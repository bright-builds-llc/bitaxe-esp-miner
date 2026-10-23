//! Admission-time upper bound for the qualified pre-reset ASIC shutdown plan.

use bitaxe_asic::asic::AsicFamily;
use bitaxe_asic::bm1366::command::Bm1366AdapterAction;
use bitaxe_asic::bm1366::mining_ready::{
    safe_shutdown_command_actions, Bm1366MiningProfile, MiningReadyConfig,
};
use bitaxe_config::catalog::BoardCatalogEntry;
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

pub const UART_WRITE_BOUND_MS: u32 = 100;
pub const POLL_ALLOWANCE_MS: u32 = 50;
pub const OWNER_AND_GPIO_MARGIN_MS: u32 = 400;
/// The Conservative (400 MHz) tail.
pub const PRE_RESET_BOUND_MS: u32 = closed_pre_reset_tail_ms(400);
/// The upstream-default (485 MHz) tail.
pub const UPSTREAM_PRE_RESET_BOUND_MS: u32 = closed_pre_reset_tail_ms(485);
/// The highest Ultra 205 BM1366 catalog frequency (`bitaxe-config` catalog).
pub const CATALOG_MAXIMUM_FREQUENCY_MHZ: u16 = 575;
/// The tail of the catalog maximum: the shutdown bound outside every budget (ADR-0037).
pub const CATALOG_MAXIMUM_PRE_RESET_TAIL_MS: u32 =
    closed_pre_reset_tail_ms(CATALOG_MAXIMUM_FREQUENCY_MHZ);
const _: () = assert!(PRE_RESET_BOUND_MS == 15_550);
const _: () = assert!(UPSTREAM_PRE_RESET_BOUND_MS == 19_050);
const _: () = assert!(CATALOG_MAXIMUM_PRE_RESET_TAIL_MS == 22_550);

/// The closed pre-reset tail for a profile frequency: 1,550 ms fixed, plus
/// 250 ms per 6.25 MHz ramp step down to 50 MHz (one 100 ms frame, a 100 ms
/// delay and a 50 ms poll allowance).
#[must_use]
pub const fn closed_pre_reset_tail_ms(frequency_mhz: u16) -> u32 {
    let steps = (4 * frequency_mhz as u32).div_ceil(25).saturating_sub(8);
    1_550 + 250 * steps
}

/// The bound of the actual shutdown plan for `profile`, or none when it holds
/// an unbounded action.
#[must_use]
pub fn pre_reset_bound_ms(profile: Bm1366MiningProfile) -> Option<u32> {
    if option_env!("BITAXE_ASIC_UART_TRACE") == Some("1") {
        return None;
    }
    let config = MiningReadyConfig::ultra_205_profile(1, profile);
    safe_shutdown_command_actions(config)
        .ok()
        .and_then(|actions| maximum_pre_reset_ms(&actions))
}

/// ADR-0037 hardware admission: the frequency is a catalog-range BM1366 profile
/// whose ordered shutdown plan is bounded by its closed tail. The shutdown runs
/// after the budget, so no budget has to reserve it (decision 5, amended). The
/// fan is not part of admission: preparation proves it at 100%, then the
/// device's fan settings drive it (decision 6).
#[must_use]
pub fn profile_admissible(profile: MiningHardwareProfile) -> bool {
    Bm1366MiningProfile::ultra_205(profile.frequency()).is_ok_and(plan_is_bounded)
}

/// Rejects a changed or unsupported shutdown plan before any hardware
/// admission: the plan must be bounded by exactly its closed tail.
#[must_use]
pub fn plan_is_bounded(profile: Bm1366MiningProfile) -> bool {
    pre_reset_bound_ms(profile) == Some(closed_pre_reset_tail_ms(profile.frequency_mhz()))
}

