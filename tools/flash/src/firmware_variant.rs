//! Test-only firmware variants (ADR-0040).
//!
//! A package records the variant compiled into its application. Installing a
//! non-production variant requires the operator to name it with
//! `--allow-test-firmware <variant>`, and the application bytes must agree with
//! the recorded variant.

use crate::*;

/// Boot-line literal compiled into the ELF only by the overheat-trip-test variant.
const OVERHEAT_TRIP_TEST_MARKER: &[u8] = b"firmware_variant=overheat-trip-test";

/// Firmware variant compiled into a package; mirrors `//firmware/bitaxe:firmware_variant`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum FirmwareVariant {
    /// The default firmware, and the variant of every package that predates ADR-0040.
    Production,
    /// Lowered local overheat trip for hardware checks; never released.
    OverheatTripTest,
}

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

/// Fails closed when the application bytes disagree with the recorded variant.
pub(crate) fn require_firmware_variant_bytes(
    recorded: FirmwareVariant,
    application: &[u8],
) -> Result<()> {
    let has_marker = application
        .windows(OVERHEAT_TRIP_TEST_MARKER.len())
        .any(|window| window == OVERHEAT_TRIP_TEST_MARKER);
    match (recorded, has_marker) {
        (FirmwareVariant::Production, true) => {
            bail!("identity_admission=blocked reason=test_firmware_bytes_in_production_package")
        }
        (FirmwareVariant::OverheatTripTest, false) => {
            bail!("identity_admission=blocked reason=test_firmware_marker_missing")
        }
        _ => Ok(()),
    }
}
