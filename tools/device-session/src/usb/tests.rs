use super::*;

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
