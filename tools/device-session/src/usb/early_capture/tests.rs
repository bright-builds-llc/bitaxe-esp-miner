use super::*;
#[cfg(target_os = "macos")]
fn pty() -> (File, File, Candidate) {
    use std::ffi::CStr;
    use std::os::fd::FromRawFd;
    let mut master = -1;
    let mut slave = -1;
    let mut name = [0i8; 128];
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
    let master = unsafe { File::from_raw_fd(master) };
    let slave = unsafe { File::from_raw_fd(slave) };
    let port = unsafe { CStr::from_ptr(name.as_ptr()) }
        .to_str()
        .expect("PTY path")
        .to_owned();
    let candidate = Candidate {
        port,
        physical: "physical".into(),
        enumeration: "enumeration".into(),
        node: NodeIdentity::from_metadata(slave.metadata().expect("metadata"))
            .expect("character device"),
    };
    (master, slave, candidate)
}
#[cfg(target_os = "macos")]
#[test]
fn slow_admission_does_not_delay_actual_receiving() {
    let _signals =
        super::super::process::SignalSupervisor::acquire().expect("capture signal owner");
    use std::io::Write;
    // Arrange
    let (mut master, _slave, candidate) = pty();
    let lease = Arc::new(File::open("/dev/null").expect("lease stand-in"));
    let mut capture = EarlyCapture::start(
        candidate,
        lease,
        Duration::from_millis(900),
        Vec::with_capacity(4096),
        None,
    )
    .expect("capture");
    // Act: a real writer produces a bootstrap-sized record before either admission completes.
    master.write_all(&[b'x'; 92]).expect("PTY write");
    thread::sleep(Duration::from_millis(200));
    let received = capture.state.lock().expect("state").captured;
    capture.admit();
    let result = capture.finish(false, Duration::from_secs(2));
    // Assert
    assert_eq!(
        received, 92,
        "bytes must be actively drained while admission is pending"
    );
    assert_eq!(result.expect("admitted result").bytes.len(), 92);
}

