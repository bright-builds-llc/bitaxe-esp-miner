use super::line_admission::ReceiveLineAdmission;
use super::*;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[test]
fn lifecycle_reaches_reflash_ready() {
    // Arrange
    let events = [
        UsbLifecycleEvent::Admit,
        UsbLifecycleEvent::BeginFlash,
        UsbLifecycleEvent::FlashComplete,
        UsbLifecycleEvent::BeginCleanup,
        UsbLifecycleEvent::CleanupComplete,
    ];

    // Act
    let state = events
        .into_iter()
        .try_fold(UsbLifecycleState::Prepared, reduce_lifecycle)
        .expect("valid lifecycle should reduce");

    // Assert
    assert_eq!(state, UsbLifecycleState::ReflashReady);
}

#[test]
fn lifecycle_rejects_illegal_transition() {
    // Arrange
    let state = UsbLifecycleState::Prepared;

    // Act
    let result = reduce_lifecycle(state, UsbLifecycleEvent::FlashComplete);

    // Assert
    assert!(result.is_err());
}

#[test]
fn retry_requires_changed_enumeration_and_first_attempt() {
    // Arrange
    let context = RetryContext {
        category: UsbTerminalCategory::BootloaderConnectFailed,
        cleanup_complete: true,
        enumeration_changed: true,
        same_physical_device: true,
        immutable_operation: true,
        repeated_boundary: false,
        attempts: 1,
    };

    // Act
    let eligible = retry_is_eligible(context);

    // Assert
    assert!(eligible);
}

#[test]
fn retry_rejects_hardware_write_failure() {
    // Arrange
    let context = RetryContext {
        category: UsbTerminalCategory::FlashFailedAfterTransfer,
        cleanup_complete: true,
        enumeration_changed: true,
        same_physical_device: true,
        immutable_operation: true,
        repeated_boundary: false,
        attempts: 1,
    };

    // Act
    let eligible = retry_is_eligible(context);

    // Assert
    assert!(!eligible);
}

#[test]
fn bootloader_failure_without_changed_enumeration_recommends_connector_power_cycle() {
    // Arrange
    let context = RetryContext {
        category: UsbTerminalCategory::BootloaderConnectFailed,
        cleanup_complete: true,
        enumeration_changed: false,
        same_physical_device: true,
        immutable_operation: true,
        repeated_boundary: false,
        attempts: 1,
    };

    // Act
    let detail = ineligible_retry_detail(
        context,
        Some(EspflashConnectionSignature::DiagnosticUnavailable),
    );

    // Assert
    assert!(detail.contains("disconnect USB and normal device power"));
    assert!(detail.contains("wait 10 seconds"));
    assert!(detail.contains("reconnect normal power, then USB"));
    assert!(detail.contains("do not use pins, headers, or test points"));
}

#[test]
fn recovery_snapshot_rejects_identity_drift() {
    // Arrange
    let snapshot = UsbDeviceSnapshot {
        port: "/dev/private".to_owned(),
        physical_identity_digest: "different".to_owned(),
        enumeration_token: "epoch".to_owned(),
        accessible: true,
        holder_count: 0,
    };

    // Act
    let error = validate_recovery_snapshot(&snapshot, "expected")
        .expect_err("identity drift must fail closed");

    // Assert
    assert_eq!(error.category, UsbTerminalCategory::IdentityDrift);
}

#[test]
fn recovery_snapshot_rejects_foreign_holder() {
    // Arrange
    let snapshot = UsbDeviceSnapshot {
        port: "/dev/private".to_owned(),
        physical_identity_digest: "expected".to_owned(),
        enumeration_token: "epoch".to_owned(),
        accessible: true,
        holder_count: 1,
    };

    // Act
    let error = validate_recovery_snapshot(&snapshot, "expected")
        .expect_err("foreign holder must fail closed");

    // Assert
    assert_eq!(error.category, UsbTerminalCategory::ForeignHolder);
}

#[test]
fn successful_write_uses_the_extended_recovery_policy() {
    // Arrange
    let args = vec!["write-bin".to_owned()];

    // Act
    let policy = successful_command_recovery_policy(&args);

    // Assert
    assert_eq!(policy, RecoveryPhase::PostFlash);
}

