//! Versioned, redaction-safe runtime proof for late post-flash observation.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::boot_identity::ResetReasonCategory;

mod accumulator;
pub use accumulator::RuntimeAttestationAccumulator;

/// Stable marker that begins every runtime boot attestation.
pub const RUNTIME_BOOT_ATTESTATION_MARKER: &str = "runtime_boot_attestation";
/// Current runtime boot attestation wire schema.
pub const RUNTIME_BOOT_ATTESTATION_SCHEMA_VERSION: u32 = 1;

const BOARD: &str = "205";
const ASIC: &str = "BM1366";
const MINING_STATE: &str = "disabled";
const WORK_SUBMISSION_STATE: &str = "disabled";
const HARDWARE_CONTROL_STATE: &str = "disabled";
const OTA_BOOT_VALIDATION_STATE: &str = "complete";
const SPIFFS_MOUNT_STATE: &str = "available";
const API_ROUTE_SHELL_STATE: &str = "started";

/// Exact package identity admitted before a flash operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedRuntimeAttestationIdentity {
    /// Full source commit from the admitted package manifest.
    pub firmware_commit: String,
    /// Full pinned reference commit from the admitted package manifest.
    pub reference_commit: String,
    /// Full application ELF SHA-256 from the admitted package manifest.
    pub app_elf_sha256: String,
}

/// One ready-state sample from the running firmware.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBootAttestation {
    session: String,
    boot_ordinal: u64,
    reset_reason: ResetReasonCategory,
    uptime_ms: u64,
    firmware_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    esp_idf_version: String,
}

impl RuntimeBootAttestation {
    /// Creates a validated sample for firmware publication.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session: &str,
        boot_ordinal: u64,
        reset_reason: ResetReasonCategory,
        uptime_ms: u64,
        firmware_commit: &str,
        reference_commit: &str,
        app_elf_sha256: &str,
        esp_idf_version: &str,
    ) -> Result<Self, RuntimeBootAttestationError> {
        validate_lower_hex("session", session, 32)?;
        if boot_ordinal == 0 {
            return Err(RuntimeBootAttestationError::InvalidField("boot_ordinal"));
        }
        validate_lower_hex("firmware_commit", firmware_commit, 40)?;
        validate_lower_hex("reference_commit", reference_commit, 40)?;
        validate_lower_hex("app_elf_sha256", app_elf_sha256, 64)?;
        validate_token("esp_idf_version", esp_idf_version)?;

        Ok(Self {
            session: session.to_owned(),
            boot_ordinal,
            reset_reason,
            uptime_ms,
            firmware_commit: firmware_commit.to_owned(),
            reference_commit: reference_commit.to_owned(),
            app_elf_sha256: app_elf_sha256.to_owned(),
            esp_idf_version: esp_idf_version.to_owned(),
        })
    }

    /// Parses one marker, tolerating a logger prefix before the stable marker.
    pub fn parse(line: &str) -> Result<Self, RuntimeBootAttestationError> {
        let marker_start = line
            .find(RUNTIME_BOOT_ATTESTATION_MARKER)
            .ok_or(RuntimeBootAttestationError::MissingMarker)?;
        let marker = &line[marker_start..];
        let mut tokens = marker.split_whitespace();
        if tokens.next() != Some(RUNTIME_BOOT_ATTESTATION_MARKER) {
            return Err(RuntimeBootAttestationError::MissingMarker);
        }

        let mut fields = BTreeMap::new();
        for token in tokens {
            let Some((key, value)) = token.split_once('=') else {
                return Err(RuntimeBootAttestationError::MalformedToken);
            };
            if fields.insert(key, value).is_some() {
                return Err(RuntimeBootAttestationError::DuplicateField);
            }
        }
        for key in fields.keys() {
            if !is_known_field(key) {
                return Err(RuntimeBootAttestationError::UnknownField);
            }
        }

        require_exact(&fields, "schema_version", "1")?;
        require_exact(&fields, "board", BOARD)?;
        require_exact(&fields, "asic", ASIC)?;
        require_exact(&fields, "mining", MINING_STATE)?;
        require_exact(&fields, "work_submission", WORK_SUBMISSION_STATE)?;
        require_exact(&fields, "hardware_control", HARDWARE_CONTROL_STATE)?;
        require_readiness(&fields, "ota_boot_validation", OTA_BOOT_VALIDATION_STATE)?;
        require_readiness(&fields, "spiffs_mount", SPIFFS_MOUNT_STATE)?;
        require_readiness(&fields, "api_route_shell", API_ROUTE_SHELL_STATE)?;
        require_exact(&fields, "redacted", "true")?;

        let reset_reason = parse_reset_reason(require(&fields, "reset_reason")?)?;
        let boot_ordinal = parse_u64(&fields, "boot_ordinal")?;
        let uptime_ms = parse_u64(&fields, "uptime_ms")?;
        Self::new(
            require(&fields, "session")?,
            boot_ordinal,
            reset_reason,
            uptime_ms,
            require(&fields, "firmware_commit")?,
            require(&fields, "reference_commit")?,
            require(&fields, "app_elf_sha256")?,
            require(&fields, "esp_idf_version")?,
        )
    }

    /// Formats the complete versioned marker.
    pub fn marker(&self) -> String {
        format!(
            "{RUNTIME_BOOT_ATTESTATION_MARKER} schema_version={RUNTIME_BOOT_ATTESTATION_SCHEMA_VERSION} session={} boot_ordinal={} reset_reason={} uptime_ms={} board={BOARD} asic={ASIC} mining={MINING_STATE} work_submission={WORK_SUBMISSION_STATE} hardware_control={HARDWARE_CONTROL_STATE} firmware_commit={} reference_commit={} app_elf_sha256={} esp_idf_version={} ota_boot_validation={OTA_BOOT_VALIDATION_STATE} spiffs_mount={SPIFFS_MOUNT_STATE} api_route_shell={API_ROUTE_SHELL_STATE} redacted=true",
            self.session,
            self.boot_ordinal,
            self.reset_reason.label(),
            self.uptime_ms,
            self.firmware_commit,
            self.reference_commit,
            self.app_elf_sha256,
            self.esp_idf_version,
        )
    }

    /// Full source commit attested by this sample.
    pub fn firmware_commit(&self) -> &str {
        &self.firmware_commit
    }

    /// Full reference commit attested by this sample.
    pub fn reference_commit(&self) -> &str {
        &self.reference_commit
    }

    fn same_session_and_ordinal(&self, other: &Self) -> bool {
        self.session == other.session && self.boot_ordinal == other.boot_ordinal
    }

    fn same_static_fields(&self, other: &Self) -> bool {
        self.reset_reason == other.reset_reason
            && self.firmware_commit == other.firmware_commit
            && self.reference_commit == other.reference_commit
            && self.app_elf_sha256 == other.app_elf_sha256
            && self.esp_idf_version == other.esp_idf_version
    }

    fn matches_expected(&self, expected: &ExpectedRuntimeAttestationIdentity) -> bool {
        self.firmware_commit == expected.firmware_commit
            && self.reference_commit == expected.reference_commit
            && self.app_elf_sha256 == expected.app_elf_sha256
    }
}

