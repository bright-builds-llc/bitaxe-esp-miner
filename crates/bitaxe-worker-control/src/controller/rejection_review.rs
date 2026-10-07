use super::*;
use crate::{
    AuthorizationOperation, AuthorizationRejectionLog, AuthorizationRejectionSource,
    ContextAttribution, LeaseAuthorizationError, StateFingerprint,
};

/// Read-only projection of the verifier: no signing, proof, or durable write is reachable.
struct VerifierRejectionView<'a, V>(&'a V);

impl<V: LeaseAuthorizationVerifier> AuthorizationRejectionSource for VerifierRejectionView<'_, V> {
    fn rejection_log(&self) -> Option<&AuthorizationRejectionLog> {
        self.0.authorization_rejections()
    }

    fn high_water_fingerprint(&self) -> Result<Option<StateFingerprint>, LeaseAuthorizationError> {
        self.0.authorization_high_water_fingerprint()
    }
}

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    /// Attributes a Start or Renew refused before signature verification.
    #[cold]
    #[inline(never)]
    pub(super) fn attribute_context_rejection(&mut self, operation: AuthorizationOperation) {
        let context = if self
            .maybe_admission
            .as_ref()
            .is_some_and(|admission| admission.generation == self.generation)
        {
            ContextAttribution::Expired
        } else {
            ContextAttribution::Absent
        };
        self.verifier.record_context_rejection(operation, context);
    }

    /// Reviews need fresh possession, no lease and no pending cleanup or admission.
    pub(super) fn require_review_idle(&self, now: u64) -> Result<(), WorkerControlError> {
        self.required_start_context(now)?;
        if self.maybe_active.is_some()
            || self.effect_cleanup_required
            || self.maybe_cleanup_reason.is_some()
            || self.maybe_pending_admission_token.is_some()
            || matches!(self.restoration, RestorationState::Pending)
        {
            return Err(WorkerControlError::InvalidTransition);
        }
        Ok(())
    }

    // Read-only and off the routing frame like the other reviews.
    #[inline(never)]
    pub(super) fn review_authorization_rejections(
        &self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<Value, WorkerControlError> {
        request.require_empty_payload()?;
        self.require_review_idle(now)?;
        let review = crate::authorization_rejection_review(&VerifierRejectionView(&self.verifier))
            .map_err(|_| WorkerControlError::PersistenceFailed)?
            .ok_or(WorkerControlError::InvalidRequest)?;
        serde_json::to_value(review).map_err(|_| WorkerControlError::Encoding)
    }
}
