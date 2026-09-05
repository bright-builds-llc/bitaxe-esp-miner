use std::collections::HashSet;
use std::fmt;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroizing;

use crate::codec::{base64_url, canonical_json, digest_text};
use crate::{WorkerLeaseGrant, WorkerLeaseRenewal};

const TRUST_PROFILE: &str = "bwg-worker-deployment-trust/0.2";
const AUTHORIZATION_PROFILE: &str = "bwg-worker-lease-authorization/0.2";
const AUTHORIZATION_TYPE: &str = "bwg-worker-lease-authorization+jws";
const CONTROLLER_AUDIENCE: &str = "bwg-worker-controller/0.4";
const MAXIMUM_AUTHORIZATION_BYTES: usize = 512;

/// Exact possession transcript binding retained only for the current admitted lease.
#[derive(Clone, Eq, PartialEq)]
pub struct WorkerLeaseAuthorizationContext {
    control_session_binding_sha256: String,
}

impl WorkerLeaseAuthorizationContext {
    /// Parses a verified possession transcript digest at an adapter/conformance boundary.
    pub fn parse(value: &str) -> Result<Self, LeaseAuthorizationError> {
        if !digest_text(value) {
            return Err(LeaseAuthorizationError::InvalidAuthorization);
        }
        Ok(Self {
            control_session_binding_sha256: value.to_owned(),
        })
    }

    fn binding(&self) -> &str {
        &self.control_session_binding_sha256
    }
}

impl fmt::Debug for WorkerLeaseAuthorizationContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WorkerLeaseAuthorizationContext([redacted])")
    }
}

/// Durable per-authority replay state. Compare-and-store must be one atomic transaction.
pub trait AcceptedSequenceStore {
    fn authorization_high_water_fingerprint(
        &self,
    ) -> Result<Option<crate::StateFingerprint>, LeaseAuthorizationError> {
        Ok(None)
    }

    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError>;
    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError>;

    fn load(&self, key_id: &str) -> Result<Option<u64>, LeaseAuthorizationError>;

    fn compare_and_store(
        &mut self,
        key_id: &str,
        expected: Option<u64>,
        next: u64,
    ) -> Result<SequenceStoreResult, LeaseAuthorizationError>;
}

/// Closed result of one durable high-water compare-and-store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceStoreResult {
    Committed,
    Stale,
    AlreadyCommitted,
}

/// Metadata-only authorization rejection categories.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum LeaseAuthorizationError {
    #[error("Work Lease authorization is invalid")]
    InvalidAuthorization,
    #[error("Work Lease authorization was replayed")]
    Replay,
    #[error("Work Lease replay state is unavailable")]
    Persistence,
}

impl LeaseAuthorizationError {
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::InvalidAuthorization => "invalid_authorization",
            Self::Replay => "replay",
            Self::Persistence => "persistence_failed",
        }
    }
}

#[derive(Clone)]
struct TrustedKey {
    key_id: String,
    public_bytes: [u8; 32],
    key: VerifyingKey,
}

/// Strict Work Lease half of the role-separated deployment trust document.
pub struct WorkLeaseAuthorityTrust {
    issuer: String,
    audience: String,
    keys: Vec<TrustedKey>,
}

impl WorkLeaseAuthorityTrust {
    pub fn from_deployment_json(input: &str) -> Result<Self, LeaseAuthorizationError> {
        let document: WireDeploymentTrust = serde_json::from_str(input)
            .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?;
        if document.profile != TRUST_PROFILE
            || document.update_authority.role != "update_authority"
            || document.update_authority.audience != "bwg-reference-firmware-capability/0.2"
            || document.work_lease_authority.profile != TRUST_PROFILE
            || document.work_lease_authority.role != "work_lease_authority"
            || document.work_lease_authority.audience != CONTROLLER_AUDIENCE
            || !label(&document.update_authority.issuer)
            || !label(&document.work_lease_authority.issuer)
        {
            return Err(LeaseAuthorizationError::InvalidAuthorization);
        }
        let update_keys = parse_keys(document.update_authority.keys)?;
        let keys = parse_keys(document.work_lease_authority.keys)?;
        if update_keys.iter().any(|update| {
            keys.iter().any(|lease| {
                update.key_id == lease.key_id || update.public_bytes == lease.public_bytes
            })
        }) {
            return Err(LeaseAuthorizationError::InvalidAuthorization);
        }
        Ok(Self {
            issuer: document.work_lease_authority.issuer,
            audience: document.work_lease_authority.audience,
            keys,
        })
    }
}

