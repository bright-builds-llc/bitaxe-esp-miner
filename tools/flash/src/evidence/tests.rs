use super::*;
use std::process::Stdio;
use tempfile::tempdir;

fn capture_shell(script: &str, path: &Utf8Path) -> Result<CaptureProcessResult> {
    let file = open_private_output(path, true)?;
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn fixture process")?;
    let stdout = child.stdout.take().context("fixture stdout")?;
    let stderr = child.stderr.take().context("fixture stderr")?;
    let (sender, receiver) = mpsc::channel();
    spawn_reader(stdout, sender.clone(), PipeStream::Stdout);
    spawn_reader(stderr, sender, PipeStream::Stderr);
    capture_pipes(
        &mut child,
        receiver,
        file,
        5,
        EvidenceRedactionMode::DeveloperRaw,
        &CommandSpec::new("espflash", ["monitor"]),
    )
}

#[test]
fn incremental_sanitizer_carries_secret_across_chunks() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");
    let file = open_private_output(&path, true).expect("output");
    let mut sanitizer = IncrementalSanitizer::new(EvidenceRedactionMode::DeveloperRaw);
    let mut file = file;

    // Act
    sanitizer
        .push(b"status password=super-", &mut file)
        .expect("chunk one");
    sanitizer
        .push(b"secret token=api-secret\n", &mut file)
        .expect("chunk two");
    sanitizer.finish(&mut file).expect("finish");
    file.sync_all().expect("sync");

    // Assert
    let captured = fs::read_to_string(path.as_std_path()).expect("captured");
    assert_eq!(captured, "status password=[redacted] token=[redacted]\n");
}

#[test]
fn interleaved_sanitizer_keeps_partial_lines_independent_by_stream() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");
    let file = open_private_output(&path, true).expect("output");
    let mut sanitizer = InterleavedSanitizer::new(file, EvidenceRedactionMode::DeveloperRaw);

    // Act
    sanitizer
        .push(PipeStream::Stdout, b"password=super-")
        .expect("stdout prefix");
    sanitizer
        .push(PipeStream::Stderr, b"status=complete\n")
        .expect("stderr line");
    sanitizer
        .push(PipeStream::Stdout, b"secret\n")
        .expect("stdout tail");
    sanitizer.finish().expect("finish");

    // Assert
    let captured = fs::read_to_string(path.as_std_path()).expect("captured");
    assert_eq!(captured, "status=complete\npassword=[redacted]\n");
    assert!(!captured.contains("super-secret"));
}

#[test]
fn real_process_capture_sanitizes_stdout_and_stderr_without_raw_sink() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");

    // Act
    let outcome = capture_shell(
        "printf 'password=super-'; printf 'secret\\n'; printf 'token=api-secret\\n' >&2",
        &path,
    )
    .expect("capture");

    // Assert
    assert!(matches!(
        outcome.status,
        CaptureProcessStatus::ExitedSuccess
    ));
    let captured = fs::read_to_string(path.as_std_path()).expect("captured");
    assert!(captured.contains("password=[redacted]"));
    assert!(captured.contains("token=[redacted]"));
    assert!(!captured.contains("super-secret"));
    assert!(!captured.contains("api-secret"));
}

#[test]
fn capture_command_binds_execution_to_trusted_program_and_sanitizes_output() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");
    let trusted_program =
        Utf8PathBuf::from_path_buf(fs::canonicalize("/bin/sh").expect("canonical shell"))
            .expect("utf8 shell");
    let command = CommandSpec::new(
        trusted_program.as_str(),
        ["-c", "printf 'password=super-secret\\n'"],
    );

    // Act
    let outcome = capture_command(
        &command,
        &trusted_program,
        &path,
        5,
        EvidenceRedactionMode::DeveloperRaw,
        true,
    )
    .expect("capture");

    // Assert
    assert_eq!(outcome.status, CaptureProcessStatus::ExitedSuccess);
    assert_eq!(
        fs::read_to_string(path.as_std_path()).expect("captured"),
        "password=[redacted]\n"
    );
}

#[test]
fn capture_command_rejects_program_other_than_trusted_before_creating_output() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");
    let trusted_program =
        Utf8PathBuf::from_path_buf(fs::canonicalize("/bin/sh").expect("canonical shell"))
            .expect("utf8 shell");
    let untrusted_program =
        Utf8PathBuf::from_path_buf(fs::canonicalize("/bin/echo").expect("canonical echo"))
            .expect("utf8 echo");
    let command = CommandSpec::new(untrusted_program.as_str(), ["unsafe"]);

    // Act
    let result = capture_command(
        &command,
        &trusted_program,
        &path,
        5,
        EvidenceRedactionMode::DeveloperRaw,
        true,
    );

    // Assert
    let error = result.expect_err("untrusted program must be rejected");
    assert!(format!("{error:#}").contains("untrusted_program"));
    assert!(!path.exists());
}

