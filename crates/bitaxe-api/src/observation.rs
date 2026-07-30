//! Read-only API projection of producer-owned observation truth.

use bitaxe_safety::observation::{
    FaultReason, MonotonicMillis, Observation, StaleReason, StampedSample, UnavailableReason,
};
use bitaxe_safety::{
    power::{INPUT_VOLTAGE_MARGIN_RATIO, INPUT_VOLTAGE_NOMINAL_VOLTS, POWER_SAMPLE_STALE_AFTER_MS},
    thermal::{ASIC_THROTTLE_TEMP_C, MIN_PLAUSIBLE_TEMP_C},
};
use serde::{Deserialize, Serialize};

const ULTRA_205_MAX_INPUT_POWER_WATTS: f64 = 15.0;

/// Stable public state labels for one observed fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStateWire {
    Fresh,
    Stale,
    Unavailable,
    Fault,
}

/// Producer-owned provenance copied to the public contract without mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationStampWire {
    pub boot_session: u64,
    pub sequence: u64,
    pub acquired_at_ms: u64,
}

impl<T> From<&StampedSample<T>> for ObservationStampWire {
    fn from(sample: &StampedSample<T>) -> Self {
        Self {
            boot_session: sample.boot_session().get(),
            sequence: sample.sequence().get(),
            acquired_at_ms: sample.acquired_at().get(),
        }
    }
}

/// Typed and redaction-safe reason attached to a non-fresh observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "code", rename_all = "snake_case")]
pub enum ObservationReasonWire {
    Stale(StaleReason),
    Unavailable(UnavailableReason),
    Fault(FaultReason),
}

/// Truth-only wire projection. Numeric compatibility values live elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationTruthWire {
    pub state: ObservationStateWire,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stamp: Option<ObservationStampWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ObservationReasonWire>,
}

impl<T> From<&Observation<T>> for ObservationTruthWire {
    fn from(observation: &Observation<T>) -> Self {
        match observation {
            Observation::Fresh { sample } => Self {
                state: ObservationStateWire::Fresh,
                stamp: Some(sample.into()),
                reason: None,
            },
            Observation::Stale { last_good, reason } => Self {
                state: ObservationStateWire::Stale,
                stamp: Some(last_good.into()),
                reason: Some(ObservationReasonWire::Stale(*reason)),
            },
            Observation::Unavailable { reason } => Self {
                state: ObservationStateWire::Unavailable,
                stamp: None,
                reason: Some(ObservationReasonWire::Unavailable(*reason)),
            },
            Observation::Fault {
                reason,
                maybe_last_good,
            } => Self {
                state: ObservationStateWire::Fault,
                stamp: maybe_last_good.as_ref().map(Into::into),
                reason: Some(ObservationReasonWire::Fault(*reason)),
            },
        }
    }
}

/// Complete stored observation state consumed by operator projections.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetryObservations {
    pub power_watts: Observation<f64>,
    pub bus_voltage_volts: Observation<f64>,
    pub current_amps: Observation<f64>,
    pub chip_temp_celsius: Observation<f64>,
    pub vr_temp_celsius: Observation<f64>,
    pub fan_rpm: Observation<u16>,
}

impl TelemetryObservations {
    #[must_use]
    pub const fn unavailable(reason: UnavailableReason) -> Self {
        Self {
            power_watts: Observation::unavailable(reason),
            bus_voltage_volts: Observation::unavailable(reason),
            current_amps: Observation::unavailable(reason),
            chip_temp_celsius: Observation::unavailable(reason),
            vr_temp_celsius: Observation::unavailable(reason),
            fan_rpm: Observation::unavailable(reason),
        }
    }

    /// Returns unavailable truth for a retained source that has no valid
    /// producer-owned session, sequence, or acquisition time.
    #[must_use]
    pub const fn unavailable_from_unstamped_legacy_source() -> Self {
        Self::unavailable(UnavailableReason::ProducerUnavailable)
    }

