use super::*;

#[test]
fn manifest_v4_rejects_wrong_factory_artifact_name() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4_with_factory_artifact(&dir, DEFAULT_ELF_NAME, "wrong.bin");
    let command = FlashCommand {
        factory_reset: false,
        common: common_args(),
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains(FACTORY_IMAGE_NAME));
    assert!(error.contains("wrong.bin"));
}

#[test]
fn zero_ports_error_includes_actionable_example() {
    // Arrange
    let environment = FakeFlashEnvironment::with_ports("");

    // Act
    let result = resolve_port(None, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("No serial ports found"));
    assert!(error.contains("--port /dev/"));
}

#[test]
fn ambiguous_ports_error_lists_each_candidate() {
    // Arrange
    let environment = FakeFlashEnvironment::with_ports(
        "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbserial-110 USB serial\n",
    );

    // Act
    let result = resolve_port(None, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("Ambiguous serial ports"));
    assert!(error.contains("--port /dev/cu.usbmodem101"));
    assert!(error.contains("--port /dev/cu.usbserial-110"));
}

#[test]
fn bare_com_is_not_a_likely_port() {
    // Arrange
    let port = "COM";

    // Act
    let likely = is_likely_port(port);

    // Assert
    assert!(!likely);
}

#[test]
fn numbered_com_is_a_likely_port() {
    // Arrange
    let port = "COM3";

    // Act
    let likely = is_likely_port(port);

    // Assert
    assert!(likely);
}

#[test]
fn evidence_monitor_command_is_receive_only() {
    // Arrange
    let common = common_args();
    let environment = FakeFlashEnvironment::default();

    // Act
    let command = prepare_evidence_monitor_command(&common, &environment).expect("command");

    // Assert
    assert_eq!(command.program, "bitaxe-receive-only");
    assert_eq!(
        command.args,
        vec!["observe", "--port", "/dev/cu.usbmodem101"]
    );
    assert!(!command.display().contains("espflash monitor"));
}

#[test]
fn routine_monitor_command_is_receive_only() {
    // Arrange
    let common = common_args();
    let environment = FakeFlashEnvironment::default();

    // Act
    let command = prepare_monitor_command(&common, &environment).expect("command");

    // Assert
    assert_eq!(
        command.args,
        vec!["observe", "--port", "/dev/cu.usbmodem101"]
    );
    assert_eq!(command.program, "bitaxe-receive-only");
    assert!(!command.display().contains("reset"));
}

#[test]
fn receive_only_console_frames_unterminated_serial_bytes() {
    // Arrange
    let mut output = Vec::new();

    // Act
    write_receive_only_console_to(&mut output, b"serial").expect("serial output");
    emit_line_to(&mut output, "usb_session", "ready").expect("ready marker");

    // Assert
    assert_eq!(output, b"serial\nusb_session: ready\n");
}

#[test]
fn receive_only_console_preserves_existing_newline() {
    // Arrange
    let mut output = Vec::new();

    // Act
    write_receive_only_console_to(&mut output, b"serial\n").expect("serial output");
    emit_line_to(&mut output, "usb_session", "ready").expect("ready marker");

    // Assert
    assert_eq!(output, b"serial\nusb_session: ready\n");
}

#[test]
fn receive_only_console_completes_trailing_carriage_return() {
    // Arrange
    let mut output = Vec::new();

    // Act
    write_receive_only_console_to(&mut output, b"serial\r").expect("serial output");
    emit_line_to(&mut output, "usb_session", "ready").expect("ready marker");

    // Assert
    assert_eq!(output, b"serial\r\nusb_session: ready\n");
}

#[test]
fn receive_only_console_keeps_empty_capture_at_line_start() {
    // Arrange
    let mut output = Vec::new();

    // Act
    write_receive_only_console_to(&mut output, b"").expect("empty serial output");
    emit_line_to(&mut output, "usb_session", "ready").expect("ready marker");

    // Assert
    assert_eq!(output, b"usb_session: ready\n");
}

#[test]
fn receive_only_console_frames_arbitrary_binary_bytes() {
    // Arrange
    let mut output = Vec::new();

    // Act
    write_receive_only_console_to(&mut output, &[0x00, 0xff, b'x']).expect("binary serial output");
    emit_line_to(&mut output, "usb_session", "ready").expect("ready marker");

    // Assert
    assert_eq!(
        output,
        [
            vec![0x00, 0xff, b'x', b'\n'],
            b"usb_session: ready\n".to_vec()
        ]
        .concat()
    );
}

#[test]
fn cleanup_success_cannot_replace_an_operation_failure() {
    // Arrange
    let operation_result = Err(anyhow::anyhow!("primary operation failure"));
    let cleanup_result = Ok(());

    // Act
    let error = combine_operation_and_cleanup(operation_result, cleanup_result)
        .expect_err("operation failure must remain terminal");

    // Assert
    assert_eq!(error.to_string(), "primary operation failure");
}

#[test]
fn cleanup_failure_is_secondary_to_an_operation_failure() {
    // Arrange
    let operation_result = Err(anyhow::anyhow!("primary operation failure"));
    let cleanup_result = Err(anyhow::anyhow!("cleanup failure"));

    // Act
    let error = combine_operation_and_cleanup(operation_result, cleanup_result)
        .expect_err("operation failure must remain terminal");

    // Assert
    assert_eq!(error.to_string(), "cleanup_failure=secondary");
    assert_eq!(
        error
            .chain()
            .nth(1)
            .map(std::string::ToString::to_string)
            .as_deref(),
        Some("primary operation failure")
    );
}
