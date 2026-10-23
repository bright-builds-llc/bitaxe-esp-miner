//! The Gamma 601 core-voltage check's outcome and its USB record (ADR-0041, phase D).
//!
//! Only the test-only `gamma601-vout-check` firmware runs the check: it configures the TPS546
//! as upstream does, then turns the output on once for a bounded window with the ASIC held in
//! reset, judged by [`crate::tps546::enable`]. Every reason is a closed label.

pub use crate::tps546::config::Readback;
use crate::tps546::enable::EnableSummary;
use crate::tps546::OffState;

/// How the check ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckResult {
    /// Not run yet: it waits for the output to be confirmed off after boot.
    Pending,
    /// The window completed within every limit.
    Passed,
    /// The output was commanded on, and the window ended early.
    Aborted,
    /// The output was never commanded on.
    Refused,
}

impl CheckResult {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Passed => "passed",
            Self::Aborted => "aborted",
            Self::Refused => "refused",
        }
    }
}

/// The check's record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoutCheckRecord {
    pub result: CheckResult,
    /// A closed label; `none` when the check passed or is pending.
    pub reason: &'static str,
    pub target_mv: u32,
    pub maybe_vout_mode: Option<u8>,
    pub readback: Readback,
    /// `STATUS_WORD`'s fault bits and `STATUS_CML` after the configuration writes, or after
    /// the first one that failed, before `CLEAR_FAULTS`: whether a write set CML.
    pub maybe_config_faults: Option<u16>,
    pub maybe_config_cml: Option<u8>,
    /// `STATUS_WORD`'s fault bits after `CLEAR_FAULTS`, before the enable.
    pub maybe_faults_before: Option<u16>,
    pub summary: EnableSummary,
    /// The output's state after the check commanded it off; `None` before that.
    pub maybe_off: Option<OffState>,
    /// The check thread's least free stack, in bytes, measured after the check.
    pub maybe_stack_free_bytes: Option<u32>,
}

impl VoutCheckRecord {
    /// Before the check runs.
    #[must_use]
    pub const fn pending(target_mv: u32) -> Self {
        Self {
            result: CheckResult::Pending,
            reason: "none",
            target_mv,
            maybe_vout_mode: None,
            readback: Readback::Unread,
            maybe_config_faults: None,
            maybe_config_cml: None,
            maybe_faults_before: None,
            summary: EnableSummary {
                samples: 0,
                vout_mv_min: 0,
                vout_mv_max: 0,
                iout_ma_max: 0,
                vr_temp_c_max: 0,
                board_temp_c_max: 0,
                vin_mv_min: 0,
                vin_mv_max: 0,
                faults_after: 0,
            },
            maybe_off: None,
            maybe_stack_free_bytes: None,
        }
    }

    /// Ends the check with `result` and `reason`.
    pub fn conclude(&mut self, result: CheckResult, reason: &'static str) {
        self.result = result;
        self.reason = reason;
    }

    /// The closed USB record. The extremes mean nothing until a sample was taken.
    #[must_use]
    pub fn usb_marker(&self) -> String {
        let summary = &self.summary;
        format!(
            "usb_gamma601_vout_check schema=v1 result={} reason={} target_mv={} vout_mode={} readback={} config_faults={} config_cml={} faults_before={} faults_after={} samples={} vout_mv_min={} vout_mv_max={} iout_ma_max={} vr_temp_c_max={} board_temp_c_max={} vin_mv_min={} vin_mv_max={} off={} stack_free={} redacted=true",
            self.result.label(),
            self.reason,
            self.target_mv,
            self.maybe_vout_mode
                .map_or_else(|| "unread".to_owned(), |mode| format!("0x{mode:02x}")),
            readback_label(self.readback),
            hex(self.maybe_config_faults.map(u32::from), 4),
            hex(self.maybe_config_cml.map(u32::from), 2),
            hex(self.maybe_faults_before.map(u32::from), 4),
            format_args!("0x{:04x}", summary.faults_after),
            summary.samples,
            summary.vout_mv_min,
            summary.vout_mv_max,
            summary.iout_ma_max,
            summary.vr_temp_c_max,
            summary.board_temp_c_max,
            summary.vin_mv_min,
            summary.vin_mv_max,
            self.maybe_off.map_or("unread", OffState::label),
            self.maybe_stack_free_bytes
                .map_or_else(|| "unread".to_owned(), |bytes| bytes.to_string()),
        )
    }
}

fn hex(maybe_value: Option<u32>, digits: usize) -> String {
    maybe_value.map_or_else(
        || "unread".to_owned(),
        |value| format!("0x{value:0digits$x}"),
    )
}

fn readback_label(readback: Readback) -> String {
    match readback {
        Readback::Unread => "unread".to_owned(),
        Readback::Matched => "match".to_owned(),
        Readback::Mismatch {
            command,
            wrote,
            read,
        } => format!("mismatch:0x{command:02x}:0x{wrote:04x}:0x{read:04x}"),
    }
}

#[cfg(test)]
mod tests;
