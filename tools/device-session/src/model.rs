use serde::{Deserialize, Serialize};

pub const PUBLIC_PROJECTION_SCHEMA: &str = "esp-device-session-v1";
pub const PRIVATE_RESULT_SCHEMA: &str = "esp-device-session-private-result-v1";
pub const REQUEST_SCHEMA: &str = "esp-device-session-reboot-request-v1";
const REQUIRED_STABLE_SAMPLES: u8 = 3;
const MAX_DURATION_MILLIS: u64 = 600_000;
const MAX_REPORTED_SERIAL_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformCategory {
    Macos,
    Linux,
    Windows,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DevicePhase {
    Initial,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerialPhase {
    PreRestart,
    PostRestart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicalMatch {
    None,
    UniqueSame,
    UniqueDifferent,
    Multiple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestOutcome {
    NotAttempted,
    NotTransmitted,
    TransmissionAmbiguous,
    ResponseMissing,
    ResponseReceived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerialDelivery {
    Correlated,
    Silent,
    Reacquired,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalCategory {
    Incomplete,
    Ready,
    ObserverUnqualified,
    RestartRequestNotSent,
    RestartAttributionAmbiguous,
    UsbIdentityUnavailable,
    UsbIdentityDrift,
    ServiceRecoveryTimeout,
    BootIdentityInvalid,
    BuildIdentityMismatch,
    SessionNotAdvanced,
    ResetReasonWrong,
    OrdinalNotNext,
    PostconditionMismatch,
}

impl TerminalCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Incomplete => "incomplete",
            Self::Ready => "ready",
            Self::ObserverUnqualified => "observer_unqualified",
            Self::RestartRequestNotSent => "restart_request_not_sent",
            Self::RestartAttributionAmbiguous => "restart_attribution_ambiguous",
            Self::UsbIdentityUnavailable => "usb_identity_unavailable",
            Self::UsbIdentityDrift => "usb_identity_drift",
            Self::ServiceRecoveryTimeout => "service_recovery_timeout",
            Self::BootIdentityInvalid => "boot_identity_invalid",
            Self::BuildIdentityMismatch => "build_identity_mismatch",
            Self::SessionNotAdvanced => "session_not_advanced",
            Self::ResetReasonWrong => "reset_reason_wrong",
            Self::OrdinalNotNext => "ordinal_not_next",
            Self::PostconditionMismatch => "postcondition_mismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineApplication {
    pub boot_session: String,
    pub boot_ordinal: u64,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedPostcondition {
    pub hostname_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRequest {
    pub schema_version: String,
    pub board_category: String,
    pub admitted_port: String,
    pub physical_identity_digest: String,
    pub trusted_origin: String,
    pub baseline: BaselineApplication,
    pub expected_postcondition: ExpectedPostcondition,
}

impl SessionRequest {
    #[must_use]
    pub fn schema_is_valid(&self) -> bool {
        self.schema_version == REQUEST_SCHEMA
            && self.board_category == "205"
            && !self.admitted_port.is_empty()
            && is_sha256(&self.physical_identity_digest)
            && (self.trusted_origin.starts_with("http://")
                || self.trusted_origin.starts_with("https://"))
            && !self.baseline.boot_session.is_empty()
            && is_sha256(&self.baseline.app_elf_sha256)
            && is_sha256(&self.expected_postcondition.hostname_sha256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrivateBootB {
    pub boot_session: String,
    pub boot_ordinal: u64,
    pub reset_reason_category: String,
    pub trusted_origin: String,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
    pub hostname_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum SessionEvent {
    PlatformObserved {
        category: PlatformCategory,
    },
    DeviceSample {
        phase: DevicePhase,
        physical_match: PhysicalMatch,
        enumeration_token: String,
        accessible: bool,
        holder_count: u16,
    },
    DeviceAbsent,
    ReaderArmed,
    ReaderLost,
    ReaderReacquired,
    ReaderStartFailed,
    SerialBytes {
        phase: SerialPhase,
        count: u64,
    },
    BaselineConfirmed,
    BaselineMismatch,
    RestartRequestStarted,
    RestartRequestBytesWritten {
        count: u64,
    },
    RestartRequestWriteComplete,
    RestartResponseReceived,
    RestartResponseRejected,
    ServiceLossObserved,
    BootBObserved {
        boot_b: PrivateBootB,
    },
    ObservationWindowExpired {
        duration_millis: u64,
    },
    CleanupComplete,
    CleanupFailed,
    AdmissionRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PublicProjection {
    pub schema_version: &'static str,
    pub terminal_category: TerminalCategory,
    pub platform_category: PlatformCategory,
    pub board_category: &'static str,
    pub same_physical_device: bool,
    pub stable_enumeration: bool,
    pub reenumerated: bool,
    pub reader_armed: bool,
    pub pre_restart_serial_delivery: bool,
    pub post_restart_serial_delivery: bool,
    pub serial_delivery: SerialDelivery,
    pub request_outcome: RequestOutcome,
    pub request_attempt_count: u8,
    pub service_loss_observed: bool,
    pub trusted_origin_preserved: bool,
    pub application_recovered: bool,
    pub build_identity_matches: bool,
    pub boot_session_changed: bool,
    pub boot_ordinal_advanced_by_one: bool,
    pub software_reset_observed: bool,
    pub postcondition_matches: bool,
    pub cleanup_complete: bool,
    pub usb_disappearance_count: u16,
    pub enumeration_change_count: u16,
    pub serial_byte_count: u64,
    pub http_observation_count: u16,
    pub duration_millis: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrivateSessionResult {
    pub schema_version: &'static str,
    pub terminal_category: TerminalCategory,
    pub request_outcome: RequestOutcome,
    pub maybe_secondary_cleanup_failure: bool,
    pub boot_b: Option<PrivateBootB>,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    baseline: BaselineApplication,
    expected_postcondition: ExpectedPostcondition,
    trusted_origin: String,
    platform_category: PlatformCategory,
    terminal_category: TerminalCategory,
    maybe_secondary_cleanup_failure: bool,
    maybe_sample_token: Option<String>,
    sample_count: u8,
    initial_device_ready: bool,
    recovery_device_ready: bool,
    same_physical_device: bool,
    maybe_initial_enumeration: Option<String>,
    maybe_current_enumeration: Option<String>,
    reader_armed: bool,
    reader_reacquired: bool,
    baseline_confirmed: bool,
    pre_restart_serial_bytes: u64,
    post_restart_serial_bytes: u64,
    request_attempt_count: u8,
    request_bytes_written: u64,
    request_write_complete: bool,
    restart_response_received: bool,
    service_loss_observed: bool,
    trusted_origin_matches: bool,
    maybe_boot_b: Option<PrivateBootB>,
    application_recovered: bool,
    build_identity_matches: bool,
    boot_session_changed: bool,
    boot_ordinal_advanced_by_one: bool,
    software_reset_observed: bool,
    postcondition_matches: bool,
    cleanup_complete: bool,
    usb_disappearance_count: u16,
    enumeration_change_count: u16,
    http_observation_count: u16,
    duration_millis: u64,
}

mod state;

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests;
