//! Durable ownership and lifecycle supervision for repository USB operations.

mod lease;
mod line_admission;
mod observation;
mod process;
mod profile_reacquire;
mod recovery;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::macos::{MacOsDeviceAdapter, UsbDeviceSnapshot};
use crate::usb_ownership::ProfileObservationTrace;
use lease::DeviceLease;
use process::{run_owned_process, OwnedProcessRequest};
use recovery::{RecoveryPhase, RecoverySample, RecoverySummary, RecoveryTracker};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(150);

mod lifecycle;
mod policy;

pub use lifecycle::*;
pub use policy::{retry_is_eligible, RetryContext};

#[cfg(test)]
use policy::EspflashConnectionSignature;
use policy::{
    classify_bootloader_diagnostic, classify_espflash_failure, classify_esptool_write_failure,
    espflash_diagnostic_filter, ineligible_retry_detail, is_esptool_write_effect, is_flash_effect,
    successful_command_recovery_policy, validate_recovery_snapshot,
};

#[derive(Clone, Copy)]
enum UsbWriteDialect {
    Espflash,
    Esptool,
}

pub struct UsbSession {
    operation: UsbOperation,
    state: UsbLifecycleState,
    lease: DeviceLease,
    physical_identity_digest: String,
    initial_enumeration_token: String,
    current_enumeration_token: String,
    current_port: String,
    device_effect_state: UsbDeviceEffectState,
    last_command_diagnostic: Option<UsbCommandDiagnostic>,
    earliest_failure: Option<UsbTerminalCategory>,
    trace_root: PathBuf,
    child_sequence: u32,
    recovery_sequence: u32,
    profile_trace_sequence: u32,
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
        let snapshot = MacOsDeviceAdapter::maybe_exact_snapshot(port)
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
            device_effect_state: UsbDeviceEffectState::None,
            last_command_diagnostic: None,
            earliest_failure: None,
            trace_root,
            child_sequence: 0,
            recovery_sequence: 0,
            profile_trace_sequence: 0,
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

    #[must_use]
    /// Returns the strongest device-write boundary observed by this session.
    pub const fn device_effect_state(&self) -> UsbDeviceEffectState {
        self.device_effect_state
    }

    #[must_use]
    pub fn last_command_diagnostic(&self) -> Option<UsbCommandDiagnostic> {
        self.last_command_diagnostic.clone()
    }

    pub fn run_espflash(
        &mut self,
        program: &Path,
        args: &[String],
        timeout: Duration,
    ) -> Result<SupervisedOutput, UsbSessionError> {
        self.run_supervised_write(program, args, timeout, UsbWriteDialect::Espflash)
    }

    pub fn run_esptool_write_flash(
        &mut self,
        program: &Path,
        args: &[String],
        timeout: Duration,
    ) -> Result<SupervisedOutput, UsbSessionError> {
        if !is_esptool_write_effect(args) {
            return Err(session_error(
                UsbTerminalCategory::FlashFailedBeforeTransfer,
                "managed esptool command is not write_flash",
            ));
        }
        self.run_supervised_write(program, args, timeout, UsbWriteDialect::Esptool)
    }

