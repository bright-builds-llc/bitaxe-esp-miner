use crate::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CommandSpec {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
}

impl CommandSpec {
    pub(crate) fn new<I, S>(program: &str, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            program: program.to_owned(),
            args: args
                .into_iter()
                .map(|arg| arg.as_ref().to_owned())
                .collect(),
        }
    }

    pub(crate) fn display(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(self.program.clone());
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }
}

#[derive(Debug)]
pub(crate) struct FlashOutcome {
    pub(crate) manifest: Option<Utf8PathBuf>,
    pub(crate) flash_image: Utf8PathBuf,
    pub(crate) runtime_identity: Option<ExpectedRuntimeAttestationIdentity>,
    pub(crate) command: CommandSpec,
    pub(crate) nvs_seed: Option<NvsSeedOutcome>,
}

pub(crate) struct PreparedFlash {
    pub(crate) outcome: FlashOutcome,
    pub(crate) execution_command: CommandSpec,
    pub(crate) _execution_snapshot: Option<AdmittedExecutionSnapshot>,
}

pub(crate) struct AdmittedFactoryImage {
    pub(crate) manifest: Utf8PathBuf,
    pub(crate) display_path: Utf8PathBuf,
    pub(crate) bytes: Vec<u8>,
    pub(crate) runtime_identity: ExpectedRuntimeAttestationIdentity,
}

pub(crate) enum AdmittedFlashImage {
    DeveloperDryRun { display_path: Utf8PathBuf },
    Factory(AdmittedFactoryImage),
}

impl AdmittedFlashImage {
    pub(crate) fn maybe_manifest(&self) -> Option<&Utf8Path> {
        match self {
            Self::DeveloperDryRun { .. } => None,
            Self::Factory(factory) => Some(&factory.manifest),
        }
    }

    pub(crate) fn display_path(&self) -> &Utf8Path {
        match self {
            Self::DeveloperDryRun { display_path } => display_path,
            Self::Factory(factory) => &factory.display_path,
        }
    }

    pub(crate) fn maybe_factory_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::DeveloperDryRun { .. } => None,
            Self::Factory(factory) => Some(&factory.bytes),
        }
    }

    pub(crate) fn maybe_runtime_identity(&self) -> Option<&ExpectedRuntimeAttestationIdentity> {
        match self {
            Self::DeveloperDryRun { .. } => None,
            Self::Factory(factory) => Some(&factory.runtime_identity),
        }
    }
}

pub(crate) struct AdmittedExecutionSnapshot {
    pub(crate) _file: tempfile::NamedTempFile,
    pub(crate) path: Utf8PathBuf,
}

impl AdmittedExecutionSnapshot {
    pub(crate) fn materialize(bytes: &[u8]) -> Result<Self> {
        let mut file = tempfile::NamedTempFile::new().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_create_failed")
        })?;
        file.as_file_mut().write_all(bytes).map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_write_failed")
        })?;
        file.as_file_mut().flush().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_write_failed")
        })?;
        file.as_file().sync_all().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_sync_failed")
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = file
                .as_file()
                .metadata()
                .map_err(|_| {
                    anyhow::anyhow!(
                        "identity_admission=blocked reason=execution_snapshot_permissions_failed"
                    )
                })?
                .permissions();
            permissions.set_mode(0o600);
            file.as_file().set_permissions(permissions).map_err(|_| {
                anyhow::anyhow!(
                    "identity_admission=blocked reason=execution_snapshot_permissions_failed"
                )
            })?;
        }
        let path = Utf8PathBuf::from_path_buf(file.path().to_path_buf()).map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_path_invalid")
        })?;

        Ok(Self { _file: file, path })
    }

    pub(crate) fn path(&self) -> &Utf8Path {
        &self.path
    }
}