/// Closed result vocabulary for runtime attestation admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAttestationStatus {
    /// Two or more exact-package, monotonic samples were admitted.
    Trusted,
    /// No attestation marker was present.
    Missing,
    /// A marker was present but malformed or used an unsupported schema.
    Malformed,
    /// Fewer than two valid samples were present.
    InsufficientSamples,
    /// Samples did not belong to one session and boot ordinal.
    MixedSessionOrOrdinal,
    /// Samples disagreed on immutable identity or readiness facts.
    StaticFieldsMismatch,
    /// Sample uptime did not increase strictly in observation order.
    NonMonotonicUptime,
    /// The stable sample identity did not match the admitted package.
    PackageIdentityMismatch,
    /// A sample did not report the required ready-state categories.
    IncompleteReadiness,
}

impl RuntimeAttestationStatus {
    /// Returns the additive evidence-record label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Missing => "missing",
            Self::Malformed => "malformed",
            Self::InsufficientSamples => "insufficient_samples",
            Self::MixedSessionOrOrdinal => "mixed_session_or_ordinal",
            Self::StaticFieldsMismatch => "static_fields_mismatch",
            Self::NonMonotonicUptime => "non_monotonic_uptime",
            Self::PackageIdentityMismatch => "package_identity_mismatch",
            Self::IncompleteReadiness => "incomplete_readiness",
        }
    }
}

/// Pure classification of replayed attestations in one serial capture.
pub fn classify_runtime_boot_attestations(
    log: &str,
    expected: &ExpectedRuntimeAttestationIdentity,
) -> RuntimeAttestationStatus {
    let mut accumulator = RuntimeAttestationAccumulator::default();
    for line in log
        .lines()
        .filter(|line| line.contains(RUNTIME_BOOT_ATTESTATION_MARKER))
    {
        accumulator.observe_line(line);
    }
    accumulator.status(expected)
}

