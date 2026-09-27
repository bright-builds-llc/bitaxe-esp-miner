use std::thread;
use std::time::{Duration, Instant};

use crate::macos::ReceiveOnlyReader;

use super::timing::TimedReader;
use super::{
    line_admission, process, session_error, write_private_trace, MonitorOutput, RecoveryPhase,
    UsbLifecycleEvent, UsbSession, UsbSessionError, UsbTerminalCategory,
};
use super::{BootstrapFailureStage, TimingStage};

const MAX_MONITOR_BYTES: usize = 16 * 1024 * 1024;

impl UsbSession {
    /// Waits for the installed Worker, then collects bounded diagnostics under this lease.
    pub fn observe_installed_worker(
        &mut self,
    ) -> Result<crate::UsbRebootLoopObservation, UsbSessionError> {
        self.reacquire_profile(crate::UsbProfile::SerialJtagRuntime)?;
        self.transition(UsbLifecycleEvent::BeginObservation)?;
        let result = self.observe_installed_worker_inner();
        let completion = self.transition(UsbLifecycleEvent::ObservationComplete);
        if let Err(error) = &result {
            self.fail_once(error.category);
        }
        match (result, completion) {
            (Err(primary), _) => Err(primary),
            (Ok(_), Err(error)) => Err(error),
            (Ok(observation), Ok(())) => Ok(observation),
        }
    }

    fn observe_installed_worker_inner(
        &mut self,
    ) -> Result<crate::UsbRebootLoopObservation, UsbSessionError> {
        let before = crate::inspect_usb_profile(self.port()).map_err(|_| {
            session_error(
                UsbTerminalCategory::RuntimeProfileUnknown,
                "Worker inspection failed",
            )
        })?;
        if before.physical_identity_digest != self.physical_identity_digest() {
            return Err(session_error(
                UsbTerminalCategory::PhysicalIdentityDrift,
                "Worker lease mismatch",
            ));
        }
        let observation = crate::observe_usb_reboot_loop(self.port(), Duration::from_secs(30))
            .map_err(|_| {
                session_error(
                    UsbTerminalCategory::MonitorFailed,
                    "Worker diagnostic observation failed",
                )
            })?;
        // Reacquisition refreshes a port renamed during the reconnecting capture
        // and proves that its final Worker still belongs to the original lease.
        self.reacquire_profile(crate::UsbProfile::SerialJtagRuntime)?;
        Ok(observation)
    }

    pub fn observe_receive_only(
        &mut self,
        duration: Duration,
    ) -> Result<MonitorOutput, UsbSessionError> {
        let result = if self.capture.reset_reaped {
            self.observe_early_capture()
        } else {
            self.observe_receive_only_inner(Some(duration), true, false, |_| false)
        };
        if let Err(error) = &result {
            self.fail_once(error.category);
        }
        result
    }

    /// Feeds newline-admitted chunks without retaining a cumulative transcript.
    pub fn observe_receive_only_ephemeral_chunks_until(
        &mut self,
        duration: Duration,
        stop: impl FnMut(&[u8]) -> bool,
    ) -> Result<MonitorOutput, UsbSessionError> {
        let result = self.observe_receive_only_inner(Some(duration), false, true, stop);
        if let Err(error) = &result {
            self.fail_once(error.category);
        }
        result
    }

    /// Observes a transaction containing persisted human checkpoints.
    ///
    /// The enclosing receive loop has no elapsed deadline because operator
    /// availability may span hours or overnight. The callback remains
    /// responsible for bounding every automated phase and for ending the
    /// capture after a terminal outcome.
    pub fn observe_receive_only_ephemeral_chunks_operator_gated(
        &mut self,
        stop: impl FnMut(&[u8]) -> bool,
    ) -> Result<MonitorOutput, UsbSessionError> {
        let result = self.observe_receive_only_inner(None, false, true, stop);
        if let Err(error) = &result {
            self.fail_once(error.category);
        }
        result
    }

