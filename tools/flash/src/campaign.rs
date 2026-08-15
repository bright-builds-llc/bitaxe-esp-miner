use crate::*;

mod admission;
mod evidence;
mod markers;
pub(crate) mod network;
mod serial;

use admission::{admit_campaign, prepare_campaign_nvs_seed};
use evidence::{finish_campaign_attempt, preflight_campaign_evidence};
use markers::CampaignMarkerAggregate;
use serial::{CampaignSerialDiagnostics, CampaignSerialOutcomeDetail};

const CAMPAIGN_MARKER_PREFIX: &str = "mining_campaign_status=";
const CAMPAIGN_MARKER_SCHEMA: &str = "mining-campaign-status-v12";
const CAMPAIGN_PREPARATION_PREFIX: &str = "mining_campaign_preparation=";
const CAMPAIGN_PREPARATION_SCHEMA: &str = "mining-campaign-preparation-v1";
const CAMPAIGN_RESULT_SCHEMA: &str = "mining-campaign-result-v8";
const CAMPAIGN_FLASH_DIAGNOSTICS_SCHEMA: &str = "mining-campaign-flash-diagnostics-v1";
const CAMPAIGN_OBSERVATIONS_SCHEMA: &str = "mining-campaign-observations-v4";
const CAMPAIGN_MINING_DIAGNOSTICS_SCHEMA: &str = "mining-campaign-asic-diagnostics-v1";
const OBSERVATION_DURATION_SECONDS: u64 = 360;
const MINING_DURATION_SECONDS: u64 = 600;
const JOB_TRANSITION_DURATION_SECONDS: u64 = 1_800;
const COMMAND_EFFECTS_DURATION_SECONDS: u64 = 600;
const MINING_TERMINAL_GRACE_SECONDS: u64 = 180;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CampaignCaptureLimit {
    Bounded(u64),
    OperatorGated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CampaignTerminalCategory {
    ObservationComplete,
    SubmitResponseObserved,
    SoakDurationComplete,
    JobTransitionComplete,
    JobTransitionNotObserved,
    CommandEffectsComplete,
    CommandRequestFailed,
    ResumeIntentUnconfirmed,
    ResumeReactivationTimedOut,
    OperatorCheckpointInvalid,
    #[cfg(test)]
    OperatorCheckpointDeclined,
    AdmissionFailed,
    PackageAdmissionFailed,
    DeviceAdmissionFailed,
    CredentialAdmissionFailed,
    FlashFailed,
    NvsSeedFailed,
    ConcurrentRepoSession,
    ForeignHolder,
    TransportAbsent,
    IdentityDrift,
    BootloaderConnectFailed,
    FlashFailedBeforeTransfer,
    FlashFailedAfterTransfer,
    RecoveryNotObserved,
    RepeatedBoundary,
    ObservationFailed,
    MarkerMissing,
    MarkerInvalid,
    StageMismatch,
    LeaseMismatch,
    ProfileMismatch,
    SafetyStale,
    MineOnBootEnabled,
    PoolReadDuringObservation,
    ActuationDuringObservation,
    ObservationContractIncomplete,
    HardwarePreparationFailed,
    CampaignActivationTimedOut,
    PoolConfigurationMissing,
    SubmitResponseMissing,
    SoakDurationShort,
    JobTransitionProtocolInconsistent,
    JobTransitionEvidenceIncomplete,
    RejectedShareObserved,
    StaleGenerationSubmissionObserved,
    ReconnectObserved,
    MarkerContinuityFailed,
    SafeStopUnconfirmed,
    NetworkTargetUnavailable,
    HttpWindowIncomplete,
    WebsocketWindowIncomplete,
    NetworkCorrelationFailed,
    WatchdogUnresponsive,
    WorkRenewalMissing,
    PoolPersistenceUnconfirmed,
    TerminalStateUnconfirmed,
    RuntimeIdentityUntrusted,
    UsbCleanupFailed,
    EvidenceSealFailed,
}

impl CampaignTerminalCategory {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationComplete => "observation_complete",
            Self::SubmitResponseObserved => "submit_response_observed",
            Self::SoakDurationComplete => "soak_duration_complete",
            Self::JobTransitionComplete => "job_transition_complete",
            Self::JobTransitionNotObserved => "job_transition_not_observed",
            Self::CommandEffectsComplete => "command_effects_complete",
            Self::CommandRequestFailed => "command_request_failed",
            Self::ResumeIntentUnconfirmed => "resume_intent_unconfirmed",
            Self::ResumeReactivationTimedOut => "resume_reactivation_timed_out",
            Self::OperatorCheckpointInvalid => "operator_checkpoint_invalid",
            #[cfg(test)]
            Self::OperatorCheckpointDeclined => "operator_checkpoint_declined",
            Self::AdmissionFailed => "admission_failed",
            Self::PackageAdmissionFailed => "package_admission_failed",
            Self::DeviceAdmissionFailed => "device_admission_failed",
            Self::CredentialAdmissionFailed => "credential_admission_failed",
            Self::FlashFailed => "flash_failed",
            Self::NvsSeedFailed => "nvs_seed_failed",
            Self::ConcurrentRepoSession => "concurrent_repo_session",
            Self::ForeignHolder => "foreign_holder",
            Self::TransportAbsent => "transport_absent",
            Self::IdentityDrift => "identity_drift",
            Self::BootloaderConnectFailed => "bootloader_connect_failed",
            Self::FlashFailedBeforeTransfer => "flash_failed_before_transfer",
            Self::FlashFailedAfterTransfer => "flash_failed_after_transfer",
            Self::RecoveryNotObserved => "recovery_not_observed",
            Self::RepeatedBoundary => "repeated_boundary",
            Self::ObservationFailed => "observation_failed",
            Self::MarkerMissing => "marker_missing",
            Self::MarkerInvalid => "marker_invalid",
            Self::StageMismatch => "stage_mismatch",
            Self::LeaseMismatch => "lease_mismatch",
            Self::ProfileMismatch => "profile_mismatch",
            Self::SafetyStale => "safety_stale",
            Self::MineOnBootEnabled => "mineonboot_enabled",
            Self::PoolReadDuringObservation => "pool_read_during_observation",
            Self::ActuationDuringObservation => "actuation_during_observation",
            Self::ObservationContractIncomplete => "observation_contract_incomplete",
            Self::HardwarePreparationFailed => "hardware_preparation_failed",
            Self::CampaignActivationTimedOut => "campaign_activation_timed_out",
            Self::PoolConfigurationMissing => "pool_configuration_missing",
            Self::SubmitResponseMissing => "submit_response_missing",
            Self::SoakDurationShort => "soak_duration_short",
            Self::JobTransitionProtocolInconsistent => "job_transition_protocol_inconsistent",
            Self::JobTransitionEvidenceIncomplete => "job_transition_evidence_incomplete",
            Self::RejectedShareObserved => "rejected_share_observed",
            Self::StaleGenerationSubmissionObserved => "stale_generation_submission_observed",
            Self::ReconnectObserved => "reconnect_observed",
            Self::MarkerContinuityFailed => "marker_continuity_failed",
            Self::SafeStopUnconfirmed => "safe_stop_unconfirmed",
            Self::NetworkTargetUnavailable => "network_target_unavailable",
            Self::HttpWindowIncomplete => "http_window_incomplete",
            Self::WebsocketWindowIncomplete => "websocket_window_incomplete",
            Self::NetworkCorrelationFailed => "network_correlation_failed",
            Self::WatchdogUnresponsive => "watchdog_unresponsive",
            Self::WorkRenewalMissing => "work_renewal_missing",
            Self::PoolPersistenceUnconfirmed => "pool_persistence_unconfirmed",
            Self::TerminalStateUnconfirmed => "terminal_state_unconfirmed",
            Self::RuntimeIdentityUntrusted => "runtime_identity_untrusted",
            Self::UsbCleanupFailed => "usb_cleanup_failed",
            Self::EvidenceSealFailed => "evidence_seal_failed",
        }
    }
}

