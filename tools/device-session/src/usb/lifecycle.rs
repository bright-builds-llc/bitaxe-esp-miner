use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbOperation {
    Detect,
    Flash,
    Recover,
    Monitor,
    FlashMonitor,
    MiningCampaign,
    VerifyDurability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbLifecycleState {
    Prepared,
    Admitted,
    Flashing,
    Reenumerating,
    Observing,
    CleaningUp,
    ReflashReady,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbLifecycleEvent {
    Admit,
    BeginFlash,
    FlashComplete,
    BeginObservation,
    ObservationComplete,
    BeginCleanup,
    CleanupComplete,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbTerminalCategory {
    Ready,
    ConcurrentRepoSession,
    ForeignHolder,
    TransportAbsent,
    IdentityDrift,
    RuntimeProfileUnknown,
    HandoffUnsupported,
    HandoffRejectedUnsafeState,
    HandoffReadyTimeout,
    HandoffTransitionTimeout,
    BootloaderAmbiguous,
    PhysicalIdentityDrift,
    BootloaderSyncFailed,
    ApplicationReappearanceTimeout,
    RecoveryRequired,
    BootloaderConnectFailed,
    FlashFailedBeforeTransfer,
    FlashFailedAfterTransfer,
    MonitorFailed,
    CleanupFailed,
    RecoveryNotObserved,
    RepeatedBoundary,
}

impl UsbTerminalCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::ConcurrentRepoSession => "concurrent_repo_session",
            Self::ForeignHolder => "foreign_holder",
            Self::TransportAbsent => "transport_absent",
            Self::IdentityDrift => "identity_drift",
            Self::RuntimeProfileUnknown => "runtime_profile_unknown",
            Self::HandoffUnsupported => "handoff_unsupported",
            Self::HandoffRejectedUnsafeState => "handoff_rejected_unsafe_state",
            Self::HandoffReadyTimeout => "handoff_ready_timeout",
            Self::HandoffTransitionTimeout => "handoff_transition_timeout",
            Self::BootloaderAmbiguous => "bootloader_ambiguous",
            Self::PhysicalIdentityDrift => "physical_identity_drift",
            Self::BootloaderSyncFailed => "bootloader_sync_failed",
            Self::ApplicationReappearanceTimeout => "application_reappearance_timeout",
            Self::RecoveryRequired => "recovery_required",
            Self::BootloaderConnectFailed => "bootloader_connect_failed",
            Self::FlashFailedBeforeTransfer => "flash_failed_before_transfer",
            Self::FlashFailedAfterTransfer => "flash_failed_after_transfer",
            Self::MonitorFailed => "monitor_failed",
            Self::CleanupFailed => "cleanup_failed",
            Self::RecoveryNotObserved => "recovery_not_observed",
            Self::RepeatedBoundary => "repeated_boundary",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbSessionError {
    pub category: UsbTerminalCategory,
    pub detail: String,
}

impl std::fmt::Display for UsbSessionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.category.as_str(), self.detail)
    }
}

impl std::error::Error for UsbSessionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisedOutput {
    pub termination: SupervisedTermination,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisedTermination {
    ExitedSuccess,
    ExitedFailure,
    TimedOut,
    Interrupted { signal: i32 },
}

/// Monotonic device-write evidence observed by the supervised USB session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbDeviceEffectState {
    /// No transfer boundary was observed.
    #[default]
    None,
    /// A transfer began but completion was not observed.
    ConfirmedPartial,
    /// The supervised flash command completed successfully.
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbCommandTermination {
    NotStarted,
    ExitedSuccess,
    ExitedFailure,
    TimedOut,
    Interrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbConnectionSignature {
    NotApplicable,
    ProcessTimeout,
    ProcessInterrupted,
    DeviceNotFound,
    SerialResetIo,
    WrongBootMode,
    NoSyncReply,
    SlipFraming,
    ReadMismatch,
    CommandTimeout,
    FlashDefinitionDataTimeout,
    GenericConnectionFailure,
    DiagnosticUnavailable,
}

/// Closed, redaction-safe facts for one supervised espflash command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsbCommandDiagnostic {
    pub schema_version: String,
    pub terminal_category: UsbTerminalCategory,
    pub device_effect_state: UsbDeviceEffectState,
    pub termination: UsbCommandTermination,
    pub attempt_count: u8,
    pub connection_signature: UsbConnectionSignature,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub transfer_started: bool,
    pub transfer_completed: bool,
    pub raw_output_included: bool,
}

impl SupervisedOutput {
    #[must_use]
    pub const fn succeeded(&self) -> bool {
        matches!(self.termination, SupervisedTermination::ExitedSuccess)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorOutput {
    pub bytes: Vec<u8>,
    pub interrupted_by: Option<i32>,
    pub reenumerated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflashReady {
    pub port: String,
    pub reenumerated: bool,
}

pub fn discover_usb_ports() -> Result<Vec<String>, UsbSessionError> {
    if !cfg!(target_os = "macos") {
        return Err(session_error(
            UsbTerminalCategory::TransportAbsent,
            "native USB discovery is qualified only on macOS",
        ));
    }
    MacOsDeviceAdapter::candidate_ports()
        .map_err(|error| session_error(UsbTerminalCategory::TransportAbsent, error))
}

pub fn reduce_lifecycle(
    state: UsbLifecycleState,
    event: UsbLifecycleEvent,
) -> Result<UsbLifecycleState, UsbSessionError> {
    use UsbLifecycleEvent as Event;
    use UsbLifecycleState as State;

    let maybe_next = match (state, event) {
        (State::Prepared, Event::Admit) => Some(State::Admitted),
        (State::Admitted | State::Reenumerating, Event::BeginFlash) => Some(State::Flashing),
        (State::Flashing, Event::FlashComplete) => Some(State::Reenumerating),
        (State::Admitted | State::Reenumerating, Event::BeginObservation) => Some(State::Observing),
        (State::Observing, Event::ObservationComplete) => Some(State::Admitted),
        (
            State::Admitted | State::Reenumerating | State::Observing | State::Failed,
            Event::BeginCleanup,
        ) => Some(State::CleaningUp),
        (State::CleaningUp, Event::CleanupComplete) => Some(State::ReflashReady),
        (State::ReflashReady, Event::BeginFlash) => Some(State::Flashing),
        (_, Event::Fail) => Some(State::Failed),
        _ => None,
    };
    maybe_next.ok_or_else(|| UsbSessionError {
        category: UsbTerminalCategory::CleanupFailed,
        detail: format!("illegal lifecycle transition: {state:?} + {event:?}"),
    })
}
