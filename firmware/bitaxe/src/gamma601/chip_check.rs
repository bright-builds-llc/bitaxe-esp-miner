//! The test-only Gamma 601 chip check (ADR-0041 item 8, phase F3.1; `gamma601-chip-check`
//! only).
//!
//! Once per power-on, from a confirmed-off output and the executor holding the ASIC reset, the
//! check:
//! 1. refuses after any reset but a power-on, software, USB or external one, or when its RTC
//!    latch shows an earlier check unfinished or already run since the last power-on;
//! 2. sets the fan to full and requires a nonzero tach reading;
//! 3. arms the latch, latches the reset pad low and turns the TPS546 on at 1.15 V, configured
//!    as upstream does (`regulator::power_on`, armed only with the reset held);
//! 4. waits 500 ms and judges the settled output (`regulator::verify_on`);
//! 5. releases the pad latch and runs the BM1370's chip detection through the executor: the
//!    reset pulse, the CHIP_ID read and the count, which must find exactly one chip;
//! 6. holds the reset low again, latches the pad and turns the output off, confirmed.
//!
//! A failure at any step goes straight to step 6, which always runs. The pad latch keeps the
//! reset low through a panic or restart while the output may be on. During the count, a
//! software restart's handler drives it low and latches it, while a panic's cutoff drives it
//! low only until the reboot; that and other resets in the window leave it undriven until the
//! next boot's first act. The count starts only within
//! `DETECT_START_CAP_MS` of the power-on sequence; each step's own timeout bounds the rest. No
//! work is sent and the frequency never leaves the chip's reset default.
//!
//! Reference: `reference/esp-miner/main/tasks/power_management_task.c` (`mining_start`) and
//! `main/power/asic_init.c` (`asic_initialize`), up to `count_asic_chips`.

use super::{check_guard, regulator};
use crate::safety_adapter::{gamma601 as hardware, RuntimeI2cOwner};
use bitaxe_asic::bm1366::command::Bm1366AdapterAction;
use bitaxe_asic::bm1366::production::ProductionAsicBlocker;
use bitaxe_asic::bm1370::{init::Bm1370InitConfig, plan};
use bitaxe_safety::gamma601_chip_check::{
    detect_start_late, CheckResult, ChipCheckRecord, ChipCount,
};
use bitaxe_safety::gamma601_safe_boot::{Controller, SafeBootReport};
use bitaxe_safety::sensor_acquisition::AcquisitionOutcome;
use bitaxe_safety::tps546::OffState;
use esp_idf_svc::sys;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

/// Boot-line literal the package and flash tools look for (ADR-0040's pattern).
pub(super) const VARIANT_MARKER: &str = "firmware_variant=gamma601-chip-check";
/// Long enough after boot for Wi-Fi, the display and the first sweeps to settle.
const START_AFTER_MS: u64 = 30_000;
const TARGET_MV: u16 = 1_150;
/// Upstream's `mining_start` waits this long after setting the voltage.
const SETTLE_MS: u64 = 500;
const FAN_FULL_PERCENT: u8 = 100;
const FAN_PROOF_TIMEOUT_MS: u64 = 3_000;
const FAN_PROOF_POLL_MS: u64 = 250;
/// The executor's state lock may be briefly busy, which reads as the reset not held.
const RESET_PROOF_TIMEOUT_MS: u64 = 1_000;
const RESET_PROOF_POLL_MS: u64 = 20;
/// Set in RTC memory before the enable and kept until the output is confirmed off after it.
const LATCH_ARMED: u32 = 0x4348_4b31;
/// Set once the check has concluded with the output confirmed off.
const LATCH_DONE: u32 = 0x4348_4b44;

/// Survives a software, panic, watchdog or USB reset but not the removal of all power, so an
/// interrupted check, or a second one, refuses until then. A power-on leaves it undefined.
#[link_section = ".rtc_noinit"]
static mut RTC_CHIP_CHECK_LATCH: u32 = 0;

