//! Fan-only shell called exclusively by the sole production owner inbox.
use super::{
    cooling_core::{self, CoolingBackend, CoolingError, FanCommand, FanReply},
    revocation,
};
use crate::safety_adapter::{
    FanDutyPercent, PendingSafetyActuation, SafetyActuationCommand, SafetyActuationPollOutcome,
    SafetyActuationQueueOutcome,
};
use bitaxe_api::TelemetryObservations;

struct Backend {
    restoring: bool,
    generation: revocation::WorkerGeneration,
    maybe_pending: Option<PendingSafetyActuation>,
    applied: bool,
}
impl CoolingBackend for Backend {
    fn now_ms(&self) -> u64 {
        crate::runtime_uptime::millis()
    }
    fn admitted(&self) -> bool {
        revocation::check_deadline(self.now_ms());
        (revocation::is_live(self.generation)
            || (self.restoring && revocation::maybe_revoked() == Some(self.generation)))
            && !revocation::permits(Some(self.generation))
    }
    fn observations(&self) -> TelemetryObservations {
        crate::safety_adapter::observation_snapshot()
    }
    fn command(&mut self, command: FanCommand) -> Result<(), CoolingError> {
        if !self.admitted() {
            return Err(CoolingError::Rejected);
        }
        let command = match command {
            FanCommand::Full => SafetyActuationCommand::SetFanDuty(FanDutyPercent::FULL),
            FanCommand::RestoreBaseline => SafetyActuationCommand::RestoreCoolingBaseline {
                generation: self.generation,
            },
        };
        self.applied = false;
        self.maybe_pending = Some(
            match crate::safety_adapter::queue_safety_actuation(command) {
                SafetyActuationQueueOutcome::Queued(pending) => pending,
                _ => return Err(CoolingError::WriteFailed),
            },
        );
        Ok(())
    }
    fn poll(&mut self) -> Result<FanReply, CoolingError> {
        if self.applied {
            return Ok(FanReply::Applied);
        }
        let pending = self
            .maybe_pending
            .as_ref()
            .ok_or(CoolingError::WriteFailed)?;
        match pending.poll() {
            SafetyActuationPollOutcome::Pending => Ok(FanReply::Pending),
            SafetyActuationPollOutcome::Applied => {
                self.applied = true;
                self.maybe_pending = None;
                Ok(FanReply::Applied)
            }
            _ => Err(CoolingError::WriteFailed),
        }
    }
    fn wait(&mut self) {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
fn backend(generation: revocation::WorkerGeneration, restoring: bool) -> Backend {
    Backend {
        restoring,
        generation,
        maybe_pending: None,
        applied: false,
    }
}
pub(super) fn qualify(
    generation: revocation::WorkerGeneration,
) -> Result<serde_json::Value, CoolingError> {
    if !revocation::is_live(generation) {
        return Err(CoolingError::Rejected);
    }
    let rpm = cooling_core::qualify(&mut backend(generation, false))?;
    Ok(
        serde_json::json!({"schema":"worker-cooling-proof-v1","fan_duty_percent":100,"fan_rpm":rpm,"post_command_fan_proven":true,"asic_effects":false,"budget_reserved":false}),
    )
}
pub(super) fn restore(
    generation: revocation::WorkerGeneration,
) -> Result<serde_json::Value, CoolingError> {
    cooling_core::restore(&mut backend(generation, true))?;
    Ok(
        serde_json::json!({"schema":"worker-cooling-baseline-v1","fan_duty_percent":30,"cooling_proven":true,"asic_effects":false,"budget_reserved":false}),
    )
}
