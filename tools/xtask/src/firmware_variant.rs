//! Firmware variant identity recorded in the package manifest (ADR-0040, ADR-0041).
//!
//! The packager passes the variant selected by `//firmware/bitaxe:firmware_variant`;
//! the manifest records it so the flash tool can refuse test firmware unless the
//! operator names it explicitly.

use crate::BoardId;
use anyhow::{bail, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Firmware build compiled into the package's application image.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FirmwareVariant {
    /// The default firmware, the only variant a release may carry.
    Production,
    /// Test-only firmware with a lowered local overheat trip, for the Ultra 205.
    OverheatTripTest,
    /// Test-only Gamma 601 firmware that runs one bounded core-voltage check.
    Gamma601VoutCheck,
}

impl FirmwareVariant {
    /// The board a test variant is built for; production builds for every board.
    const fn maybe_test_board(self) -> Option<BoardId> {
        match self {
            Self::Production => None,
            Self::OverheatTripTest => Some(BoardId::Ultra205),
            Self::Gamma601VoutCheck => Some(BoardId::Gamma601),
        }
    }
}

/// A test variant's package targets only the board it was built for.
pub(crate) fn validate_firmware_variant_board(
    maybe_variant: Option<FirmwareVariant>,
    board: BoardId,
) -> Result<()> {
    let maybe_test_board = maybe_variant.and_then(FirmwareVariant::maybe_test_board);
    if maybe_test_board.is_some_and(|test_board| test_board != board) {
        bail!("firmware variant is not built for board {board}");
    }
    Ok(())
}

/// Test variants exist for hardware checks only, so a release tag never carries one.
pub(crate) fn validate_manifest_firmware_variant(
    maybe_variant: Option<FirmwareVariant>,
    maybe_release_tag: Option<&str>,
) -> Result<()> {
    let is_test_variant =
        maybe_variant.is_some_and(|variant| variant != FirmwareVariant::Production);
    if is_test_variant && maybe_release_tag.is_some() {
        bail!("test firmware variant must never carry a release tag");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_names_serialize_as_the_build_flag_values() {
        // Arrange
        let variants = [
            FirmwareVariant::Production,
            FirmwareVariant::OverheatTripTest,
            FirmwareVariant::Gamma601VoutCheck,
        ];

        // Act
        let names: Vec<String> = variants
            .iter()
            .map(|variant| serde_json::to_string(variant).expect("serialize variant"))
            .collect();

        // Assert
        assert_eq!(
            names,
            [
                "\"production\"",
                "\"overheat-trip-test\"",
                "\"gamma601-vout-check\""
            ]
        );
    }

    #[test]
    fn a_release_tag_never_carries_a_test_variant() {
        // Arrange
        let test_variant = Some(FirmwareVariant::OverheatTripTest);

        // Act
        let result = validate_manifest_firmware_variant(test_variant, Some("v1.0.0"));

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn production_and_historical_manifests_may_carry_a_release_tag() {
        // Arrange
        let variants = [None, Some(FirmwareVariant::Production)];

        // Act
        let results: Vec<bool> = variants
            .iter()
            .map(|variant| validate_manifest_firmware_variant(*variant, Some("v1.0.0")).is_ok())
            .collect();

        // Assert
        assert_eq!(results, [true, true]);
    }

    #[test]
    fn a_test_variant_targets_only_its_own_board() {
        // Arrange
        let cases = [
            (Some(FirmwareVariant::Gamma601VoutCheck), BoardId::Gamma601),
            (Some(FirmwareVariant::Gamma601VoutCheck), BoardId::Ultra205),
            (Some(FirmwareVariant::OverheatTripTest), BoardId::Ultra205),
            (Some(FirmwareVariant::OverheatTripTest), BoardId::Gamma601),
            (Some(FirmwareVariant::Production), BoardId::Gamma601),
            (None, BoardId::Gamma601),
        ];

        // Act
        let admitted: Vec<bool> = cases
            .iter()
            .map(|(variant, board)| validate_firmware_variant_board(*variant, *board).is_ok())
            .collect();

        // Assert
        assert_eq!(admitted, [true, false, true, false, true, true]);
    }
}
