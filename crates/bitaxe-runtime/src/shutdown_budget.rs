//! Admission-time upper bound for the qualified pre-reset ASIC shutdown plan.

use bitaxe_asic::bm1366::command::Bm1366AdapterAction;
use bitaxe_asic::bm1366::mining_ready::{
    safe_shutdown_command_actions, Bm1366MiningProfile, MiningReadyConfig,
};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

pub const UART_WRITE_BOUND_MS: u32 = 100;
pub const POLL_ALLOWANCE_MS: u32 = 50;
pub const OWNER_AND_GPIO_MARGIN_MS: u32 = 400;
/// The Conservative (400 MHz) tail.
pub const PRE_RESET_BOUND_MS: u32 = closed_pre_reset_tail_ms(400);
/// The upstream-default (485 MHz) tail, which a soak reserves (ADR-0033).
pub const UPSTREAM_PRE_RESET_BOUND_MS: u32 = closed_pre_reset_tail_ms(485);
/// The highest Ultra 205 BM1366 catalog frequency (`bitaxe-config` catalog).
pub const CATALOG_MAXIMUM_FREQUENCY_MHZ: u16 = 575;
/// The tail of the catalog maximum, which ADR-0037 budgets will reserve.
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

/// ADR-0037 hardware admission: the fan is at 100% (until the fan policy
/// task), the frequency is a catalog-range BM1366 profile, and its ordered
/// shutdown fits the tail the caller reserved.
#[must_use]
pub fn profile_admissible(profile: MiningHardwareProfile, reserved_tail_ms: u32) -> bool {
    profile.fan_duty().percent() == 100
        && Bm1366MiningProfile::ultra_205(profile.frequency())
            .is_ok_and(|asic| profile_fits_tail(asic, reserved_tail_ms))
}

/// Whether `profile`'s ordered shutdown is bounded and fits `reserved_tail_ms`.
#[must_use]
pub fn profile_fits_tail(profile: Bm1366MiningProfile, reserved_tail_ms: u32) -> bool {
    plan_is_bounded(profile)
        && closed_pre_reset_tail_ms(profile.frequency_mhz()) <= reserved_tail_ms
}

/// Rejects a changed or unsupported shutdown plan before any hardware
/// admission: the plan must be bounded by exactly its closed tail.
#[must_use]
pub fn plan_is_bounded(profile: Bm1366MiningProfile) -> bool {
    pre_reset_bound_ms(profile) == Some(closed_pre_reset_tail_ms(profile.frequency_mhz()))
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
    fn a_normal_budget_admits_exactly_400_mhz() {
        // Arrange
        let reserved = PRE_RESET_BOUND_MS;
        // Act
        let admitted = [400, 401].map(|mhz| profile_admissible(profile(mhz, 100), reserved));
        // Assert
        assert_eq!(admitted, [true, false]);
    }

    #[test]
    fn a_soak_budget_admits_up_to_487_mhz() {
        // Arrange: 487 MHz shares 485's 70-step ramp; 488 needs one more step.
        let reserved = UPSTREAM_PRE_RESET_BOUND_MS;
        // Act
        let admitted = [485, 487, 488].map(|mhz| profile_admissible(profile(mhz, 100), reserved));
        // Assert
        assert_eq!(admitted, [true, true, false]);
    }

    #[test]
    fn the_catalog_maximum_tail_admits_every_catalog_frequency() {
        // Arrange
        let reserved = CATALOG_MAXIMUM_PRE_RESET_TAIL_MS;
        // Act
        let admitted = [400, 575].map(|mhz| profile_admissible(profile(mhz, 100), reserved));
        // Assert
        assert_eq!(admitted, [true, true]);
    }

    #[test]
    fn a_fan_below_100_percent_is_not_admitted_yet() {
        // Arrange / Act
        let admitted = profile_admissible(profile(400, 80), PRE_RESET_BOUND_MS);
        // Assert
        assert!(!admitted);
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
