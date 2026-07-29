use crate::*;

mod admission;
mod evidence;
mod markers;

use admission::{admit_campaign, prepare_campaign_nvs_seed};
use evidence::{finish_campaign_attempt, preflight_campaign_evidence};
use markers::{assess_campaign_markers, parse_campaign_markers, CampaignStatusMarker};

const CAMPAIGN_MARKER_PREFIX: &str = "mining_campaign_status=";
const CAMPAIGN_MARKER_SCHEMA: &str = "mining-campaign-status-v1";
const CAMPAIGN_RESULT_SCHEMA: &str = "mining-campaign-result-v1";
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

#[derive(Default)]
struct CampaignAttempt {
    package_admitted: bool,
    runtime_identity_trusted: bool,
    usb_cleanup_complete: bool,
    markers: Vec<CampaignStatusMarker>,
}

pub(crate) use markers::campaign_serial_should_stop;

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

    let serial_bytes = environment
        .receive_campaign_until(admission, admission.duration_seconds)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::ObservationFailed))?;
    let serial_text = std::str::from_utf8(&serial_bytes)
        .map_err(|_| CampaignFailure::new(CampaignTerminalCategory::MarkerInvalid))?;
    attempt.markers = parse_campaign_markers(serial_text, false)?;
    let terminal = assess_campaign_markers(&attempt.markers, admission)?;

    attempt.runtime_identity_trusted =
        prepared
            .outcome
            .runtime_identity
            .as_ref()
            .is_some_and(|identity| {
                classify_runtime_boot_attestations(serial_text, identity)
                    == RuntimeAttestationStatus::Trusted
            });
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
