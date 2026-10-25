//! The test-only Gamma 601 chip check (ADR-0041 item 8, phase F3.1; `gamma601-chip-check`
//! only).
//!
//! The shared powered check (`check_power`) with nothing after the chip detection: the BM1370
//! is powered and released from reset only for the count, which must find exactly one chip. No
//! work is sent and the frequency never leaves the chip's reset default.

use super::check_guard;
use super::check_power::{self, CheckSpec, TARGET_MV};
use crate::safety_adapter::RuntimeI2cOwner;
use bitaxe_safety::gamma601_chip_check::{CheckResult, ChipCheckRecord};
use bitaxe_safety::gamma601_safe_boot::SafeBootReport;
use std::sync::Mutex;

/// Boot-line literal the package and flash tools look for (ADR-0040's pattern).
pub(super) const VARIANT_MARKER: &str = "firmware_variant=gamma601-chip-check";
const SPEC: CheckSpec = CheckSpec {
    tag: "gamma601_chip_check",
    latch_armed: check_power::CHIP_CHECK_ARMED,
    latch_done: 0x4348_4b44,
};

static RECORD: Mutex<ChipCheckRecord> = Mutex::new(ChipCheckRecord::pending(TARGET_MV as u32));

/// Whether this boot's check has started; it never runs twice.
pub(super) struct ChipCheck {
    started: bool,
}

impl ChipCheck {
    pub(super) const fn pending() -> Self {
        Self { started: false }
    }

    /// Runs the check on the first sweep that `check_power::ready` admits. It blocks the
    /// safe-boot thread for a few seconds. Returns whether it ran, so the caller re-asserts off.
    pub(super) fn poll(
        &mut self,
        owner: &mut RuntimeI2cOwner<'_>,
        report: &SafeBootReport,
    ) -> bool {
        if !check_power::ready(self.started, report) {
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

// Kept a distinct frame so the build's stack audit can bound it (`firmware-stack-budget.ts`).
#[inline(never)]
fn run_check(owner: &mut RuntimeI2cOwner<'_>, report: SafeBootReport) -> ChipCheckRecord {
    check_power::run_powered(owner, report, &SPEC, |_| Ok(()))
}
