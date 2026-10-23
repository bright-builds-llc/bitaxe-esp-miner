//! Judges one bounded core-voltage enable on the Gamma 601, with the ASIC held in reset
//! (ADR-0041, phase D).
//!
//! The firmware samples the controller and the board while the output is on and hands each
//! sample to [`EnableJudge::observe`]. Any reading outside its limit or implausible, a fault
//! bit that was not set before the enable, a missed sample, or a clock that does not advance
//! ends the window: the caller then commands the output off. The window also ends after a fixed
//! number of samples, whatever the clock says.

use super::config::Readback;
use super::config::{TARGET_MAX_VOLTS, TARGET_MIN_VOLTS};
use super::{ConfirmedOff, StatusWord};

/// How long the output stays on.
pub const WINDOW_MS: u32 = 60_000;
/// The output must reach its band within this time of the enable.
pub const SETTLE_MS: u32 = 1_000;
/// Samples further apart than this count as missed; the firmware samples every 250 ms.
pub const MAX_SAMPLE_GAP_MS: u32 = 750;
/// The window ends after this many samples even if the clock has not reached `WINDOW_MS`:
/// one every 250 ms, plus slack.
pub const MAX_SAMPLES: u32 = WINDOW_MS / 250 + 16;

/// `STATUS_WORD`'s CML bit: the only fault the baseline may carry, since a CML bit latched at
/// boot is under investigation (D2). Any other latched bit would mask new faults under its
/// summary.
const STATUS_CML: u16 = 0x0002;

/// The band around the target, after the settle.
pub const VOUT_TOLERANCE_VOLTS: f64 = 0.050;
/// No sample may read above this, settled or not.
pub const VOUT_CEILING_VOLTS: f64 = 1.25;
/// The ASIC is held in reset, so it should draw leakage only, in either direction; upstream
/// warns at 25 A.
pub const IOUT_MAX_AMPS: f64 = 5.0;
/// A temperature below this is a sensor fault, not a reading.
pub const TEMP_MIN_C: f64 = 0.0;
pub const VR_TEMP_MAX_C: f64 = 80.0;
pub const BOARD_TEMP_MAX_C: f64 = 60.0;
pub const VIN_MIN_VOLTS: f64 = 4.8;
pub const VIN_MAX_VOLTS: f64 = 5.7;

/// `CLEAR_FAULTS`, a send-byte command, sent before the baseline is read.
pub const CLEAR_FAULTS: [u8; 1] = [0x03];
/// `OPERATION` = on; a bus sends it only with an [`EnableArmed`].
pub const OPERATION_ON: [u8; 2] = [0x01, 0x80];

/// The one value a bus takes, by value, to turn the output on. Only [`arm`] makes one, from
/// the readback, an off confirmation after the configuration, an accepted judge and the reset
/// reading. It makes the caller pass through those checks; the readings themselves are the
/// caller's, so it is a structural guard, not proof against a caller that lies.
#[derive(Debug, PartialEq, Eq)]
pub struct EnableArmed(());

/// Arms one enable, or names the first precondition that does not hold.
pub fn arm(
    _judge: &EnableJudge,
    readback: Readback,
    _off_after_config: ConfirmedOff,
    reset_held: bool,
) -> Result<EnableArmed, EnableRefusal> {
    if readback != Readback::Matched {
        Err(EnableRefusal::ReadbackUnmatched)
    } else if !reset_held {
        Err(EnableRefusal::ResetNotHeld)
    } else {
        Ok(EnableArmed(()))
    }
}

/// One sample's readings, decoded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Readings {
    pub vout_volts: f64,
    pub iout_amps: f64,
    pub vin_volts: f64,
    pub vr_temp_c: f64,
    pub board_temp_c: f64,
    pub status: StatusWord,
}

/// A sample, `elapsed_ms` after the enable. `None` readings mean a read failed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub elapsed_ms: u32,
    pub maybe_readings: Option<Readings>,
}

