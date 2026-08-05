use std::fs::{self, OpenOptions};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::lease::{maybe_process_start, DeviceLease};
use super::{
    session_error, SupervisedOutput, SupervisedTermination, UsbSessionError, UsbTerminalCategory,
};

static PENDING_SIGNAL: AtomicI32 = AtomicI32::new(0);
static SIGNAL_HANDLER_LOCK: Mutex<()> = Mutex::new(());
const FILE_MODE: u32 = 0o600;

pub(super) struct OwnedProcessRequest<'a> {
    pub(super) program: &'a Path,
    pub(super) args: &'a [String],
    pub(super) timeout: Duration,
    pub(super) trace_root: &'a Path,
    pub(super) trace_label: &'a str,
    pub(super) maybe_rust_log: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnedChildIdentity {
    pid: u32,
    process_group: i32,
    process_start: String,
    executable_path: PathBuf,
    executable_sha256: String,
}

impl OwnedChildIdentity {
    pub(super) fn is_alive(&self) -> bool {
        let result = unsafe { libc::kill(self.pid.cast_signed(), 0) };
        result == 0
    }

    pub(super) fn identity_matches(&self) -> bool {
        if maybe_process_start(self.pid).as_deref() != Some(self.process_start.as_str()) {
            return false;
        }
        let group = unsafe { libc::getpgid(self.pid.cast_signed()) };
        if group != self.process_group {
            return false;
        }
        let expected_path = fs::canonicalize(&self.executable_path).ok();
        let observed_path =
            maybe_process_executable(self.pid).and_then(|path| fs::canonicalize(path).ok());
        if observed_path != expected_path {
            return false;
        }
        executable_digest(&self.executable_path)
            .is_ok_and(|digest| digest == self.executable_sha256)
    }
}

pub(super) fn run_owned_process(
    request: OwnedProcessRequest<'_>,
    lease: &mut DeviceLease,
) -> Result<SupervisedOutput, UsbSessionError> {
    let _signal_supervisor = SignalSupervisor::acquire()?;

    let executable_path = fs::canonicalize(request.program).map_err(|error| {
        session_error(
            UsbTerminalCategory::FlashFailedBeforeTransfer,
            format!("espflash executable resolution failed: {error}"),
        )
    })?;
    let executable_sha256 = executable_digest(&executable_path)?;
    let stdout_path = request
        .trace_root
        .join(format!("{}.stdout", request.trace_label));
    let stderr_path = request
        .trace_root
        .join(format!("{}.stderr", request.trace_label));
    let stdout_file = private_output(&stdout_path)?;
    let stderr_file = private_output(&stderr_path)?;

    let mut command = Command::new(&executable_path);
    command
        .args(request.args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .process_group(0);
    if let Some(rust_log) = request.maybe_rust_log {
        command.env("RUST_LOG", rust_log);
    }
    let mut child = command.spawn().map_err(|error| {
        session_error(
            UsbTerminalCategory::FlashFailedBeforeTransfer,
            format!("espflash launch failed: {error}"),
        )
    })?;
    let pid = child.id();
    let process_group = unsafe { libc::getpgid(pid.cast_signed()) };
    if process_group != pid.cast_signed() {
        let _result = child.kill();
        let _result = child.wait();
        return Err(session_error(
            UsbTerminalCategory::CleanupFailed,
            "espflash child did not enter its isolated process group",
        ));
    }
    let identity = OwnedChildIdentity {
        pid,
        process_group,
        process_start: wait_for_process_start(pid)?,
        executable_path,
        executable_sha256,
    };
    lease.record_child(Some(identity.clone()))?;

    let deadline = Instant::now() + request.timeout;
    let mut timed_out = false;
    let mut interrupted_by = None;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| {
            session_error(
                UsbTerminalCategory::CleanupFailed,
                format!("espflash reap probe failed: {error}"),
            )
        })? {
            break status;
        }
        let signal = maybe_pending_signal();
        if signal.is_some() || Instant::now() >= deadline {
            timed_out = signal.is_none();
            interrupted_by = signal;
            terminate_owned_group(&identity, Duration::from_secs(2))?;
            break child.wait().map_err(|error| {
                session_error(
                    UsbTerminalCategory::CleanupFailed,
                    format!("espflash reap failed: {error}"),
                )
            })?;
        }
        thread::sleep(Duration::from_millis(25));
    };
    if process_group_has_live_members(identity.process_group) {
        terminate_group(identity.process_group, Duration::from_secs(2))?;
    }
    lease.record_child(None)?;
    let stdout = fs::read(&stdout_path).map_err(|error| {
        session_error(
            UsbTerminalCategory::CleanupFailed,
            format!("espflash stdout capture failed: {error}"),
        )
    })?;
    let stderr = fs::read(&stderr_path).map_err(|error| {
        session_error(
            UsbTerminalCategory::CleanupFailed,
            format!("espflash stderr capture failed: {error}"),
        )
    })?;
    let termination = if timed_out {
        SupervisedTermination::TimedOut
    } else if let Some(signal) = interrupted_by {
        SupervisedTermination::Interrupted { signal }
    } else if status.success() {
        SupervisedTermination::ExitedSuccess
    } else {
        SupervisedTermination::ExitedFailure
    };
    Ok(SupervisedOutput {
        termination,
        stdout,
        stderr,
    })
}

