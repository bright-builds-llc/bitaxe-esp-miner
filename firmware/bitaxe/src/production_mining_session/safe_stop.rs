//! The hardware safe-stop, with a terminal stop's cooling proof polled from the owner loop.
//!
//! Power is off before the proof starts. While the board cools, the loop keeps serving its
//! inbox, readiness and hashrate monitor, and the session keeps the terminal snapshot held
//! until the stop completes (ADR-0016).

use super::*;
use crate::mining_actuation::{
    PendingCoolingProof, SafeShutdownFailure, SafeShutdownStep, SafeStopProgress,
};
use crate::mining_actuation_adapter::MiningActuationAdapterError;
use bitaxe_stratum::v1::production_session::{HardwareSafeStopPurpose, MiningCampaignLeaseId};

/// A terminal stop whose power is off and whose cooling proof is still open.
pub(super) struct PendingSafeStop {
    lease_id: MiningCampaignLeaseId,
    proof: PendingCoolingProof<MiningActuationAdapterError>,
}

impl OrdinaryEspProductionSessionAdapter {
    /// Starts the stop: its event when it completes at once (timed at the effect's `now_ms`,
    /// as before), else `None` with the proof pending.
    pub(super) fn begin_hardware_safe_stop(
        &mut self,
        lease_id: MiningCampaignLeaseId,
        purpose: HardwareSafeStopPurpose,
        now_ms: u64,
        progress: &mut dyn FnMut(SafeShutdownStep),
    ) -> Option<ProductionSessionEvent> {
        match self.mining_actuation.begin_safe_stop(purpose, progress) {
            SafeStopProgress::Complete(result) => {
                Some(safe_stop_finished(lease_id, result, now_ms))
            }
            SafeStopProgress::CoolingPending(proof) => {
                self.maybe_pending_safe_stop = Some(PendingSafeStop { lease_id, proof });
                None
            }
        }
    }

    /// Whether a terminal stop is still cooling; the fan must stay at 100 % until it ends.
    pub(super) fn safe_stop_pending(&self) -> bool {
        self.maybe_pending_safe_stop.is_some()
    }

    /// Checks the open cooling proof once: the stop's event, timed at completion, when it ends.
    pub(super) fn poll_pending_safe_stop(
        &mut self,
        progress: &mut dyn FnMut(SafeShutdownStep),
    ) -> Option<ProductionSessionEvent> {
        let PendingSafeStop { lease_id, proof } = self.maybe_pending_safe_stop.take()?;
        match self.mining_actuation.poll_safe_stop(proof, progress) {
            SafeStopProgress::Complete(result) => Some(safe_stop_finished(
                lease_id,
                result,
                crate::runtime_uptime::millis(),
            )),
            SafeStopProgress::CoolingPending(proof) => {
                self.maybe_pending_safe_stop = Some(PendingSafeStop { lease_id, proof });
                None
            }
        }
    }
}

/// The stop's outcome for the session.
fn safe_stop_finished(
    lease_id: MiningCampaignLeaseId,
    result: Result<(), SafeShutdownFailure<MiningActuationAdapterError>>,
    now_ms: u64,
) -> ProductionSessionEvent {
    let Err(failure) = result else {
        return ProductionSessionEvent::HardwareSafeStopConfirmed { lease_id, now_ms };
    };
    admission_diagnostics::fail(admission_diagnostics::Failure::Cleanup);
    log::error!(
        "production_mining_session=fail_closed action=hardware_safe_stop failed_step={:?} category={}",
        failure.step(),
        failure.source().category(),
    );
    ProductionSessionEvent::EffectFailed {
        maybe_pool: None,
        reason: bitaxe_stratum::v1::production_session::HARDWARE_SAFE_STOP_FAILED,
        now_ms,
    }
}
