//! Thin BWG port into the sole boot-lifetime Production Mining Session owner.

use crate::production_mining_session::admission_diagnostics::{self, Failure, Stage};
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
    fn qualification_restart_context(
        &self,
    ) -> Result<Option<bitaxe_worker_control::QualificationRestartContext>, WorkerSessionError>
    {
        let generation = self.maybe_generation.ok_or(WorkerSessionError::Rejected)?;
        crate::qualification_restart::context(generation)
    }

    fn qualification_restart(
        &mut self,
        context: bitaxe_worker_control::QualificationRestartContext,
        expires_at_ms: u64,
    ) -> Result<(), WorkerSessionError> {
        let generation = self.maybe_generation.ok_or(WorkerSessionError::Rejected)?;
        crate::qualification_restart::restart(generation, context, expires_at_ms)
    }

    fn qualify_cooling(&mut self) -> Result<serde_json::Value, WorkerSessionError> {
        let generation = self.maybe_generation.ok_or(WorkerSessionError::Rejected)?;
        crate::production_mining_session::bwg_cooling(generation, false)
            .map_err(|_| WorkerSessionError::Rejected)
    }

    fn restore_cooling(&mut self) -> Result<serde_json::Value, WorkerSessionError> {
        let generation = self.maybe_generation.ok_or(WorkerSessionError::Rejected)?;
        crate::production_mining_session::bwg_cooling(generation, true)
            .map_err(|_| WorkerSessionError::SafeStopFailed)
    }

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

    fn serial_trace_review(
        &self,
    ) -> Result<Option<bitaxe_worker_control::serial::trace::SerialTraceSnapshot>, WorkerSessionError>
    {
        Ok(Some(crate::bwg_worker_usb::trace::snapshot()))
    }

    fn telemetry_cadence_probe_prepared(&self, request_bytes: usize, response_bytes: usize) {
        crate::telemetry_cadence::RECORDER.max_probe_prepared(
            request_bytes,
            response_bytes,
            crate::telemetry_cadence::now_us(),
        );
    }

    fn telemetry_cadence_arm(
        &mut self,
        phase: bitaxe_worker_control::cadence::CadencePhase,
    ) -> Result<Option<bitaxe_worker_control::cadence::CadenceArmReceipt>, WorkerSessionError> {
        let generation = self.maybe_generation.ok_or(WorkerSessionError::Rejected)?;
        Ok(crate::telemetry_cadence::RECORDER.maybe_arm(
            phase,
            crate::telemetry_cadence::now_us(),
            generation.raw(),
        ))
    }

    fn telemetry_cadence_review(
        &self,
    ) -> Result<Option<bitaxe_worker_control::cadence::CadenceSnapshot>, WorkerSessionError> {
        Ok(Some(crate::telemetry_cadence::snapshot()))
    }

    fn telemetry_cadence_endpoint(
        &self,
    ) -> Result<Option<bitaxe_worker_control::cadence::CadenceEndpoint>, WorkerSessionError> {
        let generation = self.maybe_generation.ok_or(WorkerSessionError::Rejected)?;
        Ok(crate::telemetry_cadence::maybe_endpoint(generation.raw()))
    }

    fn qualification_attempt_review(
        &self,
    ) -> Result<Option<serde_json::Value>, WorkerSessionError> {
        crate::worker_qualification_budget::review()
            .map(Some)
            .map_err(|_| WorkerSessionError::Rejected)
    }

    fn acceptance_budget_review(
        &self,
        expected_campaign: &str,
    ) -> Result<Option<serde_json::Value>, WorkerSessionError> {
        crate::worker_acceptance_budget::review(expected_campaign)
            .map(Some)
            .map_err(|_| WorkerSessionError::Rejected)
    }

    fn start(
        &mut self,
        grant: &WorkerLeaseGrant,
        deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        admission_diagnostics::begin();
        let generation = self.maybe_generation.ok_or_else(|| {
            admission_diagnostics::fail(Failure::Admission);
            WorkerSessionError::Rejected
        })?;
        crate::worker_acceptance_budget::admit(generation, grant).map_err(|_| {
            admission_diagnostics::fail(Failure::Admission);
            WorkerSessionError::Rejected
        })?;
        crate::production_mining_session::bwg_start(grant, deadlines, generation).map_err(|_| {
            admission_diagnostics::fail(Failure::Admission);
            WorkerSessionError::Rejected
        })
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
        admission_diagnostics::stage(Stage::Cleanup);
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
        crate::production_mining_session::bwg_safe_stop().map_err(|_| {
            admission_diagnostics::fail(Failure::Cleanup);
            WorkerSessionError::SafeStopFailed
        })?;
        if let Some(generation) = self.maybe_generation {
            // The owner acknowledged termination (or absence) of its effects.
            // A pre-owner rejection still owns a durable reservation to finalize.
            crate::worker_acceptance_budget::finish(generation).map_err(|_| {
                admission_diagnostics::fail(Failure::Cleanup);
                WorkerSessionError::SafeStopFailed
            })?;
            crate::production_mining_session::revocation::finish_shutdown(generation);
        }
        admission_diagnostics::stage(Stage::Complete);
        Ok(())
    }
}
