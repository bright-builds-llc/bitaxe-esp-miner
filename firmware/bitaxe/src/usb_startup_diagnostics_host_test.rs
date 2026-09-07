//! Runs the production single-writer loop against a host sink while startup fails or stalls.
use bitaxe_worker_control::serial::SerialKind;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use zeroize::Zeroize;

#[allow(dead_code)]
#[path = "bwg_worker_usb/startup_diagnostics.rs"]
mod startup_diagnostics;
#[allow(dead_code)]
#[path = "bwg_worker_usb/writer.rs"]
mod writer;

#[allow(dead_code)]
#[path = "bwg_worker_usb/rx_diagnostics.rs"]
mod rx_diagnostics;

static CURRENT_SESSION: AtomicU32 = AtomicU32::new(0);
static RECEIVE_CREDIT: bitaxe_worker_control::serial::ReceiveCreditMailbox =
    bitaxe_worker_control::serial::ReceiveCreditMailbox::new();
static OUTPUT: OnceLock<SyncSender<writer::Output>> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());
struct SecretBytes(Vec<u8>);
impl Drop for SecretBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
fn revoke_epoch(epoch: u32) {
    if CURRENT_SESSION
        .compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        RECEIVE_CREDIT.close(epoch);
    }
}

mod runtime_uptime {
    use super::*;
    pub fn millis() -> u64 {
        static START: OnceLock<Instant> = OnceLock::new();
        START.get_or_init(Instant::now).elapsed().as_millis() as u64
    }
}
mod wifi_adapter {
    pub fn maybe_startup_failure_marker() -> Option<String> {
        None
    }
}
mod boot_evidence {
    pub fn maybe_worker_diagnostic_line(slot: usize) -> Option<String> {
        (slot == 0).then(|| "usb_runtime_identity fixture=true redacted=true".to_owned())
    }
}
#[allow(dead_code)]
#[path = "usb_write_failure.rs"]
mod usb_write_failure;
mod usb_runtime {
    use super::*;
    pub(crate) use crate::usb_write_failure::WriteFailure;
    pub static DELAY_MS: AtomicU32 = AtomicU32::new(0);
    pub static PARTIAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    pub static SINK: Mutex<Option<mpsc::Sender<String>>> = Mutex::new(None);
    pub static BLOCK_NEXT: Mutex<Option<(mpsc::Sender<()>, Receiver<()>)>> = Mutex::new(None);
    pub fn write_if(bytes: &[u8], admitted: impl Fn() -> bool) -> anyhow::Result<()> {
        let maybe_block = BLOCK_NEXT.lock().expect("test block").take();
        if let Some((entered, release)) = maybe_block {
            entered.send(())?;
            release.recv_timeout(Duration::from_secs(2))?;
        }
        anyhow::ensure!(admitted(), "serial_output_revoked");
        std::thread::sleep(Duration::from_millis(u64::from(
            DELAY_MS.load(Ordering::Relaxed),
        )));
        anyhow::ensure!(admitted(), "serial_output_revoked");
        let sink = SINK
            .lock()
            .map_err(|_| anyhow::anyhow!("test sink poisoned"))?;
        sink.as_ref()
            .ok_or_else(|| anyhow::anyhow!("test sink missing"))?
            .send(std::str::from_utf8(bytes)?.to_owned())?;
        Ok(())
    }
    pub fn resynchronize_if(admitted: impl Fn() -> bool) -> anyhow::Result<()> {
        anyhow::ensure!(admitted(), "serial_output_revoked");
        Ok(())
    }
    pub fn has_partial_output() -> bool {
        PARTIAL.load(Ordering::Acquire)
    }
}

