//! The test-only Gamma 601 core-voltage check (ADR-0041, phase D; `gamma601-vout-check` only).
//!
//! Once per boot, from a confirmed-off output and a held ASIC reset, the check:
//! 1. refuses after any reset but a power-on, software, USB or external one, after a check that
//!    an earlier boot left unfinished, or when the controller already holds the check's
//!    configuration from earlier in its power-up;
//! 2. commands the output off again, as upstream does before configuring;
//! 3. reads `VOUT_MODE` and writes the Gamma configuration, still off;
//! 4. reads every register back, records the status the writes left, and re-confirms off;
//! 5. sends `CLEAR_FAULTS`, reads the fault baseline, and sets the fan to full;
//! 6. latches the ASIC reset pad, then turns the output on at 1.15 V;
//! 7. samples every 250 ms for 60 s through the pure judge, with the reset re-read each time;
//! 8. commands the output off and confirms it.
//!
//! Nothing here touches the ASIC's UART, or drives its reset other than low. Any failure before
//! the enable leaves the output off, and every path ends with the output commanded off.
//!
//! Reference: `reference/esp-miner/main/power/TPS546.c` (`TPS546_init`, `TPS546_set_vout`).

use super::regulator;
use crate::safety_adapter::{gamma601 as hardware, RuntimeI2cOwner};
use bitaxe_safety::gamma601_safe_boot::{Controller, SafeBootReport};
use bitaxe_safety::gamma601_vout_check::{CheckResult, Readback, VoutCheckRecord};
use bitaxe_safety::tps546::config;
use bitaxe_safety::tps546::enable::{self, EnableJudge, Readings, Sample, Step};
use bitaxe_safety::tps546::{ConfirmedOff, OffState, StatusWord};
use esp_idf_svc::sys;
use std::sync::Mutex;
use std::time::Duration;

/// Boot-line literal the package and flash tools look for (ADR-0040's pattern).
pub(super) const VARIANT_MARKER: &str = "firmware_variant=gamma601-vout-check";
/// Long enough after boot for Wi-Fi, the display and the first sweeps to settle.
const START_AFTER_MS: u64 = 30_000;
const TARGET_VOLTS: f64 = 1.15;
const TARGET_MV: u32 = 1_150;
const SAMPLE_PERIOD_MS: u64 = 250;
const FAN_FULL_PERCENT: u8 = 100;
const VOUT_MAX_COMMAND: u8 = 0x24;
/// Set in RTC memory before the enable and cleared once the output is confirmed off after it.
/// RTC memory survives a software, panic or watchdog reset but not the removal of all power
/// (barrel and USB), so a check interrupted mid-window refuses on later boots of this build.
/// A rebuilt image may place the word elsewhere; the controller's `VOUT_MAX` is the anchor
/// that no rebuild moves.
const LATCH_ARMED: u32 = 0x5643_4b31;

#[link_section = ".rtc_noinit"]
static mut RTC_CHECK_LATCH: u32 = 0;

static RECORD: Mutex<VoutCheckRecord> = Mutex::new(VoutCheckRecord::pending(TARGET_MV));

/// Whether this boot's check has started; it never runs twice.
pub(super) struct VoutCheck {
    started: bool,
}

impl VoutCheck {
    pub(super) const fn pending() -> Self {
        Self { started: false }
    }

    /// Runs the check on the first sweep after `START_AFTER_MS` that finds the controller
    /// identified, the output confirmed off and the ASIC reset held. It blocks the safe-boot
    /// thread for about a minute. Returns whether it ran, so the caller re-asserts off.
    pub(super) fn poll(
        &mut self,
        owner: &mut RuntimeI2cOwner<'_>,
        report: &SafeBootReport,
    ) -> bool {
        if self.started || crate::runtime_uptime::millis() < START_AFTER_MS {
            return false;
        }
        if !matches!(report.controller, Controller::Found(_)) || !report.reset_held {
            return false;
        }
        let Some(off) = report.maybe_off.and_then(OffState::confirmed) else {
            return false;
        };
        self.started = true;
        log::info!("gamma601_vout_check=started target_mv={TARGET_MV}");
        if let Ok(mut slot) = RECORD.lock() {
            slot.conclude(CheckResult::Running, "none");
        }
        // The safe boot's thread is busy for the window: its replayed record must not keep
        // claiming the pre-check readings. Only the boot's own facts stay; the readings read
        // unavailable and the fan duty shows the check's full duty.
        super::publish(SafeBootReport {
            reset_held: report.reset_held,
            controller: report.controller,
            maybe_pre_id_status: report.maybe_pre_id_status,
            maybe_pre_id_cml: report.maybe_pre_id_cml,
            maybe_start_status: report.maybe_start_status,
            maybe_start_cml: report.maybe_start_cml,
            ..SafeBootReport::initial(report.reset_held)
        });
        let record = run_check(owner, *report, off);
        log::info!("{}", record.usb_marker());
        if let Ok(mut slot) = RECORD.lock() {
            *slot = record;
        }
        true
    }
}