    /// Requires every supported fresh, validated Ultra 205 safety fact before mining effects.
    #[must_use]
    pub fn is_ultra_205_mining_safe_at(&self, now: MonotonicMillis) -> bool {
        let Some(power_watts) = maybe_current_value(&self.power_watts, now) else {
            return false;
        };
        let Some(bus_voltage_volts) = maybe_current_value(&self.bus_voltage_volts, now) else {
            return false;
        };
        let Some(current_amps) = maybe_current_value(&self.current_amps, now) else {
            return false;
        };
        let Some(chip_temp_celsius) = maybe_current_value(&self.chip_temp_celsius, now) else {
            return false;
        };
        if maybe_current_value(&self.fan_rpm, now).is_none() {
            return false;
        }

        let min_input_voltage = INPUT_VOLTAGE_NOMINAL_VOLTS * (1.0 - INPUT_VOLTAGE_MARGIN_RATIO);
        let max_input_voltage = INPUT_VOLTAGE_NOMINAL_VOLTS * (1.0 + INPUT_VOLTAGE_MARGIN_RATIO);
        power_watts.is_finite()
            && (0.0..=ULTRA_205_MAX_INPUT_POWER_WATTS).contains(&power_watts)
            && bus_voltage_volts.is_finite()
            && (min_input_voltage..=max_input_voltage).contains(&bus_voltage_volts)
            && current_amps.is_finite()
            && current_amps >= 0.0
            && chip_temp_celsius.is_finite()
            && (MIN_PLAUSIBLE_TEMP_C..ASIC_THROTTLE_TEMP_C).contains(&chip_temp_celsius)
    }
}

impl Default for TelemetryObservations {
    fn default() -> Self {
        Self::unavailable(UnavailableReason::NotYetObserved)
    }
}

/// Host-testable store whose reads only copy already-stamped observations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ObservationStore {
    observations: TelemetryObservations,
}

impl ObservationStore {
    #[must_use]
    pub const fn new(observations: TelemetryObservations) -> Self {
        Self { observations }
    }

    #[must_use]
    pub const fn read(&self) -> TelemetryObservations {
        self.observations
    }

    pub fn replace(&mut self, observations: TelemetryObservations) {
        self.observations = observations;
    }
}

/// Projects one fact out of an observation without changing producer metadata.
#[must_use]
pub fn project_observation<T, U: Copy>(
    observation: &Observation<T>,
    project: impl Fn(&T) -> Option<U> + Copy,
    missing_reason: UnavailableReason,
) -> Observation<U> {
    match observation {
        Observation::Fresh { sample } => maybe_project_sample(sample, project).map_or_else(
            || Observation::unavailable(missing_reason),
            |sample| Observation::Fresh { sample },
        ),
        Observation::Stale { last_good, reason } => maybe_project_sample(last_good, project)
            .map_or_else(
                || Observation::unavailable(missing_reason),
                |last_good| Observation::Stale {
                    last_good,
                    reason: *reason,
                },
            ),
        Observation::Unavailable { reason } => Observation::unavailable(*reason),
        Observation::Fault {
            reason,
            maybe_last_good,
        } => Observation::Fault {
            reason: *reason,
            maybe_last_good: maybe_last_good
                .as_ref()
                .and_then(|sample| maybe_project_sample(sample, project)),
        },
    }
}

fn maybe_project_sample<T, U>(
    sample: &StampedSample<T>,
    project: impl Fn(&T) -> Option<U>,
) -> Option<StampedSample<U>> {
    let maybe_value = project(sample.value());
    maybe_value.map(|value| {
        StampedSample::new(
            value,
            sample.boot_session(),
            sample.sequence(),
            sample.acquired_at(),
        )
    })
}

fn maybe_current_value<T: Copy>(observation: &Observation<T>, now: MonotonicMillis) -> Option<T> {
    let Observation::Fresh { sample } = observation else {
        return None;
    };
    if now.get().saturating_sub(sample.acquired_at().get()) > u64::from(POWER_SAMPLE_STALE_AFTER_MS)
    {
        return None;
    }

    Some(*sample.value())
}

#[cfg(test)]
mod tests;
