//! Lazy production-mining reads and one-shot campaign admission.

use std::ffi::CString;

use bitaxe_config::{ultra_205_defaults, NVS_NAMESPACE};
use bitaxe_stratum::v1::production_session::{
    LivePoolCredentials, LiveRuntimeConfig, MiningCampaignDuration, MiningCampaignLease,
    MiningCampaignLeaseId, MiningCampaignStopCondition, MiningHardwareProfilePreset,
    ProductionPoolConfiguration, ProductionPoolEndpoint, ProductionPoolSet,
};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::sys;

use super::SETTINGS_TRANSACTION_LOCK;

const MAX_POOL_STRING_BYTES: usize = 4_000;
const LIVE_SHARE_DURATION_MS: u64 = 600_000;
const DEFAULT_SOAK_DURATION_MS: u64 = 600_000;
const JOB_TRANSITION_DURATION_MS: u64 = 1_800_000;
const COMMAND_EFFECTS_DURATION_MS: u64 = 600_000;
const STRATUM_V2_DURATION_MS: u64 = 180_000;
const CAMPAIGN_KEYS: [&str; 4] = ["campstage", "campprofile", "camplease", "campdurms"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MiningCampaignStage {
    Observation,
    LiveShare,
    Soak,
    JobTransition,
    CommandEffects,
    StratumV2,
}

impl MiningCampaignStage {
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::LiveShare => "live-share",
            Self::Soak => "soak",
            Self::JobTransition => "job-transition",
            Self::CommandEffects => "command-effects",
            Self::StratumV2 => "stratum-v2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MiningCampaignAdmission {
    pub(crate) stage: MiningCampaignStage,
    pub(crate) maybe_profile: Option<MiningHardwareProfilePreset>,
    pub(crate) maybe_lease: Option<MiningCampaignLease>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProductionSettingsReadError {
    category: &'static str,
}

impl ProductionSettingsReadError {
    pub(super) const fn new(category: &'static str) -> Self {
        Self { category }
    }

    #[must_use]
    pub(crate) const fn category(self) -> &'static str {
        self.category
    }
}

/// Loads and consumes one device-local live campaign admission.
///
/// Observation metadata is non-authorizing and remains read-only. A complete
/// live tuple is erased and committed before it is returned so reboot cannot
/// replay it.
pub(crate) fn load_production_campaign_admission(
) -> Result<Option<MiningCampaignAdmission>, ProductionSettingsReadError> {
    let _transaction_guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| ProductionSettingsReadError::new("transaction_lock"))?;
    let partition = super::default_nvs_partition()
        .map_err(|_| ProductionSettingsReadError::new("nvs_partition"))?;
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, false)
        .map_err(|_| ProductionSettingsReadError::new("nvs_open"))?;
    let Some(stage) = read_optional_string_bounded(&nvs, "campstage")?
        .and_then(|stage| parse_campaign_stage(&stage))
    else {
        return Ok(None);
    };
    if stage == MiningCampaignStage::Observation {
        return Ok(Some(MiningCampaignAdmission {
            stage,
            maybe_profile: None,
            maybe_lease: None,
        }));
    }
    if read_optional_u16(&nvs, "mineonboot")?.unwrap_or(1) != 0 {
        return Err(ProductionSettingsReadError::new("mineonboot_not_paused"));
    }

    let profile = read_optional_string_bounded(&nvs, "campprofile")?
        .and_then(|profile| parse_campaign_profile(&profile))
        .ok_or_else(|| ProductionSettingsReadError::new("campaign_profile"))?;
    let lease_id = read_optional_u64(&nvs, "camplease")?
        .ok_or_else(|| ProductionSettingsReadError::new("campaign_lease"))?;
    let duration_ms = read_optional_u64(&nvs, "campdurms")?
        .ok_or_else(|| ProductionSettingsReadError::new("campaign_duration"))?;
    if !campaign_contract_valid(stage, profile, duration_ms) {
        return Err(ProductionSettingsReadError::new("campaign_contract"));
    }
    let lease_id = MiningCampaignLeaseId::new(lease_id)
        .map_err(|_| ProductionSettingsReadError::new("campaign_lease"))?;
    let duration = MiningCampaignDuration::new(duration_ms)
        .map_err(|_| ProductionSettingsReadError::new("campaign_duration"))?;
    let Some(stop_condition) = maybe_campaign_stop_condition(stage, duration) else {
        return Err(ProductionSettingsReadError::new("campaign_stop_condition"));
    };
    drop(nvs);

    let writable = EspNvs::new(partition, NVS_NAMESPACE, true)
        .map_err(|_| ProductionSettingsReadError::new("nvs_open_write"))?;
    erase_campaign_keys(&writable)?;
    Ok(Some(MiningCampaignAdmission {
        stage,
        maybe_profile: Some(profile),
        maybe_lease: Some(MiningCampaignLease::new(
            lease_id,
            profile.profile(),
            stop_condition,
        )),
    }))
}

