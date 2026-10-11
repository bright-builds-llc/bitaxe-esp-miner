//! Host-side checks of device output, mirroring the pinned Gate (`bitaxe-turnstile-system`
//! `web/worker-serial*.ts`): hello_ack shape and deadline, session and strictly
//! increasing sequence, device-heartbeat liveness, receive-credit bounds, full credit
//! before the next host record, the terminal credit after Close, reply correlation and
//! diagnostic fields. The first violation is kept. Increment 3 replaces this stand-in
//! with the pinned Gate controller itself.

use crate::{CheckStatus, ProfileCheck};
use bitaxe_worker_control::serial::{
    canonical_nonce, serial_manifest, SerialEnvelope, SerialFrameAccumulator, SerialKind,
    HEARTBEAT_TIMEOUT_MILLISECONDS, HOST_RECEIVE_WINDOW_BYTES,
};
use serde_json::Value;

/// The device must send its terminal credit within this window after Close.
const TERMINAL_CREDIT_MILLISECONDS: u64 = 2000;
const HELLO_ACK_KEYS: [&str; 8] = [
    "appElfSha256",
    "deviceNonce",
    "firmwareSourceCommit",
    "hostNonce",
    "op",
    "receiveWindowBytes",
    "receivedBytes",
    "serialManifest",
];

struct Admitted {
    session_id: String,
    last_sequence: u32,
    last_heartbeat_ms: u64,
    sent_bytes: u32,
    acknowledged_bytes: u32,
    maybe_pending_request: Option<String>,
    maybe_close: Option<u64>,
    terminal_credited: bool,
}

#[derive(Default)]
pub(crate) struct GatePeer {
    host_nonce: String,
    hello_sent_ms: u64,
    frames: SerialFrameAccumulator,
    maybe_admitted: Option<Admitted>,
    maybe_violation: Option<&'static str>,
}

impl GatePeer {
    /// The host sent Hello; device liveness is anchored here, as in the Gate.
    pub(crate) fn hello_sent(&mut self, host_nonce: &str, now_ms: u64) {
        self.host_nonce = host_nonce.to_owned();
        self.hello_sent_ms = now_ms;
        self.maybe_admitted = None;
        self.frames.clear();
    }

    /// The host is about to send one record after Hello.
    pub(crate) fn host_record(
        &mut self,
        wire_bytes: usize,
        maybe_request_id: Option<String>,
        close: bool,
        now_ms: u64,
    ) {
        self.settle(now_ms);
        let Some(admitted) = self.maybe_admitted.as_mut() else {
            self.violate("hello_ack");
            return;
        };
        if admitted.acknowledged_bytes != admitted.sent_bytes {
            self.violate("credit_stalled");
            return;
        }
        admitted.sent_bytes = admitted.sent_bytes.saturating_add(wire_bytes as u32);
        if maybe_request_id.is_some() {
            admitted.maybe_pending_request = maybe_request_id;
        }
        if close {
            admitted.maybe_close = Some(now_ms);
        }
    }

    /// Device output bytes observed at `now_ms`.
    pub(crate) fn device_bytes(&mut self, bytes: &[u8], now_ms: u64) {
        for byte in bytes {
            let Some(result) = self.frames.push_byte(*byte) else {
                continue;
            };
            match result
                .ok()
                .and_then(|frame| SerialEnvelope::parse(&frame).ok())
            {
                Some(envelope) => self.device_record(&envelope, now_ms),
                None => self.violate("envelope"),
            }
        }
    }

    /// Applies the host's own clock: device liveness and the terminal credit window.
    pub(crate) fn settle(&mut self, now_ms: u64) {
        let Some(admitted) = self.maybe_admitted.as_ref() else {
            return;
        };
        if let Some(closed_at) = admitted.maybe_close {
            if !admitted.terminal_credited
                && now_ms.saturating_sub(closed_at) > TERMINAL_CREDIT_MILLISECONDS
            {
                self.violate("terminal_credit");
            }
            return;
        }
        if now_ms.saturating_sub(admitted.last_heartbeat_ms) >= HEARTBEAT_TIMEOUT_MILLISECONDS {
            self.violate("continuity");
        }
    }

    /// Whether the device's final credit after Close covered every host byte.
    pub(crate) fn terminal_credited(&self) -> bool {
        self.maybe_admitted
            .as_ref()
            .is_some_and(|admitted| admitted.terminal_credited)
    }

