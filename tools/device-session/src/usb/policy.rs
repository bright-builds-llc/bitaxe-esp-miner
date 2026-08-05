use crate::macos::UsbDeviceSnapshot;

use super::recovery::RecoveryPhase;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EspflashConnectionSignature {
    ProcessTimeout,
    ProcessInterrupted,
    DeviceNotFound,
    SerialResetIo,
    WrongBootMode,
    NoSyncReply,
    SlipFraming,
    ReadMismatch,
    CommandTimeout,
    GenericConnectionFailure,
    DiagnosticUnavailable,
}

impl EspflashConnectionSignature {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessTimeout => "process_timeout",
            Self::ProcessInterrupted => "process_interrupted",
            Self::DeviceNotFound => "device_not_found",
            Self::SerialResetIo => "serial_reset_io",
            Self::WrongBootMode => "wrong_boot_mode",
            Self::NoSyncReply => "no_sync_reply",
            Self::SlipFraming => "slip_framing",
            Self::ReadMismatch => "read_mismatch",
            Self::CommandTimeout => "command_timeout",
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
    if args.first().map(String::as_str) == Some("write-bin") {
        RecoveryPhase::PostFlash
    } else {
        RecoveryPhase::PostProbe
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
    if classify_bootloader_diagnostic(output) != EspflashConnectionSignature::DiagnosticUnavailable
    {
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

pub(super) fn classify_bootloader_diagnostic(
    output: &SupervisedOutput,
) -> EspflashConnectionSignature {
    match output.termination {
        SupervisedTermination::TimedOut => return EspflashConnectionSignature::ProcessTimeout,
        SupervisedTermination::Interrupted { .. } => {
            return EspflashConnectionSignature::ProcessInterrupted;
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
            EspflashConnectionSignature::DeviceNotFound,
        ),
        (
            "connectionserial",
            EspflashConnectionSignature::SerialResetIo,
        ),
        (
            "connectionwrongbootmode",
            EspflashConnectionSignature::WrongBootMode,
        ),
        (
            "connectionnosyncreply",
            EspflashConnectionSignature::NoSyncReply,
        ),
        (
            "connectionframingerror",
            EspflashConnectionSignature::SlipFraming,
        ),
        (
            "connectionreadmismatch",
            EspflashConnectionSignature::ReadMismatch,
        ),
        (
            "connectiontimeout",
            EspflashConnectionSignature::CommandTimeout,
        ),
        (
            "connectionconnectionfailed",
            EspflashConnectionSignature::GenericConnectionFailure,
        ),
    ];
    signatures
        .into_iter()
        .find_map(|(marker, signature)| normalized.contains(marker).then_some(signature))
        .unwrap_or(EspflashConnectionSignature::DiagnosticUnavailable)
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
    maybe_signature: Option<EspflashConnectionSignature>,
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
                .unwrap_or(EspflashConnectionSignature::DiagnosticUnavailable)
                .as_str()
        );
    }
    if context.category == UsbTerminalCategory::BootloaderConnectFailed {
        return format!(
            "connection_signature={}; the supervised espflash command failed without an eligible state-changing retry",
            maybe_signature
                .unwrap_or(EspflashConnectionSignature::DiagnosticUnavailable)
                .as_str()
        );
    }
    "the supervised espflash command failed without an eligible state-changing retry".to_owned()
}
