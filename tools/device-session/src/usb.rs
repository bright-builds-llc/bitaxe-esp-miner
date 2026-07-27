//! Durable ownership and lifecycle supervision for repository USB operations.

mod lease;
mod process;
mod recovery;

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
#[cfg(test)]
use recovery::POST_FLASH_RECOVERY_TIMEOUT;
use recovery::{
    RecoveryPhase, RecoverySample, RecoverySummary, RecoveryTracker, STANDARD_RECOVERY_TIMEOUT,
};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(150);
const MAX_MONITOR_BYTES: usize = 16 * 1024 * 1024;

mod lifecycle;
mod policy;

pub use lifecycle::*;
pub use policy::{retry_is_eligible, RetryContext};

use policy::{
    classify_espflash_failure, ineligible_retry_detail, successful_command_recovery_policy,
    validate_recovery_snapshot,
};

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
    recovery_sequence: u32,
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
            recovery_sequence: 0,
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
            if process_result.succeeded() {
                self.transition(UsbLifecycleEvent::FlashComplete)?;
                let (phase, timeout) = successful_command_recovery_policy(args);
                if let Err(error) = self.reacquire(phase, timeout) {
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
            let maybe_snapshot = self
                .reacquire(RecoveryPhase::RetryAdmission, STANDARD_RECOVERY_TIMEOUT)
                .ok();
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
                return Err(session_error(category, ineligible_retry_detail(context)));
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
        if output.succeeded() {
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
                let snapshot =
                    self.reacquire(RecoveryPhase::MonitorAdmission, STANDARD_RECOVERY_TIMEOUT)?;
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
        let snapshot = self.reacquire(RecoveryPhase::FinalCleanup, STANDARD_RECOVERY_TIMEOUT)?;
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

    fn reacquire(
        &mut self,
        phase: RecoveryPhase,
        timeout: Duration,
    ) -> Result<UsbDeviceSnapshot, UsbSessionError> {
        let deadline = Instant::now() + timeout;
        let mut tracker = RecoveryTracker::new(phase, timeout);

        while Instant::now() < deadline {
            let maybe_snapshot =
                match MacOsDeviceAdapter::physical_snapshot(&self.physical_identity_digest) {
                    Ok(maybe_snapshot) => maybe_snapshot,
                    Err(_) => {
                        return Err(self.recovery_error_with_summary(
                            &tracker,
                            UsbTerminalCategory::IdentityDrift,
                            "the USB identity sampler failed",
                        ));
                    }
                };
            if let Some(snapshot) = maybe_snapshot {
                let same_device =
                    snapshot.physical_identity_digest == self.physical_identity_digest;
                let sample = RecoverySample {
                    same_device,
                    accessible: snapshot.accessible,
                    holder_free: snapshot.holder_count == 0,
                    enumeration_changed: snapshot.enumeration_token
                        != self.current_enumeration_token,
                    maybe_stability_key: same_device
                        .then(|| format!("{}\n{}", snapshot.port, snapshot.enumeration_token)),
                };
                let ready = tracker.observe(sample);
                if let Err(error) =
                    validate_recovery_snapshot(&snapshot, &self.physical_identity_digest)
                {
                    return Err(self.recovery_error_with_summary(
                        &tracker,
                        error.category,
                        &error.detail,
                    ));
                }
                if ready {
                    self.write_recovery_summary(&tracker.summary())?;
                    self.current_port = snapshot.port.clone();
                    self.current_enumeration_token = snapshot.enumeration_token.clone();
                    return Ok(snapshot);
                }
            } else {
                tracker.observe(RecoverySample::absent());
            }
            thread::sleep(SAMPLE_INTERVAL);
        }
        let summary = tracker.summary();
        let signature = summary.safe_signature();
        let trace_recorded = self.write_recovery_summary(&summary).is_ok();
        Err(session_error(
            UsbTerminalCategory::RecoveryNotObserved,
            format!("{signature},trace_recorded={trace_recorded}"),
        ))
    }

    fn write_recovery_summary(&mut self, summary: &RecoverySummary) -> Result<(), UsbSessionError> {
        self.recovery_sequence = self.recovery_sequence.saturating_add(1);
        let trace_path = self
            .trace_root
            .join(format!("recovery-{:04}.json", self.recovery_sequence));
        let mut bytes = serde_json::to_vec(summary)
            .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
        bytes.push(b'\n');
        write_private_trace(&trace_path, &bytes)
    }

    fn recovery_error_with_summary(
        &mut self,
        tracker: &RecoveryTracker,
        category: UsbTerminalCategory,
        detail: &str,
    ) -> UsbSessionError {
        let trace_recorded = self.write_recovery_summary(&tracker.summary()).is_ok();
        session_error(
            category,
            format!("{detail} trace_recorded={trace_recorded}"),
        )
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
