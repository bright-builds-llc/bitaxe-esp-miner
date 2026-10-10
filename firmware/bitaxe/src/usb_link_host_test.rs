//! Runs the production serial link loop against logging fakes and pins its ordered effects.
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Firmware parent module of `link.rs`, with each collaborator replaced by a logging fake.
mod bwg_worker_usb {
    #[allow(dead_code)]
    #[path = "link.rs"]
    mod link;
    #[allow(dead_code)]
    #[path = "rx_diagnostics.rs"]
    mod rx_diagnostics;

    pub(crate) use crate::fake_revocation as revocation;
    use crate::fake_revocation::WorkerGeneration;
    use bitaxe_runtime::queue::SyncSender;
    use bitaxe_worker_control::serial::trace::{SerialTraceCorrelation, SerialTraceStage};
    use bitaxe_worker_control::serial::{SerialKind, SerialSessionBinding};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::OnceLock;
    use std::time::Duration;
    use zeroize::Zeroize;

    pub(crate) static EVENTS: OnceLock<SyncSender<ControlEvent>> = OnceLock::new();
    pub(crate) static CURRENT_SESSION: AtomicU32 = AtomicU32::new(0);
    pub(crate) static AUTHENTICATED_SESSION: AtomicU32 = AtomicU32::new(0);
    pub(crate) static RECEIVE_CREDIT: crate::CreditLog = crate::CreditLog::new();

    pub(crate) enum ControlEvent {
        Session {
            epoch: u32,
            binding: SerialSessionBinding,
            generation: WorkerGeneration,
        },
        Frame {
            correlation: SerialTraceCorrelation,
            wire_bytes: usize,
            bytes: SecretBytes,
        },
    }
    pub(crate) struct SecretBytes(pub(crate) Vec<u8>);
    impl Drop for SecretBytes {
        fn drop(&mut self) {
            self.0.zeroize();
        }
    }

    pub(crate) fn revoke_epoch(epoch: u32) {
        crate::log(format!("revoke_epoch epoch={epoch}"));
        if CURRENT_SESSION
            .compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            RECEIVE_CREDIT.close(epoch);
        }
    }

    pub(crate) fn run_link() {
        link::run();
    }

    pub(crate) fn failure_marker() -> Option<String> {
        rx_diagnostics::FAILURE.marker()
    }

    pub(crate) fn clear_failure() {
        rx_diagnostics::FAILURE.clear();
    }

    pub(crate) mod writer {
        pub(crate) fn hello(
            epoch: u32,
            session_id: &str,
            payload: serde_json::Value,
        ) -> anyhow::Result<()> {
            crate::log(format!(
                "writer_hello epoch={epoch} session={session_id} payload={payload}"
            ));
            anyhow::ensure!(!crate::knob(&crate::HELLO_ACK_FAILS), "hello_ack_failed");
            Ok(())
        }
    }

    pub(crate) mod trace {
        use super::{SerialTraceCorrelation, SerialTraceStage};
        pub(crate) fn event(
            correlation: SerialTraceCorrelation,
            stage: SerialTraceStage,
            wire_bytes: usize,
        ) {
            crate::log(format!(
                "trace epoch={} sequence={} stage={stage:?} wire={wire_bytes}",
                correlation.epoch, correlation.request_sequence
            ));
        }
    }
}

/// The firmware revocation facade, reduced to its call log and scripted results.
mod fake_revocation {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) struct WorkerGeneration(pub(crate) u32);
    #[derive(Debug)]
    pub(crate) enum RevocationReason {
        LinkClosed,
    }
    pub(crate) fn check_deadline(now_ms: u64) {
        crate::log(format!("check_deadline now={now_ms}"));
    }
    pub(crate) fn begin_link(now_ms: u64) -> Option<WorkerGeneration> {
        let generation = (!crate::knob(&crate::BEGIN_LINK_FAILS)).then(|| {
            WorkerGeneration(crate::NEXT_GENERATION.fetch_add(1, super::Ordering::AcqRel))
        });
        crate::log(format!("begin_link now={now_ms} -> {generation:?}"));
        generation
    }
    pub(crate) fn heartbeat(generation: WorkerGeneration, now_ms: u64) -> bool {
        let live = !crate::knob(&crate::WORK_COOLING);
        crate::log(format!(
            "revocation_heartbeat {generation:?} now={now_ms} -> {live}"
        ));
        live
    }
    pub(crate) fn revoke_reason_at(
        generation: WorkerGeneration,
        now_ms: u64,
        reason: RevocationReason,
    ) {
        crate::log(format!(
            "revoke_reason_at {generation:?} now={now_ms} reason={reason:?}"
        ));
    }
}

mod runtime_uptime {
    pub(crate) fn millis() -> u64 {
        crate::CLOCK.load(super::Ordering::Acquire)
    }
}

mod usb_runtime {
    pub(crate) fn read(bytes: &mut [u8]) -> anyhow::Result<usize> {
        crate::next_read(bytes)
    }
}

