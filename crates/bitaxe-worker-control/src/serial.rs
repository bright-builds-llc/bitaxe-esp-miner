//! Bounded fixed Serial/JTAG framing and logical connection identity.

pub mod link;
mod liveness;
mod receive;
pub mod trace;
pub mod writer;
pub use liveness::SerialLinkLiveness;
pub use receive::{ReceiveCreditMailbox, SerialReceiveProgress};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{value::RawValue, Value};
use sha2::{Digest, Sha256};
use std::io::Write as _;
use thiserror::Error;
use zeroize::Zeroize;

pub const SERIAL_PROFILE: &str = "bwg-worker-serial/0.2";
pub const MAXIMUM_CONTROL_PAYLOAD_BYTES: usize = 65_536;
pub const MAXIMUM_WIRE_FRAME_BYTES: usize = 66_560;
pub const HEARTBEAT_TIMEOUT_MILLISECONDS: u64 = 2_800;
pub const HOST_RECEIVE_WINDOW_BYTES: u32 = 2048;
/// The receive buffer, reserved once at its frame bound and kept for reuse.
pub const FRAME_BUFFER_SITE: bitaxe_core::fallible::Site =
    bitaxe_core::fallible::Site("serial.frame_buffer");
/// The exact-length copy of each completed receive frame.
pub const FRAME_COPY_SITE: bitaxe_core::fallible::Site =
    bitaxe_core::fallible::Site("serial.frame_copy");
pub const MAXIMUM_HOST_WRITE_CHUNK_BYTES: u32 = 1024;
pub const RECORD_WRITE_TIMEOUT_MILLISECONDS: u64 = 2000;

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
        "hostToDeviceReceiveWindowBytes": HOST_RECEIVE_WINDOW_BYTES,
        "maximumHostWriteChunkBytes": MAXIMUM_HOST_WRITE_CHUNK_BYTES,
        "recordWriteTimeoutMilliseconds": RECORD_WRITE_TIMEOUT_MILLISECONDS,
        "payloadIntegrity": "sha256_exact_utf8_json",
        "qualificationAttemptProfile": "worker-qualification-attempt-v1",
        "poolDifficultyHintProfile": "worker-stratum-difficulty-hint-v1",
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
    payload_bytes: u32,
    payload_sha256: String,
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
    #[error("serial payload integrity failed")]
    Integrity,
}

/// A validated record borrowing its frame, so receiving one copies nothing. Canonical
/// records never escape their strings; an escaped field is invalid, not copied.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SerialRecord<'a> {
    pub profile: &'a str,
    pub kind: SerialKind,
    #[serde(borrow, deserialize_with = "required_borrowed_session_id")]
    pub session_id: Option<&'a str>,
    pub sequence: u32,
    payload_bytes: u32,
    payload_sha256: &'a str,
    #[serde(borrow)]
    pub payload: &'a RawValue,
}

impl<'a> SerialRecord<'a> {
    /// Parses one bounded UTF-8 JSON object followed by a single LF.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, SerialError> {
        if bytes.len() > MAXIMUM_WIRE_FRAME_BYTES {
            return Err(SerialError::Oversized);
        }
        let payload = bytes.strip_suffix(b"\n").ok_or(SerialError::Invalid)?;
        if payload.contains(&b'\n') || payload.contains(&b'\r') {
            return Err(SerialError::Invalid);
        }
        let record: Self = serde_json::from_slice(payload).map_err(|_| SerialError::Invalid)?;
        if record.profile != SERIAL_PROFILE
            || !record.payload.get().starts_with('{')
            || record.session_id.is_some_and(|id| !canonical_nonce(id, 16))
        {
            return Err(SerialError::Invalid);
        }
        if record.kind == SerialKind::Control
            && record.payload.get().len() > MAXIMUM_CONTROL_PAYLOAD_BYTES
        {
            return Err(SerialError::Oversized);
        }
        if record.payload.get().len() != record.payload_bytes as usize
            || !canonical_nonce(record.payload_sha256, 32)
            || record.payload_sha256
                != URL_SAFE_NO_PAD.encode(Sha256::digest(record.payload.get().as_bytes()))
        {
            return Err(SerialError::Integrity);
        }
        Ok(record)
    }

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
}

