//! Thin BWG port into the sole boot-lifetime Production Mining Session owner.

use bitaxe_worker_control::{
    LeaseDeadlines, RestorationReason, WorkerLeaseGrant, WorkerLeaseRenewal, WorkerSession,
    WorkerSessionError,
};

pub(crate) struct ProductionWorkerSession;

impl WorkerSession for ProductionWorkerSession {
    fn start(
        &mut self,
        grant: &WorkerLeaseGrant,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        crate::production_mining_session::bwg_start(grant, deadlines)
            .map_err(|_| WorkerSessionError::Rejected)
    }

    fn renew(
        &mut self,
        renewal: &WorkerLeaseRenewal,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        crate::production_mining_session::bwg_renew(renewal, deadlines)
            .map_err(|_| WorkerSessionError::Rejected)
    }

    fn safe_stop(&mut self, _reason: RestorationReason) -> Result<(), WorkerSessionError> {
        crate::production_mining_session::bwg_safe_stop()
            .map_err(|_| WorkerSessionError::SafeStopFailed)
    }
}
