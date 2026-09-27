//! One retained receive descriptor while both admission passes finish.
use super::{
    session_error, BootstrapTiming, MonitorOutput, TimingStage, UsbSessionError,
    UsbTerminalCategory,
};
use crate::macos::{ReceiveOnlyReader, UsbDeviceSnapshot};
use std::fs::{File, Metadata};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{sync_channel, Receiver},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
#[derive(Default)]
pub(super) struct CaptureState {
    pub maybe_intent: Option<(Duration, Vec<u8>)>,
    pub maybe_owner: Option<EarlyCapture>,
    pub maybe_signals: Option<super::process::SignalSupervisor>,
    pub reset_reaped: bool,
    pub join_failed: bool,
}
pub(super) const MAX_CAPTURE_BYTES: usize = 16 * 1024 * 1024;
const THREAD_STACK: usize = 256 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct NodeIdentity {
    dev: u64,
    ino: u64,
    rdev: u64,
}
impl NodeIdentity {
    fn from_metadata(metadata: Metadata) -> Result<Self, UsbSessionError> {
        if !metadata.file_type().is_char_device() {
            return Err(failure("reader_binding_changed"));
        }
        Ok(Self {
            dev: metadata.dev(),
            ino: metadata.ino(),
            rdev: metadata.rdev(),
        })
    }
}
#[derive(Clone)]
pub(super) struct Candidate {
    pub port: String,
    physical: String,
    enumeration: String,
    node: NodeIdentity,
}
impl Candidate {
    pub fn from_snapshot(snapshot: &UsbDeviceSnapshot) -> Result<Self, UsbSessionError> {
        if snapshot.profile != crate::UsbProfile::SerialJtagRuntime
            || !snapshot.accessible
            || snapshot.holder_count != 0
        {
            return Err(failure("reader_binding_changed"));
        }
        Ok(Self {
            port: snapshot.port.clone(),
            physical: snapshot.physical_identity_digest.clone(),
            enumeration: snapshot.enumeration_token.clone(),
            node: NodeIdentity::from_metadata(
                std::fs::metadata(&snapshot.port).map_err(|_| failure("reader_binding_changed"))?,
            )?,
        })
    }
    pub fn matches(&self, snapshot: &UsbDeviceSnapshot) -> Result<(), UsbSessionError> {
        let current = Self::from_snapshot(snapshot)?;
        if self.port != current.port
            || self.physical != current.physical
            || self.enumeration != current.enumeration
            || self.node != current.node
        {
            return Err(failure("reader_binding_changed"));
        }
        Ok(())
    }
    fn binding(&self) -> String {
        let value=format!("{{\"physicalIdentityDigest\":{},\"enumerationToken\":{},\"port\":{},\"fdDev\":\"{}\",\"fdIno\":\"{}\",\"fdRdev\":\"{}\"}}",serde_json::to_string(&self.physical).expect("string"),serde_json::to_string(&self.enumeration).expect("string"),serde_json::to_string(&self.port).expect("string"),self.node.dev,self.node.ino,self.node.rdev);
        super::sha256(value.as_bytes())
    }
}
#[derive(Default)]
struct State {
    opened: bool,
    captured: usize,
    released: bool,
    maybe_error: Option<UsbSessionError>,
    opened_at: Option<Instant>,
}
pub(super) struct EarlyCapture {
    candidate: Candidate,
    state: Arc<Mutex<State>>,
    admitted: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    receiver: Receiver<Result<MonitorOutput, UsbSessionError>>,
    thread: Option<JoinHandle<()>>,
    maybe_timing: Option<BootstrapTiming>,
    duration: Duration,
}
impl EarlyCapture {
    pub fn start(
        candidate: Candidate,
        lease: Arc<File>,
        duration: Duration,
        mut bytes: Vec<u8>,
        maybe_timing: Option<BootstrapTiming>,
    ) -> Result<Self, UsbSessionError> {
        bytes.clear();
        let state = Arc::new(Mutex::new(State::default()));
        let admitted = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = sync_channel(1);
        let args = (
            candidate.clone(),
            state.clone(),
            admitted.clone(),
            cancelled.clone(),
            maybe_timing.clone(),
        );
        let thread = thread::Builder::new()
            .name("usb-early-reader".into())
            .stack_size(THREAD_STACK)
            .spawn(move || {
                let (candidate, state, admitted, cancelled, timing) = args;
                // The same lock descriptor survives even when the parent cannot join.
                let _lease = lease;
                let result = receive(
                    candidate,
                    &state,
                    &admitted,
                    &cancelled,
                    duration,
                    bytes,
                    timing.as_ref(),
                );
                if let Err(error) = &result {
                    state.lock().expect("bounded state").maybe_error = Some(error.clone());
                    if let Some(t) = &timing {
                        t.early_failure(error_label(error));
                    }
                }
                let _receiver_gone = sender.send(result);
            })
            .map_err(|_| failure("reader_spawn_failed"))?;
        Ok(Self {
            candidate,
            state,
            admitted,
            cancelled,
            receiver,
            thread: Some(thread),
            maybe_timing,
            duration,
        })
    }
    pub fn check(&self, snapshot: &UsbDeviceSnapshot) -> Result<(), UsbSessionError> {
        self.candidate.matches(snapshot)?;
        self.health()
    }
    pub fn health(&self) -> Result<(), UsbSessionError> {
        if let Some(error) = &self.state.lock().expect("bounded state").maybe_error {
            return Err(error.clone());
        }
        Ok(())
    }
    pub fn expired(&self) -> bool {
        self.state
            .lock()
            .expect("bounded state")
            .opened_at
            .is_some_and(|at| at.elapsed() >= self.duration)
    }
    pub fn opened(&self) -> bool {
        self.state.lock().expect("bounded state").opened
    }
    pub fn check_profile(
        &self,
        profile: &crate::UsbProfileInspection,
    ) -> Result<(), UsbSessionError> {
        if profile.profile != crate::UsbProfile::SerialJtagRuntime
            || profile.physical_identity_digest != self.candidate.physical
            || profile.enumeration_token != self.candidate.enumeration
            || profile.port != self.candidate.port
        {
            return Err(failure("reader_binding_changed"));
        }
        self.health()
    }
    pub fn admit(&self) {
        self.admitted.store(true, Ordering::Release);
    }
    pub fn finish(
        &mut self,
        cancel: bool,
        limit: Duration,
    ) -> Result<MonitorOutput, UsbSessionError> {
        if cancel {
            self.cancelled.store(true, Ordering::Release);
            if let Some(t) = &self.maybe_timing {
                t.event(TimingStage::CancellationRequested);
            }
        }
        let end = Instant::now() + limit;
        while self
            .thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
        {
            if Instant::now() >= end {
                if let Some(t) = &self.maybe_timing {
                    t.early_failure("reader_join_timeout");
                }
                return Err(failure("reader_join_timeout"));
            }
            thread::sleep(Duration::from_millis(5));
        }
        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| failure("reader_join_failed"))?;
            if let Some(t) = &self.maybe_timing {
                t.reader_joined();
            }
        }
        self.receiver
            .try_recv()
            .map_err(|_| failure("reader_result_missing"))?
    }
}
impl Drop for EarlyCapture {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
fn receive(
    candidate: Candidate,
    state: &Mutex<State>,
    admitted: &AtomicBool,
    cancelled: &AtomicBool,
    duration: Duration,
    mut bytes: Vec<u8>,
    timing: Option<&BootstrapTiming>,
) -> Result<MonitorOutput, UsbSessionError> {
    let capacity = bytes.capacity().min(MAX_CAPTURE_BYTES);
    if cancelled.load(Ordering::Acquire) || super::process::maybe_pending_signal().is_some() {
        if let Some(t) = timing {
            t.event(TimingStage::CancellationRequested);
        }
        return Err(failure("capture_cancelled"));
    }
    if let Some(t) = timing {
        t.event(TimingStage::OpenStart);
    }
    let reader =
        ReceiveOnlyReader::open(&candidate.port).map_err(|_| failure("reader_open_failed"))?;
    let mut reader = super::timing::TimedReader::new(reader, timing.cloned());
    let opened = Instant::now();
    state.lock().expect("bounded state").opened_at = Some(opened);
    if let Some(t) = timing {
        t.event(TimingStage::Opened);
    }
    if NodeIdentity::from_metadata(
        reader
            .metadata()
            .map_err(|_| failure("reader_binding_changed"))?,
    )? != candidate.node
    {
        return Err(failure("reader_binding_changed"));
    }
    if let Some(t) = timing {
        t.reader_bound(candidate.binding());
    }
    state.lock().expect("bounded state").opened = true;
    let mut buffer = [0u8; 4096];
    let mut released = false;
    loop {
        if cancelled.load(Ordering::Acquire) || super::process::maybe_pending_signal().is_some() {
            if let Some(t) = timing {
                t.event(TimingStage::CancellationRequested);
            }
            return Err(failure("capture_cancelled"));
        }
        if opened.elapsed() >= duration {
            if let Some(t) = timing {
                t.event(TimingStage::CaptureDeadlineReached);
            }
            break;
        }
        if !released && admitted.load(Ordering::Acquire) {
            released = timing.map_or_else(
                || Instant::now() < opened + duration,
                |t| t.quarantine_released(bytes.len(), opened + duration),
            );
            state.lock().expect("bounded state").released = released;
            if !released {
                continue;
            }
        }
        match reader.read_into(&mut buffer) {
            Ok(0) => thread::sleep(Duration::from_millis(25)),
            Ok(count) => {
                if bytes.len().saturating_add(count) > capacity {
                    if let Some(t) = timing {
                        t.capture_overflow();
                    }
                    return Err(failure("capture_overflow"));
                }
                bytes.extend_from_slice(&buffer[..count]);
                state.lock().expect("bounded state").captured = bytes.len();
                if let Some(t) = timing {
                    t.first_read(count);
                    t.captured(bytes.len());
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25))
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(failure("reader_read_failed")),
        }
    }
    drop(reader);
    if !released {
        return Err(failure("admission_incomplete"));
    }
    if let Some(t) = timing {
        t.capture_complete();
    }
    Ok(MonitorOutput {
        bytes,
        interrupted_by: None,
        reenumerated: false,
    })
}
fn failure(detail: &'static str) -> UsbSessionError {
    session_error(UsbTerminalCategory::MonitorFailed, detail)
}
fn error_label(error: &UsbSessionError) -> &'static str {
    match error.detail.as_str() {
        "reader_binding_changed" => "reader_binding_changed",
        "reader_open_failed" => "reader_open_failed",
        "reader_read_failed" => "reader_read_failed",
        "capture_cancelled" => "capture_cancelled",
        "capture_overflow" => "capture_overflow",
        "reader_join_timeout" => "reader_join_timeout",
        _ => "admission_incomplete",
    }
}
#[cfg(test)]
mod tests;