const fn maybe_campaign_stop_condition(
    stage: MiningCampaignStage,
    duration: MiningCampaignDuration,
) -> Option<MiningCampaignStopCondition> {
    let condition = match stage {
        MiningCampaignStage::LiveShare | MiningCampaignStage::Soak => {
            MiningCampaignStopCondition::ActiveDuration { duration }
        }
        MiningCampaignStage::JobTransition => {
            MiningCampaignStopCondition::ActiveDuration { duration }
        }
        MiningCampaignStage::CommandEffects => MiningCampaignStopCondition::ResumableActiveEpoch {
            activation_timeout: duration,
            duration,
        },
        MiningCampaignStage::StratumV2 => {
            MiningCampaignStopCondition::FirstSubmitResponse { timeout: duration }
        }
        MiningCampaignStage::Observation => return None,
    };
    Some(condition)
}

const fn campaign_contract_valid(
    stage: MiningCampaignStage,
    profile: MiningHardwareProfilePreset,
    duration_ms: u64,
) -> bool {
    match stage {
        MiningCampaignStage::Observation => false,
        MiningCampaignStage::LiveShare => {
            matches!(profile, MiningHardwareProfilePreset::Conservative)
                && duration_ms == LIVE_SHARE_DURATION_MS
        }
        MiningCampaignStage::Soak => {
            matches!(profile, MiningHardwareProfilePreset::UpstreamDefault)
                && duration_ms == DEFAULT_SOAK_DURATION_MS
        }
        MiningCampaignStage::JobTransition => {
            matches!(profile, MiningHardwareProfilePreset::Conservative)
                && duration_ms == JOB_TRANSITION_DURATION_MS
        }
        MiningCampaignStage::CommandEffects => {
            matches!(profile, MiningHardwareProfilePreset::Conservative)
                && duration_ms == COMMAND_EFFECTS_DURATION_MS
        }
        MiningCampaignStage::StratumV2 => {
            matches!(profile, MiningHardwareProfilePreset::Conservative)
                && duration_ms == STRATUM_V2_DURATION_MS
        }
    }
}

/// Lazily reads pool secrets only in response to the session's typed effect.
pub(crate) fn read_production_pool_set(
) -> Result<Option<ProductionPoolSet>, ProductionSettingsReadError> {
    let _transaction_guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| ProductionSettingsReadError::new("transaction_lock"))?;
    let partition = super::default_nvs_partition()
        .map_err(|_| ProductionSettingsReadError::new("nvs_partition"))?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false)
        .map_err(|_| ProductionSettingsReadError::new("nvs_open"))?;
    let defaults = ultra_205_defaults();
    let model = defaults.device_model();
    let version = defaults.board_version();
    let primary = read_pool_configuration(&nvs, "", model, version)?;
    let fallback = read_pool_configuration(&nvs, "fb", model, version)?;
    if primary.is_none() && fallback.is_none() {
        return Ok(None);
    }
    Ok(Some(ProductionPoolSet {
        primary,
        fallback,
        prefer_fallback: read_optional_u16(&nvs, "usefbstartum")?.unwrap_or(0) == 1,
    }))
}

fn read_pool_configuration(
    nvs: &EspNvs<NvsDefault>,
    prefix: &str,
    model: &str,
    version: &str,
) -> Result<Option<ProductionPoolConfiguration>, ProductionSettingsReadError> {
    let protocol_key = format!("{prefix}stratumprot");
    let protocol =
        read_optional_string_bounded(nvs, &protocol_key)?.unwrap_or_else(|| "SV1".to_owned());
    let tls_key = format!("{prefix}stratumtls");
    let tls = read_optional_u16(nvs, &tls_key)?.unwrap_or(0);
    if protocol != "SV1" || tls != 0 {
        return Ok(None);
    }

    let host_key = format!("{prefix}stratumurl");
    let port_key = format!("{prefix}stratumport");
    let user_key = format!("{prefix}stratumuser");
    let password_key = format!("{prefix}stratumpass");
    let Some(host) = read_optional_string_bounded(nvs, &host_key)? else {
        return Ok(None);
    };
    let Some(port) = read_optional_u16(nvs, &port_key)? else {
        return Ok(None);
    };
    let Some(username) = read_optional_string_bounded(nvs, &user_key)? else {
        return Ok(None);
    };
    let Some(password) = read_optional_string_bounded(nvs, &password_key)? else {
        return Ok(None);
    };
    if host.trim().is_empty() || port == 0 || username.is_empty() {
        return Ok(None);
    }

    Ok(Some(ProductionPoolConfiguration {
        endpoint: ProductionPoolEndpoint { host, port },
        runtime: LiveRuntimeConfig {
            model: model.to_owned(),
            version: version.to_owned(),
            credentials: LivePoolCredentials { username, password },
        },
    }))
}