#[derive(Debug)]
pub(crate) struct NvsSeedOutcome {
    pub(crate) image: Utf8PathBuf,
    pub(crate) command: CommandSpec,
    pub(crate) _temp_dir: tempfile::TempDir,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CaptureProcessResult {
    pub(crate) status: CaptureProcessStatus,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum CaptureProcessStatus {
    SpawnFailed,
    ExitedSuccess,
    ExitedFailure(String),
    TimedOut,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CaptureStatus {
    Completed,
    TimedOutAfterTrustedOutput,
    TimedOutPendingPrivateClassification,
    TimedOutAfterPrivateClassification,
    TimedOutWithoutTrustedOutput,
    Failed,
    DryRun,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TrustedCaptureCompletion {
    Completed,
    TimedOut,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum MonitorTrustBasis {
    BootTranscript,
    RuntimeAttestation,
}

impl MonitorTrustBasis {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::BootTranscript => "boot_transcript",
            Self::RuntimeAttestation => "runtime_attestation",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum MonitorCaptureState {
    NotRequested,
    DryRun,
    Trusted {
        completion: TrustedCaptureCompletion,
        basis: MonitorTrustBasis,
    },
    PendingPrivateClassification,
    AdmittedPrivateClassification,
    Untrusted {
        timed_out: bool,
        conclusion: String,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum BootTranscriptStatus {
    Trusted,
    Missing,
    Untrusted,
    NotCaptured,
    NotRequested,
}

impl BootTranscriptStatus {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Missing => "missing",
            Self::Untrusted => "untrusted",
            Self::NotCaptured => "not_captured",
            Self::NotRequested => "not_requested",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RuntimeAttestationEvidenceStatus {
    Observed(RuntimeAttestationStatus),
    NotCaptured,
    NotRequested,
}

impl RuntimeAttestationEvidenceStatus {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Observed(status) => status.label(),
            Self::NotCaptured => "not_captured",
            Self::NotRequested => "not_requested",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct MonitorCaptureOutcome {
    pub(crate) state: MonitorCaptureState,
    pub(crate) capture_timeout_seconds: u64,
    pub(crate) observed_firmware_commit: String,
    pub(crate) observed_reference_commit: String,
    pub(crate) boot_transcript_status: BootTranscriptStatus,
    pub(crate) runtime_attestation_status: RuntimeAttestationEvidenceStatus,
}

impl MonitorCaptureOutcome {
    pub(crate) fn accepted(&self) -> bool {
        matches!(self.state, MonitorCaptureState::Trusted { .. })
    }

    pub(crate) fn ready_for_private_classification(&self) -> bool {
        self.state == MonitorCaptureState::PendingPrivateClassification
    }

    pub(crate) fn projection(&self) -> MonitorCaptureProjection<'_> {
        self.state.projection()
    }
}

impl MonitorCaptureState {
    pub(crate) fn projection(&self) -> MonitorCaptureProjection<'_> {
        let (
            capture_mode,
            capture_status,
            monitor_evidence_status,
            maybe_trust_basis,
            trusted_output,
            conclusion,
        ) = match self {
            MonitorCaptureState::NotRequested => (
                "not_applicable",
                CaptureStatus::DryRun,
                "not_requested",
                None,
                false,
                "not run - no monitor capture requested",
            ),
            MonitorCaptureState::DryRun => (
                "dry_run",
                CaptureStatus::DryRun,
                "not_captured",
                None,
                false,
                "not run - dry-run did not capture hardware evidence",
            ),
            MonitorCaptureState::Trusted {
                completion,
                basis,
            } => {
                let capture_status = match completion {
                    TrustedCaptureCompletion::Completed => CaptureStatus::Completed,
                    TrustedCaptureCompletion::TimedOut => {
                        CaptureStatus::TimedOutAfterTrustedOutput
                    }
                };
                let conclusion = match basis {
                    MonitorTrustBasis::BootTranscript => "passed - original boot transcript captured and trusted; HTTP/static/recovery/OTA/rollback parity not claimed",
                    MonitorTrustBasis::RuntimeAttestation => "passed - exact-package runtime attestation trusted; original boot transcript was not captured",
                };
                (
                    "noninteractive",
                    capture_status,
                    "trusted",
                    Some(*basis),
                    true,
                    conclusion,
                )
            }
            MonitorCaptureState::PendingPrivateClassification => (
                "noninteractive",
                CaptureStatus::TimedOutPendingPrivateClassification,
                "pending_private_classification",
                None,
                false,
                "pending - immutable private monitor input requires authoritative classification",
            ),
            MonitorCaptureState::AdmittedPrivateClassification => (
                "noninteractive",
                CaptureStatus::TimedOutAfterPrivateClassification,
                "pending_private_classification",
                None,
                false,
                "passed - immutable private monitor input was classified before admitted evidence derivation",
            ),
            MonitorCaptureState::Untrusted {
                timed_out,
                conclusion,
            } => (
                "noninteractive",
                if *timed_out {
                    CaptureStatus::TimedOutWithoutTrustedOutput
                } else {
                    CaptureStatus::Failed
                },
                "untrusted",
                None,
                false,
                conclusion.as_str(),
            ),
        };
        MonitorCaptureProjection {
            capture_mode,
            capture_status,
            monitor_evidence_status,
            trust_basis: maybe_trust_basis.map_or("none", MonitorTrustBasis::label),
            trusted_output,
            conclusion,
        }
    }
}

pub(crate) struct MonitorCaptureProjection<'a> {
    pub(crate) capture_mode: &'static str,
    pub(crate) capture_status: CaptureStatus,
    pub(crate) monitor_evidence_status: &'static str,
    pub(crate) trust_basis: &'static str,
    pub(crate) trusted_output: bool,
    pub(crate) conclusion: &'a str,
}

pub(crate) struct EvidenceRecordInput<'a> {
    pub(crate) command_kind: &'a str,
    pub(crate) command: &'a str,
    pub(crate) flash_command: &'a str,
    pub(crate) monitor_command: &'a str,
    pub(crate) log_path: &'a Utf8Path,
    pub(crate) private_log_path: Option<&'a Utf8Path>,
    pub(crate) private_log_sha256: Option<&'a str>,
    pub(crate) admitted_log_sha256: Option<&'a str>,
    pub(crate) capture_outcome: &'a MonitorCaptureOutcome,
}

pub(crate) struct MonitorEvidenceArtifacts<'a> {
    pub(crate) admitted_log: &'a Utf8Path,
    pub(crate) dual_paths: Option<&'a evidence::DualEvidencePaths>,
    pub(crate) private_log_sha256: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PackageManifest {
    pub(crate) schema_version: u32,
    pub(crate) semantic_version: String,
    pub(crate) source_commit: String,
    pub(crate) reference_commit: String,
    pub(crate) app_elf_sha256: String,
    pub(crate) build_identity: PackageBuildIdentity,
    pub(crate) default_flash_image: String,
    pub(crate) artifacts: Vec<PackageArtifact>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PackageBuildIdentity {
    pub(crate) label: String,
    pub(crate) channel: String,
    pub(crate) source_dirty: bool,
    pub(crate) release_tag: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PackageArtifact {
    pub(crate) kind: String,
    pub(crate) path: String,
    pub(crate) sha256: String,
}
