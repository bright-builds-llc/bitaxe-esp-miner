use std::thread;
use std::time::{Duration, Instant};

use crate::macos::ReceiveOnlyReader;

use super::{
    line_admission, process, session_error, write_private_trace, MonitorOutput, RecoveryPhase,
    UsbLifecycleEvent, UsbSession, UsbSessionError, UsbTerminalCategory,
};

const MAX_MONITOR_BYTES: usize = 16 * 1024 * 1024;

impl UsbSession {
    pub fn observe_receive_only(
        &mut self,
        duration: Duration,
    ) -> Result<MonitorOutput, UsbSessionError> {
        let result = self.observe_receive_only_inner(Some(duration), true, false, |_| false);
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
                let snapshot = self.reacquire(RecoveryPhase::MonitorAdmission)?;
                reenumerated |= snapshot.enumeration_token != self.initial_enumeration_token;
                maybe_reader =
                    Some(ReceiveOnlyReader::open(&snapshot.port).map_err(|error| {
                        session_error(UsbTerminalCategory::MonitorFailed, error)
                    })?);
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