/// A board's shutdown bound: the closed tail of its ASIC's highest catalog frequency. The
/// Ultra 205's is [`CATALOG_MAXIMUM_PRE_RESET_TAIL_MS`]; the Gamma 601's 625 MHz needs 24,550 ms.
#[must_use]
pub fn catalog_maximum_pre_reset_tail_ms_for(entry: BoardCatalogEntry) -> Option<u32> {
    let maximum_mhz = *entry.asic().frequency_options().iter().max()?;
    Some(closed_pre_reset_tail_ms(maximum_mhz))
}

/// [`profile_admissible`] for the boards whose shutdown plans the firmware models, the
/// single-chip Ultra 205 and Gamma 601: the profile's plan must be bounded by exactly its
/// closed tail. Any other board, even with the same chip, is not admitted.
#[must_use]
pub fn profile_admissible_for_board(
    entry: BoardCatalogEntry,
    profile: MiningHardwareProfile,
) -> bool {
    let modelled = matches!(entry.board_version(), "205" | "601") && entry.asic_count() == 1;
    if !modelled {
        return false;
    }
    match AsicFamily::from_model(entry.asic().model()) {
        Some(AsicFamily::Bm1366) => profile_admissible(profile),
        Some(AsicFamily::Bm1370) => {
            let frequency_mhz = profile.frequency().mhz();
            bm1370_pre_reset_bound_ms(frequency_mhz, entry.asic_count(), entry.asic().core_count())
                == Some(closed_pre_reset_tail_ms(frequency_mhz))
        }
        None => false,
    }
}

/// The bound of the BM1370's actual shutdown plan from `frequency_mhz`.
#[must_use]
pub fn bm1370_pre_reset_bound_ms(
    frequency_mhz: u16,
    chip_count: u8,
    core_count: u16,
) -> Option<u32> {
    if option_env!("BITAXE_ASIC_UART_TRACE") == Some("1") {
        return None;
    }
    bitaxe_asic::bm1370::shutdown::safe_shutdown_command_actions(
        frequency_mhz,
        chip_count,
        core_count,
    )
    .ok()
    .and_then(|actions| maximum_pre_reset_ms(&actions))
}

