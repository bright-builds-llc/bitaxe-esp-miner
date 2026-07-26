//! Durable ownership and lifecycle supervision for repository USB operations.

mod lease;
mod process;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::macos::{MacOsDeviceAdapter, ReceiveOnlyReader, UsbDeviceSnapshot};
use lease::DeviceLease;
use process::{run_owned_process, OwnedProcessRequest};

const REQUIRED_STABLE_SAMPLES: u8 = 3;
const SAMPLE_INTERVAL: Duration = Duration::from_millis(150);
const REACQUIRE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_MONITOR_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbOperation {
    Detect,
    Flash,
    Monitor,
    FlashMonitor,
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
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
    pub interrupted_by: Option<i32>,
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

pub struct UsbSession {
    operation: UsbOperation,
    state: UsbLifecycleState,
    lease: DeviceLease,
    physical_identity_digest: String,
    initial_enumeration_token: String,
    current_enumeration_token: String,
    current_port: String,
    earliest_failure: Option<UsbTerminalCategory>,
    trace_root: PathBuf,
    child_sequence: u32,
}

impl UsbSession {
    pub fn acquire(
        operation: UsbOperation,
        port: &str,
        trace_root: impl AsRef<Path>,
    ) -> Result<Self, UsbSessionError> {
        if !cfg!(target_os = "macos") {
            return Err(session_error(
                UsbTerminalCategory::TransportAbsent,
                "the durable USB supervisor is qualified only on macOS",
            ));
        }
        let snapshot = MacOsDeviceAdapter::exact_snapshot(port)
            .map_err(|error| session_error(UsbTerminalCategory::TransportAbsent, error))?
            .ok_or_else(|| {
                session_error(
                    UsbTerminalCategory::TransportAbsent,
                    "the selected USB transport is absent",
                )
            })?;
        if snapshot.holder_count > 0 {
            return Err(session_error(
                UsbTerminalCategory::ForeignHolder,
                "the selected serial transport has a foreign holder",
            ));
        }
        if !snapshot.accessible {
            return Err(session_error(
                UsbTerminalCategory::ForeignHolder,
                "the selected serial transport is not receive-accessible",
            ));
        }
        let trace_root = trace_root.as_ref().to_path_buf();
        let mut lease =
            DeviceLease::acquire(&snapshot.physical_identity_digest, operation, &trace_root)?;
        let state = reduce_lifecycle(UsbLifecycleState::Prepared, UsbLifecycleEvent::Admit)?;
        lease.record_state(state, None)?;
        Ok(Self {
            operation,
            state,
            lease,
            physical_identity_digest: snapshot.physical_identity_digest,
            initial_enumeration_token: snapshot.enumeration_token.clone(),
            current_enumeration_token: snapshot.enumeration_token,
            current_port: snapshot.port,
            earliest_failure: None,
            trace_root,
            child_sequence: 0,
        })
    }

    #[must_use]
    pub fn port(&self) -> &str {
        &self.current_port
    }

    #[must_use]
    pub fn physical_identity_digest(&self) -> &str {
        &self.physical_identity_digest
    }

    #[must_use]
    pub fn operation(&self) -> UsbOperation {
        self.operation
    }

    pub fn run_espflash(
        &mut self,
        program: &Path,
        args: &[String],
        timeout: Duration,
    ) -> Result<SupervisedOutput, UsbSessionError> {
        let mut first_boundary = None;
        for attempt in 1..=2 {
            self.transition(UsbLifecycleEvent::BeginFlash)?;
            let enumeration_before = self.current_enumeration_token.clone();
            self.child_sequence = self.child_sequence.saturating_add(1);
            let trace_label = format!("child-{:04}", self.child_sequence);
            let request = OwnedProcessRequest {
                program,
                args,
                timeout,
                trace_root: &self.trace_root,
                trace_label: &trace_label,
            };
            let process_result =
                run_owned_process(request, &mut self.lease).inspect_err(|error| {
                    self.fail_once(error.category);
                })?;
            if process_result.success {
                self.transition(UsbLifecycleEvent::FlashComplete)?;
                if let Err(error) = self.reacquire(REACQUIRE_TIMEOUT) {
                    self.fail_once(error.category);
                    return Err(error);
                }
                return Ok(process_result);
            }

            let category = classify_espflash_failure(&process_result);
            if attempt == 2 {
                self.fail_once(first_boundary.unwrap_or(category));
                return Err(session_error(
                    UsbTerminalCategory::RepeatedBoundary,
                    "the same supervised espflash boundary failed after one retry",
                ));
            }
            first_boundary = Some(category);
            let maybe_snapshot = self.reacquire(REACQUIRE_TIMEOUT).ok();
            let context = RetryContext {
                category,
                cleanup_complete: true,
                enumeration_changed: maybe_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.enumeration_token != enumeration_before),
                same_physical_device: maybe_snapshot.is_some(),
                immutable_operation: true,
                repeated_boundary: false,
                attempts: attempt,
            };
            if !retry_is_eligible(context) {
                self.fail_once(category);
                return Err(session_error(
                    category,
                    "the supervised espflash command failed without an eligible state-changing retry",
                ));
            }
            self.state = UsbLifecycleState::Reenumerating;
            self.lease.record_state(self.state, self.earliest_failure)?;
        }
        Err(session_error(
            UsbTerminalCategory::RepeatedBoundary,
            "the bounded espflash retry loop exhausted",
        ))
    }

    pub fn run_espflash_probe(
        &mut self,
        program: &Path,
        args: &[String],
        timeout: Duration,
    ) -> Result<SupervisedOutput, UsbSessionError> {
        self.child_sequence = self.child_sequence.saturating_add(1);
        let trace_label = format!("child-{:04}", self.child_sequence);
        let request = OwnedProcessRequest {
            program,
            args,
            timeout,
            trace_root: &self.trace_root,
            trace_label: &trace_label,
        };
        let output = run_owned_process(request, &mut self.lease).inspect_err(|error| {
            self.fail_once(error.category);
        })?;
        if output.success {
            return Ok(output);
        }
        let category = UsbTerminalCategory::FlashFailedBeforeTransfer;
        self.fail_once(category);
        Err(session_error(
            category,
            "the supervised espflash prerequisite probe failed",
        ))
    }

    pub fn observe_receive_only(
        &mut self,
        duration: Duration,
    ) -> Result<MonitorOutput, UsbSessionError> {
        let result = self.observe_receive_only_inner(duration);
        if let Err(error) = &result {
            self.fail_once(error.category);
        }
        result
    }

    fn observe_receive_only_inner(
        &mut self,
        duration: Duration,
    ) -> Result<MonitorOutput, UsbSessionError> {
        let _signal_supervisor = process::SignalSupervisor::acquire()?;
        self.transition(UsbLifecycleEvent::BeginObservation)?;
        self.child_sequence = self.child_sequence.saturating_add(1);
        let trace_path = self
            .trace_root
            .join(format!("monitor-{:04}.serial", self.child_sequence));
        let deadline = Instant::now() + duration;
        let mut bytes = Vec::new();
        let mut maybe_reader = None;
        let mut reenumerated = false;

        while Instant::now() < deadline {
            if let Some(signal) = process::pending_signal() {
                write_private_trace(&trace_path, &bytes)?;
                self.transition(UsbLifecycleEvent::ObservationComplete)?;
                return Ok(MonitorOutput {
                    bytes,
                    interrupted_by: Some(signal),
                    reenumerated,
                });
            }
            if maybe_reader.is_none() {
                let snapshot = self.reacquire(REACQUIRE_TIMEOUT)?;
                reenumerated |= snapshot.enumeration_token != self.initial_enumeration_token;
                maybe_reader =
                    Some(ReceiveOnlyReader::open(&snapshot.port).map_err(|error| {
                        session_error(UsbTerminalCategory::MonitorFailed, error)
                    })?);
            }
            let Some(reader) = maybe_reader.as_mut() else {
                return Err(session_error(
                    UsbTerminalCategory::MonitorFailed,
                    "receive-only reader admission failed",
                ));
            };
            match reader.read_available() {
                Ok(chunk) => {
                    let remaining = MAX_MONITOR_BYTES.saturating_sub(bytes.len());
                    bytes.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
                }
                Err(_) => {
                    maybe_reader = None;
                    reenumerated = true;
                }
            }
            thread::sleep(Duration::from_millis(25));
        }
        drop(maybe_reader);
        write_private_trace(&trace_path, &bytes)?;
        self.transition(UsbLifecycleEvent::ObservationComplete)?;
        Ok(MonitorOutput {
            bytes,
            interrupted_by: None,
            reenumerated,
        })
    }

    pub fn finish(mut self) -> Result<ReflashReady, UsbSessionError> {
        self.transition(UsbLifecycleEvent::BeginCleanup)?;
        let snapshot = self.reacquire(REACQUIRE_TIMEOUT)?;
        self.transition(UsbLifecycleEvent::CleanupComplete)?;
        self.lease.record_state(self.state, self.earliest_failure)?;
        self.lease.mark_complete();
        Ok(ReflashReady {
            port: snapshot.port,
            reenumerated: snapshot.enumeration_token != self.initial_enumeration_token,
        })
    }

    fn transition(&mut self, event: UsbLifecycleEvent) -> Result<(), UsbSessionError> {
        let state = reduce_lifecycle(self.state, event)?;
        self.state = state;
        self.lease.record_state(state, self.earliest_failure)
    }

    fn fail_once(&mut self, category: UsbTerminalCategory) {
        if self.earliest_failure.is_none() {
            self.earliest_failure = Some(category);
        }
        self.state = UsbLifecycleState::Failed;
        let _result = self.lease.record_state(self.state, self.earliest_failure);
    }

    fn reacquire(&mut self, timeout: Duration) -> Result<UsbDeviceSnapshot, UsbSessionError> {
        let deadline = Instant::now() + timeout;
        let mut stable_samples = 0_u8;
        let mut maybe_previous: Option<UsbDeviceSnapshot> = None;

        while Instant::now() < deadline {
            let maybe_snapshot =
                MacOsDeviceAdapter::physical_snapshot(&self.physical_identity_digest)
                    .map_err(|error| session_error(UsbTerminalCategory::IdentityDrift, error))?;
            if let Some(snapshot) = maybe_snapshot {
                if snapshot.physical_identity_digest != self.physical_identity_digest {
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
                if snapshot.accessible {
                    let same_sample = maybe_previous.as_ref().is_some_and(|previous| {
                        previous.port == snapshot.port
                            && previous.enumeration_token == snapshot.enumeration_token
                    });
                    stable_samples = if same_sample {
                        stable_samples.saturating_add(1)
                    } else {
                        1
                    };
                    maybe_previous = Some(snapshot.clone());
                    if stable_samples >= REQUIRED_STABLE_SAMPLES {
                        self.current_port = snapshot.port.clone();
                        self.current_enumeration_token = snapshot.enumeration_token.clone();
                        return Ok(snapshot);
                    }
                } else {
                    stable_samples = 0;
                    maybe_previous = None;
                }
            } else {
                stable_samples = 0;
                maybe_previous = None;
            }
            thread::sleep(SAMPLE_INTERVAL);
        }
        Err(session_error(
            UsbTerminalCategory::RecoveryNotObserved,
            "the admitted physical device did not become stably holder-free",
        ))
    }
}

fn classify_espflash_failure(output: &SupervisedOutput) -> UsbTerminalCategory {
    if output.timed_out || output.interrupted_by.is_some() {
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

fn session_error(category: UsbTerminalCategory, detail: impl std::fmt::Display) -> UsbSessionError {
    UsbSessionError {
        category,
        detail: detail.to_string(),
    }
}

fn write_private_trace(path: &Path, bytes: &[u8]) -> Result<(), UsbSessionError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    file.write_all(bytes)
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    file.sync_all()
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))
}

#[cfg(test)]
#[path = "usb/tests.rs"]
mod tests;