pub(super) fn terminate_owned_group(
    identity: &OwnedChildIdentity,
    grace: Duration,
) -> Result<(), UsbSessionError> {
    if !identity.identity_matches() {
        return Err(session_error(
            UsbTerminalCategory::ForeignHolder,
            "repository child identity changed before cleanup",
        ));
    }
    terminate_group(identity.process_group, grace)
}

pub(super) fn terminate_journal_group(
    identity: &OwnedChildIdentity,
    grace: Duration,
) -> Result<(), UsbSessionError> {
    // A reused PGID requires a new live leader with PID == PGID. If that
    // leader exists, its recorded identity must still match. Without a leader,
    // remaining group members belong to the original isolated child group.
    if identity.is_alive() && !identity.identity_matches() {
        return Err(session_error(
            UsbTerminalCategory::ForeignHolder,
            "stale child identity cannot be proven",
        ));
    }
    terminate_group(identity.process_group, grace)
}

fn terminate_group(group: i32, grace: Duration) -> Result<(), UsbSessionError> {
    if !process_group_has_live_members(group) {
        return Ok(());
    }
    signal_group(group, libc::SIGTERM)?;
    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if !process_group_has_live_members(group) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    signal_group(group, libc::SIGKILL)?;
    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if !process_group_has_live_members(group) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    Err(session_error(
        UsbTerminalCategory::CleanupFailed,
        "repository process group survived bounded cleanup",
    ))
}

fn process_group_has_live_members(group: i32) -> bool {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pgid=,stat="])
        .output();
    let Ok(output) = output else {
        return true;
    };
    if !output.status.success() {
        return true;
    }
    String::from_utf8_lossy(&output.stdout).lines().any(|line| {
        let mut fields = line.split_whitespace();
        let maybe_group = fields.next().and_then(|value| value.parse::<i32>().ok());
        let maybe_state = fields.next();
        maybe_group == Some(group) && maybe_state.is_some_and(|state| !state.starts_with('Z'))
    })
}

pub(super) fn maybe_pending_signal() -> Option<i32> {
    match PENDING_SIGNAL.load(Ordering::SeqCst) {
        0 => None,
        signal => Some(signal),
    }
}

fn signal_group(group: i32, signal: i32) -> Result<(), UsbSessionError> {
    let result = unsafe { libc::kill(-group, signal) };
    if result == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        return Ok(());
    }
    Err(session_error(
        UsbTerminalCategory::CleanupFailed,
        format!("repository process-group signal {signal} failed: {error}"),
    ))
}

fn private_output(path: &Path) -> Result<std::fs::File, UsbSessionError> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    if !file
        .metadata()
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?
        .is_file()
    {
        return Err(session_error(
            UsbTerminalCategory::CleanupFailed,
            "private process trace is not an ordinary file",
        ));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(FILE_MODE))
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    Ok(file)
}

fn executable_digest(path: &Path) -> Result<String, UsbSessionError> {
    let bytes = fs::read(path).map_err(|error| {
        session_error(
            UsbTerminalCategory::ForeignHolder,
            format!("executable digest read failed: {error}"),
        )
    })?;
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(encoded)
}

fn wait_for_process_start(pid: u32) -> Result<String, UsbSessionError> {
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        if let Some(start) = maybe_process_start(pid) {
            return Ok(start);
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err(session_error(
        UsbTerminalCategory::CleanupFailed,
        "child process identity was unavailable",
    ))
}

fn maybe_process_executable(pid: u32) -> Option<PathBuf> {
    let output = Command::new("/bin/ps")
        .args(["-o", "comm=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

fn signal_handler_lock() -> Result<MutexGuard<'static, ()>, UsbSessionError> {
    SIGNAL_HANDLER_LOCK.lock().map_err(|error| {
        session_error(
            UsbTerminalCategory::CleanupFailed,
            format!("signal supervisor lock failed: {error}"),
        )
    })
}

extern "C" fn record_signal(signal: i32) {
    PENDING_SIGNAL.store(signal, Ordering::SeqCst);
}

pub(super) struct SignalSupervisor {
    _lock: MutexGuard<'static, ()>,
    _guard: SignalGuard,
}

impl SignalSupervisor {
    pub(super) fn acquire() -> Result<Self, UsbSessionError> {
        let lock = signal_handler_lock()?;
        PENDING_SIGNAL.store(0, Ordering::SeqCst);
        let guard = SignalGuard::install()?;
        Ok(Self {
            _lock: lock,
            _guard: guard,
        })
    }
}

struct SignalGuard {
    previous_int: libc::sigaction,
    previous_term: libc::sigaction,
}

impl SignalGuard {
    fn install() -> Result<Self, UsbSessionError> {
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = record_signal as *const () as usize;
            libc::sigemptyset(&mut action.sa_mask);
            action.sa_flags = 0;
            let mut previous_int: libc::sigaction = std::mem::zeroed();
            let mut previous_term: libc::sigaction = std::mem::zeroed();
            if libc::sigaction(libc::SIGINT, &action, &mut previous_int) != 0
                || libc::sigaction(libc::SIGTERM, &action, &mut previous_term) != 0
            {
                return Err(session_error(
                    UsbTerminalCategory::CleanupFailed,
                    std::io::Error::last_os_error(),
                ));
            }
            Ok(Self {
                previous_int,
                previous_term,
            })
        }
    }
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        unsafe {
            libc::sigaction(libc::SIGINT, &self.previous_int, std::ptr::null_mut());
            libc::sigaction(libc::SIGTERM, &self.previous_term, std::ptr::null_mut());
        }
        PENDING_SIGNAL.store(0, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests;