fn maximum_pre_reset_ms(actions: &[Bm1366AdapterAction]) -> Option<u32> {
    actions
        .iter()
        .try_fold(OWNER_AND_GPIO_MARGIN_MS, |total, action| {
            let bound = match action {
                Bm1366AdapterAction::WriteFrame(_) => UART_WRITE_BOUND_MS,
                Bm1366AdapterAction::DelayMs(milliseconds) => {
                    milliseconds.checked_add(POLL_ALLOWANCE_MS)?
                }
                Bm1366AdapterAction::WaitTxDone { timeout_ms } => {
                    timeout_ms.checked_add(POLL_ALLOWANCE_MS)?
                }
                _ => return None,
            };
            total.checked_add(bound)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_preset_tails_keep_their_admitted_values() {
        // Arrange / Act
        let tails = [
            pre_reset_bound_ms(Bm1366MiningProfile::CONSERVATIVE),
            pre_reset_bound_ms(Bm1366MiningProfile::UPSTREAM_DEFAULT),
        ];
        // Assert
        assert_eq!(tails, [Some(15_550), Some(19_050)]);
    }

    #[test]
    fn every_catalog_frequency_plan_matches_its_closed_tail() {
        // Arrange
        let frequencies = 400..=CATALOG_MAXIMUM_FREQUENCY_MHZ;
        // Act
        let mismatched: Vec<u16> = frequencies
            .filter(|mhz| {
                let frequency =
                    bitaxe_config::AsicFrequencyMhz::parse(i64::from(*mhz)).expect("frequency");
                let profile = Bm1366MiningProfile::ultra_205(frequency).expect("catalog range");
                !plan_is_bounded(profile)
            })
            .collect();
        // Assert
        assert!(mismatched.is_empty(), "mismatched: {mismatched:?}");
    }

    #[test]
    fn the_catalog_maximum_is_the_catalog_s_highest_frequency() {
        // Arrange / Act
        let options = bitaxe_config::ultra_205_catalog_entry()
            .asic()
            .frequency_options();
        // Assert
        assert_eq!(options.iter().max(), Some(&CATALOG_MAXIMUM_FREQUENCY_MHZ));
    }

    fn profile(mhz: i64, fan: i64) -> MiningHardwareProfile {
        MiningHardwareProfile::ultra_205_bm1366(mhz, 1_100, fan).expect("catalog profile")
    }

    #[test]
    fn every_catalog_frequency_is_admitted() {
        // Arrange / Act
        let admitted = [400, 485, 575].map(|mhz| profile_admissible(profile(mhz, 100)));
        // Assert
        assert_eq!(admitted, [true, true, true]);
    }

    #[test]
    fn the_profile_fan_duty_does_not_decide_admission() {
        // Arrange / Act
        let admitted = profile_admissible(profile(400, 80));
        // Assert
        assert!(admitted);
    }

    #[test]
    fn the_ultra_205_board_bound_is_its_catalog_maximum() {
        // Act
        let bound = catalog_maximum_pre_reset_tail_ms_for(bitaxe_config::ultra_205_catalog_entry());
        // Assert
        assert_eq!(bound, Some(CATALOG_MAXIMUM_PRE_RESET_TAIL_MS));
    }

    #[test]
    fn the_gamma_601_bound_covers_its_625_mhz() {
        // Act
        let bound = catalog_maximum_pre_reset_tail_ms_for(
            bitaxe_config::catalog::gamma_601_catalog_entry(),
        );
        // Assert
        assert_eq!(bound, Some(24_550));
    }

    #[test]
    fn every_gamma_601_catalog_frequency_plan_matches_its_closed_tail() {
        // Arrange
        let entry = bitaxe_config::catalog::gamma_601_catalog_entry();
        // Act
        let mismatched: Vec<u16> = (400..=625)
            .filter(|&mhz| {
                let profile = MiningHardwareProfile::for_board(entry, i64::from(mhz), 1_150, 100)
                    .expect("catalog profile");
                !profile_admissible_for_board(entry, profile)
            })
            .collect();
        // Assert
        assert!(mismatched.is_empty(), "mismatched: {mismatched:?}");
    }

    #[test]
    fn a_board_whose_chip_the_firmware_does_not_drive_is_not_admitted() {
        // Arrange
        let entry = bitaxe_config::board_catalog()
            .iter()
            .copied()
            .find(|entry| entry.asic().model() == "BM1368")
            .expect("BM1368 board");
        let profile = MiningHardwareProfile::for_board(entry, 490, 1_166, 100).expect("profile");
        // Act
        let admitted = profile_admissible_for_board(entry, profile);
        // Assert
        assert!(!admitted);
    }

    #[test]
    fn a_board_sharing_a_modelled_chip_is_not_admitted() {
        // Arrange: the six-chip Hex and the Gamma Turbo use the BM1366 and the BM1370.
        let others: Vec<BoardCatalogEntry> = bitaxe_config::board_catalog()
            .iter()
            .copied()
            .filter(|entry| matches!(entry.board_version(), "302" | "801"))
            .collect();
        // Act
        let admitted: Vec<bool> = others
            .iter()
            .map(|&entry| {
                let profile =
                    MiningHardwareProfile::for_board(entry, 485, 1_150, 100).expect("profile");
                profile_admissible_for_board(entry, profile)
            })
            .collect();
        // Assert
        assert_eq!(admitted, [false, false]);
    }

    #[test]
    fn unsupported_or_overflowing_actions_cannot_establish_a_bound() {
        assert_eq!(
            maximum_pre_reset_ms(&[Bm1366AdapterAction::HOLD_RESET_LOW]),
            None
        );
        assert_eq!(
            maximum_pre_reset_ms(&[Bm1366AdapterAction::DelayMs(u32::MAX)]),
            None
        );
    }
}
