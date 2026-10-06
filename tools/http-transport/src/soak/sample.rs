//! Metadata-only soak samples: identity, mining state, safety, sequences and counters.
//! Pool endpoints, users and other operator values never enter a sample (evidence policy).
use bitaxe_api::{ObservationStateWire, SystemInfoWire};
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
    pub start_mining_on_boot: bool,
    pub safety_valid: bool,
    pub maybe_watchdog_feed_sequence: Option<u64>,
    pub maybe_checkpoint_sequence: Option<u64>,
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    pub revision: u64,
    /// HTTP only: whether the configured pool settings still equal the first HTTP sample's.
    pub maybe_pool_matches_initial: Option<bool>,
}

impl SoakSample {
    #[must_use]
    pub fn from_wire(wire: &SystemInfoWire, maybe_pool_matches_initial: Option<bool>) -> Self {
        Self {
            boot_session: wire.boot_session.to_string(),
            boot_ordinal: wire.boot_ordinal,
            source_commit: wire.source_commit.clone(),
            app_elf_sha256: wire.app_elf_sha256.clone(),
            source_dirty: wire.source_dirty,
            mining_active: !wire.mining_paused && wire.mining_activity == "active",
            mining_paused: wire.mining_paused,
            start_mining_on_boot: wire.start_mining_on_boot,
            safety_valid: safety_valid(wire),
            maybe_watchdog_feed_sequence: wire.runtime_health.maybe_task_watchdog_feed_sequence,
            maybe_checkpoint_sequence: wire.runtime_health.maybe_checkpoint_sequence,
            shares_accepted: wire.shares_accepted,
            shares_rejected: wire.shares_rejected,
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
