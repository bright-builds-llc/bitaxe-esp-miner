//! Test-only firmware variants (ADR-0040, ADR-0041).
//!
//! A package records the variant compiled into its application. Installing a
//! non-production variant requires the operator to name it with
//! `--allow-test-firmware <variant>`, and the application bytes must agree with
//! the recorded variant.

use crate::*;

/// Boot-line literals each compiled into the ELF only by its own test variant.
const OVERHEAT_TRIP_TEST_MARKER: &[u8] = b"firmware_variant=overheat-trip-test";
const GAMMA601_VOUT_CHECK_MARKER: &[u8] = b"firmware_variant=gamma601-vout-check";
const GAMMA601_CHIP_CHECK_MARKER: &[u8] = b"firmware_variant=gamma601-chip-check";
const GAMMA601_HASH_CHECK_MARKER: &[u8] = b"firmware_variant=gamma601-hash-check";
const GAMMA601_OVERHEAT_TRIP_TEST_MARKER: &[u8] = b"firmware_variant=gamma601-overheat-trip-test";

/// Firmware variant compiled into a package; mirrors `//firmware/bitaxe:firmware_variant`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum FirmwareVariant {
    /// The default firmware, and the variant of every package that predates ADR-0040.
    Production,
    /// Lowered local overheat trip for hardware checks on the Ultra 205; never released.
    OverheatTripTest,
    /// One bounded core-voltage check on the Gamma 601; never released.
    Gamma601VoutCheck,
    /// One bounded chip detection on the Gamma 601, for the phase F3.1 contract; never released.
    Gamma601ChipCheck,
    /// One bounded hash check on the Gamma 601, for the phase F3.2 contract; never released.
    Gamma601HashCheck,
    /// The production Gamma 601 build with the lowered local overheat trip, for the 601 overheat
    /// check (ADR-0041 items 12 and 15, ADR-0040); never released.
    Gamma601OverheatTripTest,
}

impl FirmwareVariant {
    /// A test variant's boot marker and the board it is built for.
    const fn maybe_test_identity(self) -> Option<(&'static [u8], BoardId)> {
        match self {
            Self::Production => None,
            Self::OverheatTripTest => Some((OVERHEAT_TRIP_TEST_MARKER, BoardId::Ultra205)),
            Self::Gamma601VoutCheck => Some((GAMMA601_VOUT_CHECK_MARKER, BoardId::Gamma601)),
            Self::Gamma601ChipCheck => Some((GAMMA601_CHIP_CHECK_MARKER, BoardId::Gamma601)),
            Self::Gamma601HashCheck => Some((GAMMA601_HASH_CHECK_MARKER, BoardId::Gamma601)),
            Self::Gamma601OverheatTripTest => {
                Some((GAMMA601_OVERHEAT_TRIP_TEST_MARKER, BoardId::Gamma601))
            }
        }
    }
}

const TEST_VARIANTS: [FirmwareVariant; 5] = [
    FirmwareVariant::OverheatTripTest,
    FirmwareVariant::Gamma601VoutCheck,
    FirmwareVariant::Gamma601ChipCheck,
    FirmwareVariant::Gamma601HashCheck,
    FirmwareVariant::Gamma601OverheatTripTest,
];

/// Reads the recorded variant; a manifest without one predates ADR-0040 and is production.
pub(crate) fn manifest_firmware_variant(maybe_recorded: Option<&str>) -> Result<FirmwareVariant> {
    let Some(recorded) = maybe_recorded else {
        return Ok(FirmwareVariant::Production);
    };
    <FirmwareVariant as ValueEnum>::from_str(recorded, false).map_err(|_| {
        anyhow::anyhow!("identity_admission=blocked reason=manifest_firmware_variant_unknown")
    })
}

/// Installs test firmware only when the operator names exactly the recorded variant.
pub(crate) fn admit_firmware_variant(
    recorded: FirmwareVariant,
    maybe_allowed: Option<FirmwareVariant>,
) -> Result<()> {
    match (recorded, maybe_allowed) {
        (FirmwareVariant::Production, None) => Ok(()),
        (FirmwareVariant::Production, Some(_)) => {
            bail!("identity_admission=blocked reason=test_firmware_allowance_without_test_package")
        }
        (_, None) => bail!("identity_admission=blocked reason=test_firmware_requires_allowance"),
        (recorded, Some(allowed)) if allowed != recorded => {
            bail!("identity_admission=blocked reason=test_firmware_variant_mismatch")
        }
        _ => Ok(()),
    }
}

/// A test variant installs only on the board it was built for.
pub(crate) fn require_firmware_variant_board(
    recorded: FirmwareVariant,
    board: BoardId,
) -> Result<()> {
    match recorded.maybe_test_identity() {
        Some((_, test_board)) if test_board != board => {
            bail!("identity_admission=blocked reason=test_firmware_board_mismatch")
        }
        _ => Ok(()),
    }
}

/// Fails closed when the application bytes disagree with the recorded variant: production
/// carries no test marker, and a test variant exactly its own.
pub(crate) fn require_firmware_variant_bytes(
    recorded: FirmwareVariant,
    application: &[u8],
) -> Result<()> {
    let contains = |marker: &[u8]| {
        application
            .windows(marker.len())
            .any(|window| window == marker)
    };
    let present: Vec<FirmwareVariant> = TEST_VARIANTS
        .into_iter()
        .filter(|variant| {
            variant
                .maybe_test_identity()
                .is_some_and(|(marker, _)| contains(marker))
        })
        .collect();
    if recorded == FirmwareVariant::Production {
        if present.is_empty() {
            return Ok(());
        }
        bail!("identity_admission=blocked reason=test_firmware_bytes_in_production_package");
    }
    if !present.contains(&recorded) {
        bail!("identity_admission=blocked reason=test_firmware_marker_missing");
    }
    if present.len() > 1 {
        bail!("identity_admission=blocked reason=test_firmware_marker_foreign");
    }
    Ok(())
}
