use super::{
    LeaseAuthorizationVerifier, PreparedEffect, PreparedResponse, RestorationReason,
    RestorationState, WorkerControl, WorkerControlError, WorkerSession, PROTOCOL_VERSION,
};
use serde_json::{json, Value};
use zeroize::Zeroize;

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    pub(super) fn with_status_evidence(
        &self,
        mut result: Value,
    ) -> Result<Value, WorkerControlError> {
        if let Some(evidence) = self.session.status_evidence() {
            result["qualification"] = evidence;
        }
        if !self.authenticated_logical_session {
            return Ok(result);
        }
        if let Some(settings) = self
            .session
            .settings_preservation()
            .map_err(|_| WorkerControlError::SessionFailed)?
        {
            let authorization = self
                .verifier
                .authorization_high_water_fingerprint()
                .map_err(|_| WorkerControlError::PersistenceFailed)?
                .ok_or(WorkerControlError::PersistenceFailed)?;
            result["preservation"] = json!({
                "schema":"worker-preservation-v1", "settings_sha256":settings.fingerprint,
                "authorization_high_water_sha256":authorization,
                "device_identity_sha256":self.identity.public_key_fingerprint(),
                "mine_on_boot":settings.mine_on_boot,
            });
        }
        Ok(result)
    }

    pub(super) fn status(&self, now: u64) -> Result<Value, WorkerControlError> {
        if let Some(active) = self.maybe_active.as_ref() {
            return Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "state": "mining",
                "monotonicMilliseconds": now,
                "lease": {
                    "leaseId": active.grant.lease_id(),
                    "challengeId": active.grant.challenge_id(),
                    "renewAtMonotonicMilliseconds": active.deadlines.renew_at_monotonic_milliseconds(),
                    "expiresAtMonotonicMilliseconds": active.deadlines.expires_at_monotonic_milliseconds(),
                },
                "restoration": { "status": "pending" },
            }));
        }
        let restoration = if self.boot_restoration_clear_required {
            json!({ "status": "confirmed", "reason": RestorationReason::Reboot })
        } else {
            match self.restoration {
                RestorationState::NotRequired => json!({ "status": "not_required" }),
                RestorationState::Confirmed(reason) => {
                    json!({ "status": "confirmed", "reason": reason })
                }
                RestorationState::Pending => return Err(WorkerControlError::RestorationPending),
            }
        };
        Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "state": "baseline",
            "monotonicMilliseconds": now,
            "restoration": restoration,
        }))
    }
}

pub(super) fn response(
    request_id: &str,
    result: Value,
    maybe_effect: Option<PreparedEffect>,
) -> Result<PreparedResponse, WorkerControlError> {
    let mut frame = serde_json::to_vec(&json!({
        "protocolVersion": PROTOCOL_VERSION,
        "requestId": request_id,
        "ok": true,
        "result": result,
    }))
    .map_err(|_| WorkerControlError::Encoding)?;
    if frame.len() > crate::serial::MAXIMUM_CONTROL_PAYLOAD_BYTES {
        frame.zeroize();
        return Err(WorkerControlError::InvalidFrame);
    }
    frame.push(b'\n');
    Ok(PreparedResponse {
        frame,
        maybe_effect,
    })
}
