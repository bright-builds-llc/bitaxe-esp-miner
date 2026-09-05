//! Admission-time upper bound for the qualified pre-reset ASIC shutdown plan.

use bitaxe_asic::bm1366::command::Bm1366AdapterAction;
use bitaxe_asic::bm1366::mining_ready::{
    safe_shutdown_command_actions, Bm1366MiningProfile, MiningReadyConfig,
};

pub(crate) const UART_WRITE_BOUND_MS: u32 = 100;
pub(crate) const POLL_ALLOWANCE_MS: u32 = 50;
pub(crate) const OWNER_AND_GPIO_MARGIN_MS: u32 = 400;
pub(crate) const PRE_RESET_BOUND_MS: u32 = 15_550;

/// Rejects changed or unsupported shutdown actions before any hardware admission.
pub(crate) fn conservative_plan_is_bounded() -> bool {
    if option_env!("BITAXE_ASIC_UART_TRACE") == Some("1") {
        return false;
    }
    let config = MiningReadyConfig::ultra_205_profile(1, Bm1366MiningProfile::Conservative);
    safe_shutdown_command_actions(config)
        .ok()
        .and_then(|actions| maximum_pre_reset_ms(&actions))
        == Some(PRE_RESET_BOUND_MS)
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
    fn closed_conservative_plan_reserves_the_entire_pre_reset_tail() {
        assert!(conservative_plan_is_bounded());
        assert!(PRE_RESET_BOUND_MS < 30_000);
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