    fn run_supervised_write(
        &mut self,
        program: &Path,
        args: &[String],
        timeout: Duration,
        dialect: UsbWriteDialect,
    ) -> Result<SupervisedOutput, UsbSessionError> {
        let mut first_boundary = None;
        let mut command_effect_state = UsbDeviceEffectState::None;
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
                maybe_rust_log: espflash_diagnostic_filter(self.operation, args),
            };
            let process_result = match run_owned_process(request, &mut self.lease) {
                Ok(output) => output,
                Err(error) => {
                    self.fail_once(error.category);
                    self.last_command_diagnostic = Some(UsbCommandDiagnostic::without_output(
                        error.category,
                        command_effect_state,
                        attempt,
                    ));
                    return Err(error);
                }
            };
            command_effect_state =
                advance_device_effect_state(command_effect_state, args, &process_result, dialect);
            self.device_effect_state = advance_device_effect_state(
                self.device_effect_state,
                args,
                &process_result,
                dialect,
            );
            if process_result.succeeded() {
                self.transition(UsbLifecycleEvent::FlashComplete)?;
                let phase = match dialect {
                    UsbWriteDialect::Espflash => successful_command_recovery_policy(args),
                    UsbWriteDialect::Esptool => RecoveryPhase::PostFlash,
                };
                if let Err(error) = self.reacquire(phase) {
                    self.fail_once(error.category);
                    self.last_command_diagnostic = Some(UsbCommandDiagnostic::from_output(
                        &process_result,
                        error.category,
                        command_effect_state,
                        attempt,
                    ));
                    return Err(error);
                }
                self.last_command_diagnostic = Some(UsbCommandDiagnostic::from_output(
                    &process_result,
                    UsbTerminalCategory::Ready,
                    command_effect_state,
                    attempt,
                ));
                return Ok(process_result);
            }