fn parse_campaign_stage(stage: &str) -> Option<MiningCampaignStage> {
    match stage {
        "observation" => Some(MiningCampaignStage::Observation),
        "live-share" => Some(MiningCampaignStage::LiveShare),
        "soak" => Some(MiningCampaignStage::Soak),
        "job-transition" => Some(MiningCampaignStage::JobTransition),
        "command-effects" => Some(MiningCampaignStage::CommandEffects),
        "stratum-v2" => Some(MiningCampaignStage::StratumV2),
        _ => None,
    }
}

fn parse_campaign_profile(profile: &str) -> Option<MiningHardwareProfilePreset> {
    match profile {
        "conservative" => Some(MiningHardwareProfilePreset::Conservative),
        "upstream-default" => Some(MiningHardwareProfilePreset::UpstreamDefault),
        _ => None,
    }
}

fn read_optional_string_bounded(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<Option<String>, ProductionSettingsReadError> {
    let Some(len) = nvs
        .str_len(key)
        .map_err(|_| ProductionSettingsReadError::new("nvs_string_length"))?
    else {
        return Ok(None);
    };
    if len == 0 || len > MAX_POOL_STRING_BYTES {
        return Err(ProductionSettingsReadError::new("nvs_string_size"));
    }
    let mut buffer = vec![0; len];
    nvs.get_str(key, &mut buffer)
        .map_err(|_| ProductionSettingsReadError::new("nvs_string_read"))
        .map(|maybe_value| maybe_value.map(str::to_owned))
}

fn read_optional_u16(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<Option<u16>, ProductionSettingsReadError> {
    nvs.get_u16(key)
        .map_err(|_| ProductionSettingsReadError::new("nvs_u16_read"))
}

fn read_optional_u64(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<Option<u64>, ProductionSettingsReadError> {
    nvs.get_u64(key)
        .map_err(|_| ProductionSettingsReadError::new("nvs_u64_read"))
}

fn erase_campaign_keys(nvs: &EspNvs<NvsDefault>) -> Result<(), ProductionSettingsReadError> {
    for key in CAMPAIGN_KEYS {
        let key =
            CString::new(key).map_err(|_| ProductionSettingsReadError::new("campaign_key"))?;
        let result = unsafe { sys::nvs_erase_key(nvs.handle(), key.as_ptr()) };
        if result != sys::ESP_OK && result != sys::ESP_ERR_NVS_NOT_FOUND {
            return Err(ProductionSettingsReadError::new("campaign_clear"));
        }
    }
    let result = unsafe { sys::nvs_commit(nvs.handle()) };
    if result != sys::ESP_OK {
        return Err(ProductionSettingsReadError::new("campaign_commit"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_transition_requires_conservative_profile_and_exact_duration() {
        // Arrange
        let stage = MiningCampaignStage::JobTransition;

        // Act
        let exact = campaign_contract_valid(
            stage,
            MiningHardwareProfilePreset::Conservative,
            JOB_TRANSITION_DURATION_MS,
        );
        let short = campaign_contract_valid(
            stage,
            MiningHardwareProfilePreset::Conservative,
            JOB_TRANSITION_DURATION_MS - 1,
        );
        let wrong_profile = campaign_contract_valid(
            stage,
            MiningHardwareProfilePreset::UpstreamDefault,
            JOB_TRANSITION_DURATION_MS,
        );

        // Assert
        assert!(exact);
        assert!(!short);
        assert!(!wrong_profile);
    }

    #[test]
    fn command_effects_maps_one_duration_to_two_typed_phases() {
        // Arrange
        let duration = MiningCampaignDuration::new(COMMAND_EFFECTS_DURATION_MS)
            .expect("command effects duration");

        // Act
        let maybe_condition =
            maybe_campaign_stop_condition(MiningCampaignStage::CommandEffects, duration);

        // Assert
        assert_eq!(
            maybe_condition,
            Some(MiningCampaignStopCondition::ResumableActiveEpoch {
                activation_timeout: duration,
                duration,
            })
        );
    }

    #[test]
    fn live_share_keeps_running_after_submit_until_the_evidence_window_completes() {
        // Arrange
        let duration =
            MiningCampaignDuration::new(MINING_DURATION_MS).expect("live-share duration");

        // Act
        let maybe_condition =
            maybe_campaign_stop_condition(MiningCampaignStage::LiveShare, duration);

        // Assert
        assert_eq!(
            maybe_condition,
            Some(MiningCampaignStopCondition::ActiveDuration { duration })
        );
    }
}
