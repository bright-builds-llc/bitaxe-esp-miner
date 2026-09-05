//! Fixed-pattern qualification payloads, bounded independently in both directions.
use super::{wire::ProbePayload, WorkerControlError, PROTOCOL_VERSION};
use serde_json::{json, Value};

pub(super) fn response(
    mut payload: ProbePayload,
    request_id: &str,
) -> Result<Value, WorkerControlError> {
    if !payload.padding.bytes().all(|byte| byte == b'x') {
        return Err(WorkerControlError::InvalidRequest);
    }
    let request_padding_bytes = payload.padding.len();
    let overhead = serde_json::to_vec(&json!({"protocolVersion": PROTOCOL_VERSION,
            "requestId": request_id, "ok": true, "result": {"padding": "", "requestPaddingBytes": request_padding_bytes}}))
    .map_err(|_| WorkerControlError::Encoding)?
    .len();
    if payload.response_padding_bytes < payload.padding.len()
        || payload.response_padding_bytes > crate::serial::MAXIMUM_CONTROL_PAYLOAD_BYTES - overhead
    {
        return Err(WorkerControlError::InvalidRequest);
    }
    let extra = payload.response_padding_bytes - payload.padding.len();
    payload
        .padding
        .try_reserve_exact(extra)
        .map_err(|_| WorkerControlError::SessionFailed)?;
    for _ in 0..extra {
        payload.padding.push('x');
    }
    Ok(json!({"padding": payload.padding, "requestPaddingBytes": request_padding_bytes}))
}
