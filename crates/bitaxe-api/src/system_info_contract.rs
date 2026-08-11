//! Pure input models for the complete upstream system-info response.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/system_api_json.c`
//! - `reference/esp-miner/main/http_server/openapi.yaml` `SystemInfo`

use core::fmt;

use bitaxe_config::{reload_snapshot, LoadedValue, NvsSnapshot};

/// Maximum number of upstream statistics samples exposed by `statsLimit`.
pub const SYSTEM_INFO_STATISTICS_LIMIT: u16 = 720;

/// Persisted settings emitted by `/api/system/info`.
///
/// Pool identity and certificate fields are intentionally retained only long
/// enough to serialize the local API response. Debug output never renders
/// their values.
#[derive(Clone, PartialEq)]
pub struct SystemInfoSettingsSnapshot {
    pub display: String,
    pub rotation: u16,
    pub invert_screen: bool,
    pub display_timeout: i32,
    pub manual_fan_speed: u16,
    pub min_fan_speed: u16,
    pub temp_target: u16,
    pub stats_frequency: u16,
    pub overclock_enabled: bool,
    pub overheat_mode: bool,
    pub primary_pool: SystemInfoPoolSnapshot,
    pub fallback_pool: SystemInfoPoolSnapshot,
}

impl SystemInfoSettingsSnapshot {
    /// Returns a secret-free fixture snapshot with exact non-secret defaults.
    #[must_use]
    pub fn safe_ultra_205() -> Self {
        Self {
            display: String::new(),
            rotation: 0,
            invert_screen: false,
            display_timeout: -1,
            manual_fan_speed: 100,
            min_fan_speed: 25,
            temp_target: 60,
            stats_frequency: 0,
            overclock_enabled: false,
            overheat_mode: false,
            primary_pool: SystemInfoPoolSnapshot::safe_default(),
            fallback_pool: SystemInfoPoolSnapshot::safe_default(),
        }
    }

    /// Loads typed response settings from one atomically observed NVS snapshot.
    #[must_use]
    pub fn from_nvs_snapshot(snapshot: &NvsSnapshot) -> Self {
        let loaded = reload_snapshot(snapshot);
        Self {
            display: loaded_string(&loaded, "display"),
            rotation: loaded_u16(&loaded, "rotation"),
            invert_screen: loaded_bool(&loaded, "invertscreen"),
            display_timeout: loaded_i32(&loaded, "displayTimeout"),
            manual_fan_speed: loaded_u16(&loaded, "manualfanspeed"),
            min_fan_speed: loaded_u16(&loaded, "minfanspeed"),
            temp_target: loaded_u16(&loaded, "temptarget"),
            stats_frequency: loaded_u16(&loaded, "statsFrequency"),
            overclock_enabled: loaded_bool(&loaded, "oc_enabled"),
            overheat_mode: loaded_bool(&loaded, "overheat_mode"),
            primary_pool: SystemInfoPoolSnapshot {
                url: loaded_string(&loaded, "stratumurl"),
                port: loaded_u16(&loaded, "stratumport"),
                user: loaded_string(&loaded, "stratumuser"),
                suggested_difficulty: loaded_u16(&loaded, "stratumdiff"),
                extranonce_subscribe: loaded_bool(&loaded, "stratumxnsub"),
                tls: loaded_u16(&loaded, "stratumtls"),
                certificate: loaded_string(&loaded, "stratumcert"),
                decode_coinbase: loaded_bool(&loaded, "stratumdecode"),
                protocol: loaded_string(&loaded, "stratumprot"),
                v2_authority_public_key: loaded_string(&loaded, "sv2authpubkey"),
                v2_channel_type: loaded_string(&loaded, "sv2chantype"),
            },
            fallback_pool: SystemInfoPoolSnapshot {
                url: loaded_string(&loaded, "fbstratumurl"),
                port: loaded_u16(&loaded, "fbstratumport"),
                user: loaded_string(&loaded, "fbstratumuser"),
                suggested_difficulty: loaded_u16(&loaded, "fbstratumdiff"),
                extranonce_subscribe: loaded_bool(&loaded, "stratumfbxnsub"),
                tls: loaded_u16(&loaded, "fbstratumtls"),
                certificate: loaded_string(&loaded, "fbstratumcert"),
                decode_coinbase: loaded_bool(&loaded, "fbstratumdecode"),
                protocol: loaded_string(&loaded, "fbstratumprot"),
                v2_authority_public_key: loaded_string(&loaded, "fbsv2authpubk"),
                v2_channel_type: loaded_string(&loaded, "fbsv2chantype"),
            },
        }
    }
}

impl fmt::Debug for SystemInfoSettingsSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SystemInfoSettingsSnapshot")
            .field("display", &self.display)
            .field("rotation", &self.rotation)
            .field("invert_screen", &self.invert_screen)
            .field("display_timeout", &self.display_timeout)
            .field("manual_fan_speed", &self.manual_fan_speed)
            .field("min_fan_speed", &self.min_fan_speed)
            .field("temp_target", &self.temp_target)
            .field("stats_frequency", &self.stats_frequency)
            .field("overclock_enabled", &self.overclock_enabled)
            .field("overheat_mode", &self.overheat_mode)
            .field("primary_pool", &"[redacted]")
            .field("fallback_pool", &"[redacted]")
            .finish()
    }
}