#[derive(Debug)]
pub(crate) struct CampaignFailure {
    pub(crate) category: CampaignTerminalCategory,
}

impl CampaignFailure {
    pub(crate) const fn new(category: CampaignTerminalCategory) -> Self {
        Self { category }
    }
}

impl fmt::Display for CampaignFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "mining_campaign=failed category={}",
            self.category.as_str()
        )
    }
}

impl std::error::Error for CampaignFailure {}

#[derive(Clone, Copy)]
pub(crate) struct CampaignAdmission {
    pub(crate) stage: MiningCampaignStage,
    pub(crate) maybe_profile: Option<MiningCampaignProfile>,
    pub(crate) duration_seconds: u64,
    pub(crate) maybe_lease_id: Option<u64>,
}

struct CampaignAttempt {
    package_admitted: bool,
    runtime_identity_trusted: bool,
    maybe_runtime_attestation_status: Option<RuntimeAttestationStatus>,
    usb_cleanup_complete: bool,
    marker_aggregate: CampaignMarkerAggregate,
    serial_diagnostics: CampaignSerialDiagnostics,
    serial_outcome_detail: CampaignSerialOutcomeDetail,
    network_evidence: network::CampaignNetworkEvidence,
    factory_flash_diagnostic: Option<UsbCommandDiagnostic>,
    nvs_flash_diagnostic: Option<UsbCommandDiagnostic>,
}

