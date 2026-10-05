//! Imperative shell: the supervised USB session, pinned espflash and private evidence files.

use std::env;
use std::fs::{self, File};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use sha2::{Digest, Sha256};

use super::identity::{IdentityScanner, IdentityVerdict};
use super::model::CycleRow;
use super::runner::{CycleEffects, ObserveFacts, ProfileFacts, ResetFacts};
use crate::evidence::{create_empty_private_root, open_private_new};
use crate::usb_ownership::installed_application_args;
use crate::{
    inspect_usb_profile, UsbCommandTermination, UsbProfile, UsbSession, UsbSessionError,
    UsbTerminalCategory,
};

const ESPFLASH_EXPECTED_VERSION: &str = "espflash 4.5.0";
const VERSION_TIMEOUT: Duration = Duration::from_secs(10);
const RESET_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_CYCLE_SERIAL_BYTES: usize = 256 * 1024;

/// Pinned espflash executable whose bytes must stay unchanged across every reset.
pub(crate) struct EspflashTool {
    path: Utf8PathBuf,
    sha256: String,
}

impl EspflashTool {
    /// Resolves `ESPFLASH_BIN` or `espflash` on `PATH` without executing it.
    pub(crate) fn resolve() -> Result<Self> {
        let requested = env::var("ESPFLASH_BIN").unwrap_or_else(|_| "espflash".to_owned());
        let requested_path = Utf8Path::new(&requested);
        let candidate = if requested_path.components().count() > 1 || requested_path.is_absolute() {
            requested_path.to_owned()
        } else {
            env::split_paths(&env::var_os("PATH").unwrap_or_default())
                .map(|directory| directory.join(&requested))
                .find(|path| path.is_file())
                .and_then(|path| Utf8PathBuf::from_path_buf(path).ok())
                .context("usb_reset_endurance=blocked reason=espflash_not_found")?
        };
        let canonical = fs::canonicalize(candidate.as_std_path())
            .context("usb_reset_endurance=blocked reason=espflash_not_found")?;
        let canonical = Utf8PathBuf::from_path_buf(canonical)
            .map_err(|_| anyhow::anyhow!("usb_reset_endurance=blocked reason=espflash_path"))?;
        let metadata = fs::metadata(canonical.as_std_path())?;
        #[cfg(unix)]
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            bail!("usb_reset_endurance=blocked reason=espflash_not_executable");
        }
        let sha256 = sha256_file(&canonical)?;
        Ok(Self {
            path: canonical,
            sha256,
        })
    }

    fn unchanged(&self) -> bool {
        sha256_file(&self.path).is_ok_and(|digest| digest == self.sha256)
    }
}

/// Requires a mode-0700 parent and exclusively creates the not-yet-existing root.
pub(crate) fn create_supervisor_root(root: &Utf8Path) -> Result<()> {
    if root
        .components()
        .any(|component| matches!(component, Utf8Component::ParentDir))
    {
        bail!("usb_reset_endurance=blocked reason=private_root_path");
    }
    if fs::symlink_metadata(root.as_std_path()).is_ok() {
        bail!("usb_reset_endurance=blocked reason=private_root_exists");
    }
    let parent = root
        .parent()
        .filter(|parent| !parent.as_str().is_empty())
        .context("usb_reset_endurance=blocked reason=private_parent_missing")?;
    let metadata = fs::symlink_metadata(parent.as_std_path())
        .context("usb_reset_endurance=blocked reason=private_parent_missing")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("usb_reset_endurance=blocked reason=private_parent_invalid");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o700 {
        bail!("usb_reset_endurance=blocked reason=private_parent_mode");
    }
    create_empty_private_root(root)
}

/// Admits the session's espflash version through the same supervised child path as resets.
pub(crate) fn admit_espflash_version(
    session: &mut UsbSession,
    espflash: &EspflashTool,
) -> Result<(), UsbSessionError> {
    let output = session.run_espflash_probe(
        espflash.path.as_std_path(),
        &["--version".to_owned()],
        VERSION_TIMEOUT,
    )?;
    if output.stdout != format!("{ESPFLASH_EXPECTED_VERSION}\n").as_bytes() {
        return Err(UsbSessionError {
            category: UsbTerminalCategory::FlashFailedBeforeTransfer,
            detail: "espflash_version_mismatch".to_owned(),
        });
    }
    Ok(())
}