#[test]
fn cancelled_ordinary_credit_preserves_the_new_terminal_close_receipt() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
    CURRENT_SESSION.store(32, Ordering::Release);
    RECEIVE_CREDIT.begin(32);
    writer
        .maybe_output
        .as_ref()
        .expect("sender")
        .send(writer::Output::Hello {
            epoch: 32,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".into(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello");
    writer.expect_marker("hello_ack");
    let (entered, waiting) = mpsc::channel();
    let (release, blocked) = mpsc::channel();
    *usb_runtime::BLOCK_NEXT.lock().expect("test block") = Some((entered, blocked));
    RECEIVE_CREDIT.publish(255);
    waiting
        .recv_timeout(Duration::from_secs(2))
        .expect("blocked ordinary credit");
    // Act: clean Close lands while an earlier credit is waiting for native admission.
    RECEIVE_CREDIT.close_record(32, 256, runtime_uptime::millis(), || revoke_epoch(32));
    assert_eq!(CURRENT_SESSION.load(Ordering::Acquire), 0);
    release.send(()).expect("release cancelled ordinary credit");
    let line = writer
        .lines
        .recv_timeout(Duration::from_secs(2))
        .expect("terminal receipt");
    // Assert: idempotent cancellation must preserve the strictly newer Close credit.
    let envelope = bitaxe_worker_control::serial::SerialEnvelope::parse(line.as_bytes())
        .expect("protected receipt");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(envelope.payload.get()).expect("credit"),
        serde_json::json!({"op":"receive_credit","receivedBytes":256})
    );
}

#[test]
fn revoked_pre_control_heartbeat_cannot_resume_a_queued_control_response() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
    CURRENT_SESSION.store(20, Ordering::Release);
    RECEIVE_CREDIT.begin(20);
    let sender = writer.maybe_output.as_ref().expect("sender");
    sender
        .send(writer::Output::Hello {
            epoch: 20,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".into(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello");
    writer.expect_marker("hello_ack");
    let (entered, waiting) = mpsc::channel();
    let (release, blocked) = mpsc::channel();
    *usb_runtime::BLOCK_NEXT.lock().expect("test block") = Some((entered, blocked));
    let (receipt, completion) = mpsc::sync_channel(1);
    sender
        .send(writer::Output::Control {
            epoch: 20,
            bytes: SecretBytes(
                serde_json::to_vec(&serde_json::json!({"padding":"x".repeat(8192)}))
                    .expect("payload"),
            ),
            receipt,
        })
        .expect("control");
    // Act
    waiting
        .recv_timeout(Duration::from_secs(2))
        .expect("blocked native output");
    revoke_epoch(20);
    release.send(()).expect("release native sink");
    let sent = completion
        .recv_timeout(Duration::from_secs(2))
        .expect("control completion");
    // Assert
    assert!(
        !sent,
        "revocation must cancel the already-selected control output"
    );
}

#[test]
fn validated_close_receives_final_credit_after_immediate_revocation() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
    CURRENT_SESSION.store(21, Ordering::Release);
    RECEIVE_CREDIT.begin(21);
    writer
        .maybe_output
        .as_ref()
        .expect("sender")
        .send(writer::Output::Hello {
            epoch: 21,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".into(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello");
    writer.expect_marker("hello_ack");
    // Act: the same close primitive is called by the production RX link.
    RECEIVE_CREDIT.publish(255);
    RECEIVE_CREDIT.close_record(21, 256, runtime_uptime::millis(), || revoke_epoch(21));
    assert_eq!(CURRENT_SESSION.load(Ordering::Acquire), 0);
    let credit = writer.expect_marker("\"receivedBytes\":256");
    // Assert
    let credit = bitaxe_worker_control::serial::SerialEnvelope::parse(credit.as_bytes())
        .expect("protected terminal receipt");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(credit.payload.get()).expect("credit"),
        serde_json::json!({"op":"receive_credit","receivedBytes":256})
    );
}