/// Why the window ended early.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnableAbort {
    MissedSample,
    ClockNotAdvancing,
    WindowOverrun,
    NonFiniteReading,
    ImplausibleReading,
    VoutOverCeiling,
    VoutOutOfBand,
    Overcurrent,
    VrOverTemperature,
    BoardOverTemperature,
    VinOutOfRange,
    NewFault,
}

impl EnableAbort {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::MissedSample => "missed_sample",
            Self::ClockNotAdvancing => "clock_not_advancing",
            Self::WindowOverrun => "window_overrun",
            Self::NonFiniteReading => "non_finite_reading",
            Self::ImplausibleReading => "implausible_reading",
            Self::VoutOverCeiling => "vout_over_ceiling",
            Self::VoutOutOfBand => "vout_out_of_band",
            Self::Overcurrent => "overcurrent",
            Self::VrOverTemperature => "vr_over_temperature",
            Self::BoardOverTemperature => "board_over_temperature",
            Self::VinOutOfRange => "vin_out_of_range",
            Self::NewFault => "new_fault",
        }
    }
}

/// The extremes seen so far, in integer units for the record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnableSummary {
    pub samples: u32,
    pub vout_mv_min: u32,
    pub vout_mv_max: u32,
    pub iout_ma_max: u32,
    pub vr_temp_c_max: i32,
    pub board_temp_c_max: i32,
    pub vin_mv_min: u32,
    pub vin_mv_max: u32,
    /// The fault bits of the last sample, `OFF` masked.
    pub faults_after: u16,
}

/// Why no enable may start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnableRefusal {
    TargetOutOfRange,
    /// A fault bit other than CML was set after `CLEAR_FAULTS`.
    BaselineFaults,
    /// The controller did not report its output off when the baseline was read.
    BaselineOutputOn,
    ReadbackUnmatched,
    ResetNotHeld,
}

impl EnableRefusal {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TargetOutOfRange => "target_out_of_range",
            Self::BaselineFaults => "baseline_faults",
            Self::BaselineOutputOn => "baseline_output_on",
            Self::ReadbackUnmatched => "readback_unmatched",
            Self::ResetNotHeld => "reset_not_held",
        }
    }
}

/// What the caller does after a sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Continue,
    /// Command the output off now.
    Abort(EnableAbort),
    /// The window completed within every limit; command the output off.
    Done,
}

/// The enable window's state.
#[derive(Clone, Copy, Debug)]
pub struct EnableJudge {
    target_volts: f64,
    baseline_faults: u16,
    last_elapsed_ms: u32,
    summary: EnableSummary,
    maybe_terminal: Option<Step>,
}

impl EnableJudge {
    /// A judge for `target_volts`, with `STATUS_WORD` read after `CLEAR_FAULTS` and before the
    /// enable as the baseline. Refused for a target outside the configuration's clamp, a
    /// baseline without the output reported off, or one with any fault bit other than CML.
    pub fn new(target_volts: f64, baseline: StatusWord) -> Result<Self, EnableRefusal> {
        if !(TARGET_MIN_VOLTS..=TARGET_MAX_VOLTS).contains(&target_volts) {
            return Err(EnableRefusal::TargetOutOfRange);
        }
        if !baseline.output_off() {
            return Err(EnableRefusal::BaselineOutputOn);
        }
        if baseline.faults() & !STATUS_CML != 0 {
            return Err(EnableRefusal::BaselineFaults);
        }
        Ok(Self {
            target_volts,
            baseline_faults: baseline.faults(),
            last_elapsed_ms: 0,
            summary: EnableSummary::default(),
            maybe_terminal: None,
        })
    }

    #[must_use]
    pub const fn summary(&self) -> EnableSummary {
        self.summary
    }

    /// Judges one sample. Once it returns `Abort` or `Done`, it returns the same for every
    /// later sample.
    pub fn observe(&mut self, sample: Sample) -> Step {
        if let Some(terminal) = self.maybe_terminal {
            return terminal;
        }
        let step = self.judge(sample);
        if step != Step::Continue {
            self.maybe_terminal = Some(step);
        }
        step
    }

