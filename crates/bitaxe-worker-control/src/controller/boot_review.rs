use super::*;

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    // Read-only and off the routing frame like the other reviews.
    #[inline(never)]
    pub(super) fn review_boot(
        &self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<Value, WorkerControlError> {
        request.require_empty_payload()?;
        self.require_review_idle(now)?;
        Ok(serde_json::json!({
            "schema": "worker-boot-review-v1",
            "resetCause": self.session.boot_reset_cause().label(),
        }))
    }
}
