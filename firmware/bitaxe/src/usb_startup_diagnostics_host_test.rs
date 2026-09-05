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
mod boot_evidence {
    pub fn maybe_worker_diagnostic_line(slot: usize) -> Option<String> {
        (slot == 0).then(|| "usb_runtime_identity fixture=true redacted=true".to_owned())
    }
}
mod usb_runtime {
    use super::*;
    pub static SINK: Mutex<Option<mpsc::Sender<String>>> = Mutex::new(None);
    pub fn write(bytes: &[u8]) -> anyhow::Result<()> {
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
