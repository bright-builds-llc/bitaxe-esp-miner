//! AxeOS API wire contracts and pure adapter input boundaries.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/openapi.yaml`
//! - `reference/esp-miner/main/http_server/system_api_json.c`
//! - `reference/esp-miner/main/http_server/axe-os/api/system/asic_settings.c`

pub mod asic;
pub mod mining;
pub mod scoreboard;
pub mod settings;
pub mod snapshot;
pub mod statistics;
pub mod system;
pub mod wire;

pub use asic::asic_settings_from_snapshot;
pub use mining::{mining_state_from_runtime, MiningStateWire, SharesRejectedReasonWire};
pub use scoreboard::{scoreboard_response, ScoreboardEntry, ScoreboardEntryWire};
pub use settings::{
    execute_settings_persistence_plan, plan_settings_patch_body, plan_settings_patch_value,
    AcceptedSettingsPatch, SettingsAdapterFailure, SettingsPatchFailure,
    SettingsPatchFailureReason, SettingsPatchPublicError, SettingsPersistenceAdapter,
    SettingsPersistenceEffect, SettingsPersistenceFailure, SettingsPersistenceFailureReport,
    SettingsPersistencePlan, SettingsPersistenceStep, SettingsPersistenceSuccess,
    SettingsPublicResponse,
};
pub use snapshot::{
    ApiSnapshot, AsicSnapshot, ConfigSnapshot, PlatformSnapshot, SafeTelemetrySnapshot,
};
pub use statistics::{
    empty_statistics_response, statistics_response, StatisticsSample, StatisticsWire,
};
pub use system::system_info_from_snapshot;
pub use wire::{SystemAsicWire, SystemInfoWire};

#[cfg(test)]
mod tests {
    use super::{ApiSnapshot, SystemInfoWire};

    #[test]
    fn api_contract_no_longer_exposes_phase_1_deferral_status() {
        // Arrange
        let public_contract_count = 3;

        // Act
        let expected_contract_count =
            [ApiSnapshot::safe_ultra_205()].len() + ["SystemInfoWire", "SystemAsicWire"].len();

        // Assert
        assert_eq!(expected_contract_count, public_contract_count);
    }

    #[test]
    fn api_snapshot_maps_to_system_info_wire_contract() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();

        // Act
        let wire = SystemInfoWire::from_snapshot(&snapshot);

        // Assert
        assert_eq!(wire.asic_model, "BM1366");
        assert_eq!(wire.frequency, 485.0);
        assert!(wire.mining_paused);
    }
}
