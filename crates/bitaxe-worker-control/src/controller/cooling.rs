use super::{
    ControllerRequest, LeaseAuthorizationVerifier, RestorationReason, RestorationState, Value,
    WorkerControl, WorkerControlError, WorkerSession,
};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CoolingRequest {
    action: CoolingAction,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum CoolingAction {
    ProveFan,
    RestoreBaseline,
}

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    pub(super) fn qualify_cooling(
        &mut self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<Value, WorkerControlError> {
        let input: CoolingRequest = request.required_payload()?;
        if self.maybe_active.is_some() || self.maybe_cleanup_reason.is_some() {
            return Err(WorkerControlError::InvalidTransition);
        }
        match input.action {
            CoolingAction::ProveFan => {
                self.required_start_context(now)?;
                if self.effect_cleanup_required || self.boot_restoration_clear_required {
                    return Err(WorkerControlError::InvalidTransition);
                }
                self.verifier
                    .mark_effect_pending()
                    .map_err(|_| WorkerControlError::PersistenceFailed)?;
                self.effect_cleanup_required = true;
                self.restoration = RestorationState::Pending;
                match self.session.qualify_cooling() {
                    Ok(proof) => Ok(proof),
                    Err(_) => {
                        self.safe_stop(RestorationReason::ControlFailed, now)?;
                        Err(WorkerControlError::SessionFailed)
                    }
                }
            }
            CoolingAction::RestoreBaseline => {
                // Cleanup retains the original authenticated context even if its
                // Start admission has expired. It can never authorize new work.
                self.required_active_context()?;
                if !self.effect_cleanup_required {
                    return Err(WorkerControlError::InvalidTransition);
                }
                let proof = match self.session.restore_cooling() {
                    Ok(proof) => proof,
                    Err(_) => {
                        self.safe_stop(RestorationReason::ControlFailed, now)?;
                        return Err(WorkerControlError::SessionFailed);
                    }
                };
                self.maybe_admission = None;
                self.maybe_pending_admission_token = None;
                self.maybe_cleanup_reason = Some(RestorationReason::Cancelled);
                self.verifier
                    .clear_effect_pending()
                    .map_err(|_| WorkerControlError::PersistenceFailed)?;
                self.effect_cleanup_required = false;
                self.maybe_cleanup_reason = None;
                self.restoration = RestorationState::Confirmed(RestorationReason::Cancelled);
                self.maybe_last_monotonic_milliseconds = Some(now);
                Ok(proof)
            }
        }
    }
}