#[test]
fn real_process_capture_keeps_interleaved_partial_lines_stream_local() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");

    // Act
    let outcome = capture_shell(
            "printf 'password=super-'; sleep 0.1; printf 'status=complete\\n' >&2; sleep 0.1; printf 'secret\\n'",
            &path,
        )
        .expect("capture");

    // Assert
    assert!(matches!(
        outcome.status,
        CaptureProcessStatus::ExitedSuccess
    ));
    let captured = fs::read_to_string(path.as_std_path()).expect("captured");
    assert!(captured.contains("status=complete\n"));
    assert!(captured.contains("password=[redacted]\n"));
    assert!(!captured.contains("super-secret"));
    assert!(!captured.contains("super-"));
}

#[test]
fn real_process_capture_rejects_invalid_utf8() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");

    // Act
    let result = capture_shell("printf '\\377\\n'", &path);

    // Assert
    let error = result.expect_err("invalid process output");
    assert!(format!("{error:#}").contains("evidence_sanitization_invalid"));
}

#[test]
fn invalid_private_capture_stops_before_admitted_projection() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = Utf8PathBuf::from_path_buf(dir.path().join("evidence")).expect("utf8 path");
    let paths = preflight_dual_paths(&evidence_dir).expect("preflight");

    // Act
    let result = capture_shell("printf '\\377\\n'", &paths.private_log);

    // Assert
    let error = result.expect_err("invalid process output");
    assert!(format!("{error:#}").contains("evidence_sanitization_invalid"));
    assert!(!paths.admitted_log.exists());
    assert!(!paths.private_record.exists());
    assert!(!paths.admitted_record.exists());
}

#[test]
fn real_process_capture_rejects_overlong_input() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");

    // Act
    let result = capture_shell(
        "awk 'BEGIN { for (i = 0; i < 65537; i++) printf \"a\" }'",
        &path,
    );

    // Assert
    let error = result.expect_err("overlong process output");
    assert!(format!("{error:#}").contains("evidence_sanitization_invalid"));
}

#[test]
fn incremental_sanitizer_rejects_invalid_utf8() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");
    let file = open_private_output(&path, true).expect("output");
    let mut sanitizer = IncrementalSanitizer::new(EvidenceRedactionMode::DeveloperRaw);
    let mut file = file;

    // Act
    let result = sanitizer.push(&[0xff, b'\n'], &mut file);

    // Assert
    let error = result.expect_err("invalid utf8");
    assert!(format!("{error:#}").contains("evidence_sanitization_invalid"));
}

#[test]
fn incremental_sanitizer_rejects_overlong_line() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().join("capture.log")).expect("utf8 path");
    let file = open_private_output(&path, true).expect("output");
    let mut sanitizer = IncrementalSanitizer::new(EvidenceRedactionMode::DeveloperRaw);
    let mut file = file;
    let overlong = vec![b'a'; MAX_PENDING_LINE_BYTES + 1];

    // Act
    let result = sanitizer.push(&overlong, &mut file);

    // Assert
    let error = result.expect_err("overlong line");
    assert!(format!("{error:#}").contains("evidence_sanitization_invalid"));
}

#[test]
fn dual_derivation_preserves_private_digest_and_secures_outputs() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = Utf8PathBuf::from_path_buf(dir.path().join("evidence")).expect("utf8 path");
    let paths = preflight_dual_paths(&evidence_dir).expect("preflight");
    let mut private = open_private_output(&paths.private_log, true).expect("private");
    private
        .write_all(b"ssid=lab password=[redacted] ipv4=192.168.1.1\n")
        .expect("private write");
    drop(private);
    let before = sha256_hex(&fs::read(paths.private_log.as_std_path()).expect("private bytes"));

    // Act
    let digests = derive_admitted_log(&paths, &before).expect("derive");

    // Assert
    assert_eq!(digests.private_sha256, before);
    assert_eq!(
        digests.private_sha256,
        sha256_hex(&fs::read(paths.private_log.as_std_path()).expect("private bytes"))
    );
    let admitted = fs::read_to_string(paths.admitted_log.as_std_path()).expect("admitted");
    assert!(!admitted.contains("lab"));
    assert!(!admitted.contains("192.168.1.1"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let root_mode = fs::metadata(evidence_dir.as_std_path())
            .expect("root metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(root_mode, 0o700);
    }
    #[cfg(unix)]
    for path in [&paths.private_log, &paths.admitted_log] {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(path.as_std_path())
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}

#[test]
fn dual_derivation_rejects_unclassified_digest_before_admitted_output() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let evidence_dir = Utf8PathBuf::from_path_buf(dir.path().join("evidence")).expect("utf8 path");
    let paths = preflight_dual_paths(&evidence_dir).expect("preflight");
    write_dual_private_text(&paths.private_log, "status=complete\n").expect("private capture");

    // Act
    let result = derive_admitted_log(&paths, &"0".repeat(64));

    // Assert
    let error = result.expect_err("unclassified digest");
    assert!(format!("{error:#}").contains("classified input"));
    assert!(!paths.admitted_log.exists());
    assert!(!paths.admitted_record.exists());
}