#[test]
fn successful_large_erase_is_a_completed_flash_effect() {
    // Arrange
    let args = vec!["erase-flash".to_owned()];
    let output = SupervisedOutput {
        termination: SupervisedTermination::ExitedSuccess,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    // Act
    let policy = successful_command_recovery_policy(&args);
    let state = advance_device_effect_state(
        UsbDeviceEffectState::None,
        &args,
        &output,
        UsbWriteDialect::Espflash,
    );

    // Assert
    assert_eq!(policy, RecoveryPhase::PostFlash);
    assert_eq!(state, UsbDeviceEffectState::Completed);
}

#[test]
fn successful_probe_keeps_the_standard_recovery_policy() {
    // Arrange
    let args = vec!["board-info".to_owned()];

    // Act
    let policy = successful_command_recovery_policy(&args);

    // Assert
    assert_eq!(policy, RecoveryPhase::PostProbe);
}

#[test]
fn every_supervised_termination_has_one_success_and_failure_classification() {
    // Arrange
    let cases = [
        (
            SupervisedTermination::ExitedSuccess,
            true,
            UsbTerminalCategory::FlashFailedBeforeTransfer,
        ),
        (
            SupervisedTermination::ExitedFailure,
            false,
            UsbTerminalCategory::FlashFailedBeforeTransfer,
        ),
        (
            SupervisedTermination::TimedOut,
            false,
            UsbTerminalCategory::BootloaderConnectFailed,
        ),
        (
            SupervisedTermination::Interrupted {
                signal: libc::SIGTERM,
            },
            false,
            UsbTerminalCategory::BootloaderConnectFailed,
        ),
    ];

    for (termination, succeeded, category) in cases {
        let output = SupervisedOutput {
            termination,
            stdout: Vec::new(),
            stderr: Vec::new(),
        };

        // Act
        let observed_success = output.succeeded();
        let observed_category = classify_espflash_failure(&output);

        // Assert
        assert_eq!(observed_success, succeeded, "{termination:?}");
        assert_eq!(observed_category, category, "{termination:?}");
    }
}

#[test]
fn bootloader_diagnostic_classifies_production_shaped_debug_transcripts() {
    // Arrange
    let cases = [
        (
            SupervisedTermination::TimedOut,
            b"private-timeout-token".as_slice(),
            EspflashConnectionSignature::ProcessTimeout,
        ),
        (
            SupervisedTermination::Interrupted {
                signal: libc::SIGTERM,
            },
            b"private-interrupt-token".as_slice(),
            EspflashConnectionSignature::ProcessInterrupted,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"[DEBUG] Failed to reset, error Connection(\n    DeviceNotFound,\n), retrying",
            EspflashConnectionSignature::DeviceNotFound,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"[DEBUG] Failed to reset, error Connection(Serial(Error { kind: Io(Other), description: private write device path failed })), retrying",
            EspflashConnectionSignature::SerialResetIo,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"[DEBUG] Failed to reset, error Connection(WrongBootMode(\"0x8\")), retrying",
            EspflashConnectionSignature::WrongBootMode,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"[DEBUG] Failed to reset, error Connection(NoSyncReply), retrying",
            EspflashConnectionSignature::NoSyncReply,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"[DEBUG] Failed to reset, error Connection(FramingError), retrying",
            EspflashConnectionSignature::SlipFraming,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"[DEBUG] Failed to reset, error Connection(ReadMismatch(1024, 0)), retrying",
            EspflashConnectionSignature::ReadMismatch,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"[DEBUG] Failed to reset, error Connection(Timeout(Sync)), retrying",
            EspflashConnectionSignature::CommandTimeout,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"Error while connecting to device\nTimeout while running FlashDeflData command",
            EspflashConnectionSignature::FlashDefinitionDataTimeout,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"[DEBUG] Failed to reset, error Connection(ConnectionFailed), retrying",
            EspflashConnectionSignature::GenericConnectionFailure,
        ),
        (
            SupervisedTermination::ExitedFailure,
            b"error: failed to connect to the device private-final-token",
            EspflashConnectionSignature::DiagnosticUnavailable,
        ),
    ];

    for (termination, stderr, expected) in cases {
        let output = SupervisedOutput {
            termination,
            stdout: Vec::new(),
            stderr: stderr.to_vec(),
        };

        // Act
        let signature = classify_bootloader_diagnostic(&output);
        let category = classify_espflash_failure(&output);

        // Assert
        assert_eq!(signature, expected);
        assert_eq!(category, UsbTerminalCategory::BootloaderConnectFailed);
    }
}

#[test]
fn bootloader_diagnostic_public_detail_excludes_private_transcript() {
    // Arrange
    let context = RetryContext {
        category: UsbTerminalCategory::BootloaderConnectFailed,
        cleanup_complete: true,
        enumeration_changed: false,
        same_physical_device: true,
        immutable_operation: true,
        repeated_boundary: false,
        attempts: 1,
    };

    // Act
    let detail = ineligible_retry_detail(context, Some(EspflashConnectionSignature::SerialResetIo));

    // Assert
    assert!(detail.contains("connection_signature=serial_reset_io"));
    assert!(!detail.contains("private-device-path"));
}

#[test]
fn bootloader_diagnostic_logging_is_limited_to_detector_board_info() {
    // Arrange
    let board_info = vec!["board-info".to_owned()];
    let write_bin = vec!["write-bin".to_owned()];
    let version = vec!["--version".to_owned()];

    // Act
    let detect_board_info = espflash_diagnostic_filter(UsbOperation::Detect, &board_info);
    let flash_board_info = espflash_diagnostic_filter(UsbOperation::Flash, &board_info);
    let detect_write = espflash_diagnostic_filter(UsbOperation::Detect, &write_bin);
    let detect_version = espflash_diagnostic_filter(UsbOperation::Detect, &version);

    // Assert
    assert_eq!(detect_board_info, Some("espflash::connection=debug"));
    assert_eq!(flash_board_info, None);
    assert_eq!(detect_write, None);
    assert_eq!(detect_version, None);
}

#[test]
fn successful_flash_records_completed_device_effect() {
    // Arrange
    let output = SupervisedOutput {
        termination: SupervisedTermination::ExitedSuccess,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    // Act
    let state = advance_device_effect_state(
        UsbDeviceEffectState::None,
        &["write-bin".to_owned()],
        &output,
        UsbWriteDialect::Espflash,
    );

    // Assert
    assert_eq!(state, UsbDeviceEffectState::Completed);
}

#[test]
fn successful_board_info_does_not_claim_a_device_write() {
    // Arrange
    let output = SupervisedOutput {
        termination: SupervisedTermination::ExitedSuccess,
        stdout: b"Chip type: ESP32-S3".to_vec(),
        stderr: Vec::new(),
    };

    // Act
    let state = advance_device_effect_state(
        UsbDeviceEffectState::None,
        &["board-info".to_owned()],
        &output,
        UsbWriteDialect::Espflash,
    );

    // Assert
    assert_eq!(state, UsbDeviceEffectState::None);
}

#[test]
fn flash_definition_timeout_diagnostic_is_closed_and_pre_transfer() {
    // Arrange
    let output = SupervisedOutput {
        termination: SupervisedTermination::ExitedFailure,
        stdout: b"private-port-token".to_vec(),
        stderr: b"Timeout while running FlashDeflData command private-network-token".to_vec(),
    };

    // Act
    let diagnostic = UsbCommandDiagnostic::from_output(
        &output,
        UsbTerminalCategory::BootloaderConnectFailed,
        UsbDeviceEffectState::None,
        1,
    );
    let encoded = serde_json::to_string(&diagnostic).expect("diagnostic JSON");

    // Assert
    assert_eq!(
        diagnostic.connection_signature,
        UsbConnectionSignature::FlashDefinitionDataTimeout
    );
    assert!(!diagnostic.transfer_started);
    assert!(!diagnostic.transfer_completed);
    assert!(!diagnostic.raw_output_included);
    assert!(!encoded.contains("private-port-token"));
    assert!(!encoded.contains("private-network-token"));
}

#[test]
fn write_failure_records_confirmed_partial_device_effect() {
    // Arrange
    let output = SupervisedOutput {
        termination: SupervisedTermination::ExitedFailure,
        stdout: Vec::new(),
        stderr: b"write failed".to_vec(),
    };

    // Act
    let state = advance_device_effect_state(
        UsbDeviceEffectState::None,
        &["write-bin".to_owned()],
        &output,
        UsbWriteDialect::Espflash,
    );

    // Assert
    assert_eq!(state, UsbDeviceEffectState::ConfirmedPartial);
}

#[test]
fn esptool_write_effect_distinguishes_pre_transfer_partial_and_complete() {
    // Arrange
    let args = vec!["write_flash".to_owned()];
    let cases = [
        (
            SupervisedOutput {
                termination: SupervisedTermination::ExitedFailure,
                stdout: Vec::new(),
                stderr: b"argument rejected".to_vec(),
            },
            UsbDeviceEffectState::None,
            UsbTerminalCategory::FlashFailedBeforeTransfer,
        ),
        (
            SupervisedOutput {
                termination: SupervisedTermination::ExitedFailure,
                stdout: b"Writing at 0x00010000".to_vec(),
                stderr: b"connection lost".to_vec(),
            },
            UsbDeviceEffectState::ConfirmedPartial,
            UsbTerminalCategory::FlashFailedAfterTransfer,
        ),
        (
            SupervisedOutput {
                termination: SupervisedTermination::ExitedSuccess,
                stdout: b"Hash of data verified".to_vec(),
                stderr: Vec::new(),
            },
            UsbDeviceEffectState::Completed,
            UsbTerminalCategory::FlashFailedAfterTransfer,
        ),
    ];

    for (output, expected_state, expected_failure) in cases {
        // Act
        let state = advance_device_effect_state(
            UsbDeviceEffectState::None,
            &args,
            &output,
            UsbWriteDialect::Esptool,
        );
        let failure = classify_esptool_write_failure(&output);

        // Assert
        assert_eq!(state, expected_state);
        assert_eq!(failure, expected_failure);
    }
}

#[test]
fn protected_recovery_summary_is_mode_0600_and_excludes_stability_key() {
    // Arrange
    let directory = tempdir().expect("temporary directory");
    let trace_path = directory.path().join("recovery.json");
    let mut tracker =
        RecoveryTracker::new(RecoveryPhase::PostFlash, RecoveryPhase::PostFlash.timeout());
    tracker.observe(RecoverySample {
        same_device: true,
        accessible: true,
        holder_free: true,
        enumeration_changed: true,
        maybe_stability_key: Some("/dev/private-secret-epoch".to_owned()),
    });
    let mut bytes = serde_json::to_vec(&tracker.summary()).expect("serialize summary");
    bytes.push(b'\n');

    // Act
    write_private_trace(&trace_path, &bytes).expect("write protected summary");

    // Assert
    let mode = std::fs::metadata(&trace_path)
        .expect("summary metadata")
        .permissions()
        .mode()
        & 0o777;
    let contents = std::fs::read_to_string(&trace_path).expect("summary contents");
    assert_eq!(mode, 0o600);
    assert!(!contents.contains("/dev/private-secret-epoch"));
    assert!(contents.contains("\"deadline_seconds\":60"));
}

#[test]
fn ephemeral_ingress_discards_the_untrusted_first_line() {
    // Arrange
    let mut admission = ReceiveLineAdmission::new();

    // Act
    let admitted = admission.admit(b"mining_campaign_status={invalid}\ntrusted\n");

    // Assert
    assert_eq!(admitted, Some(b"trusted\n".as_slice()));
}

#[test]
fn ephemeral_ingress_waits_for_a_split_initial_boundary() {
    // Arrange
    let mut admission = ReceiveLineAdmission::new();

    // Act
    let first = admission.admit(b"partial-marker");
    let second = admission.admit(b"-tail\ntrusted\n");

    // Assert
    assert_eq!(first, None);
    assert_eq!(second, Some(b"trusted\n".as_slice()));
}

#[test]
fn ephemeral_ingress_forwards_post_boundary_chunks_unchanged() {
    // Arrange
    let mut admission = ReceiveLineAdmission::new();
    let _discarded = admission.admit(b"first-line\n");

    // Act
    let admitted = admission.admit(b"second\nthird\n");

    // Assert
    assert_eq!(admitted, Some(b"second\nthird\n".as_slice()));
}

#[test]
fn ephemeral_ingress_requires_a_fresh_boundary_after_reopen() {
    // Arrange
    let mut admission = ReceiveLineAdmission::new();
    let _discarded = admission.admit(b"first-open\n");
    let _admitted = admission.admit(b"trusted-before-reopen\n");

    // Act
    admission.reset();
    let discarded_after_reopen = admission.admit(b"reopen-fragment\n");
    let admitted_after_reopen = admission.admit(b"trusted-after-reopen\n");

    // Assert
    assert_eq!(discarded_after_reopen, None);
    assert_eq!(
        admitted_after_reopen,
        Some(b"trusted-after-reopen\n".as_slice())
    );
}

#[test]
fn ephemeral_receive_session_owns_boundary_reset_and_admission() {
    // Arrange
    let implementation = include_str!("observation.rs");

    // Act
    let reader_open = implementation
        .find("ReceiveOnlyReader::open")
        .expect("reader admission");
    let boundary_reset = implementation
        .find("line_admission.reset()")
        .expect("boundary reset");
    let chunk_read = implementation
        .find("reader.read_available()")
        .expect("chunk read");
    let boundary_admission = implementation
        .find("line_admission.admit(&chunk)")
        .expect("boundary admission");

    // Assert
    assert!(reader_open < boundary_reset);
    assert!(boundary_reset < chunk_read);
    assert!(chunk_read < boundary_admission);
    assert_eq!(
        implementation
            .matches("line_admission.admit(&chunk)")
            .count(),
        1
    );
    assert!(implementation.contains("if feed_chunks"));
    assert!(implementation.contains("bytes.extend_from_slice"));
}
