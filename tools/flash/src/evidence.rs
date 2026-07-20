use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

use crate::EvidenceRedactionMode;
use crate::{
    sanitize_evidence_text, sha256_bytes, CaptureProcessResult, CaptureProcessStatus, CommandSpec,
};

const MAX_PENDING_LINE_BYTES: usize = 64 * 1024;
const READ_CHUNK_BYTES: usize = 4096;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DualEvidencePaths {
    pub(crate) private_log: Utf8PathBuf,
    pub(crate) admitted_log: Utf8PathBuf,
    pub(crate) private_record: Utf8PathBuf,
    pub(crate) admitted_record: Utf8PathBuf,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct DualEvidenceDigests {
    pub(crate) private_sha256: String,
    pub(crate) admitted_sha256: String,
}

enum PipeEvent {
    Bytes(Vec<u8>),
    ReadFailed,
    Closed,
}

pub(crate) fn preflight_dual_paths(evidence_dir: &Utf8Path) -> Result<DualEvidencePaths> {
    fs::create_dir_all(evidence_dir.as_std_path())
        .with_context(|| format!("failed to create evidence directory {evidence_dir}"))?;
    let metadata = fs::symlink_metadata(evidence_dir.as_std_path())
        .with_context(|| format!("failed to inspect evidence directory {evidence_dir}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("dual evidence path preflight failed: evidence root must be an owned directory");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            evidence_dir.as_std_path(),
            fs::Permissions::from_mode(0o700),
        )
        .with_context(|| format!("failed to secure evidence directory {evidence_dir}"))?;
        let mode = fs::metadata(evidence_dir.as_std_path())
            .with_context(|| format!("failed to verify evidence directory {evidence_dir}"))?
            .permissions()
            .mode()
            & 0o777;
        if mode != 0o700 {
            bail!("dual evidence path preflight failed: evidence root is not mode 0700");
        }
    }

    let private_log = evidence_dir.join("flash-monitor.classifier-input.log");
    let admitted_log = evidence_dir.join("flash-monitor.log");
    let private_record = evidence_dir.join("flash-command-evidence.private.json");
    let admitted_record = evidence_dir.join("flash-command-evidence.json");
    if private_log == admitted_log {
        bail!("dual evidence path preflight failed: evidence paths alias");
    }
    for path in [
        &private_log,
        &admitted_log,
        &private_record,
        &admitted_record,
    ] {
        match fs::symlink_metadata(path.as_std_path()) {
            Ok(_) => {
                bail!("dual evidence path preflight failed: destination already exists: {path}")
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to inspect evidence path {path}"));
            }
        }
        if path.parent() != Some(evidence_dir) {
            bail!("dual evidence path preflight failed: destination escapes evidence root");
        }
    }

    Ok(DualEvidencePaths {
        private_log,
        admitted_log,
        private_record,
        admitted_record,
    })
}

pub(crate) fn capture_command(
    command_spec: &CommandSpec,
    log_path: &Utf8Path,
    timeout_seconds: u64,
    redaction_mode: EvidenceRedactionMode,
    create_new: bool,
) -> Result<CaptureProcessResult> {
    if command_spec.program != "espflash" {
        bail!("unsupported command program: {}", command_spec.program);
    }
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent.as_std_path())
            .with_context(|| format!("failed to create log directory {parent}"))?;
    }

    let log_file = open_private_output(log_path, create_new)?;
    let mut command = Command::new("espflash");
    command
        .args(&command_spec.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", command_spec.display()))?;
    let stdout = child
        .stdout
        .take()
        .context("failed to capture monitor stdout")?;
    let stderr = child
        .stderr
        .take()
        .context("failed to capture monitor stderr")?;

    let (sender, receiver) = mpsc::channel();
    spawn_reader(stdout, sender.clone());
    spawn_reader(stderr, sender);
    capture_pipes(
        &mut child,
        receiver,
        log_file,
        timeout_seconds,
        redaction_mode,
        command_spec,
    )
}

pub(crate) fn derive_admitted_log(paths: &DualEvidencePaths) -> Result<DualEvidenceDigests> {
    let private_bytes = fs::read(paths.private_log.as_std_path())
        .with_context(|| format!("failed to read private evidence {}", paths.private_log))?;
    let private_text = std::str::from_utf8(&private_bytes)
        .map_err(|_| anyhow::anyhow!("evidence_sanitization_invalid"))?;
    let private_sha256 = sha256_hex(&private_bytes);
    let admitted = sanitize_evidence_text(private_text, EvidenceRedactionMode::CommitRedacted);
    let mut admitted_file = open_private_output(&paths.admitted_log, true)?;
    admitted_file
        .write_all(admitted.as_bytes())
        .with_context(|| format!("failed to write admitted evidence {}", paths.admitted_log))?;
    admitted_file
        .sync_all()
        .with_context(|| format!("failed to sync admitted evidence {}", paths.admitted_log))?;
    drop(admitted_file);

    let private_after = fs::read(paths.private_log.as_std_path())
        .with_context(|| format!("failed to verify private evidence {}", paths.private_log))?;
    if sha256_hex(&private_after) != private_sha256 {
        bail!("private evidence digest changed during admitted projection");
    }

    Ok(DualEvidenceDigests {
        private_sha256,
        admitted_sha256: sha256_hex(admitted.as_bytes()),
    })
}