#[test]
fn blocked_credit_is_discarded_when_a_new_hello_replaces_its_epoch() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
    CURRENT_SESSION.store(30, Ordering::Release);
    RECEIVE_CREDIT.begin(30);
    let sender = writer.maybe_output.as_ref().expect("sender");
    sender
        .send(writer::Output::Hello {
            epoch: 30,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".into(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello");
    writer.expect_marker("hello_ack");
    let (entered, waiting) = mpsc::channel();
    let (release, blocked) = mpsc::channel();
    *usb_runtime::BLOCK_NEXT.lock().expect("test block") = Some((entered, blocked));
    RECEIVE_CREDIT.publish(1024);
    waiting
        .recv_timeout(Duration::from_secs(2))
        .expect("blocked credit output");
    // Act
    revoke_epoch(30);
    RECEIVE_CREDIT.begin(31);
    CURRENT_SESSION.store(31, Ordering::Release);
    sender
        .send(writer::Output::Hello {
            epoch: 31,
            session_id: "AQEBAQEBAQEBAQEBAQEBAQ".into(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("new hello");
    release.send(()).expect("release native sink");
    // Assert: consume the next actual output rather than skipping stale records.
    let line = writer
        .lines
        .recv_timeout(Duration::from_secs(2))
        .expect("new hello output");
    let envelope = bitaxe_worker_control::serial::SerialEnvelope::parse(line.as_bytes())
        .expect("protected new output");
    assert_eq!(
        envelope.session_id.as_deref(),
        Some("AQEBAQEBAQEBAQEBAQEBAQ")
    );
    assert_eq!(envelope.sequence, 0);
}
struct WriterFixture {
    maybe_output: Option<SyncSender<writer::Output>>,
    maybe_thread: Option<std::thread::JoinHandle<()>>,
    lines: Receiver<String>,
}
impl WriterFixture {
    fn start(progress: Arc<startup_diagnostics::StartupProgress>) -> Self {
        let (sink, lines) = mpsc::channel();
        *usb_runtime::SINK.lock().expect("test sink") = Some(sink);
        let (output, receiver) = mpsc::sync_channel(4);
        let (_diagnostic_sender, diagnostics) = mpsc::sync_channel(8);
        let thread = std::thread::spawn(move || writer::run(receiver, diagnostics, &progress));
        Self {
            maybe_output: Some(output),
            maybe_thread: Some(thread),
            lines,
        }
    }
    fn expect_marker(&self, pattern: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let line = self
                .lines
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("bounded startup diagnostic");
            if line.contains(pattern) {
                return line;
            }
        }
    }
}
impl Drop for WriterFixture {
    fn drop(&mut self) {
        RECEIVE_CREDIT.close(CURRENT_SESSION.load(Ordering::Acquire));
        CURRENT_SESSION.store(0, Ordering::Release);
        usb_runtime::DELAY_MS.store(0, Ordering::Relaxed);
        usb_runtime::PARTIAL.store(false, Ordering::Release);
        self.maybe_output.take();
        if let Some(thread) = self.maybe_thread.take() {
            thread.join().expect("writer owner exits");
        }
        *usb_runtime::SINK.lock().expect("test sink") = None;
    }
}

#[test]
fn partial_native_output_cannot_be_spliced_with_a_terminal_close_credit() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
    CURRENT_SESSION.store(33, Ordering::Release);
    RECEIVE_CREDIT.begin(33);
    writer
        .maybe_output
        .as_ref()
        .expect("sender")
        .send(writer::Output::Hello {
            epoch: 33,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".into(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello");
    writer.expect_marker("hello_ack");
    // Act: actual native-loop tests establish when this partial-prefix flag is set.
    usb_runtime::PARTIAL.store(true, Ordering::Release);
    RECEIVE_CREDIT.publish(255);
    RECEIVE_CREDIT.close_record(33, 256, runtime_uptime::millis(), || revoke_epoch(33));
    // Assert: observe the writer return to unframed diagnostics without credit output.
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let line = writer
            .lines
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("bounded writer progress");
        assert!(
            !line.contains("receive_credit"),
            "terminal credit cannot follow partial JSON"
        );
        if line.starts_with("usb_startup ") {
            break;
        }
    }
    assert_eq!(
        RECEIVE_CREDIT.maybe_terminal_bytes(33, runtime_uptime::millis()),
        None
    );
    assert_eq!(CURRENT_SESSION.load(Ordering::Acquire), 0);
}

#[test]
fn partial_receive_credit_is_written_without_a_controller_reply() {
    // Arrange: the command owner has supplied no output at all after admission.
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
    CURRENT_SESSION.store(8, Ordering::Release);
    RECEIVE_CREDIT.begin(8);
    writer
        .maybe_output
        .as_ref()
        .expect("writer sender")
        .send(writer::Output::Hello {
            epoch: 8,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".to_owned(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello queued");
    writer.expect_marker("hello_ack");

    // Act: partial records must grant capacity before any complete command exists.
    RECEIVE_CREDIT.publish(1024);
    let first = writer.expect_marker("receive_credit");
    RECEIVE_CREDIT.publish(1536);
    let second = writer.expect_marker("receive_credit");

    // Assert: run the real producer and validate its exact protected wire records.
    let first = bitaxe_worker_control::serial::SerialEnvelope::parse(first.as_bytes())
        .expect("first protected credit");
    let second = bitaxe_worker_control::serial::SerialEnvelope::parse(second.as_bytes())
        .expect("second protected credit");
    assert_eq!(first.kind, SerialKind::Session);
    assert!(second.sequence > first.sequence);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(first.payload.get()).expect("credit JSON"),
        serde_json::json!({"op":"receive_credit", "receivedBytes":1024})
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(second.payload.get()).expect("credit JSON"),
        serde_json::json!({"op":"receive_credit", "receivedBytes":1536})
    );
}

#[test]
fn early_nvs_failure_is_replayed_after_startup_returns() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let progress = Arc::new(startup_diagnostics::StartupProgress::new());
    let writer = WriterFixture::start(Arc::clone(&progress));
    progress.enter(startup_diagnostics::Stage::Nvs);
    // Act
    let result: Result<(), ()> = progress.guard(|| Err(()));
    let marker = writer.expect_marker("stage=nvs state=failed");
    // Assert
    assert!(result.is_err());
    assert!(marker.contains("first_failure=nvs"));
    writer.expect_marker("usb_runtime_identity");
    assert!(writer
        .expect_marker("stage=nvs state=failed")
        .contains("redacted=true"));
}