/// Device effects backed by one retained physical-device lease.
pub(crate) struct SessionEffects<'a> {
    session: UsbSession,
    espflash: &'a EspflashTool,
    expected_physical_sha256: &'a str,
    root: &'a Utf8Path,
    rows: File,
    clock: Instant,
}

impl<'a> SessionEffects<'a> {
    pub(crate) fn new(
        session: UsbSession,
        espflash: &'a EspflashTool,
        expected_physical_sha256: &'a str,
        root: &'a Utf8Path,
    ) -> Result<Self> {
        Ok(Self {
            session,
            espflash,
            expected_physical_sha256,
            root,
            rows: open_private_new(&root.join("cycles.private.jsonl"))?,
            clock: Instant::now(),
        })
    }

    pub(crate) fn into_session(self) -> UsbSession {
        self.session
    }
}

impl CycleEffects for SessionEffects<'_> {
    fn monotonic_ms(&mut self) -> u64 {
        millis(self.clock.elapsed())
    }

    fn inspect_profile(&mut self) -> Result<ProfileFacts, UsbSessionError> {
        let inspection =
            inspect_usb_profile(self.session.port()).map_err(|error| UsbSessionError {
                category: UsbTerminalCategory::RuntimeProfileUnknown,
                detail: error.to_string(),
            })?;
        Ok(ProfileFacts {
            profile: inspection.profile,
            physical_identity_matches: inspection.physical_identity_digest
                == self.expected_physical_sha256
                && self.session.physical_identity_digest() == self.expected_physical_sha256,
        })
    }

    fn reset_application(&mut self) -> ResetFacts {
        if !self.espflash.unchanged() {
            return ResetFacts {
                termination: UsbCommandTermination::NotStarted,
                maybe_error: Some(UsbSessionError {
                    category: UsbTerminalCategory::FlashFailedBeforeTransfer,
                    detail: "espflash_identity_changed".to_owned(),
                }),
            };
        }
        let args = installed_application_args(self.session.port());
        let result = self.session.run_bootstrap_reset(
            self.espflash.path.as_std_path(),
            &args,
            RESET_TIMEOUT,
        );
        match result {
            Ok(_) => ResetFacts {
                termination: UsbCommandTermination::ExitedSuccess,
                maybe_error: None,
            },
            Err(error) => ResetFacts {
                termination: self
                    .session
                    .last_command_diagnostic()
                    .map_or(UsbCommandTermination::NotStarted, |diagnostic| {
                        diagnostic.termination
                    }),
                maybe_error: Some(error),
            },
        }
    }

    fn reacquire_transport(&mut self) -> Result<(UsbProfile, bool), UsbSessionError> {
        self.session.reacquire_application_transport()
    }

    fn observe(
        &mut self,
        timeout: Duration,
        scanner: &mut IdentityScanner,
    ) -> Result<ObserveFacts, UsbSessionError> {
        let started = Instant::now();
        let mut facts = ObserveFacts::default();
        let output =
            self.session
                .observe_receive_only_ephemeral_chunks_until(timeout, |chunk| {
                    let remaining = MAX_CYCLE_SERIAL_BYTES.saturating_sub(facts.serial.len());
                    facts
                        .serial
                        .extend_from_slice(&chunk[..chunk.len().min(remaining)]);
                    let verdict = scanner.feed(chunk);
                    if verdict == IdentityVerdict::Complete
                        && facts.maybe_identity_latency_ms.is_none()
                    {
                        facts.maybe_identity_latency_ms = Some(millis(started.elapsed()));
                    }
                    verdict != IdentityVerdict::Pending
                })?;
        facts.interrupted = output.interrupted_by.is_some();
        Ok(facts)
    }

    fn prove_released(&mut self) -> Result<(), UsbSessionError> {
        self.session.prove_transport_released()
    }

    fn record_cycle(&mut self, row: &CycleRow, serial: &[u8]) -> Result<()> {
        let mut encoded = serde_json::to_vec(row)?;
        encoded.push(b'\n');
        self.rows.write_all(&encoded)?;
        self.rows.sync_data()?;
        let path = self
            .root
            .join(format!("cycle-{:04}.serial.private", row.cycle));
        let mut file = open_private_new(&path)?;
        file.write_all(serial)?;
        file.sync_data()?;
        Ok(())
    }
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn sha256_file(path: &Utf8Path) -> Result<String> {
    let bytes = fs::read(path.as_std_path()).context("failed to digest espflash executable")?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