/// From the power-on sequence's start until the output is confirmed off.
static OUTPUT_MAY_BE_ON: AtomicBool = AtomicBool::new(false);

static RECORD: Mutex<ChipCheckRecord> = Mutex::new(ChipCheckRecord::pending(TARGET_MV as u32));

/// Whether this boot's check has started; it never runs twice.
pub(super) struct ChipCheck {
    started: bool,
}

impl ChipCheck {
    pub(super) const fn pending() -> Self {
        Self { started: false }
    }

    /// Runs the check on the first sweep after `START_AFTER_MS` that finds the controller
    /// identified and the output confirmed off. It blocks the safe-boot thread for a few
    /// seconds. Returns whether it ran, so the caller re-asserts off.
    pub(super) fn poll(
        &mut self,
        owner: &mut RuntimeI2cOwner<'_>,
        report: &SafeBootReport,
    ) -> bool {
        if self.started || crate::runtime_uptime::millis() < START_AFTER_MS {
            return false;
        }
        if !matches!(report.controller, Controller::Found(_)) || !report.off_confirmed() {
            return false;
        }
        self.started = true;
        log::info!("gamma601_chip_check=started target_mv={TARGET_MV}");
        if let Ok(mut slot) = RECORD.lock() {
            slot.conclude(CheckResult::Running, "none");
        }
        check_guard::publish_window_report(report);
        let record = run_check(owner, *report);
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
        |_| ChipCheckRecord::pending(u32::from(TARGET_MV)).usb_marker(),
        |record| record.usb_marker(),
    )
}

