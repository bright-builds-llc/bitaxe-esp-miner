use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroizing;

use sha2::{Digest, Sha256};

use crate::codec::{base64_url, canonical_json, digest_text, strict_json_frame};

const POSSESSION_PROFILE: &str = "bwg-worker-possession/0.2";
const PROOF_PROFILE: &str = "bwg-worker-possession-proof/0.2";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmwareSourceCommit(String);

impl FirmwareSourceCommit {
    pub fn parse(value: &str) -> Result<Self, PossessionError> {
        let valid = value.len() == 40
            && value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
        valid
            .then(|| Self(value.to_owned()))
            .ok_or(PossessionError::InvalidRequest)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exact source and application ELF identity signed in every possession proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmwareIdentity {
    pub(crate) source_commit: FirmwareSourceCommit,
    pub(crate) app_elf_sha256: String,
}

impl FirmwareIdentity {
    pub fn new(
        source_commit: FirmwareSourceCommit,
        app_elf_sha256: &str,
    ) -> Result<Self, PossessionError> {
        if app_elf_sha256.len() != 64
            || !app_elf_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(PossessionError::InvalidRequest);
        }
        Ok(Self {
            source_commit,
            app_elf_sha256: app_elf_sha256.to_owned(),
        })
    }
}

#[derive(Debug, Error)]
pub enum PossessionError {
    #[error("Worker possession request is invalid")]
    InvalidRequest,
    #[error("Worker possession frame is invalid")]
    InvalidFrame,
    #[error("Worker possession response encoding failed")]
    Encoding(#[from] serde_json::Error),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PossessionPurpose {
    InitialAdmission,
    TransportReacquisition,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WirePossessionRequest {
    profile: String,
    request_id: String,
    command: String,
    payload: PossessionPayload,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PossessionPayload {
    purpose: PossessionPurpose,
    possession_nonce: String,
    challenge_binding_sha256: String,
    controller_capability_sha256: String,
    serial_manifest_sha256: String,
    session_id: String,
    host_nonce: String,
    device_nonce: String,
}

/// Validated fresh possession request; arbitrary signing input is unrepresentable.
#[derive(Clone)]
pub struct PossessionRequest {
    request_id: String,
    payload: PossessionPayload,
}

impl PossessionRequest {
    pub fn from_frame(bytes: &[u8]) -> Result<Self, PossessionError> {
        let json = strict_json_frame(bytes).map_err(|error| match error {
            "invalid_frame" => PossessionError::InvalidFrame,
            _ => PossessionError::InvalidRequest,
        })?;
        let wire: WirePossessionRequest =
            serde_json::from_str(json).map_err(|_| PossessionError::InvalidRequest)?;
        if wire.profile != POSSESSION_PROFILE
            || wire.command != "prove_possession"
            || !identifier(&wire.request_id, "pos_")
            || !digest_text(&wire.payload.possession_nonce)
            || !digest_text(&wire.payload.challenge_binding_sha256)
            || !digest_text(&wire.payload.controller_capability_sha256)
            || !digest_text(&wire.payload.serial_manifest_sha256)
            || crate::serial::SerialSessionBinding::parse(
                &wire.payload.session_id,
                &wire.payload.host_nonce,
                &wire.payload.device_nonce,
            )
            .is_err()
        {
            return Err(PossessionError::InvalidRequest);
        }
        Ok(Self {
            request_id: wire.request_id,
            payload: wire.payload,
        })
    }

    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub(crate) fn nonce(&self) -> &str {
        &self.payload.possession_nonce
    }

    pub(crate) fn matches_bindings(
        &self,
        capability_sha256: &str,
        manifest_sha256: &str,
        binding: &crate::serial::SerialSessionBinding,
    ) -> bool {
        self.payload.controller_capability_sha256 == capability_sha256
            && self.payload.serial_manifest_sha256 == manifest_sha256
            && self.payload.session_id == binding.session_id
            && self.payload.host_nonce == binding.host_nonce
            && self.payload.device_nonce == binding.device_nonce
    }

    pub fn control_session_binding(
        &self,
        response: &PossessionResponse,
    ) -> Result<String, PossessionError> {
        let transcript = serde_json::json!({
            "profile": "bwg-worker-control-session/0.2",
            "request": {
                "profile": POSSESSION_PROFILE,
                "requestId": self.request_id,
                "command": "prove_possession",
                "payload": self.payload,
            },
            "response": response,
        });
        Ok(base64_url(Sha256::digest(
            canonical_json(&transcript)?.as_bytes(),
        )))
    }
}

impl fmt::Debug for PossessionRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PossessionRequest")
            .field("request_id", &self.request_id)
            .field("payload", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PossessionClaims {
    profile: &'static str,
    purpose: PossessionPurpose,
    possession_nonce: String,
    challenge_binding_sha256: String,
    controller_capability_sha256: String,
    serial_manifest_sha256: String,
    session_id: String,
    host_nonce: String,
    device_nonce: String,
    firmware_source_commit: String,
    app_elf_sha256: String,
    device_identity_jwk: DeviceIdentityJwk,
}

impl PossessionClaims {
    pub(crate) fn from_request(
        request: &PossessionRequest,
        firmware_source_commit: &FirmwareSourceCommit,
        public_key: String,
        app_elf_sha256: &str,
    ) -> Self {
        Self {
            profile: PROOF_PROFILE,
            purpose: request.payload.purpose,
            possession_nonce: request.payload.possession_nonce.clone(),
            challenge_binding_sha256: request.payload.challenge_binding_sha256.clone(),
            controller_capability_sha256: request.payload.controller_capability_sha256.clone(),
            serial_manifest_sha256: request.payload.serial_manifest_sha256.clone(),
            session_id: request.payload.session_id.clone(),
            host_nonce: request.payload.host_nonce.clone(),
            device_nonce: request.payload.device_nonce.clone(),
            firmware_source_commit: firmware_source_commit.as_str().to_owned(),
            app_elf_sha256: app_elf_sha256.to_owned(),
            device_identity_jwk: DeviceIdentityJwk {
                kty: "OKP",
                crv: "Ed25519",
                x: Zeroizing::new(public_key),
                alg: "Ed25519",
                use_: "sig",
                key_ops: ["verify"],
            },
        }
    }
}

#[derive(Clone, Serialize)]
struct DeviceIdentityJwk {
    kty: &'static str,
    crv: &'static str,
    x: Zeroizing<String>,
    alg: &'static str,
    #[serde(rename = "use")]
    use_: &'static str,
    key_ops: [&'static str; 1],
}

#[derive(Clone, Serialize)]
pub struct PossessionResponse {
    profile: &'static str,
    #[serde(rename = "requestId")]
    request_id: String,
    ok: bool,
    result: PossessionResult,
}

#[derive(Clone, Serialize)]
struct PossessionResult {
    claims: PossessionClaims,
    #[serde(rename = "compactJws")]
    compact_jws: Zeroizing<String>,
}

impl PossessionResponse {
    pub(crate) fn new(request_id: String, claims: PossessionClaims, compact_jws: String) -> Self {
        Self {
            profile: POSSESSION_PROFILE,
            request_id,
            ok: true,
            result: PossessionResult {
                claims,
                compact_jws: Zeroizing::new(compact_jws),
            },
        }
    }

    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    #[must_use]
    pub fn compact_jws(&self) -> &str {
        &self.result.compact_jws
    }

    pub fn to_frame(&self) -> Result<Vec<u8>, PossessionError> {
        let mut bytes = serde_json::to_vec(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

impl fmt::Debug for PossessionResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PossessionResponse")
            .field("request_id", &self.request_id)
            .field("result", &"[redacted]")
            .finish()
    }
}

fn identifier(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}
