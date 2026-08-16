use super::*;

#[test]
fn invalid_runtime_attestation_encoding_is_independent_of_valid_marker() {
    // Arrange
    let mut bytes = bitaxe_api::RUNTIME_BOOT_ATTESTATION_MARKER
        .as_bytes()
        .to_vec();
    bytes.extend_from_slice(&[b' ', 0xff, b'\n']);
    bytes.extend_from_slice(&observation_marker(CAMPAIGN_MARKER_SCHEMA));

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(capture.markers.len(), 1);
    assert_eq!(capture.outcome_detail, CampaignSerialOutcomeDetail::Clean);
    assert_eq!(
        capture
            .diagnostics
            .runtime_attestation_invalid_encoding_count,
        1
    );
}

#[test]
fn repeated_runtime_attestations_remain_trusted_after_the_old_text_cap() {
    // Arrange
    const SESSION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SOURCE: &str = "0123456789abcdef0123456789abcdef01234567";
    const REFERENCE: &str = "abcdef0123456789abcdef0123456789abcdef01";
    const APP_ELF: &str = "ca16ef5bd57d7e4b2f2f016ffb9236c426e68f16072bc1c5a53ef0e515f1d063";
    let expected = ExpectedRuntimeAttestationIdentity {
        firmware_commit: SOURCE.to_owned(),
        reference_commit: REFERENCE.to_owned(),
        app_elf_sha256: APP_ELF.to_owned(),
    };
    let mut analyzer = CampaignSerialAnalyzer::new(observation_admission());

    // Act
    for ordinal in 1..=182_u64 {
        let attestation = bitaxe_api::RuntimeBootAttestation::new(
            SESSION,
            7,
            bitaxe_api::boot_identity::ResetReasonCategory::Other,
            ordinal.saturating_mul(10_000),
            SOURCE,
            REFERENCE,
            APP_ELF,
            "v5.5.4",
        )
        .expect("fixture attestation");
        analyzer.observe_chunk(format!("I boot: {}\n", attestation.marker()).as_bytes());
    }
    let capture = analyzer.finish();

    // Assert
    assert_eq!(
        capture.runtime_attestation_status(&expected),
        RuntimeAttestationStatus::Trusted
    );
    assert_eq!(capture.diagnostics.runtime_attestation_candidate_count, 182);
}

#[test]
fn firmware_unavailable_diagnostic_does_not_poison_valid_attestations() {
    // Arrange
    const SESSION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SOURCE: &str = "0123456789abcdef0123456789abcdef01234567";
    const REFERENCE: &str = "abcdef0123456789abcdef0123456789abcdef01";
    const APP_ELF: &str = "ca16ef5bd57d7e4b2f2f016ffb9236c426e68f16072bc1c5a53ef0e515f1d063";
    let expected = ExpectedRuntimeAttestationIdentity {
        firmware_commit: SOURCE.to_owned(),
        reference_commit: REFERENCE.to_owned(),
        app_elf_sha256: APP_ELF.to_owned(),
    };
    let first = bitaxe_api::RuntimeBootAttestation::new(
        SESSION,
        7,
        bitaxe_api::boot_identity::ResetReasonCategory::Other,
        10_000,
        SOURCE,
        REFERENCE,
        APP_ELF,
        "v5.5.4",
    )
    .expect("first attestation");
    let second = bitaxe_api::RuntimeBootAttestation::new(
        SESSION,
        7,
        bitaxe_api::boot_identity::ResetReasonCategory::Other,
        20_000,
        SOURCE,
        REFERENCE,
        APP_ELF,
        "v5.5.4",
    )
    .expect("second attestation");
    let bytes = format!(
        "I boot: {}\nW boot: runtime_boot_attestation=unavailable reason=invalid_identity\nI boot: {}\n",
        first.marker(),
        second.marker()
    );

    // Act
    let capture = analyze_campaign_serial_bytes(bytes.as_bytes(), observation_admission());

    // Assert
    assert_eq!(
        capture.runtime_attestation_status(&expected),
        RuntimeAttestationStatus::Trusted
    );
    assert_eq!(capture.diagnostics.runtime_attestation_candidate_count, 2);
    assert_eq!(capture.diagnostics.runtime_attestation_lookalike_count, 1);
    assert_eq!(
        capture.diagnostics.runtime_attestation_parse_failure,
        "none"
    );
}

#[test]
fn genuine_marker_parse_failure_keeps_closed_discriminator() {
    // Arrange
    let bytes = b"I boot: runtime_boot_attestation broken\n";

    // Act
    let capture = analyze_campaign_serial_bytes(bytes, observation_admission());

    // Assert
    assert_eq!(capture.diagnostics.runtime_attestation_candidate_count, 1);
    assert_eq!(capture.diagnostics.runtime_attestation_lookalike_count, 0);
    assert_eq!(
        capture.diagnostics.runtime_attestation_parse_failure,
        "malformed_token"
    );
    assert_eq!(
        capture
            .diagnostics
            .runtime_attestation_parse_failure_counts
            .malformed_token,
        1
    );
}
