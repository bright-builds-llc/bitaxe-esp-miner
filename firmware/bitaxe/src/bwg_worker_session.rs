//! Thin BWG port into the sole boot-lifetime Production Mining Session owner.

use bitaxe_worker_control::{
    LeaseDeadlines, RestorationReason, WorkerLeaseGrant, WorkerLeaseRenewal, WorkerSession,
    WorkerSessionError,
};

#[derive(Default)]
pub(crate) struct ProductionWorkerSession {
    maybe_generation: Option<crate::production_mining_session::revocation::WorkerGeneration>,
}

impl ProductionWorkerSession {
    pub(crate) fn set_generation(
        &mut self,
        generation: crate::production_mining_session::revocation::WorkerGeneration,
    ) {
        self.maybe_generation = Some(generation);
    }
}

impl WorkerSession for ProductionWorkerSession {
    fn settings_preservation(
        &self,
    ) -> Result<Option<bitaxe_worker_control::SettingsPreservation>, WorkerSessionError> {
        crate::settings_adapter::preservation::read()
            .map(Some)
            .map_err(|_| WorkerSessionError::Rejected)
    }

    fn status_evidence(&self) -> Option<serde_json::Value> {
        crate::production_mining_session::status_evidence(self.maybe_generation)
    }

    fn start(
        &mut self,
        grant: &WorkerLeaseGrant,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        let generation = self.maybe_generation.ok_or(WorkerSessionError::Rejected)?;
        crate::worker_acceptance_budget::admit(generation, grant)
            .map_err(|_| WorkerSessionError::Rejected)?;
        crate::production_mining_session::bwg_start(grant, deadlines, generation)
            .map_err(|_| WorkerSessionError::Rejected)
    }

    fn renew(
        &mut self,
        renewal: &WorkerLeaseRenewal,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        let generation = self.maybe_generation.ok_or(WorkerSessionError::Rejected)?;
        crate::production_mining_session::bwg_renew(renewal, deadlines, generation)
            .map_err(|_| WorkerSessionError::Rejected)
    }

    fn safe_stop(&mut self, reason: RestorationReason) -> Result<(), WorkerSessionError> {
        if let Some(generation) = self.maybe_generation {
            use crate::production_mining_session::revocation::{self, RevocationReason};
            let cause = match reason {
                RestorationReason::LeaseExpired => RevocationReason::LeaseOrBudgetExpired,
                RestorationReason::ConnectivityLost => RevocationReason::LinkClosed,
                RestorationReason::ControlFailed | RestorationReason::MonotonicReset => {
                    RevocationReason::ControlFailed
                }
                _ => RevocationReason::RestorationRequested,
            };
            revocation::revoke_reason_at(generation, crate::runtime_uptime::millis(), cause);
        }
        crate::production_mining_session::bwg_safe_stop()
            .map_err(|_| WorkerSessionError::SafeStopFailed)
    }
}