/// One primary or fallback pool projection used by system-info.
#[derive(Clone, PartialEq)]
pub struct SystemInfoPoolSnapshot {
    pub url: String,
    pub port: u16,
    pub user: String,
    pub suggested_difficulty: u16,
    pub extranonce_subscribe: bool,
    pub tls: u16,
    pub certificate: String,
    pub decode_coinbase: bool,
    pub protocol: String,
    pub v2_authority_public_key: String,
    pub v2_channel_type: String,
}

impl SystemInfoPoolSnapshot {
    fn safe_default() -> Self {
        Self {
            url: String::new(),
            port: 0,
            user: String::new(),
            suggested_difficulty: 0,
            extranonce_subscribe: false,
            tls: 0,
            certificate: String::new(),
            decode_coinbase: true,
            protocol: "SV1".to_owned(),
            v2_authority_public_key: String::new(),
            v2_channel_type: "extended".to_owned(),
        }
    }
}

impl fmt::Debug for SystemInfoPoolSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SystemInfoPoolSnapshot([redacted])")
    }
}

/// Conditional block template facts emitted only after a positive block height.
#[derive(Clone, PartialEq)]
pub struct SystemInfoBlockSnapshot {
    pub height: u64,
    pub script_sig: String,
    pub network_difficulty: f64,
    pub coinbase_value_total_satoshis: u64,
    pub coinbase_value_user_satoshis: u64,
    pub signals: Vec<String>,
    pub coinbase_outputs: Vec<SystemInfoCoinbaseOutput>,
}

impl fmt::Debug for SystemInfoBlockSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SystemInfoBlockSnapshot")
            .field("height", &self.height)
            .field("script_sig", &"[redacted]")
            .field("network_difficulty", &self.network_difficulty)
            .field(
                "coinbase_value_total_satoshis",
                &self.coinbase_value_total_satoshis,
            )
            .field(
                "coinbase_value_user_satoshis",
                &self.coinbase_value_user_satoshis,
            )
            .field("signal_count", &self.signals.len())
            .field("coinbase_output_count", &self.coinbase_outputs.len())
            .finish()
    }
}

/// One conditional coinbase output in the upstream wire shape.
#[derive(Clone, PartialEq, Eq)]
pub struct SystemInfoCoinbaseOutput {
    pub value_satoshis: u64,
    pub address: String,
}

impl fmt::Debug for SystemInfoCoinbaseOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SystemInfoCoinbaseOutput")
            .field("value_satoshis", &self.value_satoshis)
            .field("address", &"[redacted]")
            .finish()
    }
}

fn loaded_string(loaded: &bitaxe_config::PersistenceDecision, key: &str) -> String {
    match loaded.maybe_loaded_value(key) {
        Some(LoadedValue::Str(value)) => value.clone(),
        _ => String::new(),
    }
}

fn loaded_u16(loaded: &bitaxe_config::PersistenceDecision, key: &str) -> u16 {
    match loaded.maybe_loaded_value(key) {
        Some(LoadedValue::U16(value)) => *value,
        _ => 0,
    }
}

fn loaded_i32(loaded: &bitaxe_config::PersistenceDecision, key: &str) -> i32 {
    match loaded.maybe_loaded_value(key) {
        Some(LoadedValue::I32(value)) => *value,
        _ => 0,
    }
}

fn loaded_bool(loaded: &bitaxe_config::PersistenceDecision, key: &str) -> bool {
    match loaded.maybe_loaded_value(key) {
        Some(LoadedValue::Bool(value)) => *value,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_config::{NvsSnapshot, StoredValue};

    use super::SystemInfoSettingsSnapshot;

    #[test]
    fn debug_output_redacts_pool_identity_and_certificate_values() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([
            StoredValue::string("stratumurl", "pool-canary"),
            StoredValue::string("stratumuser", "worker-canary"),
            StoredValue::string("stratumcert", "cert-canary"),
        ]);
        let settings = SystemInfoSettingsSnapshot::from_nvs_snapshot(&snapshot);

        // Act
        let debug = format!("{settings:?}");

        // Assert
        for canary in ["pool-canary", "worker-canary", "cert-canary"] {
            assert!(!debug.contains(canary));
        }
        assert!(debug.contains("primary_pool: \"[redacted]\""));
    }

    #[test]
    fn confirmed_snapshot_projects_exact_mixed_setting_types() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([
            StoredValue::string("display", "display-canary"),
            StoredValue::u16("rotation", 180),
            StoredValue::u16("invertscreen", 1),
            StoredValue::i32("displayTimeout", 90),
            StoredValue::u16("statsFrequency", 15),
            StoredValue::string("stratumprot", "SV2"),
        ]);

        // Act
        let settings = SystemInfoSettingsSnapshot::from_nvs_snapshot(&snapshot);

        // Assert
        assert_eq!(settings.display, "display-canary");
        assert_eq!(settings.rotation, 180);
        assert!(settings.invert_screen);
        assert_eq!(settings.display_timeout, 90);
        assert_eq!(settings.stats_frequency, 15);
        assert_eq!(settings.primary_pool.protocol, "SV2");
    }
}
