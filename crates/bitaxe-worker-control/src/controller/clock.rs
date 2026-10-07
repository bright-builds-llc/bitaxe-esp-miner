use super::*;
use crate::clock_stimulus::{ARMED_FOR_MILLISECONDS, OFFSET_MILLISECONDS};

/// The stimulated tick must land well inside the lease, never at its expiry edge.
const MINIMUM_LEASE_HEADROOM_MILLISECONDS: u64 = 5_000;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StimulusRequest {
    request_nonce: String,
}

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    /// Supervises the lease from the real uptime clock. Only this path may observe
    /// the single armed clock-reset stimulus; frames and confirmations never do.
    pub fn tick(&mut self, real_monotonic_milliseconds: u64) -> Result<(), WorkerControlError> {
        self.session.noise_poll();
        let now = self.clock_stimulus.sample(
            real_monotonic_milliseconds,
            self.maybe_last_monotonic_milliseconds,
        );
        self.enforce_clock(now)
    }

    /// Decreasing-clock detections since boot, real or stimulated.
    #[must_use]
    pub const fn clock_discontinuities_detected(&self) -> u32 {
        self.clock_discontinuities_detected
    }

    pub(super) fn enforce_clock(&mut self, now: u64) -> Result<(), WorkerControlError> {
        if let Some(reason) = self.maybe_cleanup_reason {
            self.safe_stop(reason, now)?;
        }
        if self
            .maybe_last_monotonic_milliseconds
            .is_some_and(|last| now < last)
        {
            self.clock_discontinuities_detected =
                self.clock_discontinuities_detected.saturating_add(1);
            self.safe_stop(RestorationReason::MonotonicReset, now)?;
            return Err(WorkerControlError::MonotonicReset);
        }
        self.maybe_last_monotonic_milliseconds = Some(now);
        if self
            .maybe_active
            .as_ref()
            .is_some_and(|active| now >= active.deadlines.expires_at_monotonic_milliseconds())
        {
            self.safe_stop(RestorationReason::LeaseExpired, now)?;
        }
        Ok(())
    }

    // A route target: its scratch must not inflate the shared routing frame.
    #[inline(never)]
    pub(super) fn prepare_clock_stimulus(
        &mut self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        let payload: StimulusRequest = request.required_payload()?;
        if !crate::codec::canonical_request_nonce(&payload.request_nonce) {
            return Err(WorkerControlError::InvalidRequest);
        }
        self.required_active_context()?;
        self.require_stimulus_window(now)?;
        if !self.clock_stimulus.can_prepare() {
            return Err(WorkerControlError::InvalidTransition);
        }
        self.next_response_token = self
            .next_response_token
            .checked_add(1)
            .ok_or(WorkerControlError::InvalidTransition)?;
        let token = self.next_response_token;
        self.clock_stimulus.prepare(token);
        response(
            &request.request_id,
            serde_json::json!({
                "schema": "worker-clock-discontinuity-stimulus-v1",
                "requestNonce": payload.request_nonce,
                "offsetMilliseconds": OFFSET_MILLISECONDS,
                "armedForMilliseconds": ARMED_FOR_MILLISECONDS,
            }),
            Some(PreparedEffect::ClockStimulus {
                generation: self.generation,
                token,
            }),
        )
    }

    /// Arms only after the acknowledgement was truly delivered, rechecking the window.
    pub(super) fn confirm_clock_stimulus(
        &mut self,
        generation: u64,
        token: u64,
        now: u64,
    ) -> Result<(), WorkerControlError> {
        if self.clock_stimulus.take_pending() != Some(token) || generation != self.generation {
            return Err(WorkerControlError::StaleResponse);
        }
        self.enforce_clock(now)?;
        self.required_active_context()?;
        self.require_stimulus_window(now)?;
        if !self.clock_stimulus.arm(now) {
            return Err(WorkerControlError::StaleResponse);
        }
        Ok(())
    }

    fn require_stimulus_window(&self, now: u64) -> Result<(), WorkerControlError> {
        let Some(active) = self.maybe_active.as_ref() else {
            return Err(WorkerControlError::InvalidTransition);
        };
        let headroom = active
            .deadlines
            .expires_at_monotonic_milliseconds()
            .saturating_sub(now);
        if active.grant.maybe_v2().is_some()
            || self.maybe_cleanup_reason.is_some()
            || self.session.noise_busy()
            || self.session.v2_scope_busy()
            || headroom < MINIMUM_LEASE_HEADROOM_MILLISECONDS
            || self
                .maybe_last_monotonic_milliseconds
                .is_none_or(|last| last < OFFSET_MILLISECONDS)
        {
            return Err(WorkerControlError::InvalidTransition);
        }
        Ok(())
    }

    // Read-only and off the routing frame like the other reviews.
    #[inline(never)]
    pub(super) fn review_clock_stimulus(
        &self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<Value, WorkerControlError> {
        request.require_empty_payload()?;
        self.require_review_idle(now)?;
        Ok(serde_json::json!({
            "schema": "worker-clock-discontinuity-stimulus-review-v1",
            "state": self.clock_stimulus.state_label(),
            "offsetMilliseconds": OFFSET_MILLISECONDS,
            "discontinuitiesDetected": self.clock_discontinuities_detected,
        }))
    }
}