impl SerialEnvelope {
    /// Recognizes a closed session payload; callers must still validate its session and sequence.
    #[must_use]
    pub fn is_close(&self) -> bool {
        SerialRecord {
            profile: &self.profile,
            kind: self.kind,
            session_id: self.session_id.as_deref(),
            sequence: self.sequence,
            payload_bytes: self.payload_bytes,
            payload_sha256: &self.payload_sha256,
            payload: &self.payload,
        }
        .is_close()
    }

    /// Parses one bounded record into an owned envelope; see [`SerialRecord::parse`].
    pub fn parse(bytes: &[u8]) -> Result<Self, SerialError> {
        let record = SerialRecord::parse(bytes)?;
        Ok(Self {
            profile: record.profile.to_owned(),
            kind: record.kind,
            session_id: record.session_id.map(str::to_owned),
            sequence: record.sequence,
            payload_bytes: record.payload_bytes,
            payload_sha256: record.payload_sha256.to_owned(),
            payload: record.payload.to_owned(),
        })
    }

    /// Wraps a raw JSON payload without escaping or reinterpreting its values. The
    /// record is written into one buffer reserved recoverably at its bound, so an
    /// exhausted heap is `Unavailable`, never an abort.
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
            payload_bytes: usize,
            payload_sha256: &'a str,
            payload: &'a RawValue,
        }
        let mut digest = [0u8; 43];
        URL_SAFE_NO_PAD
            .encode_slice(Sha256::digest(payload.get().as_bytes()), &mut digest)
            .map_err(|_| SerialError::Invalid)?;
        let payload_sha256 = std::str::from_utf8(&digest).map_err(|_| SerialError::Invalid)?;
        let bound = payload
            .get()
            .len()
            .checked_add(ENVELOPE_OVERHEAD_BYTES)
            .filter(|bound| *bound <= MAXIMUM_WIRE_FRAME_BYTES)
            .ok_or(SerialError::Oversized)?;
        let mut bytes = Vec::new();
        bitaxe_core::fallible::reserve_exact(&mut bytes, bound, ENCODE_SITE)
            .map_err(|_| SerialError::Unavailable)?;
        let wire = Wire {
            profile: SERIAL_PROFILE,
            kind,
            session_id,
            sequence,
            payload_bytes: payload.get().len(),
            payload_sha256,
            payload,
        };
        serde_json::to_writer(ReservedWriter(&mut bytes), &wire)
            .map_err(|_| SerialError::Oversized)?;
        ReservedWriter(&mut bytes)
            .write_all(b"\n")
            .map_err(|_| SerialError::Oversized)?;
        SerialRecord::parse(&bytes)?;
        Ok(bytes)
    }
}

/// Every envelope field but the payload: profile, kind, session id, sequence, payload
/// length and digest, and their keys, with room to spare.
const ENVELOPE_OVERHEAD_BYTES: usize = 256;

/// One outbound record's buffer.
pub const ENCODE_SITE: bitaxe_core::fallible::Site = bitaxe_core::fallible::Site("serial.encode");

/// Writes only into capacity already reserved; it never grows the buffer.
struct ReservedWriter<'a>(&'a mut Vec<u8>);

impl std::io::Write for ReservedWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.0.capacity() - self.0.len() {
            return Err(std::io::ErrorKind::WriteZero.into());
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
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
        let unreserved = MAXIMUM_WIRE_FRAME_BYTES - self.bytes.len();
        if self.bytes.len() == self.bytes.capacity()
            && bitaxe_core::fallible::reserve_exact(&mut self.bytes, unreserved, FRAME_BUFFER_SITE)
                .is_err()
        {
            self.clear();
            self.discarding = byte != b'\n';
            return Some(Err(SerialError::Unavailable));
        }
        self.bytes.push(byte);
        if byte != b'\n' {
            return None;
        }
        // Keep the receive allocation for the next record. Only the initialized
        // prefix holds secrets; completed ownership uses an exact-length buffer.
        let mut frame = Vec::new();
        let result =
            bitaxe_core::fallible::reserve_exact(&mut frame, self.bytes.len(), FRAME_COPY_SITE)
                .map(|()| {
                    frame.extend_from_slice(&self.bytes);
                    frame
                })
                .map_err(|_| SerialError::Unavailable);
        self.clear();
        Some(result)
    }

    /// Clears partial secret-bearing input and resets framing recovery.
    pub fn clear(&mut self) {
        self.bytes.as_mut_slice().zeroize();
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

fn required_borrowed_session_id<'de: 'a, 'a, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<&'a str>, D::Error> {
    Option::<&'a str>::deserialize(deserializer)
}