/// Parse and validation failures for one runtime attestation marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RuntimeBootAttestationError {
    /// The stable marker was absent.
    #[error("runtime boot attestation marker is missing")]
    MissingMarker,
    /// One whitespace-delimited field was not `key=value`.
    #[error("runtime boot attestation contains a malformed token")]
    MalformedToken,
    /// A key appeared more than once.
    #[error("runtime boot attestation contains a duplicate field")]
    DuplicateField,
    /// An unsupported key appeared.
    #[error("runtime boot attestation contains an unknown field")]
    UnknownField,
    /// A required key was absent.
    #[error("runtime boot attestation is missing a required field")]
    MissingField,
    /// A field was present but invalid.
    #[error("runtime boot attestation field {0} is invalid")]
    InvalidField(&'static str),
    /// A required ready-state category was absent or incomplete.
    #[error("runtime boot attestation readiness is incomplete")]
    IncompleteReadiness,
}

fn require<'a>(
    fields: &'a BTreeMap<&str, &str>,
    key: &'static str,
) -> Result<&'a str, RuntimeBootAttestationError> {
    fields
        .get(key)
        .copied()
        .ok_or(RuntimeBootAttestationError::MissingField)
}

fn require_exact(
    fields: &BTreeMap<&str, &str>,
    key: &'static str,
    expected: &str,
) -> Result<(), RuntimeBootAttestationError> {
    if require(fields, key)? == expected {
        return Ok(());
    }
    Err(RuntimeBootAttestationError::InvalidField(key))
}

fn require_readiness(
    fields: &BTreeMap<&str, &str>,
    key: &'static str,
    expected: &str,
) -> Result<(), RuntimeBootAttestationError> {
    if fields.get(key).copied() == Some(expected) {
        return Ok(());
    }
    Err(RuntimeBootAttestationError::IncompleteReadiness)
}

fn parse_u64(
    fields: &BTreeMap<&str, &str>,
    key: &'static str,
) -> Result<u64, RuntimeBootAttestationError> {
    require(fields, key)?
        .parse()
        .map_err(|_| RuntimeBootAttestationError::InvalidField(key))
}

fn parse_reset_reason(value: &str) -> Result<ResetReasonCategory, RuntimeBootAttestationError> {
    match value {
        "power_on" => Ok(ResetReasonCategory::PowerOn),
        "software_cpu" => Ok(ResetReasonCategory::SoftwareCpu),
        "watchdog" => Ok(ResetReasonCategory::Watchdog),
        "panic" => Ok(ResetReasonCategory::Panic),
        "brownout" => Ok(ResetReasonCategory::Brownout),
        "other" => Ok(ResetReasonCategory::Other),
        _ => Err(RuntimeBootAttestationError::InvalidField("reset_reason")),
    }
}

fn validate_lower_hex(
    field: &'static str,
    value: &str,
    expected_len: usize,
) -> Result<(), RuntimeBootAttestationError> {
    if value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(RuntimeBootAttestationError::InvalidField(field))
}

fn validate_token(field: &'static str, value: &str) -> Result<(), RuntimeBootAttestationError> {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'=')
    {
        return Ok(());
    }
    Err(RuntimeBootAttestationError::InvalidField(field))
}