type Ended = (CheckResult, &'static str);

const fn refused(reason: &'static str) -> Ended {
    (CheckResult::Refused, reason)
}

const fn aborted(reason: &'static str) -> Ended {
    (CheckResult::Aborted, reason)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Latch {
    Clear,
    Armed,
    Done,
}

// Kept a distinct frame so the build's stack audit can bound it (`firmware-stack-budget.ts`).
#[inline(never)]
fn run_check(owner: &mut RuntimeI2cOwner<'_>, report: SafeBootReport) -> ChipCheckRecord {
    let mut record = ChipCheckRecord::pending(u32::from(TARGET_MV));
    record.result = CheckResult::Running;
    record.maybe_diode_before_c = read_diode(owner, report);
    let latch = read_latch();
    let mut maybe_started_ms = None;
    match power_and_detect(owner, report, latch, &mut record, &mut maybe_started_ms) {
        Ok(()) => record.conclude(CheckResult::Passed, "none"),
        Err((result, reason)) => record.conclude(result, reason),
    }
    // Step 6 runs whatever happened: the reset first, then the output. The pad is driven low
    // directly too, in case the executor's hold failed.
    if crate::asic_adapter::production::execute_safe_shutdown_actions(&[
        Bm1366AdapterAction::HOLD_RESET_LOW,
    ])
    .is_err()
    {
        log::warn!("gamma601_chip_check=fault category=reset_hold");
    }
    if force_reset_low_and_hold().is_err() {
        log::warn!("gamma601_chip_check=fault category=reset_pad_hold");
    }
    if let Some(started_ms) = maybe_started_ms {
        record.maybe_on_ms = Some(crate::runtime_uptime::millis().saturating_sub(started_ms));
    }
    record.maybe_off = regulator::turn_off_and_confirm(owner, report);
    // Unconfirmed, the pad stays latched low and an armed latch stays armed. A check refused
    // before its power-on sequence (an armed latch among the reasons) leaves the latch as it was.
    if record.maybe_off == Some(OffState::Confirmed) {
        OUTPUT_MAY_BE_ON.store(false, Ordering::Release);
        release_reset_pad();
        if maybe_started_ms.is_some() {
            write_latch(LATCH_DONE);
        }
    }
    // SAFETY: a side-effect-free query of the calling task's own stack watermark, in bytes on
    // the pinned Xtensa port.
    record.maybe_stack_free_bytes =
        Some(unsafe { sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) });
    record
}

/// Steps 1 to 5. An `Err` after the power-on sequence starts leaves the output for step 6.
fn power_and_detect(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    latch: Latch,
    record: &mut ChipCheckRecord,
    maybe_started_ms: &mut Option<u64>,
) -> Result<(), Ended> {
    check_guard::reset_reason_admitted().map_err(refused)?;
    match latch {
        Latch::Armed => return Err(refused("previous_check_unfinished")),
        Latch::Done => return Err(refused("check_already_ran")),
        Latch::Clear => {}
    }
    hardware::configure_thermal(owner, FAN_FULL_PERCENT).map_err(|_| refused("fan_failed"))?;
    if !poll_until(FAN_PROOF_TIMEOUT_MS, FAN_PROOF_POLL_MS, || {
        fan_spinning(owner, report)
    }) {
        return Err(refused("fan_unproven"));
    }
    if !poll_until(
        RESET_PROOF_TIMEOUT_MS,
        RESET_PROOF_POLL_MS,
        crate::gamma601::reset_held,
    ) {
        return Err(refused("reset_not_held"));
    }
    register_restart_guard().map_err(|()| refused("restart_guard_failed"))?;
    write_latch(LATCH_ARMED);
    OUTPUT_MAY_BE_ON.store(true, Ordering::Release);
    hold_reset_pad().map_err(|()| refused("reset_pad_hold_failed"))?;
    let started_ms = crate::runtime_uptime::millis();
    *maybe_started_ms = Some(started_ms);
    let baseline = regulator::power_on(owner, report, TARGET_MV).map_err(aborted)?;
    std::thread::sleep(Duration::from_millis(SETTLE_MS));
    regulator::verify_on(owner, report, TARGET_MV, baseline).map_err(aborted)?;
    let reading = observe(owner, report, record);
    record.maybe_vout_mv = reading.maybe_vout_volts.map(millivolts);
    if detect_start_late(started_ms, crate::runtime_uptime::millis()) {
        return Err(aborted("detect_start_late"));
    }
    let actions = plan::chip_detect_actions(chip_config()).map_err(|_| aborted("plan_invalid"))?;
    release_reset_pad_for_count().map_err(|()| aborted("reset_pad_release_failed"))?;
    let detected =
        crate::asic_adapter::production::execute_chip_detection_actions_guarded_with_progress(
            &actions,
            None,
            &mut || {},
        );
    record.chips =
        crate::asic_adapter::last_counted_chips().map_or(ChipCount::Incomplete, ChipCount::Counted);
    record.maybe_blocker = detected.err().map(ProductionAsicBlocker::as_str);
    observe(owner, report, record);
    record.maybe_diode_after_c = read_diode(owner, report);
    match (detected, record.chips) {
        (Ok(()), ChipCount::Counted(1)) => Ok(()),
        (_, ChipCount::Counted(_)) => Err(aborted("chip_count_not_one")),
        _ => Err(aborted("chip_count_incomplete")),
    }
}

/// The 601's catalog chip count, at the BM1370 defaults; detection never sets a frequency.
fn chip_config() -> Bm1370InitConfig {
    Bm1370InitConfig {
        chip_count: crate::board_identity::catalog_entry().asic_count(),
        ..Bm1370InitConfig::gamma_single_chip()
    }
}

/// A fresh controller reading, folded into the record's extremes.
fn observe(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    record: &mut ChipCheckRecord,
) -> SafeBootReport {
    let mut reading = report;
    hardware::read_controller(owner, &mut reading);
    record.observe_on(
        reading.maybe_iout_amps,
        reading.maybe_vr_temperature_celsius,
    );
    reading
}

fn read_diode(owner: &mut RuntimeI2cOwner<'_>, report: SafeBootReport) -> Option<f64> {
    let mut reading = report;
    match hardware::read_thermal(owner, &mut reading).chip_temperature {
        AcquisitionOutcome::Success(celsius) => Some(celsius),
        _ => None,
    }
}

fn fan_spinning(owner: &mut RuntimeI2cOwner<'_>, report: SafeBootReport) -> bool {
    let mut reading = report;
    let _ = hardware::read_thermal(owner, &mut reading);
    reading.maybe_fan_rpm.is_some_and(|rpm| rpm > 0)
}

/// Whether `ready` holds within `timeout_ms`, asked every `poll_ms`.
fn poll_until(timeout_ms: u64, poll_ms: u64, mut ready: impl FnMut() -> bool) -> bool {
    let deadline_ms = crate::runtime_uptime::millis().saturating_add(timeout_ms);
    loop {
        if ready() {
            return true;
        }
        if crate::runtime_uptime::millis() >= deadline_ms {
            return false;
        }
        std::thread::sleep(Duration::from_millis(poll_ms));
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn millivolts(volts: f64) -> u32 {
    (volts * 1000.0).round().clamp(0.0, f64::from(u32::MAX)) as u32
}

fn read_latch() -> Latch {
    if check_guard::power_on_reset() {
        return Latch::Clear;
    }
    // SAFETY: only the safe-boot thread touches the latch. Volatile access keeps the retained
    // RTC word from being optimized into ordinary RAM state.
    match unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_CHIP_CHECK_LATCH)) } {
        LATCH_ARMED => Latch::Armed,
        LATCH_DONE => Latch::Done,
        _ => Latch::Clear,
    }
}

