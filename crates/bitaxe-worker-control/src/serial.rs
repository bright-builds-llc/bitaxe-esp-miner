//! Bounded fixed Serial/JTAG framing and logical connection identity.

mod liveness;
pub use liveness::SerialLinkLiveness;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{value::RawValue, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroize;

pub const SERIAL_PROFILE: &str = "bwg-worker-serial/0.1";
pub const MAXIMUM_CONTROL_PAYLOAD_BYTES: usize = 65_536;
pub const MAXIMUM_WIRE_FRAME_BYTES: usize = 66_560;
pub const HEARTBEAT_TIMEOUT_MILLISECONDS: u64 = 2_800;

#[must_use]
/// Returns the canonical signed application transport manifest.
pub fn serial_manifest() -> Value {
    serde_json::json!({
        "profile": SERIAL_PROFILE, "transport": "esp32s3_usb_serial_jtag",
        "baudRate": 115200, "framing": "utf8_ndjson",
        "maximumControlPayloadBytes": MAXIMUM_CONTROL_PAYLOAD_BYTES,
        "maximumWireFrameBytes": MAXIMUM_WIRE_FRAME_BYTES,
        "heartbeatIntervalMilliseconds": 1000,
        "heartbeatTimeoutMilliseconds": HEARTBEAT_TIMEOUT_MILLISECONDS,
        "foregroundOnly": true,
    })
}

/// Hashes the canonical manifest with the browser-compatible encoding.
pub fn serial_manifest_sha256() -> Result<String, serde_json::Error> {
    Ok(
        URL_SAFE_NO_PAD.encode(Sha256::digest(crate::codec::canonical_json(
            &serial_manifest(),
        )?)),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Fresh device session identity and both handshake nonces bound by possession.
pub struct SerialSessionBinding {
    pub session_id: String,
    pub host_nonce: String,
    pub device_nonce: String,
}

impl SerialSessionBinding {
    pub fn parse(
        session_id: &str,
        host_nonce: &str,
        device_nonce: &str,
    ) -> Result<Self, SerialError> {
        if !canonical_nonce(session_id, 16)
            || !canonical_nonce(host_nonce, 32)
            || !canonical_nonce(device_nonce, 32)
        {
            return Err(SerialError::Invalid);
        }
        Ok(Self {
            session_id: session_id.to_owned(),
            host_nonce: host_nonce.to_owned(),
            device_nonce: device_nonce.to_owned(),
        })
    }
}

#[must_use]
/// Validates base64url without padding or alternative trailing-bit encodings.
pub fn canonical_nonce(value: &str, bytes: usize) -> bool {
    URL_SAFE_NO_PAD
        .decode(value)
        .is_ok_and(|decoded| decoded.len() == bytes && URL_SAFE_NO_PAD.encode(decoded) == value)
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
/// Closed record roles; only Control records can contain Worker requests.
pub enum SerialKind {
    Session,
    Control,
    Heartbeat,
    Diagnostic,
}

/// Payload bytes never appear in Debug or framing errors.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SerialEnvelope {
    pub profile: String,
    pub kind: SerialKind,
    #[serde(deserialize_with = "required_session_id")]
    pub session_id: Option<String>,
    pub sequence: u32,
    pub payload: Box<RawValue>,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SerialError {
    #[error("serial record is invalid")]
    Invalid,
    #[error("serial record exceeds its bound")]
    Oversized,
    #[error("serial frame buffer allocation failed")]
    Unavailable,
}

impl SerialEnvelope {
    /// Recognizes a closed session payload; callers must still validate its session and sequence.
    #[must_use]
    pub fn is_close(&self) -> bool {
        #[derive(Deserialize)]
        enum Op {
            #[serde(rename = "close")]
            Close,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Close {
            #[serde(rename = "op")]
            _op: Op,
            #[serde(rename = "reason")]
            _reason: crate::RestorationReason,
        }
        self.kind == SerialKind::Session
            && serde_json::from_str::<Close>(self.payload.get()).is_ok()
    }

    /// Parses one bounded UTF-8 JSON object followed by a single LF.
    pub fn parse(bytes: &[u8]) -> Result<Self, SerialError> {
        if bytes.len() > MAXIMUM_WIRE_FRAME_BYTES {
            return Err(SerialError::Oversized);
        }
        let payload = bytes.strip_suffix(b"\n").ok_or(SerialError::Invalid)?;
        if payload.contains(&b'\n') || payload.contains(&b'\r') {
            return Err(SerialError::Invalid);
        }
        let envelope: Self = serde_json::from_slice(payload).map_err(|_| SerialError::Invalid)?;
        if envelope.profile != SERIAL_PROFILE
            || !envelope.payload.get().starts_with('{')
            || envelope
                .session_id
                .as_ref()
                .is_some_and(|id| !canonical_nonce(id, 16))
        {
            return Err(SerialError::Invalid);
        }
        if envelope.kind == SerialKind::Control
            && envelope.payload.get().len() > MAXIMUM_CONTROL_PAYLOAD_BYTES
        {
            return Err(SerialError::Oversized);
        }
        Ok(envelope)
    }

    /// Wraps a raw JSON payload without escaping or reinterpreting its values.
    pub fn encode(
        kind: SerialKind,
        session_id: Option<&str>,
        sequence: u32,
        payload: &RawValue,
    ) -> Result<Vec<u8>, SerialError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Wire<'a> {
            profile: &'a str,
            kind: SerialKind,
            session_id: Option<&'a str>,
            sequence: u32,
            payload: &'a RawValue,
        }
        let mut bytes = serde_json::to_vec(&Wire {
            profile: SERIAL_PROFILE,
            kind,
            session_id,
            sequence,
            payload,
        })
        .map_err(|_| SerialError::Invalid)?;
        bytes.push(b'\n');
        Self::parse(&bytes)?;
        Ok(bytes)
    }
}

/// Byte-at-a-time processing bounds memory even for unlimited unterminated input.
#[derive(Default)]
pub struct SerialFrameAccumulator {
    bytes: Vec<u8>,
    discarding: bool,
}

impl SerialFrameAccumulator {
    /// Emits complete records and discards oversized lines through their next LF.
    pub fn push_byte(&mut self, byte: u8) -> Option<Result<Vec<u8>, SerialError>> {
        if self.discarding {
            if byte == b'\n' {
                self.discarding = false;
            }
            return None;
        }
        if self.bytes.len() >= MAXIMUM_WIRE_FRAME_BYTES - 1 && byte != b'\n' {
            self.clear();
            self.discarding = true;
            return Some(Err(SerialError::Oversized));
        }
        if self.bytes.len() == self.bytes.capacity()
            && self
                .bytes
                .try_reserve_exact(MAXIMUM_WIRE_FRAME_BYTES - self.bytes.len())
                .is_err()
        {
            self.clear();
            self.discarding = byte != b'\n';
            return Some(Err(SerialError::Unavailable));
        }
        self.bytes.push(byte);
        (byte == b'\n').then(|| Ok(std::mem::take(&mut self.bytes)))
    }

    /// Clears partial secret-bearing input and resets framing recovery.
    pub fn clear(&mut self) {
        self.bytes.zeroize();
        self.bytes.clear();
        self.discarding = false;
    }
}

impl Drop for SerialFrameAccumulator {
    fn drop(&mut self) {
        self.clear();
    }
}

impl Drop for SerialEnvelope {
    fn drop(&mut self) {
        let payload = std::mem::replace(&mut self.payload, RawValue::NULL.to_owned());
        let text: Box<str> = payload.into();
        let mut text = text.into_string();
        text.zeroize();
    }
}

fn required_session_id<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}