    pub(crate) fn check(&self) -> ProfileCheck {
        let terminal = match self.maybe_admitted.as_ref() {
            Some(admitted) => admitted.maybe_close.is_none() || admitted.terminal_credited,
            None => true,
        };
        let passed = self.maybe_violation.is_none() && self.maybe_admitted.is_some() && terminal;
        ProfileCheck {
            id: "serial_output".into(),
            status: if passed {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            detail: match self.maybe_violation {
                Some(category) => category.into(),
                None if passed => "device_records_met_gate_rules".into(),
                None if self.maybe_admitted.is_none() => "hello_ack_missing".into(),
                None => "terminal_credit_missing".into(),
            },
        }
    }

    fn device_record(&mut self, envelope: &SerialEnvelope, now_ms: u64) {
        let Some(admitted) = self.maybe_admitted.as_mut() else {
            self.admit(envelope, now_ms);
            return;
        };
        let expired =
            now_ms.saturating_sub(admitted.last_heartbeat_ms) >= HEARTBEAT_TIMEOUT_MILLISECONDS;
        if admitted.maybe_close.is_none()
            && (expired
                || envelope.session_id.as_deref() != Some(admitted.session_id.as_str())
                || envelope.sequence <= admitted.last_sequence)
        {
            self.violate("continuity");
            return;
        }
        admitted.last_sequence = envelope.sequence;
        let payload: Value = serde_json::from_str(envelope.payload.get()).unwrap_or(Value::Null);
        if admitted.maybe_close.is_some() {
            // The Gate discards everything from the retired epoch except one final credit.
            if credit(&payload) == Some(admitted.sent_bytes) {
                admitted.terminal_credited = true;
            }
            return;
        }
        let violation = match envelope.kind {
            SerialKind::Heartbeat => {
                admitted.last_heartbeat_ms = now_ms;
                None
            }
            SerialKind::Session => match credit(&payload) {
                Some(received)
                    if received > admitted.acknowledged_bytes
                        && received <= admitted.sent_bytes =>
                {
                    admitted.acknowledged_bytes = received;
                    None
                }
                Some(_) => Some("credit_invalid"),
                None => Some("correlation"),
            },
            SerialKind::Diagnostic => {
                let line_only = payload.as_object().is_some_and(|object| {
                    object.len() == 1 && object.get("line").is_some_and(Value::is_string)
                });
                (!line_only).then_some("fields")
            }
            SerialKind::Control => {
                let request_id = payload.get("requestId").and_then(Value::as_str);
                if request_id.is_some() && request_id == admitted.maybe_pending_request.as_deref() {
                    admitted.maybe_pending_request = None;
                    None
                } else {
                    Some("correlation")
                }
            }
        };
        if let Some(category) = violation {
            self.violate(category);
        }
    }

    fn admit(&mut self, envelope: &SerialEnvelope, now_ms: u64) {
        let payload: Value = serde_json::from_str(envelope.payload.get()).unwrap_or(Value::Null);
        let mut keys: Vec<&str> = payload
            .as_object()
            .map(|object| object.keys().map(String::as_str).collect())
            .unwrap_or_default();
        keys.sort_unstable();
        let text = |key: &str| payload.get(key).and_then(Value::as_str).unwrap_or("");
        let hex = |key: &str, length: usize| {
            let value = text(key);
            value.len() == length
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        };
        let valid = envelope.kind == SerialKind::Session
            && envelope.sequence == 0
            && keys == HELLO_ACK_KEYS
            && text("op") == "hello_ack"
            && text("hostNonce") == self.host_nonce
            && canonical_nonce(text("deviceNonce"), 32)
            && hex("firmwareSourceCommit", 40)
            && hex("appElfSha256", 64)
            && payload.get("receiveWindowBytes").and_then(Value::as_u64)
                == Some(u64::from(HOST_RECEIVE_WINDOW_BYTES))
            && payload.get("receivedBytes").and_then(Value::as_u64) == Some(0)
            && payload.get("serialManifest") == Some(&serial_manifest())
            && now_ms.saturating_sub(self.hello_sent_ms) < HEARTBEAT_TIMEOUT_MILLISECONDS;
        let Some(session_id) = envelope.session_id.clone().filter(|_| valid) else {
            self.violate("hello_ack");
            return;
        };
        self.maybe_admitted = Some(Admitted {
            session_id,
            last_sequence: 0,
            last_heartbeat_ms: self.hello_sent_ms,
            sent_bytes: 0,
            acknowledged_bytes: 0,
            maybe_pending_request: None,
            maybe_close: None,
            terminal_credited: false,
        });
    }

    fn violate(&mut self, category: &'static str) {
        if self.maybe_violation.is_none() {
            self.maybe_violation = Some(category);
        }
    }
}

/// `receivedBytes` of an exact `{"op":"receive_credit","receivedBytes":N}` payload.
fn credit(payload: &Value) -> Option<u32> {
    let object = payload.as_object()?;
    if object.len() != 2 || object.get("op")?.as_str()? != "receive_credit" {
        return None;
    }
    u32::try_from(object.get("receivedBytes")?.as_u64()?).ok()
}

#[cfg(test)]
mod tests;
