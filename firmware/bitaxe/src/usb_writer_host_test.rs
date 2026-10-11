//! Runs the production serial writer loop against a scripted clock, queue and
//! transport, and pins its ordered output and effects.
use bitaxe_worker_control::serial::trace::{SerialTraceCorrelation, SerialTraceStage};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use zeroize::Zeroize;

#[allow(dead_code)]
#[path = "bwg_worker_usb/rx_diagnostics.rs"]
mod rx_diagnostics;
#[allow(dead_code)]
#[path = "bwg_worker_usb/startup_diagnostics.rs"]
mod startup_diagnostics;
#[allow(dead_code)]
#[path = "usb_tx_measurement.rs"]
mod usb_tx_measurement;
#[allow(dead_code)]
#[path = "usb_write_failure.rs"]
mod usb_write_failure;
#[allow(dead_code)]
#[path = "bwg_worker_usb/writer.rs"]
mod writer;

#[path = "usb_writer_host_test/fakes.rs"]
mod fakes;
use fakes::queue::{self, Receiver, SyncSender};
use fakes::{
    boot_evidence, heap_sample, production_mining_session, runtime_uptime, statistics_runtime,
    trace, usb_runtime, wifi_adapter,
};

static OUTPUT: OnceLock<SyncSender<writer::Output>> = OnceLock::new();
static CURRENT_SESSION: AtomicU32 = AtomicU32::new(0);
static RECEIVE_CREDIT: CreditLog = CreditLog::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());

struct SecretBytes(Vec<u8>);
impl Drop for SecretBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

fn revoke_epoch(epoch: u32) {
    let revoked = CURRENT_SESSION
        .compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire)
        .is_ok();
    fakes::log(format!("revoke_epoch epoch={epoch} revoked={revoked}"));
    if revoked {
        RECEIVE_CREDIT.mailbox().close(epoch);
    }
}

/// The writer's view of the receive-credit mailbox; mutations are logged.
struct CreditLog(Mutex<Option<Arc<bitaxe_worker_control::serial::ReceiveCreditMailbox>>>);
impl CreditLog {
    const fn new() -> Self {
        Self(Mutex::new(None))
    }
    fn mailbox(&self) -> Arc<bitaxe_worker_control::serial::ReceiveCreditMailbox> {
        self.0
            .lock()
            .expect("credit lock")
            .get_or_insert_with(Default::default)
            .clone()
    }
    fn reset(&self) {
        *self.0.lock().expect("credit lock") = None;
    }
    fn maybe_received_bytes(&self, epoch: u32) -> Option<u32> {
        self.mailbox().maybe_received_bytes(epoch)
    }
    fn maybe_terminal_bytes(&self, epoch: u32, now_ms: u64) -> Option<u32> {
        self.mailbox().maybe_terminal_bytes(epoch, now_ms)
    }
    fn finish_terminal(&self, epoch: u32) {
        fakes::log(format!("credit_finish_terminal epoch={epoch}"));
        self.mailbox().finish_terminal(epoch);
    }
}

/// Runs one script through `writer::run` and returns its complete transcript.
fn transcript(steps: Vec<fakes::Step>) -> String {
    let _guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    fakes::CLOCK.store(0, Ordering::Release);
    CURRENT_SESSION.store(0, Ordering::Release);
    RECEIVE_CREDIT.reset();
    fakes::LOG.lock().expect("log lock").clear();
    *fakes::STEPS.lock().expect("steps lock") = steps.into();
    let (outputs, output) = queue::sync_channel(4);
    let (diagnostic_sender, diagnostics) = queue::sync_channel(8);
    fakes::install(outputs, diagnostic_sender);

    writer::run(
        output,
        diagnostics,
        &startup_diagnostics::StartupProgress::new(),
    );

    let receipts: Vec<String> = fakes::RECEIPTS
        .lock()
        .expect("receipts lock")
        .iter()
        .map(|receipt| format!("{:?}", receipt.try_recv()))
        .collect();
    fakes::log(format!("receipts {}", receipts.join(" ")));
    fakes::log(format!(
        "final current={} partial={}",
        CURRENT_SESSION.load(Ordering::Acquire),
        usb_runtime::has_partial_output()
    ));
    let mut text = fakes::LOG.lock().expect("log lock").join("\n");
    text.push('\n');
    text
}

#[path = "usb_writer_host_test/scripts.rs"]
mod scripts;
