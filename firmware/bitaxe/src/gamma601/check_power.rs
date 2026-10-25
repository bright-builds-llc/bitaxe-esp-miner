//! The powered-check runner the test-only Gamma 601 chip and hash checks share (ADR-0041 items
//! 8 and 9).
//!
//! Once per power-on, from a confirmed-off output and the executor holding the ASIC reset, a
//! check:
//! 1. refuses after any reset but a power-on, software, USB or external one, or when its RTC
//!    latch shows an earlier check unfinished or already run since the last power-on;
//! 2. sets the fan to full and requires a nonzero tach reading;
//! 3. arms the latch, latches the reset pad low and turns the TPS546 on at 1.15 V, configured
//!    as upstream does (`regulator::power_on`, armed only with the reset held);
//! 4. waits 500 ms and judges the settled output (`regulator::verify_on`);
//! 5. releases the pad latch and runs the BM1370's chip detection through the executor: the
//!    reset pulse, the CHIP_ID read and the count, which must find exactly one chip; then the
//!    check's own stage runs;
//! 6. holds the reset low again, latches the pad and turns the output off, confirmed.
//!
//! A failure at any step goes straight to step 6, which always runs. The pad latch keeps the
//! reset low through a panic or restart while the output may be on. From the count on, a
//! software restart's handler drives it low and latches it, while a panic's cutoff drives it
//! low only until the reboot; that and other resets in the window leave it undriven until the
//! next boot's first act. The count starts only within `DETECT_START_CAP_MS` of the power-on
//! sequence.
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
use std::time::Duration;

/// Long enough after boot for Wi-Fi, the display and the first sweeps to settle.
const START_AFTER_MS: u64 = 30_000;
pub(super) const TARGET_MV: u16 = 1_150;
/// Upstream's `mining_start` waits this long after setting the voltage.
const SETTLE_MS: u64 = 500;
const FAN_FULL_PERCENT: u8 = 100;
const FAN_PROOF_TIMEOUT_MS: u64 = 3_000;
const FAN_PROOF_POLL_MS: u64 = 250;
/// The executor's state lock may be briefly busy, which reads as the reset not held.
const RESET_PROOF_TIMEOUT_MS: u64 = 1_000;
const RESET_PROOF_POLL_MS: u64 = 20;

/// Survives a software, panic, watchdog or USB reset but not the removal of all power, so an
/// interrupted check, or a second one, refuses until then. A power-on leaves it undefined.
#[link_section = ".rtc_noinit"]
static mut RTC_CHECK_LATCH: u32 = 0;

/// From the power-on sequence's start until the output is confirmed off.
static OUTPUT_MAY_BE_ON: AtomicBool = AtomicBool::new(false);

/// Every check's armed value: an interrupted check of either kind blocks both.
pub(super) const CHIP_CHECK_ARMED: u32 = 0x4348_4b31;
pub(super) const HASH_CHECK_ARMED: u32 = 0x4853_4b31;

/// What distinguishes one check from another.
pub(super) struct CheckSpec {
    /// The log key, such as `gamma601_chip_check`.
    pub(super) tag: &'static str,
    /// Set in RTC memory before the enable and kept until the output is confirmed off after it.
    pub(super) latch_armed: u32,
    /// Set once the check has concluded with the output confirmed off.
    pub(super) latch_done: u32,
}

