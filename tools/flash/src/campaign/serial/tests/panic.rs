use super::*;

#[test]
fn fragmented_crlf_panic_lines_preserve_first_closed_signature() {
    // Arrange
    let mut analyzer = CampaignSerialAnalyzer::new(observation_admission());

    // Act
    analyzer.observe_chunk(b"***ERROR*** A stack overflow in task production-");
    analyzer
        .observe_chunk(b"mining-session has been detected.\r\nabort() was called at PC 0x1234\n");
    let capture = analyzer.finish();

    // Assert
    assert_eq!(capture.diagnostics.panic_signature, "stack_overflow");
    assert_eq!(
        capture.diagnostics.panic_task_family,
        "production_mining_session"
    );
    assert_eq!(capture.diagnostics.panic_signature_count, 2);
    assert_eq!(
        capture
            .diagnostics
            .events
            .iter()
            .filter(|event| event.kind == CampaignSerialEventKind::PanicSignatureObserved)
            .count(),
        2
    );
}

#[test]
fn panic_diagnostics_serialize_without_raw_line_task_or_address() {
    // Arrange
    let private_task = "private-task-identity";
    let private_address = "0x4feedcab";
    let bytes = format!(
        "***ERROR*** A stack overflow in task {private_task} has been detected at {private_address}.\n"
    );

    // Act
    let capture = analyze_campaign_serial_bytes(bytes.as_bytes(), observation_admission());
    let serialized = serde_json::to_string(&capture.diagnostics).expect("diagnostics serialize");

    // Assert
    assert!(serialized.contains("\"panic_signature\":\"stack_overflow\""));
    assert!(serialized.contains("\"panic_task_family\":\"other\""));
    assert!(!serialized.contains(private_task));
    assert!(!serialized.contains(private_address));
    assert!(!serialized.contains("stack overflow in task"));
}

#[test]
fn panic_reset_without_observed_signature_is_explicitly_unknown() {
    // Arrange
    const SOURCE: &str = "1111111111111111111111111111111111111111";
    const REFERENCE: &str = "2222222222222222222222222222222222222222";
    const APP: &str = "3333333333333333333333333333333333333333333333333333333333333333";
    let first = bitaxe_api::RuntimeBootAttestation::new(
        "44444444444444444444444444444444",
        1,
        bitaxe_api::boot_identity::ResetReasonCategory::PowerOn,
        10_000,
        SOURCE,
        REFERENCE,
        APP,
        "v5.5.4",
    )
    .expect("first attestation");
    let second = bitaxe_api::RuntimeBootAttestation::new(
        "55555555555555555555555555555555",
        2,
        bitaxe_api::boot_identity::ResetReasonCategory::Panic,
        1_000,
        SOURCE,
        REFERENCE,
        APP,
        "v5.5.4",
    )
    .expect("second attestation");
    let bytes = format!("{}\n{}\n", first.marker(), second.marker());

    // Act
    let capture = analyze_campaign_serial_bytes(bytes.as_bytes(), observation_admission());

    // Assert
    assert_eq!(capture.diagnostics.panic_signature, "unknown");
    assert_eq!(capture.diagnostics.panic_task_family, "none");
    assert_eq!(capture.diagnostics.panic_signature_count, 0);
    assert_eq!(
        capture.diagnostics.runtime_attestation_mixed_reset_reason,
        "panic"
    );
}