impl fmt::Debug for WorkLeaseAuthorityTrust {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkLeaseAuthorityTrust")
            .field("issuer", &self.issuer)
            .field("audience", &self.audience)
            .field("key_count", &self.keys.len())
            .finish()
    }
}

/// Strict verifier which durably consumes a sequence before returning success.
pub struct WorkLeaseAuthorizationVerifier<S> {
    trust: WorkLeaseAuthorityTrust,
    store: S,
}

impl<S: AcceptedSequenceStore> WorkLeaseAuthorizationVerifier<S> {
    #[must_use]
    pub const fn new(trust: WorkLeaseAuthorityTrust, store: S) -> Self {
        Self { trust, store }
    }

    pub fn verify_start(
        &mut self,
        grant: &WorkerLeaseGrant,
        context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        self.verify(
            "start",
            grant.challenge_id(),
            grant.authorizationless(),
            grant.authorization(),
            context,
        )
    }

    pub fn verify_renewal(
        &mut self,
        renewal: &WorkerLeaseRenewal,
        challenge_id: &str,
        context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        self.verify(
            "renew",
            challenge_id,
            renewal.authorizationless(),
            renewal.authorization(),
            context,
        )
    }

    fn verify(
        &mut self,
        operation: &str,
        active_challenge_id: &str,
        request: impl Serialize,
        compact_jws: &str,
        context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        let verified = verify_jws(
            &self.trust,
            operation,
            active_challenge_id,
            request,
            compact_jws,
            context,
        )?;
        let maybe_current = self.store.load(&verified.key_id)?;
        if maybe_current.is_some_and(|current| verified.sequence <= current) {
            return Err(LeaseAuthorizationError::Replay);
        }
        match self
            .store
            .compare_and_store(&verified.key_id, maybe_current, verified.sequence)?
        {
            SequenceStoreResult::Committed => Ok(()),
            SequenceStoreResult::Stale | SequenceStoreResult::AlreadyCommitted => {
                Err(LeaseAuthorizationError::Replay)
            }
        }
    }
}

impl<S: AcceptedSequenceStore> crate::session::LeaseAuthorizationVerifier
    for WorkLeaseAuthorizationVerifier<S>
{
    fn authorization_high_water_fingerprint(
        &self,
    ) -> Result<Option<crate::StateFingerprint>, LeaseAuthorizationError> {
        self.store.authorization_high_water_fingerprint()
    }

    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.store.mark_effect_pending()
    }

    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.store.clear_effect_pending()
    }

    fn verify_start(
        &mut self,
        grant: &WorkerLeaseGrant,
        context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        WorkLeaseAuthorizationVerifier::verify_start(self, grant, context)
    }

    fn verify_renewal(
        &mut self,
        renewal: &WorkerLeaseRenewal,
        challenge_id: &str,
        context: &WorkerLeaseAuthorizationContext,
    ) -> Result<(), LeaseAuthorizationError> {
        WorkLeaseAuthorizationVerifier::verify_renewal(self, renewal, challenge_id, context)
    }
}

struct VerifiedAuthorization {
    key_id: String,
    sequence: u64,
}

