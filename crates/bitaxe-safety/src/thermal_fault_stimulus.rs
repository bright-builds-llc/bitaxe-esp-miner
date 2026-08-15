//! Pure, bounded EMC2101 invalid-sample regression stimulus.
//!
//! The firmware shell continues every real sensor read. This state machine
//! decides when a successful temperature outcome may be replaced by the
//! existing typed `InvalidSample` outcome and validates the producer truth
//! observed on the following sweep.

use crate::{
    observation::{FaultReason, Observation},
    sensor_acquisition::AcquisitionOutcome,
    thermal::ThermalReading,
};

pub const THERMAL_FAULT_STIMULUS_KIND: &str = "emc2101_invalid_sample";
pub const THERMAL_FAULT_STIMULUS_SAMPLE_COUNT: u16 = 5;
const BASELINE_SWEEP_LIMIT: u8 = 10;
const RECOVERY_SWEEP_LIMIT: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalFaultStimulusMarker {
    BaselineReady,
    FaultObserved,
    Recovered,
}

impl ThermalFaultStimulusMarker {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BaselineReady => "baseline_ready",
            Self::FaultObserved => "fault_observed",
            Self::Recovered => "recovered",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalFaultStimulusFailure {
    BaselineUnavailable,
    RealReadFailedDuringInjection,
    FaultProjectionMissing,
    RecoveryUnavailable,
}

impl ThermalFaultStimulusFailure {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BaselineUnavailable => "baseline_unavailable",
            Self::RealReadFailedDuringInjection => "real_read_failed_during_injection",
            Self::FaultProjectionMissing => "fault_projection_missing",
            Self::RecoveryUnavailable => "recovery_unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalFaultStimulusStep {
    pub outcome: AcquisitionOutcome<f64>,
    pub maybe_marker: Option<ThermalFaultStimulusMarker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    AwaitingBaseline { remaining: u8 },
    Injecting { injected: u16 },
    AwaitingRecovery { remaining: u8 },
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThermalFaultStimulus {
    phase: Phase,
}

impl Default for ThermalFaultStimulus {
    fn default() -> Self {
        Self {
            phase: Phase::AwaitingBaseline {
                remaining: BASELINE_SWEEP_LIMIT,
            },
        }
    }
}

impl ThermalFaultStimulus {
    pub fn step(
        &mut self,
        prior: &Observation<ThermalReading>,
        actual: AcquisitionOutcome<f64>,
    ) -> Result<ThermalFaultStimulusStep, ThermalFaultStimulusFailure> {
        match self.phase {
            Phase::AwaitingBaseline { remaining } => {
                if prior.is_fresh() && matches!(actual, AcquisitionOutcome::Success(_)) {
                    self.phase = Phase::Injecting { injected: 1 };
                    return Ok(ThermalFaultStimulusStep {
                        outcome: AcquisitionOutcome::InvalidSample,
                        maybe_marker: Some(ThermalFaultStimulusMarker::BaselineReady),
                    });
                }
                let Some(remaining) = remaining.checked_sub(1).filter(|value| *value > 0) else {
                    return Err(ThermalFaultStimulusFailure::BaselineUnavailable);
                };
                self.phase = Phase::AwaitingBaseline { remaining };
                Ok(ThermalFaultStimulusStep {
                    outcome: actual,
                    maybe_marker: None,
                })
            }
            Phase::Injecting { injected } => {
                // The first post-overlay sweep proves that the production
                // reducer published the expected fault. Later sweeps may
                // legitimately age its retained last-good sample to stale, so
                // the transaction latches this proof instead of weakening the
                // producer's ordinary one-second stale policy.
                if injected == 1 && !is_expected_fault(prior) {
                    return Err(ThermalFaultStimulusFailure::FaultProjectionMissing);
                }
                if !matches!(actual, AcquisitionOutcome::Success(_)) {
                    return Err(ThermalFaultStimulusFailure::RealReadFailedDuringInjection);
                }
                let next = injected + 1;
                self.phase = if next == THERMAL_FAULT_STIMULUS_SAMPLE_COUNT {
                    Phase::AwaitingRecovery {
                        remaining: RECOVERY_SWEEP_LIMIT,
                    }
                } else {
                    Phase::Injecting { injected: next }
                };
                Ok(ThermalFaultStimulusStep {
                    outcome: AcquisitionOutcome::InvalidSample,
                    maybe_marker: (injected == 1)
                        .then_some(ThermalFaultStimulusMarker::FaultObserved),
                })
            }
            Phase::AwaitingRecovery { remaining } => {
                if prior.is_fresh() {
                    self.phase = Phase::Complete;
                    return Ok(ThermalFaultStimulusStep {
                        outcome: actual,
                        maybe_marker: Some(ThermalFaultStimulusMarker::Recovered),
                    });
                }
                let Some(remaining) = remaining.checked_sub(1).filter(|value| *value > 0) else {
                    return Err(ThermalFaultStimulusFailure::RecoveryUnavailable);
                };
                self.phase = Phase::AwaitingRecovery { remaining };
                Ok(ThermalFaultStimulusStep {
                    outcome: actual,
                    maybe_marker: None,
                })
            }
            Phase::Complete => Ok(ThermalFaultStimulusStep {
                outcome: actual,
                maybe_marker: None,
            }),
        }
    }

    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(self.phase, Phase::Complete)
    }
}

fn is_expected_fault(observation: &Observation<ThermalReading>) -> bool {
    observation.state_label() == "fault"
        && observation.maybe_reason() == Some(FaultReason::ThermalReadingInvalid.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        observation::{BootSessionId, MonotonicMillis, ObservationSequence, UnavailableReason},
        sensor_acquisition::{
            reduce_sensor_sweep, ProducerSensorState, ProducerSequences, SensorSweepOutcomes,
        },
    };

    const SENSOR_STALE_AFTER_MS: u64 = 1_000;

    fn fresh(sequence: u64) -> Observation<ThermalReading> {
        Observation::record_success(
            ThermalReading {
                chip_temp_celsius: 50.0,
                maybe_board_temp_celsius: None,
                maybe_vr_temp_celsius: None,
            },
            BootSessionId::new(1),
            ObservationSequence::new(sequence),
            MonotonicMillis::new(sequence * 1_000),
        )
        .expect("fixture sequence advances")
        .0
    }

    #[test]
    fn exact_five_sample_fault_sequence_recovers() {
        // Arrange
        let mut stimulus = ThermalFaultStimulus::default();
        let mut prior = fresh(0);
        let mut markers = Vec::new();
        let mut injected = 0;

        // Act
        for sweep in 0..7 {
            let step = stimulus
                .step(&prior, AcquisitionOutcome::Success(50.0))
                .expect("closed healthy sequence");
            if step.outcome == AcquisitionOutcome::InvalidSample {
                injected += 1;
                prior = prior.record_fault(FaultReason::ThermalReadingInvalid);
            } else {
                prior = fresh(sweep + 1);
            }
            if let Some(marker) = step.maybe_marker {
                markers.push(marker);
            }
        }

        // Assert
        assert_eq!(injected, THERMAL_FAULT_STIMULUS_SAMPLE_COUNT);
        assert_eq!(
            markers,
            [
                ThermalFaultStimulusMarker::BaselineReady,
                ThermalFaultStimulusMarker::FaultObserved,
                ThermalFaultStimulusMarker::Recovered,
            ]
        );
        assert!(stimulus.is_complete());
    }

    #[test]
    fn production_order_fault_sequence_survives_stale_processing() {
        // Arrange
        let mut stimulus = ThermalFaultStimulus::default();
        let mut state = ProducerSensorState::default();
        let mut sequences = ProducerSequences::default();
        let mut markers = Vec::new();
        let mut injected = 0;

        // Act
        for sweep in 1..=8 {
            let acquired_at = MonotonicMillis::new(sweep * SENSOR_STALE_AFTER_MS);
            let step = stimulus
                .step(
                    state.thermal().temperature_truth(),
                    AcquisitionOutcome::Success(50.0),
                )
                .expect("production owner must retain the injected fault projection");
            if step.outcome == AcquisitionOutcome::InvalidSample {
                injected += 1;
            }
            if let Some(marker) = step.maybe_marker {
                markers.push(marker);
            }
            let (next_state, next_sequences) = reduce_sensor_sweep(
                state,
                sequences,
                SensorSweepOutcomes {
                    power: AcquisitionOutcome::Unavailable(UnavailableReason::ProducerUnavailable),
                    asic_temperature_celsius: step.outcome,
                    vr_temperature_celsius: AcquisitionOutcome::Unavailable(
                        UnavailableReason::UnsupportedOnBoard,
                    ),
                    tachometer_rpm: AcquisitionOutcome::Unavailable(
                        UnavailableReason::ProducerUnavailable,
                    ),
                },
                BootSessionId::new(1),
                acquired_at,
                12.0,
            )
            .expect("bounded fixture sequences remain representable");
            state = next_state.mark_stale_at(acquired_at, SENSOR_STALE_AFTER_MS);
            sequences = next_sequences;
        }

        // Assert
        assert_eq!(injected, THERMAL_FAULT_STIMULUS_SAMPLE_COUNT);
        assert_eq!(
            markers,
            [
                ThermalFaultStimulusMarker::BaselineReady,
                ThermalFaultStimulusMarker::FaultObserved,
                ThermalFaultStimulusMarker::Recovered,
            ]
        );
        assert!(stimulus.is_complete());
    }

    #[test]
    fn real_read_failure_aborts_injection() {
        // Arrange
        let mut stimulus = ThermalFaultStimulus::default();
        let prior = fresh(0);
        let first = stimulus
            .step(&prior, AcquisitionOutcome::Success(50.0))
            .expect("baseline starts injection");
        let fault = prior.record_fault(FaultReason::ThermalReadingInvalid);

        // Act
        let result = stimulus.step(&fault, AcquisitionOutcome::ReadFailed);

        // Assert
        assert_eq!(first.outcome, AcquisitionOutcome::InvalidSample);
        assert_eq!(
            result,
            Err(ThermalFaultStimulusFailure::RealReadFailedDuringInjection)
        );
    }

    #[test]
    fn missing_fault_projection_fails_closed() {
        // Arrange
        let mut stimulus = ThermalFaultStimulus::default();
        let prior = fresh(0);
        stimulus
            .step(&prior, AcquisitionOutcome::Success(50.0))
            .expect("baseline starts injection");

        // Act
        let result = stimulus.step(&prior, AcquisitionOutcome::Success(50.0));

        // Assert
        assert_eq!(
            result,
            Err(ThermalFaultStimulusFailure::FaultProjectionMissing)
        );
    }
}
