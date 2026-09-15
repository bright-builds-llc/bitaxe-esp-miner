//! Runs the real Worker replay dispatcher while the real general-log mutex is unavailable.
#![allow(dead_code)]
#[path = "boot_diagnostic_cache.rs"]
mod boot_diagnostic_cache;
#[path = "log_buffer.rs"]
mod log_buffer;
#[path = "boot_evidence/worker_diagnostics.rs"]
mod worker_diagnostics;

use boot_diagnostic_cache::{BootDiagnosticCache, Checkpoint, RecordOutcome, StartFailure, CACHE};
use std::sync::{mpsc, OnceLock};
use std::time::Duration;
struct ResetReceipts {
    maybe_allocation_context: Option<bitaxe_api::panic_receipt::AllocationFailureContextMarker>,
}
static RESET_RECEIPTS: OnceLock<ResetReceipts> = OnceLock::new();
fn worker_usb_boot_marker() -> String {
    "fixture boot".to_owned()
}
fn firmware_commit() -> &'static str {
    "0000000000000000000000000000000000000000"
}
fn app_elf_sha256() -> String {
    "0".repeat(64)
}
fn worker_rust_panic_marker() -> Option<String> {
    None
}
fn worker_allocation_failure_marker() -> Option<String> {
    None
}
mod storage_http_diagnostics {
    pub fn maybe_failure_marker() -> Option<String> {
        None
    }
    pub fn maybe_status_marker() -> Option<String> {
        None
    }
}
mod preparation_evidence {
    pub fn marker(_: bool) -> String {
        "fixture preparation".to_owned()
    }
}

#[test]
fn actual_replay_returns_cached_or_unavailable_values_before_log_lock_release() {
    // Arrange
    const EXPECTED: &str = "usb_memory_checkpoint stage=usb_install free_bytes=10 largest_block_bytes=8 reserve_bytes=98304 redacted=true";
    assert_eq!(
        CACHE.record_checkpoint(Checkpoint::UsbInstall, 10, 8, 98304),
        RecordOutcome::Recorded
    );
    assert!(CACHE.record_failure(StartFailure::UsbInstall));
    let mut buffer = bitaxe_api::RetainedLogBuffer::with_capacity(512 * 1024);
    buffer.append(&format!("{EXPECTED}\n"));
    buffer.append(&"unrelated=fixture\n".repeat(8192));
    log_buffer::install_retained_log_buffer_for_test(buffer);
    let (entered, started) = mpsc::channel();
    let (reply, received) = mpsc::channel();
    let (worker, before_release) = log_buffer::with_retained_lock_for_test(|| {
        let worker = std::thread::spawn(move || {
            entered.send(()).expect("announce reader");
            let values =
                [5, 6, 7, 8, 9, 10, 12, 13].map(worker_diagnostics::maybe_worker_diagnostic_line);
            reply.send(values).expect("reader result");
        });
        started
            .recv_timeout(Duration::from_secs(1))
            .expect("reader starts");
        // This bound only keeps the regression finite. The assertion is causal:
        // completion occurs while the log lock is still held, not a latency benchmark.
        let before_release = received.recv_timeout(Duration::from_secs(1));
        (worker, before_release)
    });
    worker
        .join()
        .expect("reader exits after bounded lock release");
    // Assert
    assert_eq!(
        before_release.expect("replay must not acquire or fall back to LOG_BUFFER"),
        [
            None,
            Some(EXPECTED.to_owned()),
            None,
            None,
            None,
            Some(
                "bwg_worker_start_failure category=startup_failed detail=usb_install redacted=true"
                    .to_owned()
            ),
            None,
            None
        ]
    );

    // Act: evict general-log history, then replace it with unavailable storage.
    log_buffer::append_runtime_log_line(&"unrelated=rollover\n".repeat(32768));
    assert!(!log_buffer::retained_text_for_test().contains(EXPECTED));
    log_buffer::install_retained_log_buffer_for_test(bitaxe_api::RetainedLogBuffer::empty());
    // Assert: these are retained boot observations, not a general-log append claim.
    assert_eq!(
        worker_diagnostics::maybe_worker_diagnostic_line(6).as_deref(),
        Some(EXPECTED)
    );
    assert_eq!(worker_diagnostics::maybe_worker_diagnostic_line(7), None);
}

#[test]
fn generated_cache_markers_pass_the_real_usb_retained_line_allowlist() {
    // Arrange
    let cache = BootDiagnosticCache::new();
    // Act / Assert
    for checkpoint in [
        Checkpoint::UsbInstall,
        Checkpoint::UsbInstalled,
        Checkpoint::WifiDriverPrepare,
        Checkpoint::WifiDriverPrepared,
        Checkpoint::WorkerOwnerPrepare,
        Checkpoint::StatisticsStart,
        Checkpoint::StatisticsStarted,
    ] {
        assert_eq!(
            cache.record_checkpoint(
                checkpoint,
                u32::MAX as usize,
                u32::MAX as usize,
                u32::MAX as usize
            ),
            RecordOutcome::Recorded
        );
        let marker = cache
            .maybe_checkpoint_marker(checkpoint)
            .expect("complete checkpoint");
        assert!(bitaxe_core::usb_diagnostics::is_worker_diagnostic_retained_line(&marker));
        assert!(
            !bitaxe_core::usb_diagnostics::is_worker_diagnostic_retained_line(&format!(
                "{marker} private=fixture"
            ))
        );
    }
    for failure in [
        StartFailure::OwnerSpawn,
        StartFailure::UsbInstall,
        StartFailure::ControlOwner,
    ] {
        let cache = BootDiagnosticCache::new();
        assert!(cache.record_failure(failure));
        assert!(
            bitaxe_core::usb_diagnostics::is_worker_diagnostic_retained_line(
                &cache.maybe_failure_marker().expect("complete failure")
            )
        );
    }
}
