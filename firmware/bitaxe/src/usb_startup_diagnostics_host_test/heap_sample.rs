//! Host stand-in for the native heap read, plus the writer's heap-sample scheduling tests.
use super::*;
use bitaxe_core::usb_diagnostics::{internal_heap_sample_marker, InternalHeapSample};

pub(crate) fn marker(uptime_ms: u64) -> String {
    internal_heap_sample_marker(&InternalHeapSample {
        uptime_ms,
        free_bytes: 9_687,
        allocated_bytes: 250_000,
        largest_block_bytes: 1_920,
        minimum_free_bytes: 607,
        allocated_blocks: 1_234,
        free_blocks: 56,
        revoked: false,
    })
}

#[test]
fn idle_link_carries_the_internal_heap_sample() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));

    // Act
    let line = writer.expect_marker("internal_heap_sample");

    // Assert
    assert!(line.starts_with("internal_heap_sample schema=v1 uptime_ms="));
    assert!(line.trim_end().ends_with(" redacted=true"));
}

#[test]
fn active_session_never_carries_the_internal_heap_sample() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let writer = WriterFixture::start(Arc::new(startup_diagnostics::StartupProgress::new()));
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
    writer.expect_marker("hello_ack");

    // Act
    let mut lines = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(6);
    while lines.iter().filter(|line: &&String| line.contains("\"kind\":\"heartbeat\"")).count() < 2 {
        lines.push(
            writer
                .lines
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("bounded session output"),
        );
    }

    // Assert
    assert!(lines.iter().all(|line| !line.contains("internal_heap_sample")));
    CURRENT_SESSION.store(0, Ordering::Release);
}