pub(crate) fn write_dual_private_text(path: &Utf8Path, text: &str) -> Result<()> {
    write_secure_new_text(path, text, EvidenceRedactionMode::DeveloperRaw)
}

pub(crate) fn write_dual_admitted_text(path: &Utf8Path, text: &str) -> Result<()> {
    write_secure_new_text(path, text, EvidenceRedactionMode::CommitRedacted)
}

fn write_secure_new_text(
    path: &Utf8Path,
    text: &str,
    redaction_mode: EvidenceRedactionMode,
) -> Result<()> {
    let mut file = open_private_output(path, true)?;
    let sanitized = sanitize_evidence_text(text, redaction_mode);
    file.write_all(sanitized.as_bytes())
        .with_context(|| format!("failed to write private evidence {path}"))?;
    file.sync_all()
        .with_context(|| format!("failed to sync private evidence {path}"))
}

fn open_private_output(path: &Utf8Path, create_new: bool) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true);
    if create_new {
        options.create_new(true);
    } else {
        options.create(true).truncate(true);
    }
    #[cfg(unix)]
    options.mode(0o600);
    let file = options
        .open(path.as_std_path())
        .with_context(|| format!("failed to create evidence output {path}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o600))
            .with_context(|| format!("failed to secure evidence output {path}"))?;
    }
    Ok(file)
}

fn spawn_reader(mut reader: impl Read + Send + 'static, sender: Sender<PipeEvent>) {
    std::thread::spawn(move || {
        let mut buffer = [0_u8; READ_CHUNK_BYTES];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    let _ = sender.send(PipeEvent::Closed);
                    return;
                }
                Ok(length) => {
                    if sender
                        .send(PipeEvent::Bytes(buffer[..length].to_vec()))
                        .is_err()
                    {
                        return;
                    }
                }
                Err(_) => {
                    let _ = sender.send(PipeEvent::ReadFailed);
                    return;
                }
            }
        }
    });
}

fn capture_pipes(
    child: &mut Child,
    receiver: Receiver<PipeEvent>,
    log_file: File,
    timeout_seconds: u64,
    redaction_mode: EvidenceRedactionMode,
    command_spec: &CommandSpec,
) -> Result<CaptureProcessResult> {
    let mut sanitizer = IncrementalSanitizer::new(log_file, redaction_mode);
    let started = Instant::now();
    let deadline = Duration::from_secs(timeout_seconds);
    let mut closed_pipes = 0_u8;
    let status = loop {
        while let Ok(event) = receiver.try_recv() {
            handle_pipe_event(child, &mut sanitizer, event, &mut closed_pipes)?;
        }

        if let Some(status) = child
            .try_wait()
            .with_context(|| format!("failed to poll {}", command_spec.display()))?
        {
            break if status.success() {
                CaptureProcessStatus::ExitedSuccess
            } else {
                CaptureProcessStatus::ExitedFailure(status.to_string())
            };
        }

        if started.elapsed() >= deadline {
            child
                .kill()
                .with_context(|| format!("failed to stop {}", command_spec.display()))?;
            child
                .wait()
                .with_context(|| format!("failed to reap {}", command_spec.display()))?;
            break CaptureProcessStatus::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(20));
    };

    drain_pipe_events(child, &receiver, &mut sanitizer, &mut closed_pipes)?;
    sanitizer.finish()?;
    Ok(CaptureProcessResult { status })
}

fn drain_pipe_events(
    child: &mut Child,
    receiver: &Receiver<PipeEvent>,
    sanitizer: &mut IncrementalSanitizer,
    closed_pipes: &mut u8,
) -> Result<()> {
    let drain_deadline = Instant::now() + Duration::from_secs(2);
    while *closed_pipes < 2 {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => handle_pipe_event(child, sanitizer, event, closed_pipes)?,
            Err(mpsc::RecvTimeoutError::Timeout) if Instant::now() < drain_deadline => {}
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => {
                bail!("evidence_sanitization_invalid");
            }
        }
    }
    Ok(())
}

