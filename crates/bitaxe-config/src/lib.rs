use bitaxe_core::{AsicTarget, BoardTarget};

pub mod catalog;
pub mod defaults;

pub use catalog::{
    board_catalog, ultra_205_catalog_entry, AsicProfile, BoardCapabilities, BoardCatalogEntry,
    VerificationScope,
};
pub use defaults::{ultra_205_defaults, Ultra205Defaults};

/// Phase 1 board and ASIC identity selection.
///
/// This contract is intentionally limited to typed identity. It does not perform
/// NVS mutation, mutable settings, Wi-Fi setup, mining, ASIC control, voltage,
/// fan, thermal, or power behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phase1BoardSelection {
    board: BoardTarget,
    asic: AsicTarget,
    device_model: &'static str,
    board_version: u16,
    asic_frequency_mhz: u16,
    asic_voltage_mv: u16,
}

impl Phase1BoardSelection {
    /// Returns the first Phase 1 hardware target: Ultra 205 with BM1366.
    ///
    /// This remains as a Phase 1 compatibility shim for callers that only need
    /// board and ASIC identity.
    #[must_use]
    pub const fn ultra_205() -> Self {
        Self {
            board: BoardTarget::Ultra205,
            asic: AsicTarget::Bm1366,
            device_model: "ultra",
            board_version: 205,
            asic_frequency_mhz: 485,
            asic_voltage_mv: 1200,
        }
    }

    /// Returns the selected board target.
    #[must_use]
    pub const fn board(&self) -> BoardTarget {
        self.board
    }

    /// Returns the selected ASIC target.
    #[must_use]
    pub const fn asic(&self) -> AsicTarget {
        self.asic
    }

    /// Returns the upstream `devicemodel` default for the selected board.
    #[must_use]
    pub const fn device_model(&self) -> &'static str {
        self.device_model
    }

    /// Returns the upstream `boardversion` default for the selected board.
    #[must_use]
    pub const fn board_version(&self) -> u16 {
        self.board_version
    }

    /// Returns the upstream `asicmodel` default for the selected board.
    #[must_use]
    pub const fn asic_model(&self) -> &'static str {
        self.asic.display_name()
    }

    /// Returns the upstream `asicfrequency` default in MHz.
    #[must_use]
    pub const fn asic_frequency_mhz(&self) -> u16 {
        self.asic_frequency_mhz
    }

    /// Returns the upstream `asicvoltage` default in millivolts.
    #[must_use]
    pub const fn asic_voltage_mv(&self) -> u16 {
        self.asic_voltage_mv
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_core::{AsicTarget, BoardTarget};

    use super::{
        board_catalog, ultra_205_catalog_entry, ultra_205_defaults, Phase1BoardSelection,
        VerificationScope,
    };

    #[test]
    fn ultra_205_selection_uses_ultra_205_board_target() {
        // Arrange
        let selection = Phase1BoardSelection::ultra_205();

        // Act
        let board = selection.board();

        // Assert
        assert_eq!(board, BoardTarget::Ultra205);
    }

    #[test]
    fn ultra_205_selection_uses_bm1366_asic_target() {
        // Arrange
        let selection = Phase1BoardSelection::ultra_205();

        // Act
        let asic = selection.asic();

        // Assert
        assert_eq!(asic, AsicTarget::Bm1366);
    }

    #[test]
    fn ultra_205_selection_uses_reference_config_205_defaults() {
        // Arrange
        let selection = Phase1BoardSelection::ultra_205();

        // Act
        let device_model = selection.device_model();
        let board_version = selection.board_version();
        let asic_model = selection.asic_model();
        let asic_frequency_mhz = selection.asic_frequency_mhz();
        let asic_voltage_mv = selection.asic_voltage_mv();

        // Assert
        assert_eq!(device_model, "ultra");
        assert_eq!(board_version, 205);
        assert_eq!(asic_model, "BM1366");
        assert_eq!(asic_frequency_mhz, 485);
        assert_eq!(asic_voltage_mv, 1200);
    }

    #[test]
    fn ultra_205_defaults_match_config_205_fixture() {
        // Arrange
        let defaults = ultra_205_defaults();

        // Act
        let primary_pool = defaults.primary_pool();
        let fallback_pool = defaults.fallback_pool();

        // Assert
        assert_eq!(defaults.hostname(), "bitaxe");
        assert_eq!(primary_pool.url(), "public-pool.io");
        assert_eq!(primary_pool.port(), 3333);
        assert_eq!(primary_pool.tls(), 0);
        assert_eq!(primary_pool.cert(), "x");
        assert_eq!(
            primary_pool.user(),
            "bc1qnp980s5fpp8l94p5cvttmtdqy8rvrq74qly2yrfmzkdsntqzlc5qkc4rkq.bitaxe"
        );
        assert_eq!(primary_pool.password(), "x");
        assert_eq!(primary_pool.difficulty(), 1000);
        assert_eq!(primary_pool.extranonce_subscribe(), 0);
        assert_eq!(fallback_pool.url(), "solo.ckpool.org");
        assert_eq!(fallback_pool.port(), 3333);
        assert_eq!(fallback_pool.tls(), 0);
        assert_eq!(fallback_pool.cert(), "x");
        assert_eq!(
            fallback_pool.user(),
            "bc1qnp980s5fpp8l94p5cvttmtdqy8rvrq74qly2yrfmzkdsntqzlc5qkc4rkq.bitaxe"
        );
        assert_eq!(fallback_pool.password(), "x");
        assert_eq!(fallback_pool.difficulty(), 1000);
        assert_eq!(fallback_pool.extranonce_subscribe(), 0);
        assert_eq!(defaults.asic_frequency_mhz(), 485);
        assert_eq!(defaults.asic_voltage_mv(), 1200);
        assert_eq!(defaults.asic_model(), "BM1366");
        assert_eq!(defaults.device_model(), "ultra");
        assert_eq!(defaults.board_version(), "205");
        assert_eq!(defaults.rotation(), 0);
        assert!(defaults.auto_fan_speed());
        assert_eq!(defaults.manual_fan_speed(), 100);
        assert!(defaults.self_test());
        assert!(!defaults.overheat_mode());
    }

    #[test]
    fn ultra_205_catalog_entry_uses_bm1366_reference_values() {
        // Arrange
        let entry = ultra_205_catalog_entry();

        // Act
        let asic = entry.asic();
        let capabilities = entry.capabilities();

        // Assert
        assert_eq!(entry.board_version(), "205");
        assert_eq!(entry.family(), "Ultra");
        assert_eq!(asic.model(), "BM1366");
        assert_eq!(entry.asic_count(), 1);
        assert_eq!(
            asic.frequency_options(),
            [400, 425, 450, 475, 485, 500, 525, 550, 575]
        );
        assert_eq!(asic.voltage_options(), [1100, 1150, 1200, 1250, 1300]);
        assert_eq!(asic.default_frequency_mhz(), 485);
        assert_eq!(asic.default_voltage_mv(), 1200);
        assert!(capabilities.ds4432u());
        assert!(capabilities.ina260());
        assert!(!capabilities.tps546());
        assert_eq!(entry.power_consumption_target(), 12);
    }

    #[test]
    fn non_205_board_entries_are_not_hardware_verified() {
        // Arrange
        let catalog = board_catalog();

        // Act
        let non_205_entries = catalog
            .iter()
            .filter(|entry| entry.board_version() != "205");

        // Assert
        for entry in non_205_entries {
            assert_eq!(
                entry.verification_scope(),
                VerificationScope::NotHardwareVerified
            );
        }
    }
}
