//! The built board's part of the self-test (ADR-0041 item 14): the authority its regulator runs
//! under, how it commands the fan, the first diagnostic job id and the regulator's facts.

use bitaxe_api::TelemetryObservations;
use bitaxe_asic::asic::JobId;
use bitaxe_safety::self_test::HardwareSelfTestFailure;

use crate::mining_actuation_adapter::{
    MiningActuationAdapterError, Ultra205MiningActuationAdapter,
};

/// The regulator facts one sample adds to the measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RegulatorFacts {
    pub(super) maybe_vr_temperature_c: Option<f32>,
    pub(super) fault: bool,
}

#[cfg(not(feature = "board-gamma-601"))]
mod built {
    use super::*;

    /// The 205's DS4432U and enable need no generation: its verified self-test runs as it did.
    pub(in super::super) struct BoardAuthority;

    impl BoardAuthority {
        pub(in super::super) fn begin(
            _actuation: &mut Ultra205MiningActuationAdapter,
        ) -> Result<Self, HardwareSelfTestFailure> {
            Ok(Self)
        }

        pub(in super::super) fn set_fan_duty(
            &self,
            percent: u8,
        ) -> Result<(), MiningActuationAdapterError> {
            Ultra205MiningActuationAdapter::set_self_test_fan_duty(percent)
        }

        pub(in super::super) fn end(self) {}
    }

    pub(in super::super) const fn first_job_id() -> JobId {
        JobId::new(0)
    }

    pub(in super::super) fn regulator_facts(
        _observations: &TelemetryObservations,
    ) -> RegulatorFacts {
        RegulatorFacts {
            maybe_vr_temperature_c: None,
            fault: false,
        }
    }
}

#[cfg(feature = "board-gamma-601")]
mod built {
    use super::*;
    use crate::production_mining_session::revocation::{
        self, FanControllerOwner, WorkerGeneration,
    };
    use crate::safety_adapter::{
        FanDutyPercent, SafetyActuationCommand, SafetyActuationRequestOutcome,
    };
    use bitaxe_safety::observation::Observation;

    /// The longest the regulator may stay on for one self-test: the gate's 240 s budget
    /// ceiling. It starts after the preflight, so it covers the preparation, the 180 s warm-up
    /// and the 30 s measurement.
    const OUTPUT_ON_BOUND_MS: u64 = 240_000;
    /// The preparation's allowance: the TPS546 on and verified, chip detection, the BM1370
    /// init and ramp, and the fan proof (about 25 s on hardware).
    const PREPARATION_ALLOWANCE_MS: u64 = 30_000;
    const _: () = assert!(
        PREPARATION_ALLOWANCE_MS
            + bitaxe_safety::self_test::HARDWARE_SELF_TEST_WARMUP_TIMEOUT_MS
            + bitaxe_safety::self_test::HARDWARE_SELF_TEST_MEASUREMENT_MS
            <= OUTPUT_ON_BOUND_MS
    );

    /// The 601's TPS546 turns on only under a generation's permit (ADR-0041 item 10), so the
    /// self-test runs as a local generation with a budget armed at once. The gate revokes it at
    /// the bound or on an unsafe observation, and the regulator's owner then turns the output off
    /// within 100 ms, whatever the self-test thread is doing.
    pub(in super::super) struct BoardAuthority {
        generation: WorkerGeneration,
    }

    impl BoardAuthority {
        pub(in super::super) fn begin(
            actuation: &mut Ultra205MiningActuationAdapter,
        ) -> Result<Self, HardwareSelfTestFailure> {
            let now_ms = crate::runtime_uptime::millis();
            let generation =
                revocation::begin_local().ok_or(HardwareSelfTestFailure::SafetyUnavailable)?;
            let admitted = revocation::admit_budget(generation, OUTPUT_ON_BOUND_MS)
                && revocation::activate(generation, now_ms)
                && revocation::begin_dispatch(revocation::stamp(Some(generation)), now_ms);
            if !admitted {
                revocation::revoke_at(generation, now_ms);
                revocation::finish_shutdown(generation);
                return Err(HardwareSelfTestFailure::SafetyUnavailable);
            }
            actuation.set_worker_generation(Some(generation));
            log::info!(
                "self_test_authority=granted generation={} bound_ms={OUTPUT_ON_BOUND_MS}",
                generation.raw()
            );
            Ok(Self { generation })
        }

        /// Fan duties go as the local generation's controller commands; the idle-only
        /// `SetFanDuty` path refuses below 100 % while a generation holds the gate.
        pub(in super::super) fn set_fan_duty(
            &self,
            percent: u8,
        ) -> Result<(), MiningActuationAdapterError> {
            let duty = FanDutyPercent::try_from(percent)
                .map_err(|_| MiningActuationAdapterError::UnsupportedProfile)?;
            match crate::safety_adapter::request_safety_actuation(
                SafetyActuationCommand::SetControllerFanDuty {
                    duty,
                    owner: FanControllerOwner::Local(self.generation),
                },
            ) {
                SafetyActuationRequestOutcome::Applied => Ok(()),
                SafetyActuationRequestOutcome::QueueFull => {
                    Err(MiningActuationAdapterError::SafetyQueueFull)
                }
                SafetyActuationRequestOutcome::OwnerUnavailable => {
                    Err(MiningActuationAdapterError::SafetyOwnerUnavailable)
                }
                SafetyActuationRequestOutcome::ReplyTimedOut => {
                    Err(MiningActuationAdapterError::SafetyReplyTimedOut)
                }
                SafetyActuationRequestOutcome::HardwareWriteFailed => {
                    Err(MiningActuationAdapterError::SafetyHardwareWriteFailed)
                }
                SafetyActuationRequestOutcome::Refused => {
                    Err(MiningActuationAdapterError::WorkerGenerationRevoked)
                }
            }
        }

        /// Retires the generation after the safe stop: nothing may power the regulator for it.
        pub(in super::super) fn end(self) {
            let now_ms = crate::runtime_uptime::millis();
            revocation::revoke_at(self.generation, now_ms);
            revocation::finish_shutdown(self.generation);
        }
    }

    pub(in super::super) const fn first_job_id() -> JobId {
        crate::board_identity::ASIC_FAMILY.first_job_id()
    }

    /// The TPS546's temperature, and whether its power fact is in fault: the 601's producer
    /// faults the power fact on a TPS546 power fault (`gamma601_sensors::power_outcome`), as
    /// upstream's `test_vreg_faults` reads `VCORE_check_fault`.
    pub(in super::super) fn regulator_facts(
        observations: &TelemetryObservations,
    ) -> RegulatorFacts {
        let maybe_vr_temperature_c = if observations.vr_temp_celsius.is_fresh() {
            observations
                .vr_temp_celsius
                .maybe_last_good()
                .map(|sample| *sample.value() as f32)
                .filter(|value| value.is_finite())
        } else {
            None
        };
        RegulatorFacts {
            maybe_vr_temperature_c,
            fault: matches!(observations.power_watts, Observation::Fault { .. }),
        }
    }
}

pub(super) use built::{first_job_id, regulator_facts, BoardAuthority};
