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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PipeStream {
    Stdout,
    Stderr,
}

enum PipeEvent {
    Bytes(PipeStream, Vec<u8>),
    ReadFailed(PipeStream),
    Closed(PipeStream),
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

    let paths = dual_paths(evidence_dir);
    let DualEvidencePaths {
        private_log,
        admitted_log,
        private_record,
        admitted_record,
    } = &paths;
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

    Ok(paths)
}

pub(crate) fn preflight_dual_finalization_paths(
    evidence_dir: &Utf8Path,
) -> Result<DualEvidencePaths> {
    let metadata = fs::symlink_metadata(evidence_dir.as_std_path())
        .with_context(|| format!("failed to inspect evidence directory {evidence_dir}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("dual evidence finalization failed: evidence root must be an owned directory");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o777 != 0o700 {
            bail!("dual evidence finalization failed: evidence root is not mode 0700");
        }
    }

    let paths = dual_paths(evidence_dir);
    for path in [&paths.private_log, &paths.private_record] {
        let metadata = fs::symlink_metadata(path.as_std_path())
            .with_context(|| format!("missing private evidence input {path}"))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("dual evidence finalization failed: private input is not a regular file");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o777 != 0o600 {
                bail!("dual evidence finalization failed: private input is not mode 0600");
            }
        }
        if path.parent() != Some(evidence_dir) {
            bail!("dual evidence finalization failed: private input escapes evidence root");
        }
    }
    for path in [&paths.admitted_log, &paths.admitted_record] {
        match fs::symlink_metadata(path.as_std_path()) {
            Ok(_) => bail!("dual evidence finalization failed: destination already exists"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to inspect evidence path {path}"));
            }
        }
        if path.parent() != Some(evidence_dir) {
            bail!("dual evidence finalization failed: destination escapes evidence root");
        }
    }
    Ok(paths)
}

fn dual_paths(evidence_dir: &Utf8Path) -> DualEvidencePaths {
    DualEvidencePaths {
        private_log: evidence_dir.join("flash-monitor.classifier-input.log"),
        admitted_log: evidence_dir.join("flash-monitor.log"),
        private_record: evidence_dir.join("flash-command-evidence.private.json"),
        admitted_record: evidence_dir.join("flash-command-evidence.json"),
    }
}

pub(crate) fn capture_command(
    command_spec: &CommandSpec,
    trusted_program: &Utf8Path,
    log_path: &Utf8Path,
    timeout_seconds: u64,
    redaction_mode: EvidenceRedactionMode,
    create_new: bool,
) -> Result<CaptureProcessResult> {
    validate_trusted_program(command_spec, trusted_program)?;
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent.as_std_path())
            .with_context(|| format!("failed to create log directory {parent}"))?;
    }

    let log_file = open_private_output(log_path, create_new)?;
    let mut command = Command::new(&command_spec.program);
    command
        .args(&command_spec.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => {
            let mut sanitizer = InterleavedSanitizer::new(log_file, redaction_mode);
            sanitizer.push(PipeStream::Stderr, b"phase35_child_spawn_failed\n")?;
            sanitizer.finish()?;
            return Ok(CaptureProcessResult {
                status: CaptureProcessStatus::SpawnFailed,
            });
        }
    };
    let stdout = child
        .stdout
        .take()
        .context("failed to capture monitor stdout")?;
    let stderr = child
        .stderr
        .take()
        .context("failed to capture monitor stderr")?;

    let (sender, receiver) = mpsc::channel();
    spawn_reader(stdout, sender.clone(), PipeStream::Stdout);
    spawn_reader(stderr, sender, PipeStream::Stderr);
    capture_pipes(
        &mut child,
        receiver,
        log_file,
        timeout_seconds,
        redaction_mode,
        command_spec,
    )
}

fn validate_trusted_program(command_spec: &CommandSpec, trusted_program: &Utf8Path) -> Result<()> {
    let requested_program = Utf8Path::new(&command_spec.program);
    if requested_program != trusted_program || !trusted_program.is_absolute() {
        bail!("capture_command=blocked reason=untrusted_program");
    }
    let canonical = fs::canonicalize(trusted_program.as_std_path())
        .context("capture_command=blocked reason=untrusted_program")?;
    if canonical != trusted_program.as_std_path() {
        bail!("capture_command=blocked reason=untrusted_program");
    }
    let metadata = fs::metadata(trusted_program.as_std_path())
        .context("capture_command=blocked reason=untrusted_program")?;
    if !metadata.is_file() {
        bail!("capture_command=blocked reason=untrusted_program");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            bail!("capture_command=blocked reason=untrusted_program");
        }
    }
    Ok(())
}

