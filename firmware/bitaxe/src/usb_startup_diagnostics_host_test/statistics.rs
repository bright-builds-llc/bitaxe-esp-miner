use super::*;

#[test]
fn late_usb_writer_replays_closed_statistics_spawn_failure() {
    // Arrange
    let _exclusive = TEST_LOCK.lock().expect("exclusive writer fixture");
    let record = &statistics_startup_diagnostics::STARTUP;
    assert!(record.begin(8192));
    record.observe_before(
        2052,
        statistics_startup_diagnostics::HeapObservation {
            free_bytes: 14000,
            largest_block_bytes: 7900,
        },
    );
    record.finish(
        statistics_startup_diagnostics::HeapObservation {
            free_bytes: 13912,
            largest_block_bytes: 7900,
        },
        Err(Some(12)),
    );
    let progress = Arc::new(startup_diagnostics::StartupProgress::new());
    progress.enter(startup_diagnostics::Stage::RuntimeReady);
    progress.complete();
    // Act
    let writer = WriterFixture::start(progress);
    // Assert
    writer.expect_marker("usb_reboot_discriminator schema=v1");
    let line = writer
        .lines
        .recv_timeout(Duration::from_secs(2))
        .expect("statistics follows boot");
    assert!(line.contains("state=spawn_failed errno=12 stack_bytes=8192 stack_caps=2052"));
    assert!(line.contains("redacted=true"));
    let startup = writer
        .lines
        .recv_timeout(Duration::from_secs(2))
        .expect("startup follows statistics");
    assert!(startup.contains("usb_startup schema=v1 stage=runtime_ready"));
}
