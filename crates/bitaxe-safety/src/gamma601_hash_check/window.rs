//! The hash check's telemetry judge: every sample from the init's start to the window's end.

use crate::tps546::StatusWord;

/// VOUT may stray this far from the target once the init starts.
const VOUT_TOLERANCE_VOLTS: f64 = 0.050;
/// Above the TPS546's 4.5 V `VIN_OFF`, so a sagging supply stops the check before the regulator.
const VIN_MIN_VOLTS: f64 = 4.6;
/// Under the configured 30 A fault; one BM1370 at 300 MHz draws a few amps.
const IOUT_MAX_AMPS: f64 = 20.0;
/// Under upstream's 105 C regulator throttle.
const VR_TEMP_MAX_C: f64 = 90.0;
/// The diode limit once the gate has passed; the gate's own ceiling.
const DIODE_STOP_C: f64 = super::DIODE_MAX_C;
/// `STATUS_WORD`'s communication summary bit: the safe boot's own transfers set it (phase D2), so
/// it is no power fault.
const STATUS_CML: u16 = 0x0002;
/// A good reading of each kind (the controller, the fan and, once required, the diode) may be
/// at most this old; a failed read in between is skipped.
const MAX_SAMPLE_GAP_MS: u32 = 1_500;
/// Bounds the samples over the longest stage: the init, the stabilization and the window.
const MAX_SAMPLES: u32 = 400;

/// One telemetry reading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HashReadings {
    pub vout_volts: f64,
    pub vin_volts: f64,
    pub iout_amps: f64,
    pub vr_temp_c: f64,
    pub maybe_diode_c: Option<f64>,
    pub maybe_fan_rpm: Option<u32>,
    pub status: StatusWord,
}

/// A reading, or its absence, at a time measured from the init's start.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HashSample {
    pub elapsed_ms: u32,
    pub maybe_readings: Option<HashReadings>,
}

/// Why the judge stopped the check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashStop {
    ClockNotAdvancing,
    MissedSample,
    NonFiniteReading,
    NewFault,
    VoutOutOfBand,
    VinLow,
    OverPower,
    Overcurrent,
    VrOverTemperature,
    DiodeOverTemperature,
    DiodeLost,
    FanStopped,
    SampleOverrun,
}

impl HashStop {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ClockNotAdvancing => "clock_not_advancing",
            Self::MissedSample => "missed_sample",
            Self::NonFiniteReading => "non_finite_reading",
            Self::NewFault => "new_fault",
            Self::VoutOutOfBand => "vout_out_of_band",
            Self::VinLow => "vin_low",
            Self::OverPower => "over_power",
            Self::Overcurrent => "overcurrent",
            Self::VrOverTemperature => "vr_over_temperature",
            Self::DiodeOverTemperature => "diode_over_temperature",
            Self::DiodeLost => "diode_lost",
            Self::FanStopped => "fan_stopped",
            Self::SampleOverrun => "sample_overrun",
        }
    }
}

/// The extremes of every accepted sample.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct HashExtremes {
    pub maybe_vin_mv_min: Option<u32>,
    pub iout_ma_max: u32,
    pub power_mw_max: u32,
    pub vr_temp_c_max: i32,
    pub maybe_diode_max_c: Option<f64>,
}

/// The judge's state. The diode counts only once [`HashWindowJudge::require_diode`] is called,
/// after the gate: before the init it reads no chip.
#[derive(Clone, Copy, Debug)]
pub struct HashWindowJudge {
    target_volts: f64,
    max_power_watts: f64,
    baseline_faults: u16,
    diode_required: bool,
    last_elapsed_ms: Option<u32>,
    last_controller_ms: u32,
    last_fan_ms: u32,
    last_diode_ms: u32,
    samples: u32,
    extremes: HashExtremes,
    maybe_stop: Option<HashStop>,
}

impl HashWindowJudge {
    /// A judge for `target_volts` and the board's power limit, with the faults read before the
    /// init as the baseline.
    #[must_use]
    pub fn new(target_volts: f64, max_power_watts: f64, baseline: StatusWord) -> Self {
        Self {
            target_volts,
            max_power_watts,
            baseline_faults: baseline.faults(),
            diode_required: false,
            last_elapsed_ms: None,
            last_controller_ms: 0,
            last_fan_ms: 0,
            last_diode_ms: 0,
            samples: 0,
            extremes: HashExtremes::default(),
            maybe_stop: None,
        }
    }

    /// Counts the diode from now on; its staleness is measured from `elapsed_ms`.
    pub fn require_diode(&mut self, elapsed_ms: u32) {
        self.diode_required = true;
        self.last_diode_ms = elapsed_ms;
    }