fn verify_jws(
    trust: &WorkLeaseAuthorityTrust,
    operation: &str,
    active_challenge_id: &str,
    request: impl Serialize,
    compact_jws: &str,
    context: &WorkerLeaseAuthorizationContext,
) -> Result<VerifiedAuthorization, LeaseAuthorizationError> {
    if compact_jws.is_empty() || compact_jws.len() > MAXIMUM_AUTHORIZATION_BYTES {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    }
    let mut segments = compact_jws.split('.');
    let (Some(protected), Some(payload), Some(signature), None) = (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
    ) else {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    };
    let protected_bytes = canonical_segment(protected, 512)?;
    let payload_bytes = canonical_segment(payload, 512)?;
    let signature_bytes = canonical_segment(signature, 86)?;
    if signature_bytes.len() != 64 {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    }
    let header: AuthorizationHeader = serde_json::from_slice(&protected_bytes)
        .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?;
    let claims: AuthorizationClaims = serde_json::from_slice(&payload_bytes)
        .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?;
    if header.algorithm != "Ed25519"
        || header.type_ != AUTHORIZATION_TYPE
        || !key_id(&header.key_id)
        || claims.operation != operation
        || !digest_text(&claims.request_sha256)
        || claims.control_session_binding_sha256 != context.binding()
        || canonical_json(&header)
            .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?
            .as_bytes()
            != protected_bytes
        || canonical_json(&claims)
            .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?
            .as_bytes()
            != payload_bytes
    {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    }
    let sequence = canonical_sequence(&claims.sequence)?;
    let digest_input = Zeroizing::new(
        serde_json::to_string(&AuthorizationDigestInput {
            active_challenge_id,
            audience: &trust.audience,
            issuer: &trust.issuer,
            operation,
            profile: AUTHORIZATION_PROFILE,
            request,
        })
        .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?,
    );
    let request_sha256 = base64_url(Sha256::digest(digest_input.as_bytes()));
    if claims.request_sha256 != request_sha256 {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    }
    let trusted = trust
        .keys
        .iter()
        .find(|key| key.key_id == header.key_id)
        .ok_or(LeaseAuthorizationError::InvalidAuthorization)?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?;
    let signing_input = Zeroizing::new(format!("{protected}.{payload}"));
    trusted
        .key
        .verify_strict(signing_input.as_bytes(), &signature)
        .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?;
    Ok(VerifiedAuthorization {
        key_id: header.key_id,
        sequence,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizationDigestInput<'a, R> {
    active_challenge_id: &'a str,
    audience: &'a str,
    issuer: &'a str,
    operation: &'a str,
    profile: &'static str,
    request: R,
}

fn canonical_segment(value: &str, maximum: usize) -> Result<Vec<u8>, LeaseAuthorizationError> {
    if value.is_empty() || value.len() > maximum || value.contains('=') {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?;
    if URL_SAFE_NO_PAD.encode(&bytes) != value {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    }
    Ok(bytes)
}

fn parse_keys(keys: Vec<WirePublicKey>) -> Result<Vec<TrustedKey>, LeaseAuthorizationError> {
    if keys.is_empty() || keys.len() > 8 {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    }
    let mut key_ids = HashSet::new();
    let mut public_keys = HashSet::new();
    let parsed = keys
        .into_iter()
        .map(|wire| {
            if !key_id(&wire.key_id)
                || wire.key_type != "OKP"
                || wire.curve != "Ed25519"
                || wire.algorithm != "Ed25519"
                || wire.use_ != "sig"
                || wire.key_operations != ["verify"]
            {
                return Err(LeaseAuthorizationError::InvalidAuthorization);
            }
            let bytes = canonical_segment(&wire.public_key, 43)?;
            let public_bytes: [u8; 32] = bytes
                .try_into()
                .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?;
            let key = VerifyingKey::from_bytes(&public_bytes)
                .map_err(|_| LeaseAuthorizationError::InvalidAuthorization)?;
            let torsion_free = CompressedEdwardsY(public_bytes)
                .decompress()
                .is_some_and(|point| point.is_torsion_free());
            if key.is_weak()
                || !torsion_free
                || !key_ids.insert(wire.key_id.clone())
                || !public_keys.insert(public_bytes)
            {
                return Err(LeaseAuthorizationError::InvalidAuthorization);
            }
            Ok(TrustedKey {
                key_id: wire.key_id,
                public_bytes,
                key,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parsed)
}

fn canonical_sequence(value: &str) -> Result<u64, LeaseAuthorizationError> {
    if value.is_empty()
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(LeaseAuthorizationError::InvalidAuthorization);
    }
    value
        .parse::<u64>()
        .ok()
        .filter(|sequence| *sequence > 0)
        .ok_or(LeaseAuthorizationError::InvalidAuthorization)
}

fn key_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || [b'.', b'_', b'-'].contains(&byte))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireDeploymentTrust {
    profile: String,
    update_authority: WireUpdateAuthority,
    work_lease_authority: WireWorkLeaseAuthority,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireUpdateAuthority {
    issuer: String,
    audience: String,
    role: String,
    keys: Vec<WirePublicKey>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireWorkLeaseAuthority {
    profile: String,
    issuer: String,
    audience: String,
    role: String,
    keys: Vec<WirePublicKey>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WirePublicKey {
    #[serde(rename = "kid")]
    key_id: String,
    #[serde(rename = "kty")]
    key_type: String,
    #[serde(rename = "crv")]
    curve: String,
    #[serde(rename = "x")]
    public_key: String,
    #[serde(rename = "alg")]
    algorithm: String,
    #[serde(rename = "use")]
    use_: String,
    #[serde(rename = "key_ops")]
    key_operations: Vec<String>,
}

#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct AuthorizationHeader {
    #[serde(rename = "alg")]
    algorithm: String,
    #[serde(rename = "kid")]
    key_id: String,
    #[serde(rename = "typ")]
    type_: String,
}

#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AuthorizationClaims {
    control_session_binding_sha256: String,
    operation: String,
    request_sha256: String,
    sequence: String,
}