#[test]
fn diagnostic_writer_continues_while_wifi_startup_is_blocked() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let progress = Arc::new(startup_diagnostics::StartupProgress::new());
    let writer = WriterFixture::start(Arc::clone(&progress));
    let (release, blocked) = mpsc::channel();
    progress.enter(startup_diagnostics::Stage::Network);
    // Act
    std::thread::scope(|scope| {
        let startup = scope.spawn(move || blocked.recv_timeout(Duration::from_secs(3)));
        let first = writer.expect_marker("stage=network state=entered");
        let second = writer.expect_marker("stage=network state=entered");
        // Assert
        assert_ne!(
            first, second,
            "boot time must advance while startup remains blocked"
        );
        assert!(second.contains("first_failure=none"));
        release.send(()).expect("release simulated Wi-Fi");
        startup
            .join()
            .expect("startup thread joins")
            .expect("bounded release");
    });
}

#[test]
fn later_startup_progress_preserves_the_first_failure_category() {
    // Arrange
    let progress = startup_diagnostics::StartupProgress::new();
    progress.enter(startup_diagnostics::Stage::Nvs);
    progress.fail(startup_diagnostics::Stage::Nvs);
    // Act
    progress.enter(startup_diagnostics::Stage::Network);
    progress.fail(startup_diagnostics::Stage::Network);
    progress.enter(startup_diagnostics::Stage::RuntimeReady);
    progress.complete();
    // Assert
    assert_eq!(progress.marker(42), "usb_startup schema=v1 stage=runtime_ready state=complete first_failure=nvs uptime_ms=42 redacted=true");
}