            let category = match dialect {
                UsbWriteDialect::Espflash => classify_espflash_failure(&process_result),
                UsbWriteDialect::Esptool => classify_esptool_write_failure(&process_result),
            };
            let maybe_signature = (category == UsbTerminalCategory::BootloaderConnectFailed)
                .then(|| classify_bootloader_diagnostic(&process_result));
            if attempt == 2 {
                let (first_category, first_signature) =
                    first_boundary.unwrap_or((category, maybe_signature));
                self.fail_once(first_category);
                self.last_command_diagnostic = Some(UsbCommandDiagnostic::from_output(
                    &process_result,
                    first_category,
                    command_effect_state,
                    attempt,
                ));
                return Err(session_error(
                    UsbTerminalCategory::RepeatedBoundary,
                    format!(
                        "connection_signature={}; the same supervised espflash boundary failed after one retry",
                        first_signature
                            .unwrap_or(UsbConnectionSignature::DiagnosticUnavailable)
                            .as_str()
                    ),
                ));
            }
            self.last_command_diagnostic = Some(UsbCommandDiagnostic::from_output(
                &process_result,
                category,
                command_effect_state,
                attempt,
            ));
            first_boundary = Some((category, maybe_signature));
            let maybe_snapshot = self.reacquire(RecoveryPhase::RetryAdmission).ok();
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
                    ineligible_retry_detail(context, maybe_signature),
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
            maybe_rust_log: espflash_diagnostic_filter(self.operation, args),
        };
        let output = match run_owned_process(request, &mut self.lease) {
            Ok(output) => output,
            Err(error) => {
                self.fail_once(error.category);
                self.last_command_diagnostic = Some(UsbCommandDiagnostic::without_output(
                    error.category,
                    UsbDeviceEffectState::None,
                    1,
                ));
                return Err(error);
            }
        };
        if output.succeeded() {
            self.last_command_diagnostic = Some(UsbCommandDiagnostic::from_output(
                &output,
                UsbTerminalCategory::Ready,
                UsbDeviceEffectState::None,
                1,
            ));
            return Ok(output);
        }
        let category = UsbTerminalCategory::FlashFailedBeforeTransfer;
        self.fail_once(category);
        self.last_command_diagnostic = Some(UsbCommandDiagnostic::from_output(
            &output,
            category,
            UsbDeviceEffectState::None,
            1,
        ));
        Err(session_error(
            category,
            "the supervised espflash prerequisite probe failed",
        ))
    }

    pub fn finish(mut self) -> Result<ReflashReady, UsbSessionError> {
        self.transition(UsbLifecycleEvent::BeginCleanup)?;
        let snapshot = self.reacquire(RecoveryPhase::FinalCleanup)?;
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

    fn reacquire(&mut self, phase: RecoveryPhase) -> Result<UsbDeviceSnapshot, UsbSessionError> {
        let timeout = phase.timeout();
        let deadline = Instant::now() + timeout;
        let mut tracker = RecoveryTracker::new(phase, timeout);

        while Instant::now() < deadline {
            let maybe_snapshot =
                match MacOsDeviceAdapter::maybe_physical_snapshot(&self.physical_identity_digest) {
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

    fn write_profile_observation_trace(
        &mut self,
        trace: &ProfileObservationTrace,
    ) -> Result<(), UsbSessionError> {
        self.profile_trace_sequence = self.profile_trace_sequence.saturating_add(1);
        let trace_path = self.trace_root.join(format!(
            "profile-transition-{:04}.json",
            self.profile_trace_sequence
        ));
        let mut bytes = serde_json::to_vec(trace)
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

fn advance_device_effect_state(
    current: UsbDeviceEffectState,
    args: &[String],
    output: &SupervisedOutput,
    dialect: UsbWriteDialect,
) -> UsbDeviceEffectState {
    let is_write = match dialect {
        UsbWriteDialect::Espflash => is_flash_effect(args),
        UsbWriteDialect::Esptool => is_esptool_write_effect(args),
    };
    if current == UsbDeviceEffectState::Completed || (is_write && output.succeeded()) {
        return UsbDeviceEffectState::Completed;
    }
    let category = match dialect {
        UsbWriteDialect::Espflash => classify_espflash_failure(output),
        UsbWriteDialect::Esptool => classify_esptool_write_failure(output),
    };
    if is_write && category == UsbTerminalCategory::FlashFailedAfterTransfer {
        return UsbDeviceEffectState::ConfirmedPartial;
    }
    current
}

impl UsbCommandDiagnostic {
    const SCHEMA: &'static str = "esp-usb-command-diagnostic-v1";

    pub(crate) fn from_output(
        output: &SupervisedOutput,
        terminal_category: UsbTerminalCategory,
        device_effect_state: UsbDeviceEffectState,
        attempt_count: u8,
    ) -> Self {
        let connection_signature =
            if terminal_category == UsbTerminalCategory::BootloaderConnectFailed {
                classify_bootloader_diagnostic(output)
            } else {
                UsbConnectionSignature::NotApplicable
            };
        Self {
            schema_version: Self::SCHEMA.to_owned(),
            terminal_category,
            device_effect_state,
            termination: match output.termination {
                SupervisedTermination::ExitedSuccess => UsbCommandTermination::ExitedSuccess,
                SupervisedTermination::ExitedFailure => UsbCommandTermination::ExitedFailure,
                SupervisedTermination::TimedOut => UsbCommandTermination::TimedOut,
                SupervisedTermination::Interrupted { .. } => UsbCommandTermination::Interrupted,
            },
            attempt_count,
            connection_signature,
            stdout_bytes: output.stdout.len(),
            stderr_bytes: output.stderr.len(),
            stdout_sha256: sha256(&output.stdout),
            stderr_sha256: sha256(&output.stderr),
            transfer_started: device_effect_state != UsbDeviceEffectState::None,
            transfer_completed: device_effect_state == UsbDeviceEffectState::Completed,
            raw_output_included: false,
        }
    }

    fn without_output(
        terminal_category: UsbTerminalCategory,
        device_effect_state: UsbDeviceEffectState,
        attempt_count: u8,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA.to_owned(),
            terminal_category,
            device_effect_state,
            termination: UsbCommandTermination::NotStarted,
            attempt_count,
            connection_signature: UsbConnectionSignature::DiagnosticUnavailable,
            stdout_bytes: 0,
            stderr_bytes: 0,
            stdout_sha256: sha256(&[]),
            stderr_sha256: sha256(&[]),
            transfer_started: device_effect_state != UsbDeviceEffectState::None,
            transfer_completed: device_effect_state == UsbDeviceEffectState::Completed,
            raw_output_included: false,
        }
    }
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
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
