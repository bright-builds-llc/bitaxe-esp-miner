use super::*;

#[test]
fn missing_reference_guard_failure_blocks_report_output() {
    // Arrange
    let env = FakeEnvironment::failing_guard("reference missing or not initialized");
    let request = ReportRequest {
        checklist: Utf8PathBuf::from("docs/parity/checklist.md"),
        format: ReportFormat::Text,
        fail_on_invalid_verified: true,
    };

    // Act
    let result = run_report(&request, &env);

    // Assert
    assert!(result.is_err());
    assert!(result
        .expect_err("report should fail")
        .to_string()
        .contains("reference missing"));
    assert!(!env.read_called.get());
}

#[test]
fn dirty_reference_guard_failure_blocks_report_output() {
    // Arrange
    let env = FakeEnvironment::failing_guard("reference dirty");
    let request = ReportRequest {
        checklist: Utf8PathBuf::from("docs/parity/checklist.md"),
        format: ReportFormat::Text,
        fail_on_invalid_verified: true,
    };

    // Act
    let result = run_report(&request, &env);

    // Assert
    assert!(result.is_err());
    assert!(result
        .expect_err("report should fail")
        .to_string()
        .contains("reference dirty"));
    assert!(!env.read_called.get());
}
