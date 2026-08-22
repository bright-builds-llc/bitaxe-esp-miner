use std::fmt;

use bitaxe_config::{ultra_205_defaults, NVS_NAMESPACE};
use bitaxe_stratum::v2::authority::parse_authority_public_key;
use bitaxe_stratum::v2::messages::ChannelKind;
use bitaxe_stratum::v2::session::SessionConfig;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};

use super::production::ProductionSettingsReadError;
use super::SETTINGS_TRANSACTION_LOCK;

const MAX_POOL_STRING_BYTES: usize = 4_000;

#[derive(Clone, PartialEq)]
pub(crate) struct V2PoolSettings {
    pub(crate) session: SessionConfig,
    pub(crate) maybe_authority_public_key: Option<[u8; 32]>,
}

impl fmt::Debug for V2PoolSettings {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("V2PoolSettings")
            .field("session", &"redacted")
            .field(
                "authority",
                &self.maybe_authority_public_key.map(|_| "configured"),
            )
            .finish()
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct V2PoolSet {
    pub(crate) primary: Option<V2PoolSettings>,
    pub(crate) fallback: Option<V2PoolSettings>,
    pub(crate) prefer_fallback: bool,
}

impl fmt::Debug for V2PoolSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("V2PoolSet")
            .field("primary", &self.primary.as_ref().map(|_| "configured"))
            .field("fallback", &self.fallback.as_ref().map(|_| "configured"))
            .field("prefer_fallback", &self.prefer_fallback)
            .finish()
    }
}

pub(crate) fn read_stratum_v2_pool_set() -> Result<Option<V2PoolSet>, ProductionSettingsReadError> {
    let _transaction_guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| ProductionSettingsReadError::new("transaction_lock"))?;
    let partition = super::default_nvs_partition()
        .map_err(|_| ProductionSettingsReadError::new("nvs_partition"))?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false)
        .map_err(|_| ProductionSettingsReadError::new("nvs_open"))?;
    let defaults = ultra_205_defaults();
    let hardware_version = defaults.device_model();
    let primary = read_pool(&nvs, "", hardware_version)?;
    let fallback = read_pool(&nvs, "fb", hardware_version)?;
    if primary.is_none() && fallback.is_none() {
        return Ok(None);
    }
    Ok(Some(V2PoolSet {
        primary,
        fallback,
        prefer_fallback: nvs
            .get_u16("usefbstartum")
            .map_err(|_| ProductionSettingsReadError::new("nvs_u16_read"))?
            .unwrap_or(0)
            == 1,
    }))
}

fn read_pool(
    nvs: &EspNvs<NvsDefault>,
    prefix: &str,
    hardware_version: &str,
) -> Result<Option<V2PoolSettings>, ProductionSettingsReadError> {
    let protocol = read_optional_string(nvs, &format!("{prefix}stratumprot"))?
        .unwrap_or_else(|| "SV1".to_owned());
    if protocol != "SV2" {
        return Ok(None);
    }
    if nvs
        .get_u16(&format!("{prefix}stratumtls"))
        .map_err(|_| ProductionSettingsReadError::new("nvs_u16_read"))?
        .unwrap_or(0)
        != 0
    {
        return Err(ProductionSettingsReadError::new("sv2_tls_unsupported"));
    }
    let Some(endpoint_host) = read_optional_string(nvs, &format!("{prefix}stratumurl"))? else {
        return Ok(None);
    };
    let Some(endpoint_port) = nvs
        .get_u16(&format!("{prefix}stratumport"))
        .map_err(|_| ProductionSettingsReadError::new("nvs_u16_read"))?
    else {
        return Ok(None);
    };
    let Some(user_identity) = read_optional_string(nvs, &format!("{prefix}stratumuser"))? else {
        return Ok(None);
    };
    if endpoint_host.trim().is_empty() || endpoint_port == 0 || user_identity.is_empty() {
        return Ok(None);
    }
    let channel_key = if prefix.is_empty() {
        "sv2chantype"
    } else {
        "fbsv2chantype"
    };
    let channel_kind = match read_optional_string(nvs, channel_key)?.as_deref() {
        None | Some("extended") => ChannelKind::Extended,
        Some("standard") => ChannelKind::Standard,
        Some(_) => return Err(ProductionSettingsReadError::new("sv2_channel_type")),
    };
    let authority_key = if prefix.is_empty() {
        "sv2authpubkey"
    } else {
        "fbsv2authpubk"
    };
    let authority = read_optional_string(nvs, authority_key)?.unwrap_or_default();
    let maybe_authority_public_key = parse_authority_public_key(&authority)
        .map_err(|_| ProductionSettingsReadError::new("sv2_authority_key"))?;
    Ok(Some(V2PoolSettings {
        session: SessionConfig {
            endpoint_host,
            endpoint_port,
            vendor: "bitaxe-rust".to_owned(),
            hardware_version: hardware_version.to_owned(),
            firmware: crate::build_label().to_owned(),
            device_id: String::new(),
            user_identity,
            nominal_hashrate: 1.0e12,
            channel_kind,
            minimum_extranonce_size: 6,
        },
        maybe_authority_public_key,
    }))
}

fn read_optional_string(
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
