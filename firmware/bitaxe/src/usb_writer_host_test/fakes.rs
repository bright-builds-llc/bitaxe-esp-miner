//! Scripted collaborators for the production writer: clock, queues, transport, sources.
use super::*;

/// One scripted event consumed whenever the writer waits for output.
pub(crate) enum Step {
    At(u64),
    Hello {
        epoch: u32,
        session: &'static str,
    },
    Control {
        epoch: u32,
        request_sequence: u32,
        payload: Vec<u8>,
    },
    /// The writer's 10 ms wait elapses with no output.
    Timeout,
    Link(Effect),
    Diagnostic(&'static str),
    StartupMarker(Option<&'static str>),
    /// Applies `fault` just before admission check `check` of write `write` (1-based, counted from now).
    Arm {
        write: u32,
        check: u32,
        fault: Fault,
    },
}

/// Effects of the link thread and Worker owner on the writer's shared state.
#[derive(Clone, Copy)]
pub(crate) enum Effect {
    /// RX admitted `epoch`: it is current and its credit mailbox begins.
    Begin(u32),
    Publish(u32),
    Revoke,
    CloseRecord {
        received: u32,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum Fault {
    Effect(Effect),
    /// The native driver rejects the next chunk.
    Rejected,
}

pub(crate) static CLOCK: AtomicU64 = AtomicU64::new(0);
pub(crate) static STEPS: Mutex<VecDeque<Step>> = Mutex::new(VecDeque::new());
pub(crate) static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static OUTPUTS: Mutex<Option<queue::SyncSender<writer::Output>>> = Mutex::new(None);
static DIAGNOSTICS: Mutex<Option<queue::SyncSender<String>>> = Mutex::new(None);
pub(crate) static RECEIPTS: Mutex<Vec<bitaxe_runtime::reply::ReplyReceiver<bool>>> =
    Mutex::new(Vec::new());
static STARTUP_MARKER: Mutex<Option<&'static str>> = Mutex::new(None);
static WRITES: AtomicU32 = AtomicU32::new(0);
static ARMED: Mutex<Vec<(u32, u32, Fault)>> = Mutex::new(Vec::new());

pub(crate) fn log(line: String) {
    LOG.lock().expect("log lock").push(line);
}

pub(crate) fn install(
    outputs: queue::SyncSender<writer::Output>,
    diagnostics: queue::SyncSender<String>,
) {
    *OUTPUTS.lock().expect("outputs lock") = Some(outputs);
    *DIAGNOSTICS.lock().expect("diagnostics lock") = Some(diagnostics);
    *STARTUP_MARKER.lock().expect("marker lock") = None;
    ARMED.lock().expect("armed lock").clear();
    RECEIPTS.lock().expect("receipts lock").clear();
    WRITES.store(0, Ordering::Release);
    usb_runtime::reset();
}

fn apply(effect: Effect) {
    let now = CLOCK.load(Ordering::Acquire);
    match effect {
        Effect::Begin(epoch) => {
            log(format!("link_begin epoch={epoch}"));
            CURRENT_SESSION.store(epoch, Ordering::Release);
            RECEIVE_CREDIT.mailbox().begin(epoch);
        }
        Effect::Publish(received) => {
            log(format!("link_publish received={received}"));
            RECEIVE_CREDIT.mailbox().publish(received);
        }
        Effect::Revoke => {
            let epoch = CURRENT_SESSION.load(Ordering::Acquire);
            log(format!("link_revoke epoch={epoch}"));
            revoke_epoch(epoch);
        }
        Effect::CloseRecord { received } => {
            let epoch = CURRENT_SESSION.load(Ordering::Acquire);
            log(format!(
                "link_close_record epoch={epoch} received={received}"
            ));
            RECEIVE_CREDIT
                .mailbox()
                .close_record(epoch, received, now, || revoke_epoch(epoch));
        }
    }
}

enum Next {
    Ready,
    Timeout,
    End,
}

fn next_output() -> Next {
    loop {
        let Some(step) = STEPS.lock().expect("steps lock").pop_front() else {
            return Next::End;
        };
        match step {
            Step::At(at_ms) => CLOCK.store(at_ms, Ordering::Release),
            Step::Hello { epoch, session } => {
                let payload = serde_json::json!({"op": "hello_ack", "session": session});
                log(format!("deliver hello epoch={epoch} session={session}"));
                send_output(writer::Output::Hello {
                    epoch,
                    session_id: session.to_owned(),
                    payload,
                });
                return Next::Ready;
            }
            Step::Control {
                epoch,
                request_sequence,
                payload,
            } => {
                log(format!(
                    "deliver control epoch={epoch} request_sequence={request_sequence} bytes={}",
                    payload.len()
                ));
                let (receipt, completion) = bitaxe_runtime::reply::reply();
                RECEIPTS.lock().expect("receipts lock").push(completion);
                send_output(writer::Output::Control {
                    epoch,
                    request_sequence,
                    bytes: SecretBytes(payload),
                    receipt,
                });
                return Next::Ready;
            }
            Step::Timeout => return Next::Timeout,
            Step::Link(effect) => apply(effect),
            Step::Diagnostic(line) => {
                let sent = DIAGNOSTICS
                    .lock()
                    .expect("diagnostics lock")
                    .as_ref()
                    .is_some_and(|sender| sender.try_send(line.to_owned()).is_ok());
                log(format!("queue diagnostic accepted={sent}"));
            }
            Step::StartupMarker(maybe_marker) => {
                *STARTUP_MARKER.lock().expect("marker lock") = maybe_marker;
            }
            Step::Arm {
                write,
                check,
                fault,
            } => {
                let base = WRITES.load(Ordering::Acquire);
                ARMED
                    .lock()
                    .expect("armed lock")
                    .push((base + write, check, fault));
            }
        }
    }
}

fn send_output(output: writer::Output) {
    let accepted = OUTPUTS
        .lock()
        .expect("outputs lock")
        .as_ref()
        .is_some_and(|sender| sender.try_send(output).is_ok());
    assert!(accepted, "scripted output fits the writer queue");
}

/// Runs armed effects for this check; reports whether the driver rejects the chunk.
fn before_check(write: u32, check: u32) -> bool {
    let mut rejected = false;
    let fired: Vec<Fault> = {
        let mut armed = ARMED.lock().expect("armed lock");
        let fired = armed
            .iter()
            .filter(|(w, c, _)| *w == write && *c == check)
            .map(|(_, _, fault)| *fault)
            .collect();
        armed.retain(|(w, c, _)| !(*w == write && *c == check));
        fired
    };
    for fault in fired {
        match fault {
            Fault::Effect(effect) => apply(effect),
            Fault::Rejected => rejected = true,
        }
    }
    rejected
}

pub(crate) mod queue {
    use std::sync::mpsc;
    use std::time::Duration;

    pub(crate) struct SyncSender<T>(mpsc::SyncSender<T>);
    impl<T> SyncSender<T> {
        pub(crate) fn try_send(&self, value: T) -> Result<(), mpsc::TrySendError<T>> {
            self.0.try_send(value)
        }
    }

    pub(crate) struct Receiver<T>(mpsc::Receiver<T>);
    impl<T> Receiver<T> {
        /// Consumes script steps instead of waiting in real time.
        pub(crate) fn recv_timeout(&self, _timeout: Duration) -> Result<T, mpsc::RecvTimeoutError> {
            match super::next_output() {
                super::Next::Ready => self
                    .0
                    .try_recv()
                    .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
                super::Next::Timeout => Err(mpsc::RecvTimeoutError::Timeout),
                super::Next::End => Err(mpsc::RecvTimeoutError::Disconnected),
            }
        }
        pub(crate) fn try_recv(&self) -> Result<T, mpsc::TryRecvError> {
            self.0.try_recv()
        }
    }

    pub(crate) fn sync_channel<T>(bound: usize) -> (SyncSender<T>, Receiver<T>) {
        let (sender, receiver) = mpsc::sync_channel(bound);
        (SyncSender(sender), Receiver(receiver))
    }
}

/// Mirrors `usb_runtime.rs`: admission before every 512-byte chunk, before the
/// drain and after the flush; partial output only when some but not all bytes queued.
pub(crate) mod usb_runtime {
    use super::*;
    pub(crate) use crate::usb_tx_measurement as measurement;
    pub(crate) use crate::usb_write_failure::{
        WriteFailure, WriteObservation, WriteObservationStage, WriteStage,
    };
    use measurement::{Category, RecordKind, Retained};

    static PARTIAL: AtomicBool = AtomicBool::new(false);

    pub(crate) fn reset() {
        PARTIAL.store(false, Ordering::Release);
    }

    pub(crate) fn has_partial_output() -> bool {
        PARTIAL.load(Ordering::Acquire)
    }

    pub(crate) fn write_measured_if<F: FnMut(WriteObservation)>(
        _retained: &mut Retained,
        bytes: &[u8],
        admitted: impl Fn() -> bool,
        mut observe: F,
        category: Category,
        kind: RecordKind,
    ) -> anyhow::Result<()> {
        let write = WRITES.fetch_add(1, Ordering::AcqRel) + 1;
        let now = || CLOCK.load(Ordering::Acquire);
        log(format!(
            "write #{write} {category:?}/{kind:?} bytes={}",
            bytes.len()
        ));
        let fail = |stage: WriteStage, queued: usize, observe: &mut F| {
            if queued > 0 && queued < bytes.len() {
                PARTIAL.store(true, Ordering::Release);
            }
            observe(WriteObservation {
                stage: WriteObservationStage::Abandoned,
                at_ms: now(),
                queued_bytes: queued,
                record_bytes: bytes.len(),
            });
            log(format!("abandon #{write} {stage:?} queued={queued}"));
            WriteFailure {
                stage,
                elapsed_ms: 0,
                queued_bytes: queued,
                record_bytes: bytes.len(),
            }
        };
        let mut queued = 0;
        let mut check = 0;
        while queued < bytes.len() {
            check += 1;
            let rejected = before_check(write, check);
            if !admitted() {
                return Err(fail(WriteStage::Cancelled, queued, &mut observe).into());
            }
            if rejected {
                return Err(fail(WriteStage::Write, queued, &mut observe).into());
            }
            queued += (bytes.len() - queued).min(512);
            observe(WriteObservation {
                stage: WriteObservationStage::Queued,
                at_ms: now(),
                queued_bytes: queued,
                record_bytes: bytes.len(),
            });
        }
        for _ in 0..2 {
            check += 1;
            before_check(write, check);
            if !admitted() {
                return Err(fail(WriteStage::Cancelled, bytes.len(), &mut observe).into());
            }
        }
        observe(WriteObservation {
            stage: WriteObservationStage::Completed,
            at_ms: now(),
            queued_bytes: bytes.len(),
            record_bytes: bytes.len(),
        });
        log(format!(
            "wire #{write} {:?}",
            String::from_utf8_lossy(bytes)
        ));
        Ok(())
    }

    pub(crate) fn resynchronize_if(
        retained: &mut Retained,
        admitted: impl Fn() -> bool,
    ) -> anyhow::Result<()> {
        if PARTIAL.load(Ordering::Acquire) {
            write_measured_if(
                retained,
                b"\n",
                admitted,
                |_| {},
                Category::Resynchronization,
                RecordKind::Resynchronization,
            )?;
            PARTIAL.store(false, Ordering::Release);
        }
        Ok(())
    }
}

pub(crate) mod trace {
    use super::*;
    pub(crate) fn event(
        correlation: SerialTraceCorrelation,
        stage: SerialTraceStage,
        wire_bytes: usize,
    ) {
        log(format!(
            "trace epoch={} sequence={} stage={stage:?} wire={wire_bytes}",
            correlation.epoch, correlation.request_sequence
        ));
    }
    pub(crate) fn observe(
        correlation: SerialTraceCorrelation,
        observation: usb_runtime::WriteObservation,
    ) {
        log(format!(
            "observe epoch={} sequence={} {:?} queued={}/{}",
            correlation.epoch,
            correlation.request_sequence,
            observation.stage,
            observation.queued_bytes,
            observation.record_bytes
        ));
    }
}

pub(crate) mod heap_sample {
    pub(crate) fn marker(now_ms: u64) -> String {
        format!("internal_heap fixture=true uptime_ms={now_ms} redacted=true")
    }
}

pub(crate) mod runtime_uptime {
    pub(crate) fn millis() -> u64 {
        super::CLOCK.load(super::Ordering::Acquire)
    }
}

pub(crate) mod boot_evidence {
    pub(crate) fn worker_usb_boot_marker() -> String {
        "usb_reboot_discriminator schema=v1 boot_ordinal=2 reset_reason=software_cpu uptime_ms=1 redacted=true".to_owned()
    }
    pub(crate) fn maybe_worker_diagnostic_line(slot: usize) -> Option<String> {
        (slot == 0).then(|| "usb_runtime_identity fixture=true redacted=true".to_owned())
    }
}

pub(crate) mod wifi_adapter {
    pub(crate) fn maybe_startup_failure_marker() -> Option<String> {
        None
    }
}

pub(crate) mod production_mining_session {
    pub(crate) mod admission_diagnostics {
        pub(crate) fn marker() -> String {
            "worker_admission fixture=true redacted=true".to_owned()
        }
    }
    pub(crate) mod revocation {
        pub(crate) fn maybe_unsafe_detail_marker() -> Option<String> {
            None
        }
    }
}

pub(crate) mod statistics_runtime {
    pub(crate) mod diagnostics {
        pub(crate) struct Startup;
        impl Startup {
            pub(crate) fn maybe_marker(&self) -> Option<String> {
                super::super::STARTUP_MARKER
                    .lock()
                    .expect("marker lock")
                    .map(str::to_owned)
            }
        }
        pub(crate) static STARTUP: Startup = Startup;
    }
}