impl Default for CampaignAttempt {
    fn default() -> Self {
        Self {
            package_admitted: false,
            runtime_identity_trusted: false,
            maybe_runtime_attestation_status: None,
            usb_cleanup_complete: false,
            marker_aggregate: CampaignMarkerAggregate::default(),
            serial_diagnostics: CampaignSerialDiagnostics::not_observed(),
            serial_outcome_detail: CampaignSerialOutcomeDetail::Clean,
            network_evidence: network::CampaignNetworkEvidence::not_required(),
            factory_flash_diagnostic: None,
            nvs_flash_diagnostic: None,
        }
    }
}

#[cfg(test)]
pub(crate) use serial::{analyze_campaign_serial_bytes, campaign_serial_should_stop};
pub(crate) use serial::{CampaignSerialAnalyzer, CampaignSerialCapture};

pub(crate) fn run_mining_campaign(
    command: &MiningCampaignCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let evidence_root = environment.workspace_path(&command.evidence_dir);
    environment
        .approve_private_evidence_root(&evidence_root)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::AdmissionFailed))?;
    let paths = preflight_campaign_evidence(&evidence_root)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::AdmissionFailed))?;

    let mut attempt = CampaignAttempt::default();
    let admission = match admit_campaign(command, environment) {
        Ok(admission) => admission,
        Err(failure) => {
            return finish_campaign_attempt(command, None, &paths, &attempt, Err(failure));
        }
    };

    let operation_result = execute_campaign(
        command,
        admission,
        &evidence_root,
        &mut attempt,
        environment,
    );
    let cleanup_result = environment.finish_usb_session();
    if cleanup_result.is_ok() {
        attempt.usb_cleanup_complete = true;
    }
    let result = match (operation_result, cleanup_result) {
        (Ok(category), Ok(())) => Ok(category),
        (Err(failure), _) => Err(failure),
        (Ok(_), Err(_)) => Err(CampaignFailure::new(
            CampaignTerminalCategory::UsbCleanupFailed,
        )),
    };
    finish_campaign_attempt(command, Some(admission), &paths, &attempt, result)
}

pub(crate) fn run_signal_identify(
    command: &SignalIdentifyCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let evidence_root = environment.workspace_path(&command.evidence_dir);
    environment.approve_private_evidence_root(&evidence_root)?;
    let metadata = fs::symlink_metadata(evidence_root.as_std_path())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("identify_checkpoint=blocked reason=attempt_root_invalid");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o700 {
        bail!("identify_checkpoint=blocked reason=attempt_root_not_private");
    }
    network::respond_identify_checkpoint(&evidence_root, command.checkpoint, command.outcome)?;
    emit_line("identify_checkpoint", command.checkpoint.as_str())?;
    emit_line("identify_outcome", command.outcome.as_str())
}