impl super::UsbSession {
    /// Prepare bounded ordinary capture; the receiver cannot start until reset is reaped.
    pub fn prepare_post_reset_capture(
        &mut self,
        duration: Duration,
    ) -> Result<(), UsbSessionError> {
        if duration.is_zero()
            || self.capture.maybe_intent.is_some()
            || self.capture.maybe_owner.is_some()
        {
            return Err(failure("capture_preparation_failed"));
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(MAX_CAPTURE_BYTES)
            .map_err(|_| failure("capture_preparation_failed"))?;
        if let Some(t) = &self.maybe_timing {
            t.early_mode(duration);
        }
        self.capture.maybe_intent = Some((duration, bytes));
        Ok(())
    }
    pub(super) fn early_candidate(
        &mut self,
        snapshot: &UsbDeviceSnapshot,
    ) -> Result<(), UsbSessionError> {
        let result = self.early_candidate_inner(snapshot);
        if let Err(error) = &result {
            if let Some(t) = &self.maybe_timing {
                t.early_failure(error_label(error));
            }
        }
        result
    }
    fn early_candidate_inner(
        &mut self,
        snapshot: &UsbDeviceSnapshot,
    ) -> Result<(), UsbSessionError> {
        if !self.capture.reset_reaped {
            return Ok(());
        }
        if super::process::maybe_pending_signal().is_some() {
            return Err(failure("capture_cancelled"));
        }
        if let Some(capture) = &self.capture.maybe_owner {
            return capture.check(snapshot);
        }
        if self.capture.maybe_intent.is_none() {
            return Ok(());
        }
        if snapshot.physical_identity_digest != self.physical_identity_digest {
            return Err(failure("reader_binding_changed"));
        }
        let candidate = Candidate::from_snapshot(snapshot)?;
        self.bootstrap_event(TimingStage::CandidateObserved);
        let (duration, bytes) = self
            .capture
            .maybe_intent
            .take()
            .expect("capture intent checked");
        self.capture.maybe_owner = Some(EarlyCapture::start(
            candidate,
            self.lease.receiver_guard(),
            duration,
            bytes,
            self.maybe_timing.clone(),
        )?);
        Ok(())
    }
    pub(super) fn stop_early_capture(&mut self) -> Result<(), UsbSessionError> {
        if self.capture.join_failed {
            return Err(failure("reader_join_timeout"));
        }
        if let Some(mut capture) = self.capture.maybe_owner.take() {
            let result = capture.finish(true, Duration::from_secs(5));
            if let Err(error) = result {
                if error.detail == "reader_join_timeout" || capture.thread.is_some() {
                    return Err(error);
                }
            }
        }
        self.capture.maybe_signals = None;
        Ok(())
    }
    pub(super) fn observe_early_capture(&mut self) -> Result<MonitorOutput, UsbSessionError> {
        self.transition(super::UsbLifecycleEvent::BeginObservation)?;
        self.bootstrap_event(TimingStage::MonitorStart);
        let snapshot = self.reacquire(super::RecoveryPhase::MonitorAdmission)?;
        self.bootstrap_event(TimingStage::MonitorAdmitted);
        let capture = self
            .capture
            .maybe_owner
            .as_ref()
            .ok_or_else(|| failure("admission_incomplete"))?;
        capture.check(&snapshot)?;
        capture.admit();
        // Continue polling signals while the same owner retains the descriptor and buffer.
        while self.capture.maybe_owner.as_ref().is_some_and(|capture| {
            capture
                .thread
                .as_ref()
                .is_some_and(|thread| !thread.is_finished())
        }) {
            if super::process::maybe_pending_signal().is_some()
                || self
                    .capture
                    .maybe_owner
                    .as_ref()
                    .is_some_and(EarlyCapture::expired)
            {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        let mut capture = self
            .capture
            .maybe_owner
            .take()
            .expect("capture remains owned");
        let result = capture.finish(
            super::process::maybe_pending_signal().is_some(),
            Duration::from_secs(5),
        );
        if capture.thread.is_some() {
            self.capture.join_failed = true;
            self.capture.maybe_owner = Some(capture);
            return Err(failure("reader_join_timeout"));
        }
        self.capture.maybe_signals = None;
        let output = result?;
        self.child_sequence = self.child_sequence.saturating_add(1);
        let trace = self
            .trace_root
            .join(format!("monitor-{:04}.serial", self.child_sequence));
        super::write_private_trace(&trace, &output.bytes)?;
        self.transition(super::UsbLifecycleEvent::ObservationComplete)?;
        Ok(MonitorOutput {
            reenumerated: snapshot.enumeration_token != self.initial_enumeration_token,
            ..output
        })
    }
}
