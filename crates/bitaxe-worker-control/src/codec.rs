use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use zeroize::Zeroize;

pub(crate) const MAXIMUM_CONTROL_FRAME_BYTES: usize = 65_536;

/// Bounded secret-bearing JSON-lines accumulator for fragmented USB packets.
pub struct WorkerControlFrameAccumulator {
    bytes: Vec<u8>,
}

/// Closed framing failure with no frame contents.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WorkerControlFrameAccumulatorError {
    #[error("Worker control transfer is oversized")]
    Oversized,
    #[error("Worker control transfer contains multiple frames")]
    MultipleFrames,
}

impl WorkerControlFrameAccumulator {
    #[must_use]
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn push(
        &mut self,
        chunk: &[u8],
    ) -> Result<Option<Vec<u8>>, WorkerControlFrameAccumulatorError> {
        if self.bytes.len().saturating_add(chunk.len()) > MAXIMUM_CONTROL_FRAME_BYTES {
            self.clear();
            return Err(WorkerControlFrameAccumulatorError::Oversized);
        }
        self.bytes.extend_from_slice(chunk);
        let Some(newline) = self.bytes.iter().position(|byte| *byte == b'\n') else {
            return Ok(None);
        };
        if self.bytes[newline + 1..].contains(&b'\n') {
            self.clear();
            return Err(WorkerControlFrameAccumulatorError::MultipleFrames);
        }
        Ok(Some(self.bytes.drain(..=newline).collect()))
    }

    pub fn clear(&mut self) {
        self.bytes.zeroize();
        self.bytes.clear();
    }
}

impl Default for WorkerControlFrameAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WorkerControlFrameAccumulator {
    fn drop(&mut self) {
        self.clear();
    }
}

pub(crate) fn strict_json_frame(bytes: &[u8]) -> Result<&str, &'static str> {
    if bytes.is_empty() || bytes.len() > MAXIMUM_CONTROL_FRAME_BYTES {
        return Err("invalid_frame");
    }
    let Some(payload) = bytes.strip_suffix(b"\n") else {
        return Err("invalid_frame");
    };
    if payload.is_empty() || payload.contains(&b'\n') || payload.contains(&b'\r') {
        return Err("invalid_frame");
    }
    std::str::from_utf8(payload).map_err(|_| "invalid_frame")
}

pub(crate) fn digest_text(value: &str) -> bool {
    value.len() == 43
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

pub(crate) fn base64_url(bytes: impl AsRef<[u8]>) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub(crate) fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    canonical_value(&value)
}

fn canonical_value(value: &Value) -> Result<String, serde_json::Error> {
    match value {
        Value::Array(values) => values
            .iter()
            .map(canonical_value)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| format!("[{}]", values.join(","))),
        Value::Object(record) => record
            .iter()
            .map(|(key, value)| {
                Ok(format!(
                    "{}:{}",
                    serde_json::to_string(key)?,
                    canonical_value(value)?
                ))
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()
            .map(|fields| format!("{{{}}}", fields.join(","))),
        primitive => serde_json::to_string(primitive),
    }
}
