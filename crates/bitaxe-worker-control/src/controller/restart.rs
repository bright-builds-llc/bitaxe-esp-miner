use super::*;
use base64::Engine;

const MAX_SAFE_BOOT_ORDINAL: u64 = 9_007_199_254_740_990;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RestartRequest {
    request_nonce: String,
    expected_boot_ordinal: u64,
}

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    // A route target: its scratch must not inflate the shared routing frame.
    #[inline(never)]
    pub(super) fn prepare_qualification_restart(
        &mut self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        let payload: RestartRequest = request.required_payload()?;
        if payload.request_nonce.len() != 22 {
            return Err(WorkerControlError::InvalidRequest);
        }
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&payload.request_nonce)
            .map_err(|_| WorkerControlError::InvalidRequest)?;
        if decoded.len() != 16
            || base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&decoded)
                != payload.request_nonce
            || payload.expected_boot_ordinal == 0
            || payload.expected_boot_ordinal > MAX_SAFE_BOOT_ORDINAL
        {
            return Err(WorkerControlError::InvalidRequest);
        }
        self.required_start_context(now)?;
        self.require_restart_idle()?;
        if self.restart_consumed {
            return Err(WorkerControlError::InvalidTransition);
        }
        let kind = if request.command == "qualification_core_dump_self_test" {
            crate::QualificationResetKind::CoreDumpSelfTest
        } else {
            crate::QualificationResetKind::Restart
        };
        let context = self
            .reset_context(kind)
            .map_err(|_| WorkerControlError::SessionFailed)?
            .ok_or(WorkerControlError::InvalidTransition)?;
        if context.boot_ordinal != payload.expected_boot_ordinal
            || context.worker_generation == 0
            || context.transport_epoch == 0
        {
            return Err(WorkerControlError::InvalidTransition);
        }
        let expires_at_ms = self
            .maybe_admission
            .as_ref()
            .ok_or(WorkerControlError::AdmissionRequired)?
            .established_at_monotonic_milliseconds
            .checked_add(60_000)
            .ok_or(WorkerControlError::InvalidTransition)?;
        self.next_response_token = self
            .next_response_token
            .checked_add(1)
            .ok_or(WorkerControlError::InvalidTransition)?;
        self.restart_consumed = true;
        self.maybe_restart_token = Some(self.next_response_token);
        response(
            &request.request_id,
            serde_json::json!({
                "schema": match kind {
                    crate::QualificationResetKind::Restart => "worker-qualification-restart-v1",
                    crate::QualificationResetKind::CoreDumpSelfTest => "worker-qualification-core-dump-self-test-v1",
                }, "requestNonce":payload.request_nonce,
                "bootOrdinal": context.boot_ordinal, "nextBootOrdinal": context.boot_ordinal + 1
            }),
            Some(PreparedEffect::QualificationRestart {
                kind,
                generation: self.generation,
                token: self.next_response_token,
                context,
                expires_at_ms,
            }),
        )
    }

    fn reset_context(
        &self,
        kind: crate::QualificationResetKind,
    ) -> Result<Option<crate::QualificationRestartContext>, crate::WorkerSessionError> {
        match kind {
            crate::QualificationResetKind::Restart => self.session.qualification_restart_context(),
            crate::QualificationResetKind::CoreDumpSelfTest => {
                self.session.core_dump_self_test_context()
            }
        }
    }

    fn require_restart_idle(&self) -> Result<(), WorkerControlError> {
        if self.session.v2_scope_busy()
            || self.maybe_active.is_some()
            || self.maybe_pending_admission_token.is_some()
            || self.effect_cleanup_required
            || self.boot_restoration_clear_required
            || matches!(self.restoration, RestorationState::Pending)
        {
            return Err(WorkerControlError::InvalidTransition);
        }
        Ok(())
    }

    /// Confirms a reply using a newly observed clock. The native shell must also check its epoch.
    pub fn confirm_sent_at(
        &mut self,
        mut response: PreparedResponse,
        now: u64,
    ) -> Result<(), WorkerControlError> {
        if matches!(
            response.maybe_effect.as_ref(),
            Some(PreparedEffect::V2Dispatch { .. })
        ) {
            return self.confirm_v2_dispatch(response, now);
        }
        if matches!(
            response.maybe_effect.as_ref(),
            Some(PreparedEffect::NoiseDispatch { .. })
        ) {
            return self.confirm_noise_dispatch(response, now);
        }
        if !matches!(
            response.maybe_effect.as_ref(),
            Some(PreparedEffect::QualificationRestart { .. })
        ) {
            return self.confirm_sent(response);
        }
        let Some(PreparedEffect::QualificationRestart {
            kind,
            generation,
            token,
            context,
            expires_at_ms,
        }) = response.maybe_effect.take()
        else {
            return Err(WorkerControlError::StaleResponse);
        };
        let expected = self.maybe_restart_token.take();
        if !self.restart_consumed
            || expected != Some(token)
            || token != self.next_response_token
            || generation != self.generation
        {
            return Err(WorkerControlError::StaleResponse);
        }
        self.enforce_clock(now)?;
        self.required_start_context(now)?;
        self.require_restart_idle()?;
        if now >= expires_at_ms {
            return Err(WorkerControlError::StaleResponse);
        }
        let fresh = self
            .reset_context(kind)
            .map_err(|_| WorkerControlError::SessionFailed)?;
        if fresh != Some(context) {
            return Err(WorkerControlError::StaleResponse);
        }
        match kind {
            crate::QualificationResetKind::Restart => {
                self.session.qualification_restart(context, expires_at_ms)
            }
            crate::QualificationResetKind::CoreDumpSelfTest => {
                self.session.core_dump_self_test(context, expires_at_ms)
            }
        }
        .map_err(|_| WorkerControlError::SessionFailed)
    }
}