fn execute_campaign(
    command: &MiningCampaignCommand,
    admission: CampaignAdmission,
    evidence_root: &Utf8Path,
    attempt: &mut CampaignAttempt,
    environment: &impl FlashEnvironment,
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    let flash_command = FlashCommand {
        common: CommonArgs {
            board: command.board,
            port: command.port.clone(),
            dry_run: false,
            redact_evidence: true,
            evidence_mode: None,
            evidence_dir: None,
        },
        image: None,
        manifest: command.manifest.clone(),
        wifi_credentials: None,
    };
    let prepared = prepare_flash(&flash_command, environment)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::PackageAdmissionFailed))?;
    attempt.package_admitted = true;
    let port = maybe_command_port(&prepared.execution_command)
        .ok_or_else(|| CampaignFailure::new(CampaignTerminalCategory::DeviceAdmissionFailed))?;
    environment
        .begin_usb_session(UsbOperation::MiningCampaign, &port)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::DeviceAdmissionFailed))?;
    environment
        .execute(&campaign_board_info_command(&port))
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::DeviceAdmissionFailed))?;

    let nvs_seed = prepare_campaign_nvs_seed(command, admission, &port, environment)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::CredentialAdmissionFailed))?;
    let factory_result = environment.execute(&prepared.execution_command);
    attempt.factory_flash_diagnostic = environment.last_usb_command_diagnostic();
    if factory_result.is_err() {
        return Err(campaign_flash_failure(
            attempt.factory_flash_diagnostic.as_ref(),
            CampaignTerminalCategory::FlashFailed,
        ));
    }
    let nvs_result = environment.execute(&nvs_seed.command);
    attempt.nvs_flash_diagnostic = environment.last_usb_command_diagnostic();
    if nvs_result.is_err() {
        return Err(campaign_flash_failure(
            attempt.nvs_flash_diagnostic.as_ref(),
            CampaignTerminalCategory::NvsSeedFailed,
        ));
    }

    let expected_runtime =
        prepared.outcome.runtime_identity.clone().ok_or_else(|| {
            CampaignFailure::new(CampaignTerminalCategory::PackageAdmissionFailed)
        })?;
    let capture = environment
        .receive_campaign_until(
            admission,
            expected_runtime.clone(),
            evidence_root,
            campaign_capture_limit(admission),
        )
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::ObservationFailed))?;
    attempt.maybe_runtime_attestation_status =
        Some(capture.serial.runtime_attestation_status(&expected_runtime));
    attempt.runtime_identity_trusted =
        attempt.maybe_runtime_attestation_status == Some(RuntimeAttestationStatus::Trusted);
    attempt.serial_diagnostics = capture.serial.diagnostics;
    attempt.serial_outcome_detail = capture.serial.outcome_detail;
    attempt.marker_aggregate = capture.serial.aggregate;
    attempt.network_evidence = capture.network;
    if let Some(category) = capture.serial.maybe_failure {
        return Err(CampaignFailure::new(category));
    }
    if let Some(category) = attempt.network_evidence.maybe_failure {
        return Err(CampaignFailure::new(category));
    }
    let terminal = attempt.marker_aggregate.assess(admission)?;
    if !attempt.runtime_identity_trusted {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::RuntimeIdentityUntrusted,
        ));
    }
    Ok(terminal)
}

pub(crate) fn campaign_flash_failure(
    maybe_diagnostic: Option<&UsbCommandDiagnostic>,
    fallback: CampaignTerminalCategory,
) -> CampaignFailure {
    let category =
        maybe_diagnostic.map_or(fallback, |diagnostic| match diagnostic.terminal_category {
            UsbTerminalCategory::Ready => fallback,
            UsbTerminalCategory::ConcurrentRepoSession => {
                CampaignTerminalCategory::ConcurrentRepoSession
            }
            UsbTerminalCategory::ForeignHolder => CampaignTerminalCategory::ForeignHolder,
            UsbTerminalCategory::TransportAbsent => CampaignTerminalCategory::TransportAbsent,
            UsbTerminalCategory::IdentityDrift => CampaignTerminalCategory::IdentityDrift,
            UsbTerminalCategory::BootloaderConnectFailed => {
                CampaignTerminalCategory::BootloaderConnectFailed
            }
            UsbTerminalCategory::FlashFailedBeforeTransfer => {
                CampaignTerminalCategory::FlashFailedBeforeTransfer
            }
            UsbTerminalCategory::FlashFailedAfterTransfer => {
                CampaignTerminalCategory::FlashFailedAfterTransfer
            }
            UsbTerminalCategory::MonitorFailed => fallback,
            UsbTerminalCategory::CleanupFailed => CampaignTerminalCategory::UsbCleanupFailed,
            UsbTerminalCategory::RecoveryNotObserved => {
                CampaignTerminalCategory::RecoveryNotObserved
            }
            UsbTerminalCategory::RepeatedBoundary => CampaignTerminalCategory::RepeatedBoundary,
        });
    CampaignFailure::new(category)
}

fn campaign_capture_limit(admission: CampaignAdmission) -> CampaignCaptureLimit {
    match admission.stage {
        MiningCampaignStage::Observation => {
            CampaignCaptureLimit::Bounded(admission.duration_seconds)
        }
        MiningCampaignStage::LiveShare
        | MiningCampaignStage::Soak
        | MiningCampaignStage::JobTransition => CampaignCaptureLimit::Bounded(
            admission
                .duration_seconds
                .saturating_add(MINING_TERMINAL_GRACE_SECONDS),
        ),
        // This transaction contains safe, persisted checkpoints at which the
        // owner may be absent for hours or overnight. Its automated phases
        // enforce their own deadlines; the enclosing capture must not guess a
        // human-response duration.
        MiningCampaignStage::CommandEffects => CampaignCaptureLimit::OperatorGated,
    }
}

fn campaign_board_info_command(port: &str) -> CommandSpec {
    CommandSpec::new(
        "espflash",
        [
            "board-info",
            "--chip",
            "esp32s3",
            "--port",
            port,
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "hard-reset",
        ],
    )
}
