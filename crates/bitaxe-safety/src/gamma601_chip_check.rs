//! The Gamma 601 chip check's outcome and its USB record (ADR-0041 item 8, phase F3.1).
//!
//! Only the test-only `gamma601-chip-check` firmware runs the check: it turns the TPS546 on at
//! the check's voltage, releases the ASIC reset only for the chip count, then holds the reset
//! and turns the output off. Every reason is a closed label. The record shares the core-voltage
//! check's replay alternation and result vocabulary.

pub use crate::gamma601_vout_check::{CheckResult, ReplayRecord, ReplayTurn};
use crate::tps546::OffState;

/// The chip count starts only within this long of the power-on sequence's start; later, the
/// check turns the output off instead. The on time's bound otherwise comes from each step's own
/// timeout.
pub const DETECT_START_CAP_MS: u64 = 5_000;

/// Whether the chip count would start more than [`DETECT_START_CAP_MS`] after the power-on
/// sequence started.
#[must_use]
pub const fn detect_start_late(power_on_started_ms: u64, now_ms: u64) -> bool {
    now_ms.saturating_sub(power_on_started_ms) > DETECT_START_CAP_MS
}

/// What the chip count found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipCount {
    /// No count ran.
    Unread,
    /// The count ran to completion and found this many answering chips.
    Counted(u8),
    /// The count stopped early: a frame of the wrong length, a UART fault or its budget.
    Incomplete,
}

impl ChipCount {
    pub(crate) fn label(self) -> String {
        match self {
            Self::Unread => "unread".to_owned(),
            Self::Counted(chips) => chips.to_string(),
            Self::Incomplete => "incomplete".to_owned(),
        }
    }
}

/// The check's record.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChipCheckRecord {
    pub result: CheckResult,
    /// A closed label; `none` when the check passed or is pending.
    pub reason: &'static str,
    pub target_mv: u32,
    pub chips: ChipCount,
    /// The executor's closed blocker label when the detection failed; `None` otherwise.
    pub maybe_blocker: Option<&'static str>,
    /// The external diode before the enable, with the ASIC held in reset and unpowered.
    pub maybe_diode_before_c: Option<f64>,
    /// The external diode after the count, the chip powered.
    pub maybe_diode_after_c: Option<f64>,
    /// VOUT when the settled output was judged.
    pub maybe_vout_mv: Option<u32>,
    /// The highest IOUT and regulator temperature read while the output was on.
    pub iout_ma_max: u32,
    pub vr_temp_c_max: i32,
    /// How long the output may have been on: from the start of the power-on sequence to the
    /// off command.
    pub maybe_on_ms: Option<u64>,
    /// The output's state after the check commanded it off; `None` before that.
    pub maybe_off: Option<OffState>,
    /// The check thread's least free stack, in bytes, measured after the check.
    pub maybe_stack_free_bytes: Option<u32>,
}

impl ChipCheckRecord {
    /// Before the check runs.
    #[must_use]
    pub const fn pending(target_mv: u32) -> Self {
        Self {
            result: CheckResult::Pending,
            reason: "none",
            target_mv,
            chips: ChipCount::Unread,
            maybe_blocker: None,
            maybe_diode_before_c: None,
            maybe_diode_after_c: None,
            maybe_vout_mv: None,
            iout_ma_max: 0,
            vr_temp_c_max: 0,
            maybe_on_ms: None,
            maybe_off: None,
            maybe_stack_free_bytes: None,
        }
    }

    /// Ends the check with `result` and `reason`.
    pub fn conclude(&mut self, result: CheckResult, reason: &'static str) {
        self.result = result;
        self.reason = reason;
    }

    /// Folds one reading taken while the output was on into the extremes.
    pub fn observe_on(&mut self, maybe_iout_amps: Option<f64>, maybe_vr_temp_c: Option<f64>) {
        if let Some(iout) = maybe_iout_amps.filter(|amps| amps.is_finite()) {
            self.iout_ma_max = self.iout_ma_max.max(milli(iout));
        }
        if let Some(celsius) = maybe_vr_temp_c.filter(|celsius| celsius.is_finite()) {
            self.vr_temp_c_max = self.vr_temp_c_max.max(celsius.round() as i32);
        }
    }

    /// The closed USB record.
    #[must_use]
    pub fn usb_marker(&self) -> String {
        format!(
            "usb_gamma601_chip_check schema=v1 result={} reason={} target_mv={} chips={} blocker={} diode_before_c={} diode_after_c={} vout_mv={} iout_ma_max={} vr_temp_c_max={} on_ms={} off={} stack_free={} redacted=true",
            self.result.label(),
            self.reason,
            self.target_mv,
            self.chips.label(),
            self.maybe_blocker.unwrap_or("none"),
            celsius(self.maybe_diode_before_c),
            celsius(self.maybe_diode_after_c),
            unread(self.maybe_vout_mv),
            self.iout_ma_max,
            self.vr_temp_c_max,
            unread(self.maybe_on_ms),
            self.maybe_off.map_or("unread", OffState::label),
            unread(self.maybe_stack_free_bytes),
        )
    }
}

/// Amps as whole milliamps; a negative reading (an idle offset) counts as zero.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn milli(amps: f64) -> u32 {
    (amps * 1000.0).round().clamp(0.0, f64::from(u32::MAX)) as u32
}

fn celsius(maybe_celsius: Option<f64>) -> String {
    maybe_celsius
        .filter(|celsius| celsius.is_finite())
        .map_or_else(|| "unread".to_owned(), |celsius| format!("{celsius:.1}"))
}

fn unread<T: ToString>(maybe_value: Option<T>) -> String {
    maybe_value.map_or_else(|| "unread".to_owned(), |value| value.to_string())
}

#[cfg(test)]
mod tests;