mod crypto_entropy {
    #[derive(Debug)]
    pub(crate) struct EntropyError;
    pub(crate) fn fill(bytes: &mut [u8]) -> Result<(), EntropyError> {
        crate::log(format!("entropy bytes={}", bytes.len()));
        if crate::knob(&crate::ENTROPY_FAILS) {
            return Err(EntropyError);
        }
        for byte in bytes {
            *byte = crate::ENTROPY.fetch_add(1, super::Ordering::AcqRel);
        }
        Ok(())
    }
}

fn firmware_commit() -> &'static str {
    "0123456789abcdef0123456789abcdef01234567"
}

fn app_elf_sha256() -> String {
    "e".repeat(64)
}

/// Logs every mailbox call and delegates to a real mailbox replaced per script.
pub(crate) struct CreditLog(
    Mutex<Option<std::sync::Arc<bitaxe_worker_control::serial::ReceiveCreditMailbox>>>,
);
impl CreditLog {
    const fn new() -> Self {
        Self(Mutex::new(None))
    }
    fn mailbox(&self) -> std::sync::Arc<bitaxe_worker_control::serial::ReceiveCreditMailbox> {
        self.0
            .lock()
            .expect("credit lock")
            .get_or_insert_with(Default::default)
            .clone()
    }
    fn reset(&self) {
        *self.0.lock().expect("credit lock") = None;
    }
    pub(crate) fn begin(&self, epoch: u32) {
        log(format!("credit_begin epoch={epoch}"));
        self.mailbox().begin(epoch);
    }
    pub(crate) fn publish(&self, received_bytes: u32) {
        log(format!("credit_publish received={received_bytes}"));
        self.mailbox().publish(received_bytes);
    }
    pub(crate) fn close(&self, epoch: u32) {
        log(format!("credit_close epoch={epoch}"));
        self.mailbox().close(epoch);
    }
    pub(crate) fn close_record(
        &self,
        epoch: u32,
        received: u32,
        now_ms: u64,
        revoke: impl FnOnce(),
    ) {
        log(format!(
            "credit_close_record epoch={epoch} received={received} now={now_ms}"
        ));
        self.mailbox().close_record(epoch, received, now_ms, revoke);
    }
    fn summary(&self, epoch: u32) -> String {
        let mailbox = self.mailbox();
        format!(
            "credit received={:?} terminal={:?}",
            mailbox.maybe_received_bytes(epoch),
            mailbox.maybe_terminal_bytes(epoch, CLOCK.load(Ordering::Acquire))
        )
    }
}

/// One scripted host interaction consumed by the fake USB read.
pub(crate) enum Step {
    Chunk {
        at_ms: u64,
        bytes: Vec<u8>,
    },
    ReadError {
        at_ms: u64,
    },
    /// The Worker owner confirms possession for the current epoch.
    Authenticate,
    /// The Worker owner revokes the current epoch.
    Revoke,
    /// The Worker owner drains its bounded queue.
    Drain,
    Set(&'static AtomicBool, bool),
}

struct ScriptComplete;

static TEST_LOCK: Mutex<()> = Mutex::new(());
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static STEPS: Mutex<VecDeque<Step>> = Mutex::new(VecDeque::new());
static OWNER: OnceLock<Mutex<bitaxe_runtime::queue::Receiver<bwg_worker_usb::ControlEvent>>> =
    OnceLock::new();
static CLOCK: AtomicU64 = AtomicU64::new(0);
static ENTROPY: AtomicU8 = AtomicU8::new(0);
static NEXT_GENERATION: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
pub(crate) static ENTROPY_FAILS: AtomicBool = AtomicBool::new(false);
pub(crate) static BEGIN_LINK_FAILS: AtomicBool = AtomicBool::new(false);
pub(crate) static HELLO_ACK_FAILS: AtomicBool = AtomicBool::new(false);
pub(crate) static WORK_COOLING: AtomicBool = AtomicBool::new(false);

fn knob(flag: &AtomicBool) -> bool {
    flag.load(Ordering::Acquire)
}

fn failure() -> String {
    format!("failure={:?}", bwg_worker_usb::failure_marker())
}

fn log(line: String) {
    LOG.lock().expect("log lock").push(line);
}

fn owner() -> MutexGuard<'static, bitaxe_runtime::queue::Receiver<bwg_worker_usb::ControlEvent>> {
    OWNER
        .get_or_init(|| {
            let (sender, receiver) = bitaxe_runtime::queue::sync_channel(4);
            assert!(
                bwg_worker_usb::EVENTS.set(sender).is_ok(),
                "events installed once"
            );
            Mutex::new(receiver)
        })
        .lock()
        .expect("owner lock")
}

fn drain() {
    use bwg_worker_usb::ControlEvent;
    let receiver = owner();
    while let Ok(event) = receiver.try_recv() {
        match event {
            ControlEvent::Session {
                epoch,
                binding,
                generation,
            } => log(format!(
                "owner_session epoch={epoch} session={} host={} device={} {generation:?}",
                binding.session_id, binding.host_nonce, binding.device_nonce
            )),
            ControlEvent::Frame {
                correlation,
                wire_bytes,
                bytes,
            } => log(format!(
                "owner_frame epoch={} sequence={} wire={wire_bytes} bytes={:?}",
                correlation.epoch,
                correlation.request_sequence,
                String::from_utf8_lossy(&bytes.0)
            )),
        }
    }
}