/// The check's latest record, logged with the safe boot's and replayed in turn with it.
pub(super) fn usb_marker() -> String {
    RECORD.lock().map_or_else(
        |_| VoutCheckRecord::pending(TARGET_MV).usb_marker(),
        |record| record.usb_marker(),
    )
}

// Kept a distinct frame so the build's stack audit can bound it (`firmware-stack-budget.ts`).
#[inline(never)]
fn run_check(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    off: ConfirmedOff,
) -> VoutCheckRecord {
    let mut record = VoutCheckRecord::pending(TARGET_MV);
    // An earlier boot's unfinished check keeps its latch until all power is removed.
    let interrupted = check_latch_armed();
    if let Err((result, reason)) =
        configure_and_enable(owner, report, off, interrupted, &mut record)
    {
        record.conclude(result, reason);
    }
    record.maybe_off = regulator::turn_off_and_confirm(owner, report);
    if record.maybe_off == Some(OffState::Confirmed) && !interrupted {
        release_reset_latch();
    }
    // SAFETY: a side-effect-free query of the calling task's own stack watermark, in bytes on
    // the pinned Xtensa port.
    record.maybe_stack_free_bytes =
        Some(unsafe { sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) });
    record
}

type Ended = (CheckResult, &'static str);

const fn refused(reason: &'static str) -> Ended {
    (CheckResult::Refused, reason)
}

/// Everything up to and including the window. An `Err` before the enable leaves the output
/// off; the caller turns it off in every case.
fn configure_and_enable(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    off: ConfirmedOff,
    interrupted: bool,
    record: &mut VoutCheckRecord,
) -> Result<(), Ended> {
    reset_reason_admitted()?;
    if interrupted {
        return Err(refused("previous_check_unfinished"));
    }
    let mut budget = hardware::budget();
    hardware::command_off(&mut owner.tps546(&mut budget))
        .map_err(|_| refused("off_command_failed"))?;
    let vout_mode = regulator::read_vout_mode(owner).ok_or(refused("vout_mode_unread"))?;
    record.maybe_vout_mode = Some(vout_mode);
    let plan = config::gamma_config_plan(off, vout_mode, TARGET_VOLTS)
        .map_err(|refusal| refused(refusal.label()))?;
    let off_after_config = configure(owner, report, plan.writes(), vout_mode, record)?;
    let mut budget = hardware::budget();
    owner
        .tps546(&mut budget)
        .clear_faults()
        .map_err(|_| refused("clear_faults_failed"))?;
    let baseline = regulator::read_status(owner).ok_or(refused("baseline_unread"))?;
    record.maybe_faults_before = Some(baseline.faults());
    let mut judge =
        EnableJudge::new(TARGET_VOLTS, baseline).map_err(|refusal| refused(refusal.label()))?;
    hardware::configure_thermal(owner, FAN_FULL_PERCENT).map_err(|_| refused("fan_failed"))?;
    let armed = enable::arm(
        &judge,
        record.readback,
        off_after_config,
        super::reset_held(),
    )
    .map_err(|refusal| refused(refusal.label()))?;
    hold_reset_latch().map_err(|()| refused("reset_hold_failed"))?;
    let mut budget = hardware::budget();
    let enabled = owner.tps546(&mut budget).operation_on(armed);
    let started_ms = crate::runtime_uptime::millis();
    // The write may have reached the controller even when it reports an error.
    enabled.map_err(|_| (CheckResult::Aborted, "enable_write_failed"))?;
    let ended = run_window(owner, report, &mut judge, started_ms);
    record.summary = judge.summary();
    match ended {
        Ok(Step::Done) => {
            record.conclude(CheckResult::Passed, "none");
            Ok(())
        }
        Ok(Step::Abort(abort)) => Err((CheckResult::Aborted, abort.label())),
        Ok(Step::Continue) => Err((CheckResult::Aborted, "window_unfinished")),
        Err(reason) => Err((CheckResult::Aborted, reason)),
    }
}

/// Only a power-on, a software restart, a USB or an external reset admits the check. Any other
/// reset, such as a panic, a watchdog or a brown-out, may have cut an earlier check's window
/// short.
fn reset_reason_admitted() -> Result<(), Ended> {
    // SAFETY: ESP-IDF exposes the current reset reason as a side-effect-free query.
    let reason = unsafe { sys::esp_reset_reason() };
    match reason {
        sys::esp_reset_reason_t_ESP_RST_POWERON
        | sys::esp_reset_reason_t_ESP_RST_SW
        | sys::esp_reset_reason_t_ESP_RST_USB
        | sys::esp_reset_reason_t_ESP_RST_EXT => Ok(()),
        sys::esp_reset_reason_t_ESP_RST_PANIC => Err(refused("previous_reset_panic")),
        sys::esp_reset_reason_t_ESP_RST_BROWNOUT => Err(refused("previous_reset_brownout")),
        sys::esp_reset_reason_t_ESP_RST_INT_WDT
        | sys::esp_reset_reason_t_ESP_RST_TASK_WDT
        | sys::esp_reset_reason_t_ESP_RST_WDT => Err(refused("previous_reset_watchdog")),
        _ => Err(refused("previous_reset_other")),
    }
}

/// Writes the plan and reads it back, recording the status the writes left even when one
/// failed, then re-confirms the output off. Refuses when the controller already holds the
/// check's `VOUT_MAX`: only a check since its last power-up writes that value, so a second
/// enable on the same power-up is refused whatever this firmware's RTC memory says.
fn configure(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    writes: &[config::ConfigWrite],
    vout_mode: u8,
    record: &mut VoutCheckRecord,
) -> Result<ConfirmedOff, Ended> {
    if already_configured(owner, writes, vout_mode)? {
        return Err(refused("controller_already_configured"));
    }
    let written = regulator::write_and_read_back(owner, writes, vout_mode).map_err(refused);
    let mut configured = report;
    hardware::read_controller(owner, &mut configured);
    record.maybe_config_faults = configured.maybe_faults;
    record.maybe_config_cml = configured.maybe_cml;
    record.readback = written?;
    if record.readback != Readback::Matched {
        return Err(refused("config_readback_mismatch"));
    }
    configured
        .maybe_off
        .and_then(OffState::confirmed)
        .ok_or(refused("off_unconfirmed_after_config"))
}

/// Whether `VOUT_MAX` already reads as the check's value, by the same rule the readback uses,
/// so any value that would admit an enable also counts as already configured.
fn already_configured(
    owner: &mut RuntimeI2cOwner<'_>,
    writes: &[config::ConfigWrite],
    vout_mode: u8,
) -> Result<bool, Ended> {
    let vout_max = writes
        .iter()
        .find(|write| write.command() == VOUT_MAX_COMMAND)
        .ok_or(refused("vout_max_unplanned"))?;
    let mut budget = hardware::budget();
    let read = owner
        .tps546(&mut budget)
        .read_config(vout_max)
        .map_err(|_| refused("vout_max_unread"))?;
    Ok(config::readback_matches(vout_max, read, vout_mode))
}

/// Samples every `SAMPLE_PERIOD_MS` until the judge ends the window, or the ASIC reset is
/// found released.
fn run_window(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    judge: &mut EnableJudge,
    started_ms: u64,
) -> Result<Step, &'static str> {
    let mut next_ms = started_ms;
    loop {
        next_ms += SAMPLE_PERIOD_MS;
        let now_ms = crate::runtime_uptime::millis();
        std::thread::sleep(Duration::from_millis(next_ms.saturating_sub(now_ms)));
        if !super::reset_held() {
            return Err("reset_released");
        }
        let maybe_readings = sample(owner, report);
        let elapsed_ms = crate::runtime_uptime::millis().saturating_sub(started_ms);
        let step = judge.observe(Sample {
            elapsed_ms: u32::try_from(elapsed_ms).unwrap_or(u32::MAX),
            maybe_readings,
        });
        if step != Step::Continue {
            return Ok(step);
        }
    }
}

