//! Metadata-only soak samples: identity, mining state, safety, hardware settings and readings,
//! sequences and counters.
//! Pool endpoints, users and other operator values never enter a sample (evidence policy).
use bitaxe_api::{reference_u64, ObservationStateWire, SystemInfoWire};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SoakTransport {
    Http,
    Websocket,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SoakSample {
    pub boot_session: String,
    pub boot_ordinal: u64,
    pub source_commit: String,
    pub app_elf_sha256: String,
    pub source_dirty: bool,
    pub mining_active: bool,
    pub mining_paused: bool,
    /// A hardware safe stop is in progress (`miningActivity` `stopping`): not active,
    /// and not yet a confirmed stop. Defaulted so journals from before the state still parse.
    #[serde(default)]
    pub mining_stopping: bool,
    pub start_mining_on_boot: bool,
    pub safety_valid: bool,
    pub maybe_watchdog_feed_sequence: Option<u64>,
    pub maybe_checkpoint_sequence: Option<u64>,
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    /// The all-time and session best nonce difficulty (`bestDiff`, `bestSessionDiff`).
    /// Defaulted so journals from before these fields still parse.
    #[serde(default)]
    pub best_diff: u64,
    #[serde(default)]
    pub best_session_diff: u64,
    /// HTTP server task sends (`runtimeHealth.httpServerSends`): sends, failures and
    /// the worst duration since boot, and the worst in the last minute or two.
    #[serde(default)]
    pub http_response_sends: u64,
    #[serde(default)]
    pub http_response_worst_ms: u32,
    #[serde(default)]
    pub http_response_recent_worst_ms: u32,
    #[serde(default)]
    pub http_response_failures: u64,
    #[serde(default)]
    pub websocket_sends: u64,
    #[serde(default)]
    pub websocket_send_worst_ms: u32,
    #[serde(default)]
    pub websocket_send_recent_worst_ms: u32,
    #[serde(default)]
    pub websocket_send_failures: u64,
    #[serde(default)]
    pub wifi_rssi: i16,
    /// The saved frequency and core voltage (ADR-0037: the device settings are the profile).
    #[serde(default)]
    pub frequency_mhz: f64,
    #[serde(default)]
    pub core_voltage_mv: u16,
    /// The measured core voltage, only while its reading is fresh.
    #[serde(default)]
    pub maybe_core_voltage_actual_mv: Option<f64>,
    #[serde(default)]
    pub power_watts: f64,
    #[serde(default)]
    pub hash_rate_ghs: f64,
    /// The saved fan settings and the fan's reading, so a soak can judge a live fan change.
    #[serde(default)]
    pub auto_fan_speed: u8,
    #[serde(default)]
    pub manual_fan_speed: u16,
    #[serde(default)]
    pub fan_rpm: u16,
    pub revision: u64,
    /// HTTP only: whether the configured pool settings still equal the first HTTP sample's.
    pub maybe_pool_matches_initial: Option<bool>,
}

impl SoakSample {
    #[must_use]
    pub fn from_wire(wire: &SystemInfoWire, maybe_pool_matches_initial: Option<bool>) -> Self {
        let sends = wire.runtime_health.http_server_sends;
        Self {
            boot_session: wire.boot_session.to_string(),
            boot_ordinal: wire.boot_ordinal,
            source_commit: wire.source_commit.clone(),
            app_elf_sha256: wire.app_elf_sha256.clone(),
            source_dirty: wire.source_dirty,
            mining_active: !wire.mining_paused && wire.mining_activity == "active",
            mining_paused: wire.mining_paused,
            mining_stopping: wire.mining_activity == "stopping",
            start_mining_on_boot: wire.start_mining_on_boot,
            safety_valid: safety_valid(wire),
            maybe_watchdog_feed_sequence: wire.runtime_health.maybe_task_watchdog_feed_sequence,
            maybe_checkpoint_sequence: wire.runtime_health.maybe_checkpoint_sequence,
            shares_accepted: wire.shares_accepted,
            shares_rejected: wire.shares_rejected,
            best_diff: reference_u64(wire.best_diff),
            best_session_diff: reference_u64(wire.best_session_diff),
            http_response_sends: sends.http_responses.sends,
            http_response_worst_ms: sends.http_responses.worst_millis,
            http_response_recent_worst_ms: sends.http_responses.recent_worst_millis,
            http_response_failures: sends.http_responses.failures,
            websocket_sends: sends.websocket_sends.sends,
            websocket_send_worst_ms: sends.websocket_sends.worst_millis,
            websocket_send_recent_worst_ms: sends.websocket_sends.recent_worst_millis,
            websocket_send_failures: sends.websocket_sends.failures,
            wifi_rssi: wire.wifi_rssi,
            frequency_mhz: wire.frequency,
            core_voltage_mv: wire.core_voltage,
            maybe_core_voltage_actual_mv: (wire.core_voltage_actual_status.state
                == ObservationStateWire::Fresh)
                .then_some(wire.core_voltage_actual),
            power_watts: wire.power,
            hash_rate_ghs: wire.hash_rate,
            auto_fan_speed: wire.auto_fan_speed,
            manual_fan_speed: wire.manual_fan_speed,
            fan_rpm: wire.fan_rpm,
            revision: wire.operator_snapshot_revision.get(),
            maybe_pool_matches_initial,
        }
    }

    /// Same boot, same exact package, clean source.
    #[must_use]
    pub fn same_identity(&self, other: &Self) -> bool {
        self.boot_session == other.boot_session
            && self.boot_ordinal == other.boot_ordinal
            && self.source_commit == other.source_commit
            && self.app_elf_sha256 == other.app_elf_sha256
            && !self.source_dirty
            && !other.source_dirty
    }
}

/// The soak's live stop limits: fresh readings, 4.5–5.5 V input, at most 15 W, ASIC below 75 C,
/// nonzero fan. The same bounds the legacy campaign judged.
#[must_use]
pub fn safety_valid(sample: &SystemInfoWire) -> bool {
    [
        sample.power_status.state,
        sample.voltage_status.state,
        sample.current_status.state,
        sample.chip_temp_status.state,
        sample.fan_rpm_status.state,
    ]
    .into_iter()
    .all(|state| state == ObservationStateWire::Fresh)
        && sample.power.is_finite()
        && (0.0..=15.0).contains(&sample.power)
        && sample.voltage_millivolts.is_finite()
        && (4_500.0..=5_500.0).contains(&sample.voltage_millivolts)
        && sample.current_milliamps.is_finite()
        && sample.current_milliamps >= 0.0
        && sample.temp.is_finite()
        && sample.temp < 75.0
        && sample.fan_rpm > 0
}

/// Configured primary and fallback pool settings, held only in zeroizing memory for comparison.
#[derive(PartialEq, Eq)]
pub struct PoolSettings(Zeroizing<String>);

impl PoolSettings {
    #[must_use]
    pub fn from_wire(wire: &SystemInfoWire) -> Self {
        Self(Zeroizing::new(format!(
            "{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}",
            wire.stratum_url,
            wire.stratum_port,
            wire.stratum_user,
            wire.stratum_protocol,
            wire.fallback_stratum_url,
            wire.fallback_stratum_port,
            wire.fallback_stratum_user,
            wire.fallback_stratum_protocol,
        )))
    }
}
