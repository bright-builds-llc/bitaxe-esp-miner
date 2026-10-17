//! The crash-loop guard (ADR-0038 sub-decision 3, owner decisions 2026-10-17). The
//! device starts paused after [`CRASH_LOOP_RESETS`] abnormal resets in a row, each
//! before its boot reached [`STABLE_UPTIME_MS`] of uptime. Pure: the firmware reads
//! and writes the streak in NVS, which counts brownouts and survives power loss.
//!
//! A panic, a watchdog or a brownout reset extends the streak. A power-on, a normal
//! restart or any other reset clears it, as do a boot that stays up long enough and
//! the operator's resume.

use bitaxe_api::boot_identity::ResetReasonCategory;
use bitaxe_config::NvsWrite;

pub use bitaxe_api::CRASH_STREAK_KEY;

/// Abnormal resets in a row after which the device starts paused.
pub const CRASH_LOOP_RESETS: u16 = 3;
/// Uptime after which a boot counts as stable and clears the streak.
pub const STABLE_UPTIME_MS: u64 = 10 * 60_000;

/// The streak after a boot whose reset was `reset`.
#[must_use]
pub const fn next_streak(previous: u16, reset: ResetReasonCategory) -> u16 {
    match reset {
        ResetReasonCategory::Panic
        | ResetReasonCategory::Watchdog
        | ResetReasonCategory::Brownout => previous.saturating_add(1),
        ResetReasonCategory::PowerOn
        | ResetReasonCategory::SoftwareCpu
        | ResetReasonCategory::Other => 0,
    }
}

/// Whether a boot with this streak starts paused.
#[must_use]
pub const fn starts_paused(streak: u16) -> bool {
    streak >= CRASH_LOOP_RESETS
}

/// Whether a boot up for `uptime_ms` is stable enough to clear the streak.
#[must_use]
pub const fn boot_is_stable(uptime_ms: u64) -> bool {
    uptime_ms >= STABLE_UPTIME_MS
}

/// The device-owned write that saves `streak`.
#[must_use]
pub fn streak_write(streak: u16) -> NvsWrite {
    NvsWrite::u16(CRASH_STREAK_KEY, streak)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_watchdog_and_brownout_extend_the_streak() {
        // Arrange
        let abnormal = [
            ResetReasonCategory::Panic,
            ResetReasonCategory::Watchdog,
            ResetReasonCategory::Brownout,
        ];
        // Act / Assert
        for reset in abnormal {
            assert_eq!(next_streak(1, reset), 2, "{reset:?}");
        }
    }

    #[test]
    fn power_on_restart_and_other_resets_clear_the_streak() {
        // Arrange
        let normal = [
            ResetReasonCategory::PowerOn,
            ResetReasonCategory::SoftwareCpu,
            ResetReasonCategory::Other,
        ];
        // Act / Assert
        for reset in normal {
            assert_eq!(next_streak(2, reset), 0, "{reset:?}");
        }
    }

    #[test]
    fn the_third_abnormal_reset_in_a_row_starts_paused() {
        // Arrange
        let streak = [ResetReasonCategory::Panic; 3]
            .into_iter()
            .fold(0, next_streak);
        // Act / Assert
        assert!(!starts_paused(streak - 1));
        assert!(starts_paused(streak));
    }

    #[test]
    fn the_streak_saturates() {
        // Arrange / Act / Assert
        assert_eq!(
            next_streak(u16::MAX, ResetReasonCategory::Watchdog),
            u16::MAX
        );
    }

    #[test]
    fn ten_minutes_of_uptime_is_stable() {
        // Arrange / Act / Assert
        assert!(!boot_is_stable(STABLE_UPTIME_MS - 1));
        assert!(boot_is_stable(STABLE_UPTIME_MS));
    }

    #[test]
    fn the_streak_is_a_device_owned_write() {
        // Arrange / Act
        let plan = bitaxe_api::SettingsPersistencePlan::device_owned(vec![streak_write(3)]);
        // Assert
        assert!(plan.is_ok());
    }
}
