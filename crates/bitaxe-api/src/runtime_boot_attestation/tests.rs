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
fn unavailable_diagnostic_is_not_a_stable_marker_candidate() {
    // Arrange
    let log = format!(
        "runtime_boot_attestation=unavailable reason=invalid_identity\n{}\n{}\n",
        sample(10_000).marker(),
        sample(20_000).marker()
    );

    // Act
    let status = classify_runtime_boot_attestations(&log, &expected());

    // Assert
    assert_eq!(status, RuntimeAttestationStatus::Trusted);
}

#[test]
fn embedded_marker_text_is_not_a_stable_marker_candidate() {
    // Arrange
    let log = format!(
        "diagnostic_runtime_boot_attestation broken\n{}\n{}\n",
        sample(10_000).marker(),
        sample(20_000).marker()
    );

    // Act
    let status = classify_runtime_boot_attestations(&log, &expected());

    // Assert
    assert_eq!(status, RuntimeAttestationStatus::Trusted);
}

#[test]
fn accumulator_counts_every_closed_parse_failure_without_values() {
    // Arrange
    let valid = sample(10_000).marker();
    let mut accumulator = RuntimeAttestationAccumulator::default();
    let candidates = [
        "runtime_boot_attestation=unavailable".to_owned(),
        format!("{valid} broken"),
        format!("{valid} schema_version=1"),
        format!("{valid} unsupported=value"),
        valid.replace(" redacted=true", ""),
        valid.replace("board=205", "board=999"),
        valid.replace("spiffs_mount=available", "spiffs_mount=unavailable"),
    ];

    // Act
    for candidate in candidates {
        accumulator.observe_line(&candidate);
    }
    let counts = accumulator.parse_failure_counts();

    // Assert
    assert_eq!(
        accumulator.maybe_first_parse_failure(),
        Some(RuntimeAttestationParseFailure::MissingMarker)
    );
    for failure in [
        RuntimeAttestationParseFailure::MissingMarker,
        RuntimeAttestationParseFailure::MalformedToken,
        RuntimeAttestationParseFailure::DuplicateField,
        RuntimeAttestationParseFailure::UnknownField,
        RuntimeAttestationParseFailure::MissingField,
        RuntimeAttestationParseFailure::InvalidField,
        RuntimeAttestationParseFailure::IncompleteReadiness,
    ] {
        assert_eq!(counts.count(failure), 1, "{}", failure.label());
    }
}

#[test]
fn parse_failure_counts_saturate() {
    // Arrange
    let mut counts = RuntimeAttestationParseFailureCounts {
        counts: [0, u64::MAX, 0, 0, 0, 0, 0],
    };

    // Act
    counts.record(RuntimeAttestationParseFailure::MalformedToken);

    // Assert
    assert_eq!(
        counts.count(RuntimeAttestationParseFailure::MalformedToken),
        u64::MAX
    );
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
