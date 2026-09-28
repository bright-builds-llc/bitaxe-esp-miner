use super::*;

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    // Keep signature verification off the larger general-command dispatch frame.
    #[inline(never)]
    pub(super) fn prepare_start_controller(
        &mut self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        if self.session.noise_busy() || self.session.v2_scope_busy() {
            return Err(WorkerControlError::InvalidTransition);
        }
        let mut result = self.start(request.required_payload()?, now)?;
        if request.includes_status_evidence() {
            result = self.with_status_evidence(result)?;
        }
        response(&request.request_id, result, None)
    }
}