    #[must_use]
    pub const fn extremes(&self) -> HashExtremes {
        self.extremes
    }

    /// Judges one sample. Once it stops, it returns the same stop for every later sample.
    pub fn observe(&mut self, sample: HashSample) -> Option<HashStop> {
        if self.maybe_stop.is_none() {
            self.maybe_stop = self.judge(sample);
        }
        self.maybe_stop
    }

    fn judge(&mut self, sample: HashSample) -> Option<HashStop> {
        let now = sample.elapsed_ms;
        if self.last_elapsed_ms.is_some_and(|last| now <= last) {
            return Some(HashStop::ClockNotAdvancing);
        }
        self.last_elapsed_ms = Some(now);
        self.samples += 1;
        if self.samples > MAX_SAMPLES {
            return Some(HashStop::SampleOverrun);
        }
        let stale = |last_ms: u32| now - last_ms.min(now) > MAX_SAMPLE_GAP_MS;
        // The cadence bound: whatever this sample holds, the last good controller reading must
        // be recent. A single failed read in between is skipped.
        if stale(self.last_controller_ms) {
            return Some(HashStop::MissedSample);
        }
        let readings = sample.maybe_readings?;
        if !finite(&readings) {
            return Some(HashStop::NonFiniteReading);
        }
        self.last_controller_ms = now;
        if readings.maybe_fan_rpm.is_some() {
            self.last_fan_ms = now;
        } else if stale(self.last_fan_ms) {
            return Some(HashStop::FanStopped);
        }
        if readings.maybe_diode_c.is_some() {
            self.last_diode_ms = now;
        } else if self.diode_required && stale(self.last_diode_ms) {
            return Some(HashStop::DiodeLost);
        }
        self.fold(&readings);
        self.breach(&readings)
    }

    fn breach(&self, readings: &HashReadings) -> Option<HashStop> {
        let power_watts = readings.vout_volts * readings.iout_amps;
        if readings.status.faults() & !(self.baseline_faults | STATUS_CML) != 0 {
            Some(HashStop::NewFault)
        } else if (readings.vout_volts - self.target_volts).abs() > VOUT_TOLERANCE_VOLTS {
            Some(HashStop::VoutOutOfBand)
        } else if readings.vin_volts < VIN_MIN_VOLTS {
            Some(HashStop::VinLow)
        } else if power_watts > self.max_power_watts {
            Some(HashStop::OverPower)
        } else if readings.iout_amps > IOUT_MAX_AMPS {
            Some(HashStop::Overcurrent)
        } else if readings.vr_temp_c > VR_TEMP_MAX_C {
            Some(HashStop::VrOverTemperature)
        } else if readings.maybe_fan_rpm == Some(0) {
            Some(HashStop::FanStopped)
        } else if self.diode_required
            && readings
                .maybe_diode_c
                .is_some_and(|celsius| celsius >= DIODE_STOP_C)
        {
            Some(HashStop::DiodeOverTemperature)
        } else {
            None
        }
    }

    fn fold(&mut self, readings: &HashReadings) {
        let extremes = &mut self.extremes;
        let vin_mv = milli(readings.vin_volts);
        extremes.maybe_vin_mv_min = Some(
            extremes
                .maybe_vin_mv_min
                .map_or(vin_mv, |min| min.min(vin_mv)),
        );
        extremes.iout_ma_max = extremes.iout_ma_max.max(milli(readings.iout_amps));
        extremes.power_mw_max = extremes
            .power_mw_max
            .max(milli(readings.vout_volts * readings.iout_amps));
        #[allow(clippy::cast_possible_truncation)]
        let vr_temp_c = readings.vr_temp_c.round() as i32;
        extremes.vr_temp_c_max = extremes.vr_temp_c_max.max(vr_temp_c);
        if self.diode_required {
            if let Some(celsius) = readings.maybe_diode_c {
                extremes.maybe_diode_max_c = Some(
                    extremes
                        .maybe_diode_max_c
                        .map_or(celsius, |max| max.max(celsius)),
                );
            }
        }
    }
}

fn finite(readings: &HashReadings) -> bool {
    [
        readings.vout_volts,
        readings.vin_volts,
        readings.iout_amps,
        readings.vr_temp_c,
    ]
    .iter()
    .chain(readings.maybe_diode_c.iter())
    .all(|value| value.is_finite())
}

/// Whole thousandths; a negative value (an idle offset) counts as zero.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn milli(value: f64) -> u32 {
    (value * 1000.0).round().clamp(0.0, f64::from(u32::MAX)) as u32
}
