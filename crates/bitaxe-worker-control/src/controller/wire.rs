use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use zeroize::Zeroize;

use super::{RestorationReason, WorkerControlError, PROTOCOL_VERSION};

pub(super) fn classify_json_error(error: serde_json::Error) -> WorkerControlError {
    match error.classify() {
        serde_json::error::Category::Io
        | serde_json::error::Category::Syntax
        | serde_json::error::Category::Eof => WorkerControlError::InvalidFrame,
        serde_json::error::Category::Data => WorkerControlError::InvalidRequest,
    }
}

#[derive(Deserialize)]
pub(super) struct FrameDiscriminator {
    #[serde(default)]
    pub(super) profile: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ControllerRequest {
    protocol_version: String,
    pub(super) request_id: String,
    pub(super) command: String,
    #[serde(default)]
    payload: Option<SecretJson>,
}

impl ControllerRequest {
    pub(super) fn validate(&self) -> Result<(), WorkerControlError> {
        if self.protocol_version != PROTOCOL_VERSION || !request_id(&self.request_id) {
            return Err(WorkerControlError::InvalidRequest);
        }
        Ok(())
    }

    pub(super) fn require_no_payload(&self) -> Result<(), WorkerControlError> {
        if self.payload.is_none() {
            Ok(())
        } else {
            Err(WorkerControlError::InvalidRequest)
        }
    }

    pub(super) fn required_payload<T: DeserializeOwned>(&self) -> Result<T, WorkerControlError> {
        self.payload
            .as_ref()
            .ok_or(WorkerControlError::InvalidRequest)?
            .parse()
    }
}

struct SecretJson {
    raw: Option<Box<RawValue>>,
}

impl SecretJson {
    fn parse<T: DeserializeOwned>(&self) -> Result<T, WorkerControlError> {
        let raw = self
            .raw
            .as_ref()
            .ok_or(WorkerControlError::InvalidRequest)?;
        serde_json::from_str(raw.get()).map_err(|_| WorkerControlError::InvalidRequest)
    }
}

impl<'de> Deserialize<'de> for SecretJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Box::<RawValue>::deserialize(deserializer).map(|raw| Self { raw: Some(raw) })
    }
}

impl Drop for SecretJson {
    fn drop(&mut self) {
        let Some(raw) = self.raw.take() else {
            return;
        };
        let mut secret: Box<str> = raw.into();
        secret.zeroize();
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RestorePayload {
    pub(super) reason: RestorationReason,
}

fn request_id(value: &str) -> bool {
    value.starts_with("serial_")
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ProbePayload {
    pub padding: String,
    pub response_padding_bytes: usize,
}
