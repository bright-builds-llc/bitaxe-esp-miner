use crate::*;

mod admission;
mod evidence;
mod markers;
mod serial;

use admission::{admit_campaign, prepare_campaign_nvs_seed};
use evidence::{finish_campaign_attempt, preflight_campaign_evidence};
use markers::{assess_campaign_markers, CampaignStatusMarker};
use serial::{CampaignSerialDiagnostics, CampaignSerialOutcomeDetail};

const CAMPAIGN_MARKER_PREFIX: &str = "mining_campaign_status=";
const CAMPAIGN_MARKER_SCHEMA: &str = "mining-campaign-status-v1";
const CAMPAIGN_RESULT_SCHEMA: &str = "mining-campaign-result-v2";
const CAMPAIGN_OBSERVATIONS_SCHEMA: &str = "mining-campaign-observations-v1";
const OBSERVATION_DURATION_SECONDS: u64 = 360;
const MINING_DURATION_SECONDS: u64 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CampaignTerminalCategory {
    ObservationComplete,
    SubmitResponseObserved,
    SoakDurationComplete,
    AdmissionFailed,
    PackageAdmissionFailed,
    DeviceAdmissionFailed,
    CredentialAdmissionFailed,
    FlashFailed,
    NvsSeedFailed,
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
    PoolConfigurationMissing,
    SubmitResponseMissing,
    SoakDurationShort,
    SafeStopUnconfirmed,
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
            Self::AdmissionFailed => "admission_failed",
            Self::PackageAdmissionFailed => "package_admission_failed",
            Self::DeviceAdmissionFailed => "device_admission_failed",
            Self::CredentialAdmissionFailed => "credential_admission_failed",
            Self::FlashFailed => "flash_failed",
            Self::NvsSeedFailed => "nvs_seed_failed",
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
            Self::PoolConfigurationMissing => "pool_configuration_missing",
            Self::SubmitResponseMissing => "submit_response_missing",
            Self::SoakDurationShort => "soak_duration_short",
            Self::SafeStopUnconfirmed => "safe_stop_unconfirmed",
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
    markers: Vec<CampaignStatusMarker>,
    serial_diagnostics: CampaignSerialDiagnostics,
    serial_outcome_detail: CampaignSerialOutcomeDetail,
}

impl Default for CampaignAttempt {
    fn default() -> Self {
        Self {
            package_admitted: false,
            runtime_identity_trusted: false,
            maybe_runtime_attestation_status: None,
            usb_cleanup_complete: false,
            markers: Vec::new(),
            serial_diagnostics: CampaignSerialDiagnostics::not_observed(),
            serial_outcome_detail: CampaignSerialOutcomeDetail::Clean,
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

    let operation_result = execute_campaign(command, admission, &mut attempt, environment);
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

fn execute_campaign(
    command: &MiningCampaignCommand,
    admission: CampaignAdmission,
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
    environment
        .execute(&prepared.execution_command)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::FlashFailed))?;
    environment
        .execute(&nvs_seed.command)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::NvsSeedFailed))?;

    let capture = environment
        .receive_campaign_until(admission, admission.duration_seconds)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::ObservationFailed))?;
    attempt.maybe_runtime_attestation_status = prepared
        .outcome
        .runtime_identity
        .as_ref()
        .map(|identity| capture.runtime_attestation_status(identity));
    attempt.runtime_identity_trusted =
        attempt.maybe_runtime_attestation_status == Some(RuntimeAttestationStatus::Trusted);
    attempt.serial_diagnostics = capture.diagnostics;
    attempt.serial_outcome_detail = capture.outcome_detail;
    attempt.markers = capture.markers;
    if let Some(category) = capture.maybe_failure {
        return Err(CampaignFailure::new(category));
    }
    let terminal = assess_campaign_markers(&attempt.markers, admission)?;
    if !attempt.runtime_identity_trusted {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::RuntimeIdentityUntrusted,
        ));
    }
    Ok(terminal)
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
