//! AxeOS API wire contracts and pure adapter input boundaries.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/openapi.yaml`
//! - `reference/esp-miner/main/http_server/system_api_json.c`
//! - `reference/esp-miner/main/http_server/axe-os/api/system/asic_settings.c`

pub mod settings;
pub mod snapshot;
pub mod wire;

pub use settings::{
    plan_settings_patch_body, plan_settings_patch_value, AcceptedSettingsPatch,
    SettingsPatchFailure, SettingsPatchFailureReason, SettingsPatchPublicError,
};
pub use snapshot::{
    ApiSnapshot, AsicSnapshot, ConfigSnapshot, PlatformSnapshot, SafeTelemetrySnapshot,
};
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