fn is_known_field(key: &str) -> bool {
    matches!(
        key,
        "schema_version"
            | "session"
            | "boot_ordinal"
            | "reset_reason"
            | "uptime_ms"
            | "board"
            | "asic"
            | "mining"
            | "work_submission"
            | "hardware_control"
            | "firmware_commit"
            | "reference_commit"
            | "app_elf_sha256"
            | "esp_idf_version"
            | "ota_boot_validation"
            | "spiffs_mount"
            | "api_route_shell"
            | "redacted"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SOURCE: &str = "0123456789abcdef0123456789abcdef01234567";
    const REFERENCE: &str = "abcdef0123456789abcdef0123456789abcdef01";
    const APP_ELF: &str = "ca16ef5bd57d7e4b2f2f016ffb9236c426e68f16072bc1c5a53ef0e515f1d063";

    fn sample(uptime_ms: u64) -> RuntimeBootAttestation {
        RuntimeBootAttestation::new(
            SESSION,
            7,
            ResetReasonCategory::Other,
            uptime_ms,
            SOURCE,
            REFERENCE,
            APP_ELF,
            "v5.5.4",
        )
        .expect("fixture is valid")
    }

    fn expected() -> ExpectedRuntimeAttestationIdentity {
        ExpectedRuntimeAttestationIdentity {
            firmware_commit: SOURCE.to_owned(),
            reference_commit: REFERENCE.to_owned(),
            app_elf_sha256: APP_ELF.to_owned(),
        }
    }

    fn two_sample_log() -> String {
        format!(
            "I boot: {}\nI boot: {}\n",
            sample(10_000).marker(),
            sample(20_000).marker()
        )
    }

    #[test]
    fn marker_round_trips_through_logger_prefix() {
        // Arrange
        let original = sample(10_000);
        let line = format!("I (10000) bitaxe: {}", original.marker());

        // Act
        let parsed = RuntimeBootAttestation::parse(&line).expect("marker parses");

        // Assert
        assert_eq!(parsed, original);
    }

    #[test]
    fn two_exact_monotonic_samples_are_trusted() {
        // Arrange
        let log = two_sample_log();

        // Act
        let status = classify_runtime_boot_attestations(&log, &expected());

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::Trusted);
    }

    #[test]
    fn malformed_sample_is_rejected() {
        // Arrange
        let log = format!(
            "{}\n{} broken\n",
            sample(10_000).marker(),
            sample(20_000).marker()
        );

        // Act
        let status = classify_runtime_boot_attestations(&log, &expected());

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::Malformed);
    }

    #[test]
    fn stale_package_identity_is_rejected() {
        // Arrange
        let mut stale = expected();
        stale.firmware_commit = "1111111111111111111111111111111111111111".to_owned();

        // Act
        let status = classify_runtime_boot_attestations(&two_sample_log(), &stale);

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::PackageIdentityMismatch);
    }

    #[test]
    fn wrong_digest_is_rejected() {
        // Arrange
        let mut wrong_digest = expected();
        wrong_digest.app_elf_sha256 =
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned();

        // Act
        let status = classify_runtime_boot_attestations(&two_sample_log(), &wrong_digest);

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::PackageIdentityMismatch);
    }

    #[test]
    fn wrong_reference_commit_is_rejected() {
        // Arrange
        let mut wrong_reference = expected();
        wrong_reference.reference_commit = "1111111111111111111111111111111111111111".to_owned();

        // Act
        let status = classify_runtime_boot_attestations(&two_sample_log(), &wrong_reference);

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::PackageIdentityMismatch);
    }

    #[test]
    fn mixed_session_is_rejected() {
        // Arrange
        let other = RuntimeBootAttestation::new(
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            7,
            ResetReasonCategory::Other,
            20_000,
            SOURCE,
            REFERENCE,
            APP_ELF,
            "v5.5.4",
        )
        .expect("fixture is valid");
        let log = format!("{}\n{}\n", sample(10_000).marker(), other.marker());

        // Act
        let status = classify_runtime_boot_attestations(&log, &expected());

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::MixedSessionOrOrdinal);
    }

    #[test]
    fn mixed_boot_ordinal_is_rejected() {
        // Arrange
        let other = RuntimeBootAttestation::new(
            SESSION,
            8,
            ResetReasonCategory::Other,
            20_000,
            SOURCE,
            REFERENCE,
            APP_ELF,
            "v5.5.4",
        )
        .expect("fixture is valid");
        let log = format!("{}\n{}\n", sample(10_000).marker(), other.marker());

        // Act
        let status = classify_runtime_boot_attestations(&log, &expected());

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::MixedSessionOrOrdinal);
    }

    #[test]
    fn non_monotonic_uptime_is_rejected() {
        // Arrange
        let log = format!("{}\n{}\n", sample(20_000).marker(), sample(10_000).marker());

        // Act
        let status = classify_runtime_boot_attestations(&log, &expected());

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::NonMonotonicUptime);
    }

    #[test]
    fn one_sample_is_rejected() {
        // Arrange
        let log = sample(10_000).marker();

        // Act
        let status = classify_runtime_boot_attestations(&log, &expected());

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::InsufficientSamples);
    }

    #[test]
    fn incomplete_readiness_is_rejected() {
        // Arrange
        let log = two_sample_log().replace("spiffs_mount=available", "spiffs_mount=unavailable");

        // Act
        let status = classify_runtime_boot_attestations(&log, &expected());

        // Assert
        assert_eq!(status, RuntimeAttestationStatus::IncompleteReadiness);
    }
}