fn handle_pipe_event(
    child: &mut Child,
    sanitizer: &mut IncrementalSanitizer,
    event: PipeEvent,
    closed_pipes: &mut u8,
) -> Result<()> {
    match event {
        PipeEvent::Bytes(bytes) => {
            if let Err(error) = sanitizer.push(&bytes) {
                stop_child_after_capture_failure(child)?;
                return Err(error);
            }
        }
        PipeEvent::ReadFailed => {
            stop_child_after_capture_failure(child)?;
            bail!("evidence_sanitization_invalid");
        }
        PipeEvent::Closed => *closed_pipes += 1,
    }
    Ok(())
}

fn stop_child_after_capture_failure(child: &mut Child) -> Result<()> {
    if child
        .try_wait()
        .context("failed to poll child after capture failure")?
        .is_some()
    {
        return Ok(());
    }
    child
        .kill()
        .context("failed to stop child after capture failure")?;
    child
        .wait()
        .context("failed to reap child after capture failure")?;
    Ok(())
}

struct IncrementalSanitizer {
    output: File,
    pending: Vec<u8>,
    redaction_mode: EvidenceRedactionMode,
}

impl IncrementalSanitizer {
    fn new(output: File, redaction_mode: EvidenceRedactionMode) -> Self {
        Self {
            output,
            pending: Vec::new(),
            redaction_mode,
        }
    }

    fn push(&mut self, bytes: &[u8]) -> Result<()> {
        self.pending.extend_from_slice(bytes);
        if self.pending.len() > MAX_PENDING_LINE_BYTES && !self.pending.contains(&b'\n') {
            bail!("evidence_sanitization_invalid");
        }
        while let Some(newline) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line = self.pending.drain(..=newline).collect::<Vec<_>>();
            self.write_valid(&line)?;
        }
        if self.pending.len() > MAX_PENDING_LINE_BYTES {
            bail!("evidence_sanitization_invalid");
        }
        Ok(())
    }

    fn finish(mut self) -> Result<()> {
        if !self.pending.is_empty() {
            let pending = std::mem::take(&mut self.pending);
            self.write_valid(&pending)?;
        }
        self.output
            .sync_all()
            .context("failed to sync captured evidence")
    }

    fn write_valid(&mut self, bytes: &[u8]) -> Result<()> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| anyhow::anyhow!("evidence_sanitization_invalid"))?;
        let sanitized = sanitize_evidence_text(text, self.redaction_mode);
        self.output
            .write_all(sanitized.as_bytes())
            .context("failed to write sanitized evidence")
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    sha256_bytes(bytes)
}

#[cfg(test)]
mod tests {
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
        spawn_reader(stdout, sender.clone());
        spawn_reader(stderr, sender);
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
        let mut sanitizer = IncrementalSanitizer::new(file, EvidenceRedactionMode::DeveloperRaw);

        // Act
        sanitizer
            .push(b"status password=super-")
            .expect("chunk one");
        sanitizer
            .push(b"secret token=api-secret\n")
            .expect("chunk two");
        sanitizer.finish().expect("finish");

        // Assert
        let captured = fs::read_to_string(path.as_std_path()).expect("captured");
        assert_eq!(captured, "status password=[redacted] token=[redacted]\n");
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
        let evidence_dir =
            Utf8PathBuf::from_path_buf(dir.path().join("evidence")).expect("utf8 path");
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
        let mut sanitizer = IncrementalSanitizer::new(file, EvidenceRedactionMode::DeveloperRaw);

        // Act
        let result = sanitizer.push(&[0xff, b'\n']);

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
        let mut sanitizer = IncrementalSanitizer::new(file, EvidenceRedactionMode::DeveloperRaw);
        let overlong = vec![b'a'; MAX_PENDING_LINE_BYTES + 1];

        // Act
        let result = sanitizer.push(&overlong);

        // Assert
        let error = result.expect_err("overlong line");
        assert!(format!("{error:#}").contains("evidence_sanitization_invalid"));
    }

    #[test]
    fn dual_derivation_preserves_private_digest_and_secures_outputs() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir =
            Utf8PathBuf::from_path_buf(dir.path().join("evidence")).expect("utf8 path");
        let paths = preflight_dual_paths(&evidence_dir).expect("preflight");
        let mut private = open_private_output(&paths.private_log, true).expect("private");
        private
            .write_all(b"ssid=lab password=[redacted] ipv4=192.168.1.1\n")
            .expect("private write");
        drop(private);
        let before = sha256_hex(&fs::read(paths.private_log.as_std_path()).expect("private bytes"));

        // Act
        let digests = derive_admitted_log(&paths).expect("derive");

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
}