fn write_latch(value: u32) {
    // SAFETY: as `read_latch`.
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(RTC_CHIP_CHECK_LATCH), value) }
}

/// Latches the reset pad at the low level the executor drives, through any reset, until the
/// next boot drives the pin itself.
fn hold_reset_pad() -> Result<(), ()> {
    // SAFETY: GPIO1 is the executor's reset output, driven low when this is called.
    if unsafe { sys::gpio_hold_en(sys::gpio_num_t_GPIO_NUM_1) } == sys::ESP_OK {
        Ok(())
    } else {
        Err(())
    }
}

/// Drives the reset pad low and latches it, whoever owns the pin.
fn force_reset_low_and_hold() -> Result<(), ()> {
    // SAFETY: GPIO1 is the executor's reset output; low is its safe level.
    if unsafe { sys::gpio_set_level(sys::gpio_num_t_GPIO_NUM_1, 0) } != sys::ESP_OK {
        return Err(());
    }
    hold_reset_pad()
}

/// A software restart (the HTTP restart or OTA path) bypasses the panic cutoff, so while the
/// output may be on it drives the reset low and latches it first. Other resets, such as a host
/// USB reset, a brown-out or a hardware watchdog, run no handler (ADR-0041 item 8).
fn register_restart_guard() -> Result<(), ()> {
    // SAFETY: registers a handler with no captured state; it runs once, in `esp_restart`.
    if unsafe { sys::esp_register_shutdown_handler(Some(on_restart)) } == sys::ESP_OK {
        Ok(())
    } else {
        Err(())
    }
}

extern "C" fn on_restart() {
    if OUTPUT_MAY_BE_ON.load(Ordering::Acquire) && force_reset_low_and_hold().is_err() {
        log::warn!("gamma601_chip_check=fault category=restart_reset_hold");
    }
}

/// Frees the pad for the executor's reset pulse.
fn release_reset_pad_for_count() -> Result<(), ()> {
    // SAFETY: as `hold_reset_pad`; the executor still drives the pin low.
    if unsafe { sys::gpio_hold_dis(sys::gpio_num_t_GPIO_NUM_1) } == sys::ESP_OK {
        Ok(())
    } else {
        Err(())
    }
}

/// Frees the pad once the output is confirmed off; the executor keeps driving the reset low.
fn release_reset_pad() {
    if release_reset_pad_for_count().is_err() {
        log::warn!("gamma601_chip_check=fault category=reset_pad_release");
    }
}