#[cfg(target_os = "macos")]
#[test]
fn admission_after_capture_expiry_never_releases_quarantine() {
    let _signals =
        super::super::process::SignalSupervisor::acquire().expect("capture signal owner");
    // Arrange
    let (_master, _slave, candidate) = pty();
    let mut capture = EarlyCapture::start(
        candidate,
        Arc::new(File::open("/dev/null").expect("lease")),
        Duration::from_millis(40),
        Vec::with_capacity(512),
        None,
    )
    .expect("capture");
    // Act
    thread::sleep(Duration::from_millis(100));
    capture.admit();
    let result = capture.finish(false, Duration::from_secs(1));
    // Assert
    assert_eq!(
        result.expect_err("late admission").detail,
        "admission_incomplete"
    );
    assert!(!capture.state.lock().expect("state").released);
}
#[cfg(target_os = "macos")]
#[test]
fn overflow_preserves_first_failure_and_never_returns_truncated_success() {
    let _signals =
        super::super::process::SignalSupervisor::acquire().expect("capture signal owner");
    use std::io::Write;
    // Arrange
    let (mut master, _slave, candidate) = pty();
    let mut capture = EarlyCapture::start(
        candidate,
        Arc::new(File::open("/dev/null").expect("lease")),
        Duration::from_secs(1),
        Vec::with_capacity(16),
        None,
    )
    .expect("capture");
    // Act
    while !capture.opened() {
        capture.health().expect("opening");
        thread::sleep(Duration::from_millis(1));
    }
    capture.admit();
    master.write_all(&[b'x'; 92]).expect("write");
    let result = capture.finish(false, Duration::from_secs(2));
    // Assert
    assert_eq!(result.expect_err("overflow").detail, "capture_overflow");
    assert_eq!(
        capture.health().expect_err("retained error").detail,
        "capture_overflow"
    );
}
#[cfg(target_os = "macos")]
#[test]
fn cancellation_drops_reader_and_releases_worker_lease_reference() {
    let _signals =
        super::super::process::SignalSupervisor::acquire().expect("capture signal owner");
    // Arrange
    let (_master, _slave, candidate) = pty();
    let lease = Arc::new(File::open("/dev/null").expect("lease"));
    let mut capture = EarlyCapture::start(
        candidate,
        lease.clone(),
        Duration::from_secs(10),
        Vec::with_capacity(512),
        None,
    )
    .expect("capture");
    // Act
    let result = capture.finish(true, Duration::from_secs(1));
    // Assert
    assert_eq!(result.expect_err("cancelled").detail, "capture_cancelled");
    assert!(capture.thread.is_none());
    assert_eq!(Arc::strong_count(&lease), 1);
}
#[cfg(target_os = "macos")]
#[test]
fn candidate_binding_rejects_profile_physical_enumeration_node_and_holder_changes() {
    let _signals =
        super::super::process::SignalSupervisor::acquire().expect("capture signal owner");
    // Arrange
    let (_master, _slave, candidate) = pty();
    let base = UsbDeviceSnapshot {
        port: candidate.port.clone(),
        physical_identity_digest: candidate.physical.clone(),
        enumeration_token: candidate.enumeration.clone(),
        accessible: true,
        holder_count: 0,
        profile: crate::UsbProfile::SerialJtagRuntime,
    };
    // Act / Assert
    candidate.matches(&base).expect("same candidate");
    for field in 0..5 {
        let mut changed = base.clone();
        match field {
            0 => changed.profile = crate::UsbProfile::Unknown,
            1 => changed.physical_identity_digest.push('x'),
            2 => changed.enumeration_token.push('x'),
            3 => changed.port = "/dev/null".into(),
            _ => changed.holder_count = 1,
        }
        assert!(candidate.matches(&changed).is_err());
    }
}
#[cfg(target_os = "macos")]
#[test]
fn failed_join_retains_actual_flock_until_worker_finishes() {
    let _signals =
        super::super::process::SignalSupervisor::acquire().expect("capture signal owner");
    use std::os::fd::AsRawFd;
    // Arrange: real kernel lock, intentionally blocked worker seam, no device.
    let lock = tempfile::NamedTempFile::new().expect("lockfile");
    assert_eq!(
        unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
    let lease = Arc::new(lock.reopen().expect("independent file"));
    // Lock the descriptor actually retained by the worker.
    assert_eq!(unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) }, 0);
    assert_eq!(
        unsafe { libc::flock(lease.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
    let (_master, _slave, candidate) = pty();
    let (send, receiver) = sync_channel(1);
    let retained = lease.clone();
    let (release, blocked) = sync_channel::<()>(1);
    let worker = thread::spawn(move || {
        let _lease = retained;
        blocked.recv().expect("release");
        send.send(Err(failure("capture_cancelled")))
            .expect("result");
    });
    let mut capture = EarlyCapture {
        candidate,
        state: Arc::new(Mutex::new(State::default())),
        admitted: Arc::new(AtomicBool::new(false)),
        cancelled: Arc::new(AtomicBool::new(false)),
        receiver,
        thread: Some(worker),
        maybe_timing: None,
        duration: Duration::from_secs(1),
    };
    drop(lease);
    // Act
    assert_eq!(
        capture
            .finish(true, Duration::from_millis(10))
            .expect_err("join timeout")
            .detail,
        "reader_join_timeout"
    );
    let contender = lock.reopen().expect("contender");
    // Assert
    assert_ne!(
        unsafe { libc::flock(contender.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
    release.send(()).expect("release worker");
    assert!(capture.finish(true, Duration::from_secs(1)).is_err());
    assert_eq!(
        unsafe { libc::flock(contender.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
}

#[cfg(target_os = "macos")]
#[test]
fn signal_handoff_preserves_pending_interrupt_and_prevents_open() {
    // Arrange: the first guard represents the reaped reset command's signal scope.
    let reset = super::super::process::SignalSupervisor::acquire().expect("signal guard");
    assert_eq!(unsafe { libc::raise(libc::SIGINT) }, 0);
    let capture_guard = reset;
    let (_master, _slave, candidate) = pty();
    // Act
    let mut capture = EarlyCapture::start(
        candidate,
        Arc::new(File::open("/dev/null").expect("lease")),
        Duration::from_secs(1),
        Vec::with_capacity(512),
        None,
    )
    .expect("capture");
    let result = capture.finish(false, Duration::from_secs(1));
    // Assert
    assert_eq!(
        result.expect_err("signal cancellation").detail,
        "capture_cancelled"
    );
    assert!(!capture.opened());
    drop(capture_guard);
    drop(super::super::process::SignalSupervisor::acquire().expect("clear test signal"));
}
#[cfg(target_os = "macos")]
#[test]
fn timing_records_one_reader_and_truthful_quarantine_byte_count() {
    let _signals =
        super::super::process::SignalSupervisor::acquire().expect("capture signal owner");
    use std::io::Write;
    // Arrange
    let timing = BootstrapTiming::default();
    timing.bind(Instant::now(), "physical", "nonce");
    timing.early_mode(Duration::from_millis(250));
    let (mut master, _slave, candidate) = pty();
    let mut capture = EarlyCapture::start(
        candidate,
        Arc::new(File::open("/dev/null").expect("lease")),
        Duration::from_millis(250),
        Vec::with_capacity(512),
        Some(timing.clone()),
    )
    .expect("capture");
    while !capture.opened() {
        capture.health().expect("opening");
        thread::sleep(Duration::from_millis(1));
    }
    // Act
    master.write_all(&[b'x'; 92]).expect("write");
    thread::sleep(Duration::from_millis(50));
    assert_eq!(timing.snapshot()["quarantineReleased"], false);
    assert!(capture.receiver.try_recv().is_err());
    capture.admit();
    let output = capture
        .finish(false, Duration::from_secs(1))
        .expect("complete");
    // Assert
    let value = timing.snapshot();
    assert_eq!(output.bytes.len(), 92);
    assert_eq!(value["readerOpenCount"], 1);
    assert_eq!(value["readerReopenCount"], 0);
    assert_eq!(value["quarantinedBytes"], 92);
    assert_eq!(value["capturedBytes"], 92);
    assert_eq!(value["readerJoined"], true);
    let stages: Vec<_> = value["events"]
        .as_array()
        .expect("events")
        .iter()
        .map(|event| event["stage"].as_str().expect("stage"))
        .collect();
    assert!(
        stages.iter().position(|s| *s == "reader_closed")
            < stages.iter().position(|s| *s == "reader_joined")
    );
}

#[test]
fn pending_before_reset_boundary_rejects_child() {
    // Arrange: an exact-test child isolates a signal arriving outside any guard.
    const CHILD: &str = "BITAXE_TEST_PENDING_RESET_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "usb::early_capture::tests::pending_before_reset_boundary_rejects_child",
                "--test-threads=1",
            ])
            .env(CHILD, "1")
            .output()
            .expect("isolated test process");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        return;
    }
    let dir = tempfile::tempdir().expect("fixture");
    let marker = dir.path().join("reset-child-started");
    let physical = super::super::sha256(dir.path().as_os_str().as_encoded_bytes());
    let lease = super::super::lease::DeviceLease::acquire(
        &physical,
        super::super::UsbOperation::FlashMonitor,
        dir.path(),
    )
    .expect("test lease");
    let mut session = super::super::UsbSession {
        maybe_timing: None,
        capture: CaptureState::default(),
        acquired_at: Instant::now(),
        operation: super::super::UsbOperation::FlashMonitor,
        state: super::super::UsbLifecycleState::Admitted,
        lease,
        physical_identity_digest: physical,
        initial_enumeration_token: "synthetic".into(),
        current_enumeration_token: "synthetic".into(),
        current_port: "/dev/null".into(),
        device_effect_state: super::super::UsbDeviceEffectState::None,
        last_command_diagnostic: None,
        earliest_failure: None,
        trace_root: dir.path().to_path_buf(),
        child_sequence: 0,
        recovery_sequence: 0,
        profile_trace_sequence: 0,
        profile_observation_counts: Default::default(),
    };
    session
        .prepare_post_reset_capture(Duration::from_secs(1))
        .expect("prepare");
    super::super::process::raise_pending_at_boundary_for_test(libc::SIGINT);
    // Act: actual production reset entry, with a harmless child that would leave a marker.
    let args = vec![
        "-c".to_owned(),
        "printf executed > \"$1\"".to_owned(),
        "sh".to_owned(),
        marker.to_string_lossy().into_owned(),
    ];
    let result = session.run_bootstrap_reset(
        std::path::Path::new("/bin/sh"),
        &args,
        Duration::from_secs(1),
    );
    // Assert
    assert_eq!(
        result.expect_err("pending reset must not launch").detail,
        "capture_cancelled"
    );
    assert!(!marker.exists());
    assert_eq!(session.child_sequence, 0);
    assert!(!session.capture.reset_reaped);
    assert!(session.capture.maybe_owner.is_none());
    session.lease.mark_complete();
}
