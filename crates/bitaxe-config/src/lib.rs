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
}

impl Phase1BoardSelection {
    /// Returns the first Phase 1 hardware target: Gamma 601 with BM1370.
    #[must_use]
    pub const fn gamma_601() -> Self {
        Self {
            board: BoardTarget::Gamma601,
            asic: AsicTarget::Bm1370,
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
}

#[cfg(test)]
mod tests {
    use bitaxe_core::{AsicTarget, BoardTarget};

    use super::Phase1BoardSelection;

    #[test]
    fn gamma_601_selection_uses_gamma_601_board_target() {
        // Arrange
        let selection = Phase1BoardSelection::gamma_601();

        // Act
        let board = selection.board();

        // Assert
        assert_eq!(board, BoardTarget::Gamma601);
    }

    #[test]
    fn gamma_601_selection_uses_bm1370_asic_target() {
        // Arrange
        let selection = Phase1BoardSelection::gamma_601();

        // Act
        let asic = selection.asic();

        // Assert
        assert_eq!(asic, AsicTarget::Bm1370);
    }
}
