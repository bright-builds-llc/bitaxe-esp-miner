use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Serialize;
use serde_json::Value;

pub(crate) const MAXIMUM_CONTROL_FRAME_BYTES: usize = 65_537;

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
