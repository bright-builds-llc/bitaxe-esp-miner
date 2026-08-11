/// Pure action selected for an ESP-IDF OTA boot-validation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootValidationAction {
    /// The running image is not awaiting validation.
    ReportNotPending,
    /// Accept the pending image after normal startup diagnostics pass.
    MarkValid,
    /// Reject the pending image and let ESP-IDF reboot into its rollback target.
    MarkInvalidAndRollback,
    /// Keep the test-only probe pending so its next normal reset proves rollback.
    HoldPendingRollbackProbe,
}

/// Selects the only allowed action from partition state, diagnostics, and the
/// build-isolated rollback-probe flag.
#[must_use]
pub const fn boot_validation_action(
    requires_validation: bool,
    startup_diagnostics_passed: bool,
    rollback_probe: bool,
) -> BootValidationAction {
    if !requires_validation {
        return BootValidationAction::ReportNotPending;
    }
    if rollback_probe {
        return BootValidationAction::HoldPendingRollbackProbe;
    }
    if startup_diagnostics_passed {
        BootValidationAction::MarkValid
    } else {
        BootValidationAction::MarkInvalidAndRollback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_pending_image_is_validated_only_after_diagnostics() {
        // Arrange / Act / Assert
        assert_eq!(
            boot_validation_action(true, true, false),
            BootValidationAction::MarkValid
        );
        assert_eq!(
            boot_validation_action(true, false, false),
            BootValidationAction::MarkInvalidAndRollback
        );
    }

    #[test]
    fn rollback_probe_holds_only_a_pending_image() {
        // Arrange / Act / Assert
        assert_eq!(
            boot_validation_action(true, true, true),
            BootValidationAction::HoldPendingRollbackProbe
        );
        assert_eq!(
            boot_validation_action(false, true, true),
            BootValidationAction::ReportNotPending
        );
    }
}
