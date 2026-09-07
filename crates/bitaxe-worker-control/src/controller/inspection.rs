use super::{
    strict_json_frame, ControllerRequest, LeaseAuthorizationVerifier, PreparedResponse, Value,
    WorkerControl, WorkerControlError, WorkerSession, PROTOCOL_VERSION,
};

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    /// Builds only a closed, correlated rejection for a structurally valid request.
    /// It grants no authority and has no send-confirmation state effect.
    pub fn prepare_rejection(
        &self,
        frame: &[u8],
        error: &WorkerControlError,
    ) -> Result<PreparedResponse, WorkerControlError> {
        let json = strict_json_frame(frame).map_err(|_| WorkerControlError::InvalidFrame)?;
        let request: ControllerRequest =
            serde_json::from_str(json).map_err(|_| WorkerControlError::InvalidRequest)?;
        request.validate()?;
        let mut bytes = serde_json::to_vec(&serde_json::json!({
            "protocolVersion": PROTOCOL_VERSION, "requestId": request.request_id,
            "ok": false, "error": {"code":"command_rejected", "message":error.category()}
        }))
        .map_err(|_| WorkerControlError::Encoding)?;
        bytes.push(b'\n');
        Ok(PreparedResponse {
            frame: bytes,
            maybe_effect: None,
        })
    }

    pub(super) fn review_qualification_attempt(
        &self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<Value, WorkerControlError> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Empty {}
        let _: Empty = request.required_payload()?;
        self.required_start_context(now)?;
        if self.maybe_active.is_some() || self.effect_cleanup_required {
            return Err(WorkerControlError::InvalidTransition);
        }
        self.session
            .qualification_attempt_review()
            .map_err(|_| WorkerControlError::SessionFailed)?
            .ok_or(WorkerControlError::InvalidRequest)
    }

    pub(super) fn review_acceptance_budget(
        &self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<Value, WorkerControlError> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct ReviewPayload {
            campaign_id: String,
        }
        let payload: ReviewPayload = request.required_payload()?;
        self.required_start_context(now)?;
        if self.maybe_active.is_some() || self.effect_cleanup_required {
            return Err(WorkerControlError::InvalidTransition);
        }
        use base64::Engine as _;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&payload.campaign_id)
            .map_err(|_| WorkerControlError::InvalidRequest)?;
        if decoded.len() != 16
            || base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&decoded)
                != payload.campaign_id
        {
            return Err(WorkerControlError::InvalidRequest);
        }
        self.session
            .acceptance_budget_review(&payload.campaign_id)
            .map_err(|_| WorkerControlError::SessionFailed)?
            .ok_or(WorkerControlError::InvalidRequest)
    }
}
