use super::*;

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    // Signature verification must not retain the large general-command frame.
    #[inline(never)]
    pub(super) fn prepare_renew_controller(
        &mut self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        let mut result = self.renew(request.required_payload()?, now)?;
        if request.includes_status_evidence() {
            result = self.with_status_evidence(result)?;
        }
        response(&request.request_id, result, None)
    }
}
