use bitaxe_core::{AsicTarget, BoardTarget};

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

    use super::Phase1BoardSelection;

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
}