#[test]
fn active_serial_session_emits_advancing_peer_heartbeats_without_control_requests() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let progress = Arc::new(startup_diagnostics::StartupProgress::new());
    let writer = WriterFixture::start(progress);
    CURRENT_SESSION.store(1, Ordering::Release);
    writer
        .maybe_output
        .as_ref()
        .expect("writer sender")
        .send(writer::Output::Hello {
            epoch: 1,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".to_owned(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello queued");
    // Act
    let first = writer.expect_marker("\"kind\":\"heartbeat\"");
    let second = writer.expect_marker("\"kind\":\"heartbeat\"");
    // Assert
    let first: serde_json::Value = serde_json::from_str(&first).expect("heartbeat envelope");
    let second: serde_json::Value = serde_json::from_str(&second).expect("heartbeat envelope");
    assert_eq!(first["sessionId"], "AAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(first["payload"], serde_json::json!({}));
    assert!(
        second["sequence"].as_u64().expect("second sequence")
            > first["sequence"].as_u64().expect("first sequence")
    );
    CURRENT_SESSION.store(0, Ordering::Release);
}

#[test]
fn peer_heartbeat_has_priority_during_continuous_control_output() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
    usb_runtime::DELAY_MS.store(2, Ordering::Relaxed);
    CURRENT_SESSION.store(2, Ordering::Release);
    let sender = writer.maybe_output.as_ref().expect("writer sender").clone();
    sender
        .send(writer::Output::Hello {
            epoch: 2,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".to_owned(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello queued");
    // Act
    let producer = std::thread::spawn(move || {
        for _ in 0..600 {
            let (receipt, completion) = mpsc::sync_channel(1);
            sender
                .send(writer::Output::Control {
                    epoch: 2,
                    bytes: SecretBytes(b"{}".to_vec()),
                    receipt,
                })
                .expect("control queued");
            assert!(completion
                .recv_timeout(Duration::from_secs(1))
                .expect("confirmed control"));
        }
    });
    let heartbeat = writer.expect_marker("\"kind\":\"heartbeat\"");
    // Assert
    assert!(heartbeat.contains("\"payload\":{}"));
    producer.join().expect("bounded control producer");
}

#[test]
fn long_control_reply_refreshes_peer_heartbeat_before_the_indivisible_record() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
    CURRENT_SESSION.store(4, Ordering::Release);
    let sender = writer.maybe_output.as_ref().expect("writer sender");
    sender
        .send(writer::Output::Hello {
            epoch: 4,
            session_id: "AAAAAAAAAAAAAAAAAAAAAA".to_owned(),
            payload: serde_json::json!({"op":"hello_ack"}),
        })
        .expect("hello queued");
    writer.expect_marker("\"kind\":\"session\"");
    let (receipt, completion) = mpsc::sync_channel(1);
    let bytes = serde_json::to_vec(&serde_json::json!({"padding":"x".repeat(8192)}))
        .expect("fixed large payload");
    // Act
    sender
        .send(writer::Output::Control {
            epoch: 4,
            bytes: SecretBytes(bytes),
            receipt,
        })
        .expect("control queued");
    let heartbeat: serde_json::Value =
        serde_json::from_str(&writer.expect_marker("\"kind\":\"heartbeat\"")).expect("heartbeat");
    let control: serde_json::Value =
        serde_json::from_str(&writer.expect_marker("\"kind\":\"control\"")).expect("control");
    // Assert
    assert!(completion
        .recv_timeout(Duration::from_secs(1))
        .expect("confirmed response"));
    assert_eq!(
        control["sequence"].as_u64().expect("control sequence"),
        heartbeat["sequence"].as_u64().expect("heartbeat sequence") + 1
    );
}