fn next_read(buffer: &mut [u8]) -> anyhow::Result<usize> {
    loop {
        let Some(step) = STEPS.lock().expect("steps lock").pop_front() else {
            std::panic::resume_unwind(Box::new(ScriptComplete));
        };
        match step {
            Step::Chunk { at_ms, mut bytes } => {
                CLOCK.store(at_ms, Ordering::Release);
                if bytes.len() > buffer.len() {
                    let rest = bytes.split_off(buffer.len());
                    STEPS
                        .lock()
                        .expect("steps lock")
                        .push_front(Step::Chunk { at_ms, bytes: rest });
                }
                buffer[..bytes.len()].copy_from_slice(&bytes);
                log(format!(
                    "read at={at_ms} bytes={} {}",
                    bytes.len(),
                    failure()
                ));
                return Ok(bytes.len());
            }
            Step::ReadError { at_ms } => {
                CLOCK.store(at_ms, Ordering::Release);
                log(format!("read at={at_ms} error {}", failure()));
                anyhow::bail!("read_failed");
            }
            Step::Authenticate => {
                let epoch = bwg_worker_usb::CURRENT_SESSION.load(Ordering::Acquire);
                log(format!("owner_authenticate epoch={epoch}"));
                bwg_worker_usb::AUTHENTICATED_SESSION.store(epoch, Ordering::Release);
            }
            Step::Revoke => {
                let epoch = bwg_worker_usb::CURRENT_SESSION.load(Ordering::Acquire);
                log(format!("owner_revoke epoch={epoch}"));
                bwg_worker_usb::revoke_epoch(epoch);
            }
            Step::Drain => drain(),
            Step::Set(flag, value) => flag.store(value, Ordering::Release),
        }
    }
}

/// Runs one script to exhaustion and returns its complete ordered effect transcript.
pub(crate) fn transcript(steps: Vec<Step>) -> String {
    let _guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    drop(owner());
    drain();
    for flag in [
        &ENTROPY_FAILS,
        &BEGIN_LINK_FAILS,
        &HELLO_ACK_FAILS,
        &WORK_COOLING,
    ] {
        flag.store(false, Ordering::Release);
    }
    CLOCK.store(0, Ordering::Release);
    ENTROPY.store(0, Ordering::Release);
    NEXT_GENERATION.store(1, Ordering::Release);
    bwg_worker_usb::CURRENT_SESSION.store(0, Ordering::Release);
    bwg_worker_usb::AUTHENTICATED_SESSION.store(0, Ordering::Release);
    bwg_worker_usb::clear_failure();
    bwg_worker_usb::RECEIVE_CREDIT.reset();
    LOG.lock().expect("log lock").clear();
    *STEPS.lock().expect("steps lock") = steps.into();

    let outcome = std::thread::spawn(bwg_worker_usb::run_link).join();
    let Err(payload) = outcome else {
        panic!("the link loop returned instead of consuming the script");
    };
    assert!(payload.is::<ScriptComplete>(), "the link loop panicked");

    drain();
    let epoch = bwg_worker_usb::CURRENT_SESSION.load(Ordering::Acquire);
    log(format!(
        "final current={epoch} authenticated={}",
        bwg_worker_usb::AUTHENTICATED_SESSION.load(Ordering::Acquire)
    ));
    log(bwg_worker_usb::RECEIVE_CREDIT.summary(epoch));
    log(failure());
    let lines = std::mem::take(&mut *LOG.lock().expect("log lock"));
    let mut text = collapse_credit_runs(lines).join("\n");
    text.push('\n');
    text
}

/// Folds each run of consecutive single-byte credit publishes into one ranged line.
fn collapse_credit_runs(lines: Vec<String>) -> Vec<String> {
    const PREFIX: &str = "credit_publish received=";
    let mut folded: Vec<String> = Vec::with_capacity(lines.len());
    let mut maybe_run: Option<(u32, u32)> = None;
    let flush = |folded: &mut Vec<String>, maybe_run: &mut Option<(u32, u32)>| {
        if let Some((first, last)) = maybe_run.take() {
            folded.push(match first == last {
                true => format!("{PREFIX}{first}"),
                false => format!("{PREFIX}{first}..={last}"),
            });
        }
    };
    for line in lines {
        let Some(value) = line
            .strip_prefix(PREFIX)
            .and_then(|text| text.parse::<u32>().ok())
        else {
            flush(&mut folded, &mut maybe_run);
            folded.push(line);
            continue;
        };
        match maybe_run {
            Some((first, last)) if last.checked_add(1) == Some(value) => {
                maybe_run = Some((first, value))
            }
            _ => {
                flush(&mut folded, &mut maybe_run);
                maybe_run = Some((value, value));
            }
        }
    }
    flush(&mut folded, &mut maybe_run);
    folded
}

#[path = "usb_link_host_test/scripts.rs"]
mod scripts;