/// One sample: the controller's telemetry and status, and the board temperature. Any failed
/// read makes the whole sample missing.
fn sample(owner: &mut RuntimeI2cOwner<'_>, report: SafeBootReport) -> Option<Readings> {
    let mut reading = report;
    hardware::read_controller(owner, &mut reading);
    let _ = hardware::read_thermal(owner, &mut reading);
    Some(Readings {
        vout_volts: reading.maybe_vout_volts?,
        iout_amps: reading.maybe_iout_amps?,
        vin_volts: reading.maybe_vin_volts?,
        vr_temp_c: reading.maybe_vr_temperature_celsius?,
        board_temp_c: reading.maybe_board_temperature_celsius?,
        status: StatusWord(reading.maybe_faults?),
    })
}

fn check_latch_armed() -> bool {
    // SAFETY: only the safe-boot thread touches the latch. Volatile access keeps the retained
    // RTC word from being optimized into ordinary RAM state.
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_CHECK_LATCH)) == LATCH_ARMED }
}

/// Arms the RTC latch and holds the ASIC reset pad low, so a reset while the output may be on
/// cannot release the ASIC before the next boot drives the pin itself.
fn hold_reset_latch() -> Result<(), ()> {
    // SAFETY: same single-thread latch access as `check_latch_armed`; GPIO1 is driven low by
    // `hold_asic_reset_early` and no HAL driver owns it.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(RTC_CHECK_LATCH), LATCH_ARMED);
        if sys::gpio_hold_en(sys::gpio_num_t_GPIO_NUM_1) == sys::ESP_OK {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Clears the latch and the pad hold once the output is confirmed off; the safe boot keeps
/// driving the reset low.
fn release_reset_latch() {
    // SAFETY: as `hold_reset_latch`.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(RTC_CHECK_LATCH), 0);
        if sys::gpio_hold_dis(sys::gpio_num_t_GPIO_NUM_1) != sys::ESP_OK {
            log::warn!("gamma601_vout_check=fault category=reset_hold_release");
        }
    }
}
