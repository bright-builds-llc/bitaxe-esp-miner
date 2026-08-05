use super::*;
use crate::{UsbOperation, UsbTerminalCategory};
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[test]
fn supervised_process_captures_output_in_private_files() {
    // Arrange
    let directory = tempdir().expect("temporary directory");
    let trace_root = directory.path().join("trace");
    fs::create_dir(&trace_root).expect("trace root");
    fs::set_permissions(&trace_root, fs::Permissions::from_mode(0o700))
        .expect("private trace root");
    let mut lease = DeviceLease::acquire("process-output-test", UsbOperation::Flash, &trace_root)
        .expect("device lease");
    let args = vec!["-c".to_owned(), "printf supervised-output".to_owned()];

    // Act
    let output = run_owned_process(
        OwnedProcessRequest {
            program: Path::new("/bin/sh"),
            args: &args,
            timeout: Duration::from_secs(2),
            trace_root: &trace_root,
            trace_label: "child-0001",
            maybe_rust_log: None,
        },
        &mut lease,
    )
    .expect("supervised process");

    // Assert
    assert_eq!(output.termination, SupervisedTermination::ExitedSuccess);
    assert_eq!(output.stdout, b"supervised-output");
    let mode = fs::metadata(trace_root.join("child-0001.stdout"))
        .expect("stdout metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
    lease.mark_complete();
}

#[test]
fn bootloader_diagnostic_real_child_receives_private_rust_log_override() {
    // Arrange
    let directory = tempdir().expect("temporary directory");
    let trace_root = directory.path().join("trace");
    fs::create_dir(&trace_root).expect("trace root");
    fs::set_permissions(&trace_root, fs::Permissions::from_mode(0o700))
        .expect("private trace root");
    let mut lease = DeviceLease::acquire(
        "process-environment-test",
        UsbOperation::Detect,
        &trace_root,
    )
    .expect("device lease");
    let args = vec!["-c".to_owned(), "printf '%s' \"$RUST_LOG\"".to_owned()];

    // Act
    let output = run_owned_process(
        OwnedProcessRequest {
            program: Path::new("/bin/sh"),
            args: &args,
            timeout: Duration::from_secs(2),
            trace_root: &trace_root,
            trace_label: "child-0001",
            maybe_rust_log: Some("espflash::connection=debug"),
        },
        &mut lease,
    )
    .expect("supervised process");

    // Assert
    assert_eq!(output.termination, SupervisedTermination::ExitedSuccess);
    assert_eq!(output.stdout, b"espflash::connection=debug");
    let mode = fs::metadata(trace_root.join("child-0001.stdout"))
        .expect("stdout metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
    lease.mark_complete();
}

#[test]
fn timeout_terminates_descendant_process_group() {
    // Arrange
    let directory = tempdir().expect("temporary directory");
    let trace_root = directory.path().join("trace");
    fs::create_dir(&trace_root).expect("trace root");
    fs::set_permissions(&trace_root, fs::Permissions::from_mode(0o700))
        .expect("private trace root");
    let descendant_path = directory.path().join("descendant.pid");
    let script = format!(
        "sleep 30 & child=$!; printf '%s' \"$child\" > '{}'; wait",
        descendant_path.display()
    );
    let args = vec!["-c".to_owned(), script];
    let mut lease = DeviceLease::acquire("process-timeout-test", UsbOperation::Flash, &trace_root)
        .expect("device lease");

    // Act
    let output = run_owned_process(
        OwnedProcessRequest {
            program: Path::new("/bin/sh"),
            args: &args,
            timeout: Duration::from_millis(150),
            trace_root: &trace_root,
            trace_label: "child-0001",
            maybe_rust_log: None,
        },
        &mut lease,
    )
    .expect("bounded timeout");

    // Assert
    assert_eq!(output.termination, SupervisedTermination::TimedOut);
    let descendant_pid = fs::read_to_string(descendant_path)
        .expect("descendant pid")
        .parse::<i32>()
        .expect("numeric descendant pid");
    let deadline = Instant::now() + Duration::from_secs(2);
    while unsafe { libc::kill(descendant_pid, 0) } == 0 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(25));
    }
    assert_ne!(unsafe { libc::kill(descendant_pid, 0) }, 0);
    lease.mark_complete();
}

#[test]
fn successful_parent_cannot_leave_a_descendant_running() {
    // Arrange
    let directory = tempdir().expect("temporary directory");
    let trace_root = directory.path().join("trace");
    fs::create_dir(&trace_root).expect("trace root");
    fs::set_permissions(&trace_root, fs::Permissions::from_mode(0o700))
        .expect("private trace root");
    let descendant_path = directory.path().join("descendant.pid");
    let script = format!(
        "sleep 30 & child=$!; printf '%s' \"$child\" > '{}'; exit 0",
        descendant_path.display()
    );
    let args = vec!["-c".to_owned(), script];
    let mut lease = DeviceLease::acquire(
        "successful-parent-descendant-test",
        UsbOperation::Flash,
        &trace_root,
    )
    .expect("device lease");

    // Act
    let output = run_owned_process(
        OwnedProcessRequest {
            program: Path::new("/bin/sh"),
            args: &args,
            timeout: Duration::from_secs(2),
            trace_root: &trace_root,
            trace_label: "child-0001",
            maybe_rust_log: None,
        },
        &mut lease,
    )
    .expect("supervised successful parent");

    // Assert
    assert_eq!(output.termination, SupervisedTermination::ExitedSuccess);
    let descendant_pid = fs::read_to_string(descendant_path)
        .expect("descendant pid")
        .parse::<i32>()
        .expect("numeric descendant pid");
    let deadline = Instant::now() + Duration::from_secs(2);
    while unsafe { libc::kill(descendant_pid, 0) } == 0 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(25));
    }
    assert_ne!(unsafe { libc::kill(descendant_pid, 0) }, 0);
    lease.mark_complete();
}

#[test]
fn identity_mismatch_refuses_termination() {
    // Arrange
    let identity = OwnedChildIdentity {
        pid: std::process::id(),
        process_group: unsafe { libc::getpgid(0) },
        process_start: "not-the-current-start".to_owned(),
        executable_path: PathBuf::from("/bin/sh"),
        executable_sha256: "not-the-current-digest".to_owned(),
    };

    // Act
    let error = terminate_owned_group(&identity, Duration::from_millis(10))
        .expect_err("unproven process must be refused");

    // Assert
    assert_eq!(error.category, UsbTerminalCategory::ForeignHolder);
}

#[test]
fn sigint_is_recorded_for_bounded_cleanup() {
    // Arrange
    let _supervisor = SignalSupervisor::acquire().expect("signal supervisor");

    // Act
    let result = unsafe { libc::raise(libc::SIGINT) };

    // Assert
    assert_eq!(result, 0);
    assert_eq!(maybe_pending_signal(), Some(libc::SIGINT));
}

#[test]
fn sigterm_is_recorded_for_bounded_cleanup() {
    // Arrange
    let _supervisor = SignalSupervisor::acquire().expect("signal supervisor");

    // Act
    let result = unsafe { libc::raise(libc::SIGTERM) };

    // Assert
    assert_eq!(result, 0);
    assert_eq!(maybe_pending_signal(), Some(libc::SIGTERM));
}
