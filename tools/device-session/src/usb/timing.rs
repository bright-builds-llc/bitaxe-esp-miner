//! Bounded host-clock observations, independent of ordinary capture qualification.
use super::UsbTerminalCategory;
use super::{SupervisedOutput, UsbSession, UsbSessionError};
use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, Mutex};

use std::time::{Duration, Instant};

const STAGES: [&str; 16] = [
    "reset_command_call_start",
    "reset_command_call_end",
    "handoff_start",
    "handoff_admitted",
    "monitor_admission_start",
    "monitor_admitted",
    "reader_open_start",
    "reader_opened",
    "first_nonempty_read",
    "reader_closed",
    "candidate_observed",
    "reader_bound",
    "quarantine_released",
    "capture_deadline_reached",
    "reader_joined",
    "cancellation_requested",
];
#[derive(Clone, Copy, Debug)]
pub enum TimingStage {
    ResetStart,
    ResetEnd,
    HandoffStart,
    HandoffAdmitted,
    MonitorStart,
    MonitorAdmitted,
    OpenStart,
    Opened,
    FirstRead,
    Closed,
    CandidateObserved,
    ReaderBound,
    QuarantineReleased,
    CaptureDeadlineReached,
    ReaderJoined,
    CancellationRequested,
}
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct Event {
    stage: &'static str,
    elapsed_us: u64,
}
/// Closed failure stage names; arbitrary error text cannot enter the sidecar.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapFailureStage {
    Preparation,
    SessionAdmission,
    Reset,
    Handoff,
    MonitorAdmission,
    Open,
    Read,
    Close,
    Clock,
    Overflow,
    Cleanup,
    Evidence,
    Binding,
    Quarantine,
    Join,
}
#[derive(Clone, Copy, Serialize)]
struct Failure {
    stage: BootstrapFailureStage,
    category: &'static str,
}
#[derive(Clone)]
pub struct BootstrapTiming(Arc<Mutex<Recorder>>);
struct Recorder {
    origin: Option<Instant>,
    identity: Option<String>,
    nonce: Option<String>,
    events: [Option<Event>; 24],
    duration_ms: Option<u64>,
    binding: Option<String>,
    captured: usize,
    quarantined: usize,
    released: bool,
    joined: bool,
    capture_overflow: bool,
    count: usize,
    last: u64,
    reset: Option<u32>,
    opens: u32,
    reopens: u32,
    first: Option<usize>,
    overflow: bool,
    clock: bool,
    capture: bool,
    cleanup: bool,
    failure: Option<Failure>,
}
impl Default for BootstrapTiming {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Recorder {
            origin: None,
            identity: None,
            nonce: None,
            events: [None; 24],
            duration_ms: None,
            binding: None,
            captured: 0,
            quarantined: 0,
            released: false,
            joined: false,
            capture_overflow: false,
            count: 0,
            last: 0,
            reset: None,
            opens: 0,
            reopens: 0,
            first: None,
            overflow: false,
            clock: false,
            capture: false,
            cleanup: false,
            failure: None,
        })))
    }
}
impl BootstrapTiming {
    /// Binds the actual acquired session; never synthesizes a lease on failure.
    pub(crate) fn bind(&self, origin: Instant, identity: &str, nonce: &str) {
        let mut r = self
            .0
            .lock()
            .expect("bounded recorder operations do not panic");
        r.origin = Some(origin);
        r.identity = Some(identity.to_owned());
        r.nonce = Some(super::sha256(nonce.as_bytes()));
    }
    pub fn event(&self, stage: TimingStage) {
        let mut r = self
            .0
            .lock()
            .expect("bounded recorder operations do not panic");
        if let Some(origin) = r.origin {
            let us = origin.elapsed().as_micros();
            Self::record_locked(&mut r, stage, us);
        }
    }
    #[cfg(test)]
    fn record(&self, stage: TimingStage, us: u128) {
        let mut r = self
            .0
            .lock()
            .expect("bounded recorder operations do not panic");
        Self::record_locked(&mut r, stage, us);
    }
    fn record_locked(r: &mut Recorder, stage: TimingStage, us: u128) {
        let Ok(us) = u64::try_from(us) else {
            r.overflow = true;
            r.failure.get_or_insert(Failure {
                stage: BootstrapFailureStage::Overflow,
                category: "observation_overflow",
            });
            return;
        };
        if us < r.last {
            r.clock = true;
            r.failure.get_or_insert(Failure {
                stage: BootstrapFailureStage::Clock,
                category: "clock_discontinuity",
            });
        }
        r.last = us;
        if matches!(stage, TimingStage::OpenStart) {
            if r.opens == u32::MAX {
                r.overflow = true;
                r.failure.get_or_insert(Failure {
                    stage: BootstrapFailureStage::Overflow,
                    category: "observation_overflow",
                });
            } else {
                r.opens += 1;
            }
            r.reopens = r.opens.saturating_sub(1);
        }
        let name = STAGES[stage as usize];
        if r.events.iter().flatten().any(|e| e.stage == name) {
            return;
        }
        if r.count == r.events.len() {
            r.overflow = true;
            r.failure.get_or_insert(Failure {
                stage: BootstrapFailureStage::Overflow,
                category: "observation_overflow",
            });
            return;
        }
        let index = r.count;
        r.events[index] = Some(Event {
            stage: name,
            elapsed_us: us,
        });
        r.count += 1;
    }
    pub fn reset_sequence(&self, sequence: u32) {
        self.0
            .lock()
            .expect("bounded recorder operations do not panic")
            .reset
            .get_or_insert(sequence);
    }
    pub fn first_read(&self, bytes: usize) {
        if bytes > 0
            && self
                .0
                .lock()
                .expect("bounded recorder operations do not panic")
                .first
                .is_none()
        {
            self.0
                .lock()
                .expect("bounded recorder operations do not panic")
                .first = Some(bytes);
            self.event(TimingStage::FirstRead);
        }
    }
    pub fn failure(&self, stage: BootstrapFailureStage, category: UsbTerminalCategory) {
        self.fail_label(stage, category.as_str());
    }
    pub fn preparation_failed(&self) {
        if !self
            .0
            .lock()
            .expect("bounded recorder operations do not panic")
            .capture
        {
            self.fail_label(BootstrapFailureStage::Preparation, "preparation_failed");
        }
    }
    fn fail_label(&self, stage: BootstrapFailureStage, category: &'static str) {
        self.0
            .lock()
            .expect("bounded recorder operations do not panic")
            .failure
            .get_or_insert(Failure { stage, category });
    }
    pub fn capture_complete(&self) {
        self.0
            .lock()
            .expect("bounded recorder operations do not panic")
            .capture = true;
    }
    pub fn cleanup_complete(&self, complete: bool) {
        self.0
            .lock()
            .expect("bounded recorder operations do not panic")
            .cleanup = complete;
        if !complete {
            self.failure(
                BootstrapFailureStage::Cleanup,
                UsbTerminalCategory::CleanupFailed,
            );
        }
    }
    /// Serializes only after the measured operation and cleanup attempts finish.
    pub fn snapshot(&self) -> serde_json::Value {
        let r = self
            .0
            .lock()
            .expect("bounded recorder operations do not panic");
        let events: Vec<_> = r.events.iter().flatten().collect();
        let required = if r.duration_ms.is_some() {
            &STAGES[..15]
        } else {
            &STAGES[..10]
        };
        let missing: Vec<_> = required
            .iter()
            .filter(|s| !events.iter().any(|e| e.stage == **s))
            .collect();
        let mut value = serde_json::json!({"schema":"bootstrap-host-timing-v1","clock":"host_monotonic","origin":r.origin.map(|_|"usb_session_acquired"),"physicalIdentityDigest":r.identity,"sessionNonceSha256":r.nonce,"events":events,"resetChildSequence":r.reset,"readerOpenCount":r.opens,"readerReopenCount":r.reopens,"firstReadBytes":r.first,"missingStages":missing,"overflow":r.overflow,"clockDiscontinuity":r.clock,"captureComplete":r.capture && missing.is_empty() && r.reopens==0 && !r.overflow && !r.clock,"cleanupComplete":r.cleanup,"earliestFailure":r.failure});
        if let Some(duration) = r.duration_ms {
            let extra = serde_json::json!({"schema":"bootstrap-host-timing-v2", "captureMode":"early_quarantined", "captureDurationMs":duration,"readerBindingSha256":r.binding,"quarantinedBytes":if r.released {r.quarantined} else {r.captured},"capturedBytes":r.captured,"quarantineReleased":r.released,"readerJoined":r.joined,"captureOverflow":r.capture_overflow});
            value
                .as_object_mut()
                .expect("object")
                .extend(extra.as_object().expect("object").clone());
            value["captureComplete"] = serde_json::json!(
                r.capture
                    && missing.is_empty()
                    && r.opens == 1
                    && !r.overflow
                    && !r.clock
                    && !r.capture_overflow
                    && r.released
                    && r.joined
                    && r.failure.is_none()
                    && !events.iter().any(|e| e.stage == "cancellation_requested")
            );
        }
        value
    }
    pub fn early_mode(&self, duration: Duration) {
        self.0.lock().expect("bounded recorder").duration_ms =
            Some(duration.as_millis().min(u128::from(u64::MAX)) as u64);
    }
    pub(super) fn reader_bound(&self, binding: String) {
        self.0.lock().expect("bounded recorder").binding = Some(binding);
        self.event(TimingStage::ReaderBound);
    }
    pub(super) fn quarantine_released(&self, bytes: usize, deadline: Instant) -> bool {
        let mut r = self.0.lock().expect("bounded recorder");
        let now = Instant::now();
        if now >= deadline {
            return false;
        }
        r.released = true;
        r.quarantined = bytes;
        if let Some(origin) = r.origin {
            Self::record_locked(
                &mut r,
                TimingStage::QuarantineReleased,
                now.duration_since(origin).as_micros(),
            );
        }
        true
    }
    pub(super) fn captured(&self, bytes: usize) {
        self.0.lock().expect("bounded recorder").captured = bytes;
    }
    pub(super) fn reader_joined(&self) {
        self.0.lock().expect("bounded recorder").joined = true;
        self.event(TimingStage::ReaderJoined);
    }
    pub(super) fn capture_overflow(&self) {
        self.0.lock().expect("bounded recorder").capture_overflow = true;
        self.fail_label(BootstrapFailureStage::Quarantine, "capture_overflow");
    }
    pub(super) fn early_failure(&self, category: &'static str) {
        let (stage, label) = match category {
            "reader_binding_changed" => (BootstrapFailureStage::Binding, "reader_binding_changed"),
            "reader_join_timeout" => (BootstrapFailureStage::Join, "reader_join_timeout"),
            "capture_overflow" => (BootstrapFailureStage::Quarantine, "capture_overflow"),
            "reader_open_failed" => (BootstrapFailureStage::Open, "monitor_failed"),
            "reader_read_failed" | "capture_cancelled" => {
                (BootstrapFailureStage::Read, "monitor_failed")
            }
            _ => (BootstrapFailureStage::Quarantine, "admission_incomplete"),
        };
        self.fail_label(stage, label);
    }
}

