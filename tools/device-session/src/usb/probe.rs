use super::*;

#[test]
fn flash_read_device_loss_has_a_closed_transport_category() {
    // Arrange
    let output = SupervisedOutput {
        termination: SupervisedTermination::ExitedFailure,
        stdout: Vec::new(),
        stderr: b"A fatal error occurred: read failed: Device not configured".to_vec(),
    };

    // Act
    let category = classify_probe_failure(&["read_flash".to_owned()], &output);

    // Assert
    assert_eq!(category, UsbTerminalCategory::UsbEnumerationLost);
}
