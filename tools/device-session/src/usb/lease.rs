use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    process::{terminate_journal_group, OwnedChildIdentity},
    session_error, UsbLifecycleState, UsbOperation, UsbSessionError, UsbTerminalCategory,
};

const ROOT_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CrashJournal {
    schema: String,
    session_nonce: String,
    physical_identity_digest: String,
    owner_pid: u32,
    owner_start: String,
    operation: UsbOperation,
    state: UsbLifecycleState,
    earliest_failure: Option<UsbTerminalCategory>,
    child: Option<OwnedChildIdentity>,
}

pub(super) struct DeviceLease {
    _lock: File,
    journal_file: File,
    journal: CrashJournal,
    journal_path: PathBuf,
    remove_journal_on_drop: bool,
}

impl DeviceLease {
    pub(super) fn acquire(
        physical_identity_digest: &str,
        operation: UsbOperation,
        trace_root: &Path,
    ) -> Result<Self, UsbSessionError> {
        create_private_dir(trace_root)?;
        let state_root = std::env::temp_dir().join(format!("bitaxe-device-sessions-{}", unsafe {
            libc::getuid()
        }));
        create_private_dir(&state_root)?;
        let device_root = state_root.join(physical_identity_digest);
        create_private_dir(&device_root)?;
        let lock_path = device_root.join("lease.lock");
        let lock = private_file(&lock_path)?;
        let lock_result = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if lock_result != 0 {
            return Err(session_error(
                UsbTerminalCategory::ConcurrentRepoSession,
                "another repository session owns this physical device",
            ));
        }
        let journal_path = device_root.join("crash-journal.json");
        reconcile_stale_journal(&journal_path)?;
        let journal_file = private_file(&journal_path)?;
        let owner_pid = std::process::id();
        let journal = CrashJournal {
            schema: "bitaxe-usb-crash-journal-v1".to_owned(),
            session_nonce: nonce(owner_pid),
            physical_identity_digest: physical_identity_digest.to_owned(),
            owner_pid,
            owner_start: process_start(owner_pid).unwrap_or_else(|| "current".to_owned()),
            operation,
            state: UsbLifecycleState::Prepared,
            earliest_failure: None,
            child: None,
        };
        let mut lease = Self {
            _lock: lock,
            journal_file,
            journal,
            journal_path,
            remove_journal_on_drop: false,
        };
        lease.persist()?;
        Ok(lease)
    }

    pub(super) fn record_state(
        &mut self,
        state: UsbLifecycleState,
        earliest_failure: Option<UsbTerminalCategory>,
    ) -> Result<(), UsbSessionError> {
        self.journal.state = state;
        self.journal.earliest_failure = earliest_failure.or(self.journal.earliest_failure);
        self.persist()
    }

    pub(super) fn record_child(
        &mut self,
        maybe_child: Option<OwnedChildIdentity>,
    ) -> Result<(), UsbSessionError> {
        self.journal.child = maybe_child;
        self.persist()
    }

    pub(super) fn mark_complete(&mut self) {
        self.remove_journal_on_drop = true;
    }

    fn persist(&mut self) -> Result<(), UsbSessionError> {
        write_journal(&self.journal_file, &self.journal)
    }
}

impl Clone for CrashJournal {
    fn clone(&self) -> Self {
        Self {
            schema: self.schema.clone(),
            session_nonce: self.session_nonce.clone(),
            physical_identity_digest: self.physical_identity_digest.clone(),
            owner_pid: self.owner_pid,
            owner_start: self.owner_start.clone(),
            operation: self.operation,
            state: self.state,
            earliest_failure: self.earliest_failure,
            child: self.child.clone(),
        }
    }
}

impl Drop for DeviceLease {
    fn drop(&mut self) {
        if self.remove_journal_on_drop {
            let _result = fs::remove_file(&self.journal_path);
        }
    }
}

fn reconcile_stale_journal(path: &Path) -> Result<(), UsbSessionError> {
    let Ok(mut file) = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
    else {
        return Ok(());
    };
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|error| {
        session_error(
            UsbTerminalCategory::CleanupFailed,
            format!("stale journal read failed: {error}"),
        )
    })?;
    let journal: CrashJournal = serde_json::from_str(&contents).map_err(|error| {
        session_error(
            UsbTerminalCategory::ForeignHolder,
            format!("stale journal could not be authenticated: {error}"),
        )
    })?;
    if journal.schema != "bitaxe-usb-crash-journal-v1" {
        return Err(session_error(
            UsbTerminalCategory::ForeignHolder,
            "stale journal schema is not repository-owned",
        ));
    }
    if process_matches(journal.owner_pid, &journal.owner_start) {
        return Err(session_error(
            UsbTerminalCategory::ConcurrentRepoSession,
            "the recorded repository session is still alive",
        ));
    }
    if let Some(child) = journal.child {
        terminate_journal_group(&child, Duration::from_secs(2))?;
    }
    fs::remove_file(path).map_err(|error| {
        session_error(
            UsbTerminalCategory::CleanupFailed,
            format!("stale journal removal failed: {error}"),
        )
    })
}

fn private_file(path: &Path) -> Result<File, UsbSessionError> {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .mode(FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    let metadata = file
        .metadata()
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    if !metadata.is_file() {
        return Err(session_error(
            UsbTerminalCategory::CleanupFailed,
            "private session artifact is not an ordinary file",
        ));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(FILE_MODE))
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    Ok(file)
}

fn create_private_dir(path: &Path) -> Result<(), UsbSessionError> {
    fs::create_dir_all(path)
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(session_error(
            UsbTerminalCategory::CleanupFailed,
            "private session root is not an ordinary directory",
        ));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(ROOT_MODE))
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))
}

