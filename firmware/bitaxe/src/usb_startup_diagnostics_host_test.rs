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

static CURRENT_SESSION: AtomicU32 = AtomicU32::new(0);
static OUTPUT: OnceLock<SyncSender<writer::Output>> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());
struct SecretBytes(Vec<u8>);
impl Drop for SecretBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
fn revoke_epoch(epoch: u32) {
    let _ = CURRENT_SESSION.compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire);
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
mod usb_runtime {
    use super::*;
    pub static DELAY_MS: AtomicU32 = AtomicU32::new(0);
    pub static SINK: Mutex<Option<mpsc::Sender<String>>> = Mutex::new(None);
    pub fn write(bytes: &[u8]) -> anyhow::Result<()> {
        std::thread::sleep(Duration::from_millis(u64::from(
            DELAY_MS.load(Ordering::Relaxed),
        )));
        let sink = SINK
            .lock()
            .map_err(|_| anyhow::anyhow!("test sink poisoned"))?;
        sink.as_ref()
            .ok_or_else(|| anyhow::anyhow!("test sink missing"))?
            .send(std::str::from_utf8(bytes)?.to_owned())?;
        Ok(())
    }
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
        CURRENT_SESSION.store(0, Ordering::Release);
        usb_runtime::DELAY_MS.store(0, Ordering::Relaxed);
        self.maybe_output.take();
        if let Some(thread) = self.maybe_thread.take() {
            thread.join().expect("writer owner exits");
        }
        *usb_runtime::SINK.lock().expect("test sink") = None;
    }
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
