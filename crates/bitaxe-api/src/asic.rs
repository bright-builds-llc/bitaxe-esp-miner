//! Pure `/api/system/asic` response mapping.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/axe-os/api/system/asic_settings.c`
//! - `crates/bitaxe-config/src/catalog.rs`

use crate::{ApiSnapshot, SystemAsicWire};

/// Maps typed catalog and ASIC snapshot facts into the AxeOS ASIC response.
#[must_use]
pub fn asic_settings_from_snapshot(snapshot: &ApiSnapshot) -> SystemAsicWire {
    SystemAsicWire::from_snapshot(snapshot)
}

#[cfg(test)]
mod tests {
    use crate::asic::asic_settings_from_snapshot;
    use crate::ApiSnapshot;

    #[test]
    fn asic_settings_maps_ultra_205_catalog_options_from_typed_config() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();

        // Act
        let response = asic_settings_from_snapshot(&snapshot);

        // Assert
        assert_eq!(response.asic_model, "BM1366");
        assert_eq!(response.device_model, "Ultra");
        assert_eq!(response.asic_count, 1);
        assert_eq!(
            response.frequency_options,
            vec![400, 425, 450, 475, 485, 500, 525, 550, 575]
        );
        assert_eq!(response.default_frequency, 485);
        assert_eq!(response.voltage_options, vec![1100, 1150, 1200, 1250, 1300]);
        assert_eq!(response.default_voltage, 1200);
    }
}