fn write_journal(file: &File, journal: &CrashJournal) -> Result<(), UsbSessionError> {
    let mut file = file;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    file.set_len(0)
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    serde_json::to_writer(&mut file, journal)
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    file.write_all(b"\n")
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))?;
    file.sync_all()
        .map_err(|error| session_error(UsbTerminalCategory::CleanupFailed, error))
}

pub(super) fn process_start(pid: u32) -> Option<String> {
    let output = Command::new("/bin/ps")
        .args(["-o", "lstart=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn process_matches(pid: u32, expected_start: &str) -> bool {
    process_start(pid).is_some_and(|observed| observed == expected_start)
}

fn nonce(pid: u32) -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(pid.to_le_bytes());
    hasher.update(duration.as_nanos().to_le_bytes());
    encode_digest(hasher.finalize())
}

fn encode_digest(digest: impl AsRef<[u8]>) -> String {
    let digest = digest.as_ref();
    let mut encoded = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Instant;
    use tempfile::tempdir;

    #[test]
    fn second_lease_for_same_physical_device_is_refused() {
        // Arrange
        let directory = tempdir().expect("temporary directory");
        let mut first = DeviceLease::acquire(
            "lock-contention-test",
            UsbOperation::Flash,
            directory.path(),
        )
        .expect("first lease");

        // Act
        let result = DeviceLease::acquire(
            "lock-contention-test",
            UsbOperation::Monitor,
            directory.path(),
        );
        let Err(error) = result else {
            panic!("second lease must fail");
        };

        // Assert
        assert_eq!(error.category, UsbTerminalCategory::ConcurrentRepoSession);
        first.mark_complete();
        drop(first);
    }

    #[test]
    fn fresh_process_lease_contention_is_refused() {
        // Arrange
        let directory = tempdir().expect("temporary directory");
        let ready_path = directory.path().join("ready");
        let stop_path = directory.path().join("stop");
        let digest = format!("fresh-process-lock-{}", std::process::id());
        let mut child = Command::new(std::env::current_exe().expect("current test executable"))
            .args([
                "--exact",
                "usb::lease::tests::lease_helper_process",
                "--nocapture",
            ])
            .env("BITAXE_LEASE_HELPER_DIGEST", &digest)
            .env("BITAXE_LEASE_HELPER_ROOT", directory.path())
            .env("BITAXE_LEASE_HELPER_READY", &ready_path)
            .env("BITAXE_LEASE_HELPER_STOP", &stop_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("lease helper process");
        wait_for_path(&ready_path);

        // Act
        let result = DeviceLease::acquire(&digest, UsbOperation::Monitor, directory.path());

        // Assert
        let Err(error) = result else {
            panic!("fresh process contention must fail");
        };
        assert_eq!(error.category, UsbTerminalCategory::ConcurrentRepoSession);
        fs::write(&stop_path, b"stop").expect("stop helper");
        assert!(child.wait().expect("helper exit").success());
    }

    #[test]
    fn owner_crash_journal_is_reconciled_after_lock_release() {
        // Arrange
        let directory = tempdir().expect("temporary directory");
        let ready_path = directory.path().join("crash-ready");
        let stop_path = directory.path().join("unused-stop");
        let digest = format!("owner-crash-lock-{}", std::process::id());
        let mut child = Command::new(std::env::current_exe().expect("current test executable"))
            .args([
                "--exact",
                "usb::lease::tests::lease_helper_process",
                "--nocapture",
            ])
            .env("BITAXE_LEASE_HELPER_DIGEST", &digest)
            .env("BITAXE_LEASE_HELPER_ROOT", directory.path())
            .env("BITAXE_LEASE_HELPER_READY", &ready_path)
            .env("BITAXE_LEASE_HELPER_STOP", &stop_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("lease helper process");
        wait_for_path(&ready_path);
        let signal_result = unsafe { libc::kill(child.id().cast_signed(), libc::SIGKILL) };
        assert_eq!(signal_result, 0);
        let _status = child.wait().expect("crashed helper reap");

        // Act
        let recovered = DeviceLease::acquire(&digest, UsbOperation::Flash, directory.path());

        // Assert
        assert!(recovered.is_ok());
    }

    #[test]
    fn lease_helper_process() {
        // Arrange
        let Ok(digest) = std::env::var("BITAXE_LEASE_HELPER_DIGEST") else {
            return;
        };
        let root =
            PathBuf::from(std::env::var_os("BITAXE_LEASE_HELPER_ROOT").expect("helper root"));
        let ready =
            PathBuf::from(std::env::var_os("BITAXE_LEASE_HELPER_READY").expect("helper ready"));
        let stop =
            PathBuf::from(std::env::var_os("BITAXE_LEASE_HELPER_STOP").expect("helper stop"));

        // Act
        let mut lease =
            DeviceLease::acquire(&digest, UsbOperation::Flash, &root).expect("helper device lease");
        fs::write(ready, b"ready").expect("helper readiness");
        let deadline = Instant::now() + Duration::from_secs(5);
        while !stop.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }

        // Assert
        assert!(stop.exists(), "helper stop signal was not received");
        lease.mark_complete();
    }

    fn wait_for_path(path: &Path) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !path.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(path.exists(), "helper readiness timed out");
    }
}
