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
    let detail = ineligible_retry_detail(context);

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
