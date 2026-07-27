use std::time::Duration;

use crate::macos::UsbDeviceSnapshot;

use super::recovery::{RecoveryPhase, POST_FLASH_RECOVERY_TIMEOUT, STANDARD_RECOVERY_TIMEOUT};
use super::{
    session_error, SupervisedOutput, SupervisedTermination, UsbSessionError, UsbTerminalCategory,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryContext {
    pub category: UsbTerminalCategory,
    pub cleanup_complete: bool,
    pub enumeration_changed: bool,
    pub same_physical_device: bool,
    pub immutable_operation: bool,
    pub repeated_boundary: bool,
    pub attempts: u8,
}

#[must_use]
pub const fn retry_is_eligible(context: RetryContext) -> bool {
    matches!(
        context.category,
        UsbTerminalCategory::BootloaderConnectFailed
            | UsbTerminalCategory::MonitorFailed
            | UsbTerminalCategory::RecoveryNotObserved
    ) && context.cleanup_complete
        && context.enumeration_changed
        && context.same_physical_device
        && context.immutable_operation
        && !context.repeated_boundary
        && context.attempts == 1
}

pub(super) fn successful_command_recovery_policy(args: &[String]) -> (RecoveryPhase, Duration) {
    if args.first().map(String::as_str) == Some("write-bin") {
        (RecoveryPhase::PostFlash, POST_FLASH_RECOVERY_TIMEOUT)
    } else {
        (RecoveryPhase::PostProbe, STANDARD_RECOVERY_TIMEOUT)
    }
}

pub(super) fn validate_recovery_snapshot(
    snapshot: &UsbDeviceSnapshot,
    expected_physical_identity: &str,
) -> Result<(), UsbSessionError> {
    if snapshot.physical_identity_digest != expected_physical_identity {
        return Err(session_error(
            UsbTerminalCategory::IdentityDrift,
            "the observed USB transport belongs to a different physical device",
        ));
    }
    if snapshot.holder_count > 0 {
        return Err(session_error(
            UsbTerminalCategory::ForeignHolder,
            "a foreign process acquired the serial transport",
        ));
    }
    Ok(())
}

pub(super) fn classify_espflash_failure(output: &SupervisedOutput) -> UsbTerminalCategory {
    if matches!(
        output.termination,
        SupervisedTermination::TimedOut | SupervisedTermination::Interrupted { .. }
    ) {
        return UsbTerminalCategory::BootloaderConnectFailed;
    }
    let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
    if stderr.contains("write") || stderr.contains("verify") || stderr.contains("checksum") {
        UsbTerminalCategory::FlashFailedAfterTransfer
    } else if stderr.contains("connect") || stderr.contains("serial") || stderr.contains("port") {
        UsbTerminalCategory::BootloaderConnectFailed
    } else {
        UsbTerminalCategory::FlashFailedBeforeTransfer
    }
}

pub(super) fn ineligible_retry_detail(context: RetryContext) -> &'static str {
    if context.category == UsbTerminalCategory::BootloaderConnectFailed
        && context.cleanup_complete
        && !context.enumeration_changed
        && context.same_physical_device
    {
        return "the supervised espflash command could not synchronize with the bootloader and \
                USB enumeration did not change; disconnect USB and normal device power, wait 10 \
                seconds, reconnect normal power, then USB, and rerun detection; do not use pins, \
                headers, or test points";
    }
    "the supervised espflash command failed without an eligible state-changing retry"
}
