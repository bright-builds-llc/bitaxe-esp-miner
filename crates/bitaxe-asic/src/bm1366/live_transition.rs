//! Live frequency changes while mining (ADR-0037 decision 4), as upstream's power
//! management task does: the same 6.25 MHz ramp as preparation and shutdown,
//! followed by the nonce space for the new frequency
//! (`reference/esp-miner/main/tasks/power_management_task.c`).

use super::command::{Bm1366AdapterAction, Bm1366Command, NonceSpacePlan};
use super::frequency_voltage::actual_frequency_mhz;
use super::mining_ready::{
    encode_commands, frequency_transition_targets, hash_counting_number, transition_plan,
    MiningReadyConfig, FREQ_RAMP_DELAY_MS,
};

/// One ramp step of a live change: the frequency it sets and its encoded actions,
/// including the 100 ms settle that follows every on-grid step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveFrequencyStep {
    pub quarter_mhz: u32,
    pub actions: Vec<Bm1366AdapterAction>,
}

/// The ramp from `from_quarter_mhz` to `to_quarter_mhz`, one entry per frequency set.
pub fn live_frequency_steps(
    from_quarter_mhz: u32,
    to_quarter_mhz: u32,
) -> Result<Vec<LiveFrequencyStep>, &'static str> {
    frequency_transition_targets(from_quarter_mhz, to_quarter_mhz)
        .into_iter()
        .map(|(quarter_mhz, settle)| {
            let mut commands = vec![Bm1366Command::SetFrequency(transition_plan(quarter_mhz)?)];
            if settle {
                commands.push(Bm1366Command::DelayMs(FREQ_RAMP_DELAY_MS));
            }
            Ok(LiveFrequencyStep {
                quarter_mhz,
                actions: encode_commands(&commands)?,
            })
        })
        .collect()
}

/// The nonce space for `quarter_mhz`, computed from the PLL's actual output as the
/// mining-ready and shutdown plans do.
pub fn live_nonce_space_actions(
    config: MiningReadyConfig,
    quarter_mhz: u32,
) -> Result<Vec<Bm1366AdapterAction>, &'static str> {
    let hash_counting = hash_counting_number(
        config.nonce_percent,
        actual_frequency_mhz(transition_plan(quarter_mhz)?),
        config.asic_count,
        config.core_count,
    );
    let mut actions = encode_commands(&[Bm1366Command::SetNonceSpace(NonceSpacePlan {
        hash_counting_number: hash_counting,
    })])?;
    actions.push(Bm1366AdapterAction::WAIT_TX_DONE);
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bm1366::mining_ready::Bm1366MiningProfile;

    fn quarters(steps: &[LiveFrequencyStep]) -> Vec<u32> {
        steps.iter().map(|step| step.quarter_mhz).collect()
    }

    #[test]
    fn raising_frequency_ramps_up_in_6_25_mhz_steps() {
        // Arrange / Act
        let steps = live_frequency_steps(485 * 4, 500 * 4).expect("ramp");
        // Assert: 485 rounds down to the 481.25 grid, then 487.5, 493.75, 500.
        assert_eq!(quarters(&steps), vec![1_950, 1_975, 2_000]);
    }

    #[test]
    fn lowering_frequency_ramps_down_and_ends_exactly_on_target() {
        // Arrange / Act
        let steps = live_frequency_steps(500 * 4, 485 * 4).expect("ramp");
        // Assert
        assert_eq!(quarters(&steps), vec![1_975, 1_950, 1_940]);
    }

    #[test]
    fn a_change_smaller_than_one_step_is_a_single_write() {
        // Arrange / Act
        let steps = live_frequency_steps(485 * 4, 487 * 4).expect("ramp");
        // Assert
        assert_eq!(quarters(&steps), vec![1_948]);
    }

    #[test]
    fn every_on_grid_step_carries_its_settle_delay() {
        // Arrange / Act
        let steps = live_frequency_steps(400 * 4, 425 * 4).expect("ramp");
        // Assert
        assert!(steps
            .iter()
            .all(|step| step.actions.contains(&Bm1366AdapterAction::DelayMs(100))));
    }

    #[test]
    fn no_change_has_no_steps() {
        assert!(live_frequency_steps(485 * 4, 485 * 4)
            .expect("ramp")
            .is_empty());
    }

    #[test]
    fn the_nonce_space_follows_the_new_frequency() {
        // Arrange
        let config = MiningReadyConfig::ultra_205_profile(1, Bm1366MiningProfile::UPSTREAM_DEFAULT);
        // Act
        let at_485 = live_nonce_space_actions(config, 485 * 4).expect("nonce space");
        let at_575 = live_nonce_space_actions(config, 575 * 4).expect("nonce space");
        // Assert
        assert_ne!(at_485, at_575);
        assert_eq!(at_485.last(), Some(&Bm1366AdapterAction::WAIT_TX_DONE));
    }

    #[test]
    fn live_steps_from_50_mhz_encode_exactly_the_preparation_ramp() {
        // Arrange
        let preparation = encode_commands(
            &crate::bm1366::mining_ready::mining_ready_frequency_ramp_commands(
                Bm1366MiningProfile::UPSTREAM_DEFAULT,
            )
            .expect("ramp"),
        )
        .expect("encoded");
        // Act
        let live: Vec<Bm1366AdapterAction> = live_frequency_steps(200, 485 * 4)
            .expect("ramp")
            .into_iter()
            .flat_map(|step| step.actions)
            .collect();
        // Assert
        assert_eq!(live, preparation);
    }
}
