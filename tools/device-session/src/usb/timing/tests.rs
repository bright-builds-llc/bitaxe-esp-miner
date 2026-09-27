use super::*;

fn bound() -> BootstrapTiming {
    let timing = BootstrapTiming::default();
    timing.bind(Instant::now(), "physical", "private-nonce");
    timing
}
#[test]
fn absent_session_never_manufactures_identity() {
    // Arrange / Act
    let timing = BootstrapTiming::default();
    timing.preparation_failed();
    let value = timing.snapshot();
    // Assert
    assert!(value["origin"].is_null());
    assert!(value["sessionNonceSha256"].is_null());
    assert_eq!(value["missingStages"].as_array().expect("array").len(), 10);
    assert_eq!(value["earliestFailure"]["category"], "preparation_failed");
}
#[test]
fn first_failure_and_first_traversal_survive_reopening() {
    // Arrange
    let timing = bound();
    // Act
    timing.record(TimingStage::OpenStart, 10);
    timing.failure(
        BootstrapFailureStage::Read,
        super::super::UsbTerminalCategory::MonitorFailed,
    );
    timing.record(TimingStage::OpenStart, 20);
    timing.failure(
        BootstrapFailureStage::Cleanup,
        super::super::UsbTerminalCategory::CleanupFailed,
    );
    // Assert
    let value = timing.snapshot();
    assert_eq!(value["readerOpenCount"], 2);
    assert_eq!(value["readerReopenCount"], 1);
    assert_eq!(value["events"][0]["elapsedUs"], 10);
    assert_eq!(value["earliestFailure"]["stage"], "read");
    assert_eq!(value["captureComplete"], false);
}
#[test]
fn clock_regression_and_overflow_fail_capture() {
    // Arrange
    let timing = bound();
    // Act
    timing.record(TimingStage::ResetStart, 10);
    timing.record(TimingStage::ResetEnd, 9);
    timing.record(TimingStage::Opened, u128::MAX);
    // Assert
    let value = timing.snapshot();
    assert_eq!(value["clockDiscontinuity"], true);
    assert_eq!(value["overflow"], true);
    assert_eq!(value["earliestFailure"]["stage"], "clock");
}
#[test]
fn reader_drop_precedes_closed_observation() {
    struct Reader(BootstrapTiming);
    impl Drop for Reader {
        fn drop(&mut self) {
            assert!(!self.0.snapshot()["events"]
                .as_array()
                .expect("events")
                .iter()
                .any(|e| e["stage"] == "reader_closed"));
        }
    }
    // Arrange
    let timing = bound();
    let reader = TimedReader::new(Reader(timing.clone()), Some(timing.clone()));
    // Act
    drop(reader);
    // Assert
    assert_eq!(timing.snapshot()["events"][0]["stage"], "reader_closed");
}
#[cfg(target_os = "macos")]
#[test]
fn real_pty_reader_preserves_fragmentation_and_descriptor_drop() {
    use std::ffi::CStr;
    use std::fs::File;
    use std::io::Write;
    use std::os::fd::{AsRawFd, FromRawFd};
    // Arrange
    let mut master = -1;
    let mut slave = -1;
    let mut name = [0_i8; 128];
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                name.as_mut_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        0
    );
    let mut master = unsafe { File::from_raw_fd(master) };
    let slave = unsafe { File::from_raw_fd(slave) };
    let path = unsafe { CStr::from_ptr(name.as_ptr()) }
        .to_str()
        .expect("PTY path");
    let timing = bound();
    timing.event(TimingStage::OpenStart);
    let mut reader = TimedReader::new(
        crate::macos::ReceiveOnlyReader::open(path).expect("production reader"),
        Some(timing.clone()),
    );
    timing.event(TimingStage::Opened);
    // Act
    master.write_all(b"first").expect("first fragment");
    let first = reader.read_available().expect("read fragment");
    timing.first_read(first.len());
    master.write_all(b"second\n").expect("second fragment");
    let second = reader.read_available().expect("second fragment read");
    drop(reader);
    // Assert
    assert_eq!(first, b"first");
    assert_eq!(second, b"second\n");
    assert_eq!(timing.snapshot()["firstReadBytes"], 5);
    assert_eq!(timing.snapshot()["events"][3]["stage"], "reader_closed");
    assert!(unsafe { libc::fcntl(slave.as_raw_fd(), libc::F_GETFD) } >= 0);
}

#[test]
fn complete_chain_joins_reset_and_nonce_without_claiming_device_delivery() {
    // Arrange
    let timing = bound();
    // Act
    timing.reset_sequence(7);
    for (index, stage) in [
        TimingStage::ResetStart,
        TimingStage::ResetEnd,
        TimingStage::HandoffStart,
        TimingStage::HandoffAdmitted,
        TimingStage::MonitorStart,
        TimingStage::MonitorAdmitted,
        TimingStage::OpenStart,
        TimingStage::Opened,
        TimingStage::FirstRead,
        TimingStage::Closed,
    ]
    .into_iter()
    .enumerate()
    {
        timing.record(stage, index as u128);
    }
    timing.capture_complete();
    timing.cleanup_complete(true);
    timing.preparation_failed();
    // Assert
    let value = timing.snapshot();
    assert_eq!(value["captureComplete"], true);
    assert_eq!(value["resetChildSequence"], 7);
    assert_eq!(
        value["sessionNonceSha256"],
        super::super::sha256(b"private-nonce")
    );
    assert!(value["earliestFailure"].is_null());
    assert!(!value.to_string().contains("private-nonce"));
}
