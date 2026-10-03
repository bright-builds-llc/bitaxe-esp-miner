//! First failing Ultra 205 mining-safety fact. Diagnostic detail only: the
//! verdict decides exactly what `is_ultra_205_mining_safe_at` decides.

use bitaxe_safety::observation::{MonotonicMillis, Observation, StampedSample};
use bitaxe_safety::power::POWER_SAMPLE_STALE_AFTER_MS;

use super::{maybe_out_of_range_fact, TelemetryObservations};

/// Safety facts in the order the mining predicate evaluates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SafetyFact {
    Power = 1,
    BusVoltage = 2,
    Current = 3,
    ChipTemperature = 4,
    FanRpm = 5,
}

impl SafetyFact {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Power => "power",
            Self::BusVoltage => "bus_voltage",
            Self::Current => "current",
            Self::ChipTemperature => "chip_temperature",
            Self::FanRpm => "fan_rpm",
        }
    }

    #[must_use]
    pub const fn maybe_from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Power),
            2 => Some(Self::BusVoltage),
            3 => Some(Self::Current),
            4 => Some(Self::ChipTemperature),
            5 => Some(Self::FanRpm),
            _ => None,
        }
    }
}

/// Why the first failing fact was not admissible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SafetyFactState {
    /// Fresh, but acquired longer ago than the one-second sample window.
    Expired = 1,
    Stale = 2,
    Unavailable = 3,
    Fault = 4,
    OutOfRange = 5,
}

impl SafetyFactState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Expired => "expired",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
            Self::Fault => "fault",
            Self::OutOfRange => "out_of_range",
        }
    }

    #[must_use]
    pub const fn maybe_from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Expired),
            2 => Some(Self::Stale),
            3 => Some(Self::Unavailable),
            4 => Some(Self::Fault),
            5 => Some(Self::OutOfRange),
            _ => None,
        }
    }
}

/// The first failing fact, with its last known value (in thousandths of the
/// fact's unit: mW, mV, mA, m°C or milli-RPM) and sample age when known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafetyVerdict {
    pub fact: SafetyFact,
    pub state: SafetyFactState,
    pub maybe_value_milli: Option<i64>,
    pub maybe_age_ms: Option<u64>,
}

struct Rejection {
    state: SafetyFactState,
    maybe_value_milli: Option<i64>,
    maybe_age_ms: Option<u64>,
}

impl Rejection {
    const fn verdict(self, fact: SafetyFact) -> SafetyVerdict {
        SafetyVerdict {
            fact,
            state: self.state,
            maybe_value_milli: self.maybe_value_milli,
            maybe_age_ms: self.maybe_age_ms,
        }
    }
}

impl TelemetryObservations {
    /// Returns `None` exactly when every supported fact is fresh, current and in range.
    #[must_use]
    pub fn ultra_205_mining_safety_verdict_at(
        &self,
        now: MonotonicMillis,
    ) -> Option<SafetyVerdict> {
        let power = match admissible(&self.power_watts, now) {
            Ok(fact) => fact,
            Err(rejection) => return Some(rejection.verdict(SafetyFact::Power)),
        };
        let bus_voltage = match admissible(&self.bus_voltage_volts, now) {
            Ok(fact) => fact,
            Err(rejection) => return Some(rejection.verdict(SafetyFact::BusVoltage)),
        };
        let current = match admissible(&self.current_amps, now) {
            Ok(fact) => fact,
            Err(rejection) => return Some(rejection.verdict(SafetyFact::Current)),
        };
        let chip_temperature = match admissible(&self.chip_temp_celsius, now) {
            Ok(fact) => fact,
            Err(rejection) => return Some(rejection.verdict(SafetyFact::ChipTemperature)),
        };
        if let Err(rejection) = admissible(&self.fan_rpm, now) {
            return Some(rejection.verdict(SafetyFact::FanRpm));
        }

        let fact = maybe_out_of_range_fact(power.0, bus_voltage.0, current.0, chip_temperature.0)?;
        let (value, age_ms) = match fact {
            SafetyFact::Power => power,
            SafetyFact::BusVoltage => bus_voltage,
            SafetyFact::Current => current,
            // Fan RPM has freshness but no range limit, so it never fails here.
            SafetyFact::ChipTemperature | SafetyFact::FanRpm => chip_temperature,
        };
        Some(SafetyVerdict {
            fact,
            state: SafetyFactState::OutOfRange,
            maybe_value_milli: maybe_milli(value),
            maybe_age_ms: Some(age_ms),
        })
    }
}

/// A fresh value within the sample window, with its age in milliseconds.
fn admissible<T: Copy + Into<f64>>(
    observation: &Observation<T>,
    now: MonotonicMillis,
) -> Result<(f64, u64), Rejection> {
    let rejection = |state, maybe_sample: Option<&StampedSample<T>>| Rejection {
        state,
        maybe_value_milli: maybe_sample.and_then(|sample| maybe_milli((*sample.value()).into())),
        maybe_age_ms: maybe_sample.map(|sample| age_ms(sample, now)),
    };
    match observation {
        Observation::Fresh { sample } => {
            let age = age_ms(sample, now);
            if age > u64::from(POWER_SAMPLE_STALE_AFTER_MS) {
                return Err(rejection(SafetyFactState::Expired, Some(sample)));
            }
            Ok(((*sample.value()).into(), age))
        }
        Observation::Stale { last_good, .. } => {
            Err(rejection(SafetyFactState::Stale, Some(last_good)))
        }
        Observation::Unavailable { .. } => Err(rejection(SafetyFactState::Unavailable, None)),
        Observation::Fault {
            maybe_last_good, ..
        } => Err(rejection(SafetyFactState::Fault, maybe_last_good.as_ref())),
    }
}

fn age_ms<T>(sample: &StampedSample<T>, now: MonotonicMillis) -> u64 {
    now.get().saturating_sub(sample.acquired_at().get())
}

#[allow(clippy::cast_possible_truncation)]
fn maybe_milli(value: f64) -> Option<i64> {
    let milli = (value * 1000.0).round();
    // The bound keeps the cast exact; non-finite values have no closed projection.
    (milli.is_finite() && milli.abs() < 9.0e15).then_some(milli as i64)
}

#[cfg(test)]
#[path = "safety_verdict_tests.rs"]
mod tests;
