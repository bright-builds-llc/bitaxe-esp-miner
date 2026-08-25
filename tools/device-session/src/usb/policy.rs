use crate::macos::UsbDeviceSnapshot;

use super::recovery::RecoveryPhase;
use super::{
    session_error, SupervisedOutput, SupervisedTermination, UsbConnectionSignature,
    UsbSessionError, UsbTerminalCategory,
};

#[cfg(test)]
pub(super) type EspflashConnectionSignature = UsbConnectionSignature;

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

impl UsbConnectionSignature {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::ProcessTimeout => "process_timeout",
            Self::ProcessInterrupted => "process_interrupted",
            Self::DeviceNotFound => "device_not_found",
            Self::SerialResetIo => "serial_reset_io",
            Self::WrongBootMode => "wrong_boot_mode",
            Self::NoSyncReply => "no_sync_reply",
            Self::SlipFraming => "slip_framing",
            Self::ReadMismatch => "read_mismatch",
            Self::CommandTimeout => "command_timeout",
            Self::FlashDefinitionDataTimeout => "flash_definition_data_timeout",
            Self::GenericConnectionFailure => "generic_connection_failure",
            Self::DiagnosticUnavailable => "diagnostic_unavailable",
        }
    }
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

pub(super) fn successful_command_recovery_policy(args: &[String]) -> RecoveryPhase {
    if is_flash_effect(args) {
        RecoveryPhase::PostFlash
    } else {
        RecoveryPhase::PostProbe
    }
}

pub(super) fn is_flash_effect(args: &[String]) -> bool {
    matches!(
        args.first().map(String::as_str),
        Some("write-bin" | "erase-flash")
    )
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
    if classify_bootloader_diagnostic(output) != UsbConnectionSignature::DiagnosticUnavailable {
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

pub(super) fn classify_esptool_write_failure(output: &SupervisedOutput) -> UsbTerminalCategory {
    if esptool_transfer_started(output) {
        return UsbTerminalCategory::FlashFailedAfterTransfer;
    }
    if matches!(
        output.termination,
        SupervisedTermination::TimedOut | SupervisedTermination::Interrupted { .. }
    ) || classify_bootloader_diagnostic(output) != UsbConnectionSignature::DiagnosticUnavailable
    {
        return UsbTerminalCategory::BootloaderConnectFailed;
    }
    UsbTerminalCategory::FlashFailedBeforeTransfer
}

pub(super) fn esptool_transfer_started(output: &SupervisedOutput) -> bool {
    let mut bytes = Vec::with_capacity(output.stdout.len() + output.stderr.len());
    bytes.extend_from_slice(&output.stdout);
    bytes.extend_from_slice(&output.stderr);
    let normalized = String::from_utf8_lossy(&bytes).to_ascii_lowercase();
    [
        "erasing flash",
        "writing at ",
        "wrote ",
        "hash of data verified",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

pub(super) fn is_esptool_write_effect(args: &[String]) -> bool {
    args.iter().any(|argument| argument == "write_flash")
}

pub(super) fn classify_bootloader_diagnostic(output: &SupervisedOutput) -> UsbConnectionSignature {
    match output.termination {
        SupervisedTermination::TimedOut => return UsbConnectionSignature::ProcessTimeout,
        SupervisedTermination::Interrupted { .. } => {
            return UsbConnectionSignature::ProcessInterrupted;
        }
        SupervisedTermination::ExitedSuccess | SupervisedTermination::ExitedFailure => {}
    }

    let normalized = String::from_utf8_lossy(&output.stderr)
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    let signatures = [
        (
            "connectiondevicenotfound",
            UsbConnectionSignature::DeviceNotFound,
        ),
        ("connectionserial", UsbConnectionSignature::SerialResetIo),
        (
            "connectionwrongbootmode",
            UsbConnectionSignature::WrongBootMode,
        ),
        ("connectionnosyncreply", UsbConnectionSignature::NoSyncReply),
        (
            "connectionframingerror",
            UsbConnectionSignature::SlipFraming,
        ),
        (
            "connectionreadmismatch",
            UsbConnectionSignature::ReadMismatch,
        ),
        ("connectiontimeout", UsbConnectionSignature::CommandTimeout),
        (
            "timeoutwhilerunningflashdefldatacommand",
            UsbConnectionSignature::FlashDefinitionDataTimeout,
        ),
        (
            "connectionconnectionfailed",
            UsbConnectionSignature::GenericConnectionFailure,
        ),
    ];
    signatures
        .into_iter()
        .find_map(|(marker, signature)| normalized.contains(marker).then_some(signature))
        .unwrap_or(UsbConnectionSignature::DiagnosticUnavailable)
}

pub(super) fn espflash_diagnostic_filter(
    operation: super::UsbOperation,
    args: &[String],
) -> Option<&'static str> {
    (operation == super::UsbOperation::Detect
        && args.first().map(String::as_str) == Some("board-info"))
    .then_some("espflash::connection=debug")
}

pub(super) fn ineligible_retry_detail(
    context: RetryContext,
    maybe_signature: Option<UsbConnectionSignature>,
) -> String {
    if context.category == UsbTerminalCategory::BootloaderConnectFailed
        && context.cleanup_complete
        && !context.enumeration_changed
        && context.same_physical_device
    {
        return format!(
            "connection_signature={}; the supervised espflash command could not synchronize with the bootloader and \
                USB enumeration did not change; disconnect USB and normal device power, wait 10 \
                seconds, reconnect normal power, then USB, and rerun detection; do not use pins, \
                headers, or test points",
            maybe_signature
                .unwrap_or(UsbConnectionSignature::DiagnosticUnavailable)
                .as_str()
        );
    }
    if context.category == UsbTerminalCategory::BootloaderConnectFailed {
        return format!(
            "connection_signature={}; the supervised espflash command failed without an eligible state-changing retry",
            maybe_signature
                .unwrap_or(UsbConnectionSignature::DiagnosticUnavailable)
                .as_str()
        );
    }
    "the supervised espflash command failed without an eligible state-changing retry".to_owned()
}