pub(crate) fn private_log_sha256(path: &Utf8Path) -> Result<String> {
    let bytes = fs::read(path.as_std_path())
        .with_context(|| format!("failed to read private evidence {path}"))?;
    Ok(sha256_hex(&bytes))
}

pub(crate) fn derive_admitted_log(
    paths: &DualEvidencePaths,
    expected_private_sha256: &str,
) -> Result<DualEvidenceDigests> {
    let private_bytes = fs::read(paths.private_log.as_std_path())
        .with_context(|| format!("failed to read private evidence {}", paths.private_log))?;
    let private_text = std::str::from_utf8(&private_bytes)
        .map_err(|_| anyhow::anyhow!("evidence_sanitization_invalid"))?;
    let private_sha256 = sha256_hex(&private_bytes);
    if private_sha256 != expected_private_sha256 {
        bail!("private evidence digest does not match classified input");
    }
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

fn spawn_reader(
    mut reader: impl Read + Send + 'static,
    sender: Sender<PipeEvent>,
    stream: PipeStream,
) {
    std::thread::spawn(move || {
        let mut buffer = [0_u8; READ_CHUNK_BYTES];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    let _ = sender.send(PipeEvent::Closed(stream));
                    return;
                }
                Ok(length) => {
                    if sender
                        .send(PipeEvent::Bytes(stream, buffer[..length].to_vec()))
                        .is_err()
                    {
                        return;
                    }
                }
                Err(_) => {
                    let _ = sender.send(PipeEvent::ReadFailed(stream));
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
    let mut sanitizer = InterleavedSanitizer::new(log_file, redaction_mode);
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
    sanitizer: &mut InterleavedSanitizer,
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
    sanitizer: &mut InterleavedSanitizer,
    event: PipeEvent,
    closed_pipes: &mut u8,
) -> Result<()> {
    match event {
        PipeEvent::Bytes(stream, bytes) => {
            if let Err(error) = sanitizer.push(stream, &bytes) {
                stop_child_after_capture_failure(child)?;
                return Err(error);
            }
        }
        PipeEvent::ReadFailed(_stream) => {
            stop_child_after_capture_failure(child)?;
            bail!("evidence_sanitization_invalid");
        }
        PipeEvent::Closed(_stream) => *closed_pipes += 1,
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
    pending: Vec<u8>,
    redaction_mode: EvidenceRedactionMode,
}

impl IncrementalSanitizer {
    fn new(redaction_mode: EvidenceRedactionMode) -> Self {
        Self {
            pending: Vec::new(),
            redaction_mode,
        }
    }

    fn push(&mut self, bytes: &[u8], output: &mut File) -> Result<()> {
        self.pending.extend_from_slice(bytes);
        if self.pending.len() > MAX_PENDING_LINE_BYTES && !self.pending.contains(&b'\n') {
            bail!("evidence_sanitization_invalid");
        }
        while let Some(newline) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line = self.pending.drain(..=newline).collect::<Vec<_>>();
            self.write_valid(&line, output)?;
        }
        if self.pending.len() > MAX_PENDING_LINE_BYTES {
            bail!("evidence_sanitization_invalid");
        }
        Ok(())
    }

    fn finish(&mut self, output: &mut File) -> Result<()> {
        if !self.pending.is_empty() {
            let pending = std::mem::take(&mut self.pending);
            self.write_valid(&pending, output)?;
        }
        Ok(())
    }

    fn write_valid(&self, bytes: &[u8], output: &mut File) -> Result<()> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| anyhow::anyhow!("evidence_sanitization_invalid"))?;
        let sanitized = sanitize_evidence_text(text, self.redaction_mode);
        output
            .write_all(sanitized.as_bytes())
            .context("failed to write sanitized evidence")
    }
}

struct InterleavedSanitizer {
    output: File,
    stdout: IncrementalSanitizer,
    stderr: IncrementalSanitizer,
}

impl InterleavedSanitizer {
    fn new(output: File, redaction_mode: EvidenceRedactionMode) -> Self {
        Self {
            output,
            stdout: IncrementalSanitizer::new(redaction_mode),
            stderr: IncrementalSanitizer::new(redaction_mode),
        }
    }

    fn push(&mut self, stream: PipeStream, bytes: &[u8]) -> Result<()> {
        match stream {
            PipeStream::Stdout => self.stdout.push(bytes, &mut self.output),
            PipeStream::Stderr => self.stderr.push(bytes, &mut self.output),
        }
    }

    fn finish(mut self) -> Result<()> {
        self.stdout.finish(&mut self.output)?;
        self.stderr.finish(&mut self.output)?;
        self.output
            .sync_all()
            .context("failed to sync captured evidence")
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    sha256_bytes(bytes)
}

#[cfg(test)]
mod tests;
