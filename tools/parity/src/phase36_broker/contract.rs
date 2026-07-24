use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase36AllowedOperation {
    ExactPackageAdmission,
    Board205DetectorProbe,
    ExactPackageFlash,
    PassiveSerialObservation,
    ReadOnlySystemInfo,
    ReadOnlyWebSocket,
    ReadOnlyRetainedFacts,
    TypedRecovery,
    Cleanup,
}

impl Phase36AllowedOperation {
    pub const SUCCESS_ORDER: [Self; 8] = [
        Self::ExactPackageAdmission,
        Self::Board205DetectorProbe,
        Self::ExactPackageFlash,
        Self::PassiveSerialObservation,
        Self::ReadOnlySystemInfo,
        Self::ReadOnlyWebSocket,
        Self::ReadOnlyRetainedFacts,
        Self::Cleanup,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase36BrokerFailure {
    AdmissionFailed,
    DetectorFailed,
    FlashFailed,
    CaptureFailed,
    RecoveryFailed,
    CleanupFailed,
}

impl Phase36BrokerFailure {
    pub(crate) const fn valid_for(self, operation: Phase36AllowedOperation) -> bool {
        matches!(
            (self, operation),
            (
                Self::AdmissionFailed,
                Phase36AllowedOperation::ExactPackageAdmission
            ) | (
                Self::DetectorFailed,
                Phase36AllowedOperation::Board205DetectorProbe
            ) | (
                Self::FlashFailed,
                Phase36AllowedOperation::ExactPackageFlash
            ) | (
                Self::CaptureFailed,
                Phase36AllowedOperation::PassiveSerialObservation
                    | Phase36AllowedOperation::ReadOnlySystemInfo
                    | Phase36AllowedOperation::ReadOnlyWebSocket
                    | Phase36AllowedOperation::ReadOnlyRetainedFacts
            ) | (Self::RecoveryFailed, Phase36AllowedOperation::TypedRecovery)
                | (Self::CleanupFailed, Phase36AllowedOperation::Cleanup)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Phase36CapabilityScope {
    attempt_ordinal: u32,
    source_identity_digest: String,
    evaluator_identity_digest: String,
    package_identity_digest: String,
    peer_identity_digest: String,
    protected_root_identity_digest: String,
}

impl Phase36CapabilityScope {
    pub fn new(
        attempt_ordinal: u32,
        source_identity_digest: String,
        evaluator_identity_digest: String,
        package_identity_digest: String,
        peer_identity_digest: String,
        protected_root_identity_digest: String,
    ) -> Result<Self, Phase36CapabilityError> {
        if attempt_ordinal == 0 {
            return Err(Phase36CapabilityError::WrongAttempt);
        }
        for digest in [
            &source_identity_digest,
            &evaluator_identity_digest,
            &package_identity_digest,
            &peer_identity_digest,
            &protected_root_identity_digest,
        ] {
            if !valid_digest(digest) {
                return Err(Phase36CapabilityError::InvalidDigest);
            }
        }
        Ok(Self {
            attempt_ordinal,
            source_identity_digest,
            evaluator_identity_digest,
            package_identity_digest,
            peer_identity_digest,
            protected_root_identity_digest,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Phase36BrokerCapability {
    schema_version: String,
    scope: Phase36CapabilityScope,
    nonce_digest: String,
    issued_at_millis: u64,
    expires_at_millis: u64,
    capability_digest: String,
}

impl Phase36BrokerCapability {
    pub fn issue(
        scope: Phase36CapabilityScope,
        nonce_digest: String,
        issued_at_millis: u64,
        expires_at_millis: u64,
    ) -> Result<Self, Phase36CapabilityError> {
        if !valid_digest(&nonce_digest) {
            return Err(Phase36CapabilityError::InvalidDigest);
        }
        if issued_at_millis == 0 || expires_at_millis <= issued_at_millis {
            return Err(Phase36CapabilityError::InvalidLifetime);
        }
        let capability_digest =
            capability_digest(&scope, &nonce_digest, issued_at_millis, expires_at_millis)?;
        Ok(Self {
            schema_version: "phase36-broker-capability-v1".to_owned(),
            scope,
            nonce_digest,
            issued_at_millis,
            expires_at_millis,
            capability_digest,
        })
    }

    #[must_use]
    pub fn presentation(&self) -> Phase36CapabilityPresentation {
        Phase36CapabilityPresentation {
            schema_version: self.schema_version.clone(),
            attempt_ordinal: self.scope.attempt_ordinal,
            source_identity_digest: self.scope.source_identity_digest.clone(),
            evaluator_identity_digest: self.scope.evaluator_identity_digest.clone(),
            package_identity_digest: self.scope.package_identity_digest.clone(),
            peer_identity_digest: self.scope.peer_identity_digest.clone(),
            protected_root_identity_digest: self.scope.protected_root_identity_digest.clone(),
            nonce_digest: self.nonce_digest.clone(),
            issued_at_millis: self.issued_at_millis,
            expires_at_millis: self.expires_at_millis,
            capability_digest: self.capability_digest.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Phase36CapabilityPresentation {
    pub(crate) schema_version: String,
    pub(crate) attempt_ordinal: u32,
    pub(crate) source_identity_digest: String,
    pub(crate) evaluator_identity_digest: String,
    pub(crate) package_identity_digest: String,
    pub(crate) peer_identity_digest: String,
    pub(crate) protected_root_identity_digest: String,
    pub(crate) nonce_digest: String,
    pub(crate) issued_at_millis: u64,
    pub(crate) expires_at_millis: u64,
    pub(crate) capability_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase36ValidatedCapability {
    scope: Phase36CapabilityScope,
    capability_digest: String,
}

impl Phase36ValidatedCapability {
    #[must_use]
    pub const fn attempt_ordinal(&self) -> u32 {
        self.scope.attempt_ordinal
    }

    #[must_use]
    pub fn capability_digest(&self) -> &str {
        &self.capability_digest
    }
}

#[derive(Debug)]
pub struct Phase36CapabilityGuard {
    capability: Phase36BrokerCapability,
    consumed: bool,
}

impl Phase36CapabilityGuard {
    #[must_use]
    pub const fn new(capability: Phase36BrokerCapability) -> Self {
        Self {
            capability,
            consumed: false,
        }
    }

    pub fn admit(
        &mut self,
        presentation: &Phase36CapabilityPresentation,
        now_millis: u64,
    ) -> Result<Phase36ValidatedCapability, Phase36CapabilityError> {
        if self.consumed {
            return Err(Phase36CapabilityError::Replay);
        }
        if now_millis < self.capability.issued_at_millis {
            return Err(Phase36CapabilityError::NotYetValid);
        }
        if now_millis > self.capability.expires_at_millis {
            return Err(Phase36CapabilityError::Expired);
        }
        if presentation.schema_version != self.capability.schema_version {
            return Err(Phase36CapabilityError::UnsupportedSchema);
        }
        if presentation.attempt_ordinal != self.capability.scope.attempt_ordinal {
            return Err(Phase36CapabilityError::WrongAttempt);
        }
        if presentation.source_identity_digest != self.capability.scope.source_identity_digest {
            return Err(Phase36CapabilityError::WrongSource);
        }
        if presentation.evaluator_identity_digest != self.capability.scope.evaluator_identity_digest
        {
            return Err(Phase36CapabilityError::WrongEvaluator);
        }
        if presentation.package_identity_digest != self.capability.scope.package_identity_digest {
            return Err(Phase36CapabilityError::WrongPackage);
        }
        if presentation.peer_identity_digest != self.capability.scope.peer_identity_digest {
            return Err(Phase36CapabilityError::WrongPeer);
        }
        if presentation.protected_root_identity_digest
            != self.capability.scope.protected_root_identity_digest
        {
            return Err(Phase36CapabilityError::WrongProtectedRoot);
        }
        if presentation.nonce_digest != self.capability.nonce_digest
            || presentation.issued_at_millis != self.capability.issued_at_millis
            || presentation.expires_at_millis != self.capability.expires_at_millis
            || presentation.capability_digest != self.capability.capability_digest
        {
            return Err(Phase36CapabilityError::InvalidCapability);
        }
        let recomputed = capability_digest(
            &self.capability.scope,
            &self.capability.nonce_digest,
            self.capability.issued_at_millis,
            self.capability.expires_at_millis,
        )?;
        if recomputed != self.capability.capability_digest {
            return Err(Phase36CapabilityError::InvalidCapability);
        }

        self.consumed = true;
        Ok(Phase36ValidatedCapability {
            scope: self.capability.scope.clone(),
            capability_digest: self.capability.capability_digest.clone(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Phase36CapabilityError {
    #[error("phase36_capability_unsupported_schema")]
    UnsupportedSchema,
    #[error("phase36_capability_invalid_digest")]
    InvalidDigest,
    #[error("phase36_capability_invalid_lifetime")]
    InvalidLifetime,
    #[error("phase36_capability_not_yet_valid")]
    NotYetValid,
    #[error("phase36_capability_expired")]
    Expired,
    #[error("phase36_capability_replay")]
    Replay,
    #[error("phase36_capability_wrong_peer")]
    WrongPeer,
    #[error("phase36_capability_wrong_attempt")]
    WrongAttempt,
    #[error("phase36_capability_wrong_source")]
    WrongSource,
    #[error("phase36_capability_wrong_evaluator")]
    WrongEvaluator,
    #[error("phase36_capability_wrong_package")]
    WrongPackage,
    #[error("phase36_capability_wrong_protected_root")]
    WrongProtectedRoot,
    #[error("phase36_capability_invalid")]
    InvalidCapability,
    #[error("phase36_capability_encoding_failed")]
    EncodingFailed,
}

fn capability_digest(
    scope: &Phase36CapabilityScope,
    nonce_digest: &str,
    issued_at_millis: u64,
    expires_at_millis: u64,
) -> Result<String, Phase36CapabilityError> {
    let bytes = serde_json::to_vec(&(
        "phase36-broker-capability-v1",
        scope,
        nonce_digest,
        issued_at_millis,
        expires_at_millis,
    ))
    .map_err(|_| Phase36CapabilityError::EncodingFailed)?;
    Ok(sha256_hex(&bytes))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("formatting into String cannot fail");
    }
    output
}