/// Records local descriptor ownership release after the actual reader drops.
pub(super) struct TimedReader<T> {
    reader: Option<T>,
    timing: Option<BootstrapTiming>,
}
impl<T> TimedReader<T> {
    pub(super) fn new(reader: T, timing: Option<BootstrapTiming>) -> Self {
        Self {
            reader: Some(reader),
            timing,
        }
    }
}
impl<T> std::ops::Deref for TimedReader<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.reader.as_ref().expect("reader exists until Drop")
    }
}
impl<T> std::ops::DerefMut for TimedReader<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.reader.as_mut().expect("reader exists until Drop")
    }
}
impl<T> Drop for TimedReader<T> {
    fn drop(&mut self) {
        drop(self.reader.take());
        if let Some(timing) = &self.timing {
            timing.event(TimingStage::Closed);
        }
    }
}

#[cfg(test)]
mod tests;

impl UsbSession {
    /// Enables bounded observations using this session's actual lease and clock.
    pub fn enable_bootstrap_timing(&mut self, timing: BootstrapTiming) {
        timing.bind(
            self.acquired_at,
            &self.physical_identity_digest,
            self.lease.session_nonce(),
        );
        self.maybe_timing = Some(timing);
    }
    pub fn bootstrap_event(&self, stage: TimingStage) {
        if let Some(timing) = &self.maybe_timing {
            timing.event(stage);
        }
    }
    pub fn bootstrap_failure(&self, stage: BootstrapFailureStage, error: &UsbSessionError) {
        if let Some(timing) = &self.maybe_timing {
            timing.failure(stage, error.category);
        }
    }
    /// Includes supervised launch, execution and reap, not physical reset edges.
    pub fn run_bootstrap_reset(
        &mut self,
        program: &Path,
        args: &[String],
        timeout: Duration,
    ) -> Result<SupervisedOutput, UsbSessionError> {
        if self.capture.maybe_intent.is_some() {
            self.capture.maybe_signals =
                Some(super::process::SignalSupervisor::acquire_preserving_pending()?);
            if super::process::maybe_pending_signal().is_some() {
                let error =
                    super::session_error(UsbTerminalCategory::MonitorFailed, "capture_cancelled");
                self.bootstrap_event(TimingStage::CancellationRequested);
                self.bootstrap_failure(BootstrapFailureStage::Reset, &error);
                self.fail_once(error.category);
                return Err(error);
            }
        }
        if let Some(timing) = &self.maybe_timing {
            timing.reset_sequence(self.child_sequence.saturating_add(1));
        }
        self.bootstrap_event(TimingStage::ResetStart);
        let result = self.run_espflash_probe(program, args, timeout);
        self.bootstrap_event(TimingStage::ResetEnd);
        if let Err(error) = &result {
            self.bootstrap_failure(BootstrapFailureStage::Reset, error);
        } else if self.capture.maybe_intent.is_some() {
            self.capture.reset_reaped = true;
        }
        result
    }
}