pub(super) type Ended = (CheckResult, &'static str);

pub(super) const fn refused(reason: &'static str) -> Ended {
    (CheckResult::Refused, reason)
}

pub(super) const fn aborted(reason: &'static str) -> Ended {
    (CheckResult::Aborted, reason)
}

/// What a check's stage may use once exactly one chip answered.
pub(super) struct Powered<'a, 'b> {
    pub(super) owner: &'a mut RuntimeI2cOwner<'b>,
    pub(super) report: SafeBootReport,
    pub(super) record: &'a mut ChipCheckRecord,
    /// When the power-on sequence started, and when the detection returned.
    pub(super) started_ms: u64,
    pub(super) detected_ms: u64,
}

/// Whether a check that has not yet started may: `START_AFTER_MS` after boot, with the
/// controller identified and the output confirmed off.
pub(super) fn ready(started: bool, report: &SafeBootReport) -> bool {
    !started
        && crate::runtime_uptime::millis() >= START_AFTER_MS
        && matches!(report.controller, Controller::Found(_))
        && report.off_confirmed()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Latch {
    Clear,
    Armed,
    Done,
}

/// Steps 1 to 6 around `stage`. Inlined so each check's own `run_check` frame, which the
/// build's stack audit bounds, holds it.
#[inline(always)]
pub(super) fn run_powered(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    spec: &CheckSpec,
    stage: impl FnOnce(&mut Powered<'_, '_>) -> Result<(), Ended>,
) -> ChipCheckRecord {
    let mut record = ChipCheckRecord::pending(u32::from(TARGET_MV));
    record.result = CheckResult::Running;
    record.maybe_diode_before_c = read_diode(owner, report);
    let latch = read_latch(spec);
    let mut maybe_started_ms = None;
    match power_and_detect(
        owner,
        report,
        spec,
        latch,
        &mut record,
        &mut maybe_started_ms,
        stage,
    ) {
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
        log::warn!("{}=fault category=reset_hold", spec.tag);
    }
    if force_reset_low_and_hold().is_err() {
        log::warn!("{}=fault category=reset_pad_hold", spec.tag);
    }
    if let Some(started_ms) = maybe_started_ms {
        record.maybe_on_ms = Some(crate::runtime_uptime::millis().saturating_sub(started_ms));
    }
    record.maybe_off = regulator::turn_off_and_confirm(owner, report);
    // Unconfirmed, the pad stays latched low and an armed latch stays armed. A check refused
    // before its power-on sequence (an armed latch among the reasons) leaves the latch as it was.
    if record.maybe_off == Some(OffState::Confirmed) {
        OUTPUT_MAY_BE_ON.store(false, Ordering::Release);
        release_reset_pad(spec);
        if maybe_started_ms.is_some() {
            write_latch(spec.latch_done);
        }
    }
    // SAFETY: a side-effect-free query of the calling task's own stack watermark, in bytes on
    // the pinned Xtensa port.
    record.maybe_stack_free_bytes =
        Some(unsafe { sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) });
    record
}

/// Steps 1 to 5 and the stage. An `Err` after the power-on sequence starts leaves the output for
/// step 6.
#[inline(always)]
fn power_and_detect(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    spec: &CheckSpec,
    latch: Latch,
    record: &mut ChipCheckRecord,
    maybe_started_ms: &mut Option<u64>,
    stage: impl FnOnce(&mut Powered<'_, '_>) -> Result<(), Ended>,
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
    write_latch(spec.latch_armed);
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
    let detected_ms = crate::runtime_uptime::millis();
    record.chips =
        crate::asic_adapter::last_counted_chips().map_or(ChipCount::Incomplete, ChipCount::Counted);
    record.maybe_blocker = detected.err().map(ProductionAsicBlocker::as_str);
    observe(owner, report, record);
    record.maybe_diode_after_c = read_diode(owner, report);
    match (detected, record.chips) {
        (Ok(()), ChipCount::Counted(1)) => {}
        (_, ChipCount::Counted(_)) => return Err(aborted("chip_count_not_one")),
        _ => return Err(aborted("chip_count_incomplete")),
    }
    stage(&mut Powered {
        owner,
        report,
        record,
        started_ms,
        detected_ms,
    })
}

/// The 601's catalog chip count, at the BM1370 defaults; detection never sets a frequency.
pub(super) fn chip_config() -> Bm1370InitConfig {
    Bm1370InitConfig {
        chip_count: crate::board_identity::catalog_entry().asic_count(),
        ..Bm1370InitConfig::gamma_single_chip()
    }
}

/// A fresh controller reading, folded into the record's extremes.
pub(super) fn observe(
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

pub(super) fn read_diode(owner: &mut RuntimeI2cOwner<'_>, report: SafeBootReport) -> Option<f64> {
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

fn read_latch(spec: &CheckSpec) -> Latch {
    if check_guard::power_on_reset() {
        return Latch::Clear;
    }
    // SAFETY: only the safe-boot thread touches the latch. Volatile access keeps the retained
    // RTC word from being optimized into ordinary RAM state.
    let value = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(RTC_CHECK_LATCH)) };
    if value == spec.latch_armed || value == CHIP_CHECK_ARMED || value == HASH_CHECK_ARMED {
        Latch::Armed
    } else if value == spec.latch_done {
        Latch::Done
    } else {
        Latch::Clear
    }
}

fn write_latch(value: u32) {
    // SAFETY: as `read_latch`.
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(RTC_CHECK_LATCH), value) }
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
        log::warn!("gamma601_check=fault category=restart_reset_hold");
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
fn release_reset_pad(spec: &CheckSpec) {
    if release_reset_pad_for_count().is_err() {
        log::warn!("{}=fault category=reset_pad_release", spec.tag);
    }
}