    fn judge(&mut self, sample: Sample) -> Step {
        if sample.elapsed_ms <= self.last_elapsed_ms {
            return Step::Abort(EnableAbort::ClockNotAdvancing);
        }
        let gap = sample.elapsed_ms - self.last_elapsed_ms;
        self.last_elapsed_ms = sample.elapsed_ms;
        let Some(readings) = sample.maybe_readings else {
            return Step::Abort(EnableAbort::MissedSample);
        };
        if gap > MAX_SAMPLE_GAP_MS {
            return Step::Abort(EnableAbort::MissedSample);
        }
        if !all_finite(&readings) {
            return Step::Abort(EnableAbort::NonFiniteReading);
        }
        self.record(&readings);
        if let Some(abort) = self.limit_breached(sample.elapsed_ms, &readings) {
            return Step::Abort(abort);
        }
        if sample.elapsed_ms >= WINDOW_MS {
            Step::Done
        } else if self.summary.samples >= MAX_SAMPLES {
            Step::Abort(EnableAbort::WindowOverrun)
        } else {
            Step::Continue
        }
    }

    fn limit_breached(&self, elapsed_ms: u32, readings: &Readings) -> Option<EnableAbort> {
        let settled = elapsed_ms >= SETTLE_MS;
        if readings.vout_volts > VOUT_CEILING_VOLTS {
            Some(EnableAbort::VoutOverCeiling)
        } else if settled && (readings.vout_volts - self.target_volts).abs() > VOUT_TOLERANCE_VOLTS
        {
            Some(EnableAbort::VoutOutOfBand)
        } else if readings.vr_temp_c < TEMP_MIN_C || readings.board_temp_c < TEMP_MIN_C {
            Some(EnableAbort::ImplausibleReading)
        } else if readings.iout_amps.abs() > IOUT_MAX_AMPS {
            Some(EnableAbort::Overcurrent)
        } else if readings.vr_temp_c > VR_TEMP_MAX_C {
            Some(EnableAbort::VrOverTemperature)
        } else if readings.board_temp_c > BOARD_TEMP_MAX_C {
            Some(EnableAbort::BoardOverTemperature)
        } else if !(VIN_MIN_VOLTS..=VIN_MAX_VOLTS).contains(&readings.vin_volts) {
            Some(EnableAbort::VinOutOfRange)
        } else if readings.status.faults() & !self.baseline_faults != 0 {
            Some(EnableAbort::NewFault)
        } else {
            None
        }
    }

    fn record(&mut self, readings: &Readings) {
        let vout_mv = milli(readings.vout_volts);
        let vin_mv = milli(readings.vin_volts);
        let first = self.summary.samples == 0;
        let summary = &mut self.summary;
        summary.vout_mv_min = if first {
            vout_mv
        } else {
            summary.vout_mv_min.min(vout_mv)
        };
        summary.vin_mv_min = if first {
            vin_mv
        } else {
            summary.vin_mv_min.min(vin_mv)
        };
        summary.vout_mv_max = summary.vout_mv_max.max(vout_mv);
        summary.vin_mv_max = summary.vin_mv_max.max(vin_mv);
        summary.iout_ma_max = summary.iout_ma_max.max(milli(readings.iout_amps.abs()));
        let vr_temp_c = readings.vr_temp_c as i32;
        let board_temp_c = readings.board_temp_c as i32;
        summary.vr_temp_c_max = if first {
            vr_temp_c
        } else {
            summary.vr_temp_c_max.max(vr_temp_c)
        };
        summary.board_temp_c_max = if first {
            board_temp_c
        } else {
            summary.board_temp_c_max.max(board_temp_c)
        };
        summary.faults_after = readings.status.faults();
        summary.samples += 1;
    }
}

fn all_finite(readings: &Readings) -> bool {
    [
        readings.vout_volts,
        readings.iout_amps,
        readings.vin_volts,
        readings.vr_temp_c,
        readings.board_temp_c,
    ]
    .iter()
    .all(|value| value.is_finite())
}

/// Thousandths, saturating at zero.
fn milli(value: f64) -> u32 {
    (value * 1000.0).max(0.0) as u32
}

#[cfg(test)]
mod tests;
