//! Closed normal-path preparation evidence instrumentation.
use super::*;
impl Ultra205MiningActuationAdapter {
    pub(super) fn record_preparation_started(&self, step: PreparationStep) {
        self.record_preparation_progress(
            step,
            bitaxe_api::preparation_receipt::Outcome::Started,
            bitaxe_api::preparation_receipt::Failure::None,
        );
    }

    pub(super) fn record_preparation_result(
        &self,
        step: PreparationStep,
        result: &Result<(), MiningActuationAdapterError>,
    ) {
        use bitaxe_api::preparation_receipt::{Failure, Outcome};
        let (outcome, failure) = match result {
            Ok(()) => (Outcome::Completed, Failure::None),
            Err(error) => (Outcome::Failed, Self::preparation_failure(*error)),
        };
        self.record_preparation_progress(step, outcome, failure);
    }

    fn record_preparation_progress(
        &self,
        step: PreparationStep,
        outcome: bitaxe_api::preparation_receipt::Outcome,
        failure: bitaxe_api::preparation_receipt::Failure,
    ) {
        let Some(generation) = self.maybe_worker_generation else {
            return;
        };
        let number = match step {
            PreparationStep::RequireFreshSafetyObservations => 1,
            PreparationStep::SetFanDutyTo100Percent => 2,
            PreparationStep::RequireFreshNonzeroFanRpm => 3,
            PreparationStep::SetCoreVoltage(_) => 4,
            PreparationStep::WaitForCoreVoltageStabilization500Ms => 5,
            PreparationStep::EnableAsic => 6,
            PreparationStep::ResetAndDetectExactlyOneChip => 7,
            PreparationStep::InitializeMiningReadyWithFrequencyRamp(_) => 8,
            PreparationStep::RetainProductionUart => 9,
        };
        let position = crate::preparation_evidence::current_position();
        // Commit the boundary before optional runtime queries, which can fail
        // independently. The second receipt enriches the same boundary only.
        crate::preparation_evidence::record(generation.raw(), number, outcome, failure, position);
        let resources = crate::preparation_evidence::observe_resources(position);
        crate::preparation_evidence::record(generation.raw(), number, outcome, failure, resources);
    }

    fn preparation_failure(
        error: MiningActuationAdapterError,
    ) -> bitaxe_api::preparation_receipt::Failure {
        use bitaxe_api::preparation_receipt::Failure;
        match error {
            MiningActuationAdapterError::WorkerGenerationRevoked => Failure::Cancelled,
            MiningActuationAdapterError::SafetyObservationsUnavailable => {
                Failure::SafetyUnavailable
            }
            MiningActuationAdapterError::SafetyOwnerUnavailable => Failure::OwnerUnavailable,
            MiningActuationAdapterError::SafetyQueueFull => Failure::QueueFull,
            MiningActuationAdapterError::SafetyReplyTimedOut => Failure::ReplyTimeout,
            MiningActuationAdapterError::SafetyHardwareWriteFailed => Failure::HardwareWriteFailed,
            MiningActuationAdapterError::FanRpmProofTimedOut => Failure::FanTimeout,
            MiningActuationAdapterError::UnsupportedProfile => Failure::UnsupportedProfile,
            MiningActuationAdapterError::Asic(_) => Failure::AsicFailed,
            MiningActuationAdapterError::AsicPlanInvalid => Failure::AsicPlanInvalid,
            MiningActuationAdapterError::CoolingProofTimedOut => Failure::CoolingTimeout,
            MiningActuationAdapterError::CoolingProofRequired => Failure::CoolingProofRequired,
        }
    }
}