    fn observe_receive_only_inner(
        &mut self,
        maybe_duration: Option<Duration>,
        persist_trace: bool,
        feed_chunks: bool,
        mut stop: impl FnMut(&[u8]) -> bool,
    ) -> Result<MonitorOutput, UsbSessionError> {
        let _signal_supervisor = process::SignalSupervisor::acquire()?;
        self.transition(UsbLifecycleEvent::BeginObservation)?;
        self.child_sequence = self.child_sequence.saturating_add(1);
        let trace_path = self
            .trace_root
            .join(format!("monitor-{:04}.serial", self.child_sequence));
        let maybe_deadline = maybe_duration.map(|duration| Instant::now() + duration);
        let mut bytes = Vec::new();
        let mut maybe_reader = None;
        let mut reenumerated = false;
        let mut line_admission = line_admission::ReceiveLineAdmission::new();

        while maybe_deadline.is_none_or(|deadline| Instant::now() < deadline) {
            if let Some(signal) = process::maybe_pending_signal() {
                if persist_trace {
                    write_private_trace(&trace_path, &bytes)?;
                }
                self.transition(UsbLifecycleEvent::ObservationComplete)?;
                return Ok(MonitorOutput {
                    bytes,
                    interrupted_by: Some(signal),
                    reenumerated,
                });
            }
            if maybe_reader.is_none() {
                self.bootstrap_event(TimingStage::MonitorStart);
                let snapshot = self
                    .reacquire(RecoveryPhase::MonitorAdmission)
                    .inspect_err(|error| {
                        self.bootstrap_failure(BootstrapFailureStage::MonitorAdmission, error)
                    })?;
                self.bootstrap_event(TimingStage::MonitorAdmitted);
                reenumerated |= snapshot.enumeration_token != self.initial_enumeration_token;
                self.bootstrap_event(TimingStage::OpenStart);
                let reader = ReceiveOnlyReader::open(&snapshot.port)
                    .map_err(|error| session_error(UsbTerminalCategory::MonitorFailed, error))
                    .inspect_err(|error| {
                        self.bootstrap_failure(BootstrapFailureStage::Open, error)
                    })?;
                self.bootstrap_event(TimingStage::Opened);
                maybe_reader = Some(TimedReader::new(reader, self.maybe_timing.clone()));
                line_admission.reset();
            }
            let Some(reader) = maybe_reader.as_mut() else {
                return Err(session_error(
                    UsbTerminalCategory::MonitorFailed,
                    "receive-only reader admission failed",
                ));
            };
            let mut callback_was_polled = false;
            match reader.read_available() {
                Ok(chunk) => {
                    if let Some(timing) = &self.maybe_timing {
                        timing.first_read(chunk.len());
                    }
                    let should_stop = if feed_chunks {
                        line_admission.admit(&chunk).is_some_and(|admitted| {
                            callback_was_polled = true;
                            stop(admitted)
                        })
                    } else {
                        let remaining = MAX_MONITOR_BYTES.saturating_sub(bytes.len());
                        bytes.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
                        callback_was_polled = true;
                        stop(&bytes)
                    };
                    if should_stop {
                        break;
                    }
                }
                Err(_) => {
                    if let Some(timing) = &self.maybe_timing {
                        timing.failure(
                            BootstrapFailureStage::Read,
                            UsbTerminalCategory::MonitorFailed,
                        );
                    }
                    maybe_reader = None;
                    reenumerated = true;
                }
            }
            // Poll even while the transport is silent so an automated phase
            // deadline or recovery decision can terminate an operator-gated
            // observation without waiting for another serial line.
            if !callback_was_polled && stop(&[]) {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        drop(maybe_reader);
        if maybe_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            if let Some(timing) = &self.maybe_timing {
                timing.capture_complete();
            }
        }
        if persist_trace {
            write_private_trace(&trace_path, &bytes)?;
        }
        self.transition(UsbLifecycleEvent::ObservationComplete)?;
        Ok(MonitorOutput {
            bytes,
            interrupted_by: None,
            reenumerated,
        })
    }
}
