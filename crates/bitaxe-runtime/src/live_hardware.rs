//! Live hardware settings while a lease mines (ADR-0037 decision 4). Each owner tick
//! takes the next action that moves the applied hardware toward the saved settings,
//! in safe order: voltage up before frequency up, frequency down before voltage down,
//! and the nonce space for the frequency actually set. The decision is re-made every
//! tick, so a change that arrives mid-ramp is simply re-planned.

use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

const QUARTERS_PER_MHZ: u32 = 4;

/// What the hardware is running now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppliedHardware {
    pub frequency_quarter_mhz: u32,
    /// The frequency the ASIC's nonce space was last set for.
    pub nonce_space_quarter_mhz: u32,
    pub core_voltage_mv: u16,
}

impl AppliedHardware {
    /// The hardware a completed preparation leaves running.
    #[must_use]
    pub const fn prepared(profile: MiningHardwareProfile) -> Self {
        let frequency_quarter_mhz = profile.frequency().mhz() as u32 * QUARTERS_PER_MHZ;
        Self {
            frequency_quarter_mhz,
            nonce_space_quarter_mhz: frequency_quarter_mhz,
            core_voltage_mv: profile.core_voltage().millivolts(),
        }
    }

    /// The whole MHz the ordered shutdown must ramp down from: at or one step above
    /// the applied frequency, so the shutdown never starts below it.
    #[must_use]
    pub const fn shutdown_frequency_mhz(self) -> u16 {
        self.frequency_quarter_mhz.div_ceil(QUARTERS_PER_MHZ) as u16
    }

    /// What shutdown must assume while a ramp step to `step_quarter_mhz` is being
    /// written: the higher of the applied and attempted frequencies, since the write
    /// may or may not have reached the ASIC.
    #[must_use]
    pub const fn while_stepping_to(self, step_quarter_mhz: u32) -> Self {
        let frequency_quarter_mhz = if step_quarter_mhz > self.frequency_quarter_mhz {
            step_quarter_mhz
        } else {
            self.frequency_quarter_mhz
        };
        Self {
            frequency_quarter_mhz,
            ..self
        }
    }
}

/// The next live action, or none when the hardware matches the saved settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveHardwareAction {
    /// Raise the core voltage, then let it settle, before any frequency increase.
    RaiseVoltage { millivolts: u16 },
    /// Ramp the frequency toward this target, a bounded number of steps per tick.
    StepFrequency { target_quarter_mhz: u32 },
    /// Set the nonce space for the frequency now applied.
    SetNonceSpace { quarter_mhz: u32 },
    /// Lower the core voltage, only once the frequency is in place.
    LowerVoltage { millivolts: u16 },
}

#[must_use]
pub fn next_live_hardware_action(
    applied: AppliedHardware,
    target: MiningHardwareProfile,
) -> Option<LiveHardwareAction> {
    let target_quarter_mhz = u32::from(target.frequency().mhz()) * QUARTERS_PER_MHZ;
    let target_mv = target.core_voltage().millivolts();
    if target_mv > applied.core_voltage_mv {
        return Some(LiveHardwareAction::RaiseVoltage {
            millivolts: target_mv,
        });
    }
    if target_quarter_mhz != applied.frequency_quarter_mhz {
        return Some(LiveHardwareAction::StepFrequency { target_quarter_mhz });
    }
    if applied.nonce_space_quarter_mhz != applied.frequency_quarter_mhz {
        return Some(LiveHardwareAction::SetNonceSpace {
            quarter_mhz: applied.frequency_quarter_mhz,
        });
    }
    if target_mv < applied.core_voltage_mv {
        return Some(LiveHardwareAction::LowerVoltage {
            millivolts: target_mv,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(mhz: i64, mv: i64) -> MiningHardwareProfile {
        MiningHardwareProfile::ultra_205_bm1366(mhz, mv, 100).expect("catalog profile")
    }

    fn applied(mhz: u32, mv: u16) -> AppliedHardware {
        AppliedHardware::prepared(profile(i64::from(mhz), i64::from(mv)))
    }

    #[test]
    fn matching_settings_need_no_action() {
        assert_eq!(
            next_live_hardware_action(applied(485, 1_200), profile(485, 1_200)),
            None
        );
    }

    #[test]
    fn voltage_rises_before_the_frequency_does() {
        // Arrange / Act
        let action = next_live_hardware_action(applied(485, 1_200), profile(525, 1_250));
        // Assert
        assert_eq!(
            action,
            Some(LiveHardwareAction::RaiseVoltage { millivolts: 1_250 })
        );
    }

    #[test]
    fn frequency_ramps_once_the_voltage_is_in_place() {
        // Arrange / Act
        let action = next_live_hardware_action(applied(485, 1_250), profile(525, 1_250));
        // Assert
        assert_eq!(
            action,
            Some(LiveHardwareAction::StepFrequency {
                target_quarter_mhz: 2_100
            })
        );
    }

    #[test]
    fn frequency_falls_before_the_voltage_does() {
        // Arrange / Act
        let action = next_live_hardware_action(applied(525, 1_250), profile(485, 1_200));
        // Assert
        assert_eq!(
            action,
            Some(LiveHardwareAction::StepFrequency {
                target_quarter_mhz: 1_940
            })
        );
    }

    #[test]
    fn the_nonce_space_follows_a_completed_ramp_before_the_voltage_falls() {
        // Arrange: the ramp reached 485 MHz; the nonce space is still for 525.
        let mut ramped = applied(525, 1_250);
        ramped.frequency_quarter_mhz = 1_940;
        // Act
        let action = next_live_hardware_action(ramped, profile(485, 1_200));
        // Assert
        assert_eq!(
            action,
            Some(LiveHardwareAction::SetNonceSpace { quarter_mhz: 1_940 })
        );
    }

    #[test]
    fn voltage_falls_last() {
        // Arrange / Act
        let action = next_live_hardware_action(applied(485, 1_250), profile(485, 1_200));
        // Assert
        assert_eq!(
            action,
            Some(LiveHardwareAction::LowerVoltage { millivolts: 1_200 })
        );
    }

    #[test]
    fn the_shutdown_starts_at_or_one_step_above_an_off_grid_frequency() {
        // Arrange: 406.25 MHz mid-ramp.
        let mut ramping = applied(400, 1_100);
        ramping.frequency_quarter_mhz = 1_625;
        // Act / Assert
        assert_eq!(ramping.shutdown_frequency_mhz(), 407);
    }

    #[test]
    fn frequency_up_with_voltage_down_ramps_at_the_old_voltage_first() {
        // Arrange / Act
        let action = next_live_hardware_action(applied(485, 1_250), profile(525, 1_200));
        // Assert
        assert_eq!(
            action,
            Some(LiveHardwareAction::StepFrequency {
                target_quarter_mhz: 2_100
            })
        );
    }

    #[test]
    fn frequency_down_with_voltage_up_raises_the_voltage_first() {
        // Arrange / Act
        let action = next_live_hardware_action(applied(525, 1_200), profile(485, 1_250));
        // Assert
        assert_eq!(
            action,
            Some(LiveHardwareAction::RaiseVoltage { millivolts: 1_250 })
        );
    }

    #[test]
    fn an_on_grid_frequency_shuts_down_from_itself() {
        // Arrange
        let at_575 = applied(575, 1_300);
        // Act / Assert
        assert_eq!(at_575.shutdown_frequency_mhz(), 575);
    }

    #[test]
    fn shutdown_assumes_the_higher_frequency_while_a_step_is_written() {
        // Arrange
        let at_485 = applied(485, 1_200);
        // Act
        let rising = at_485.while_stepping_to(1_950).frequency_quarter_mhz;
        let falling = at_485.while_stepping_to(1_925).frequency_quarter_mhz;
        // Assert
        assert_eq!((rising, falling), (1_950, 1_940));
    }

    #[test]
    fn any_sequence_of_changes_keeps_the_safe_ordering() {
        // Arrange: targets change mid-ramp; frequency moves one ramp step per action.
        let targets = [(525, 1_250), (450, 1_150), (575, 1_300), (400, 1_100)];
        let mut hardware = applied(485, 1_200);
        // Act / Assert
        for (mhz, mv) in targets {
            let target = profile(mhz, mv);
            for _ in 0..200 {
                let Some(action) = next_live_hardware_action(hardware, target) else {
                    break;
                };
                match action {
                    LiveHardwareAction::RaiseVoltage { millivolts } => {
                        hardware.core_voltage_mv = millivolts;
                    }
                    LiveHardwareAction::StepFrequency { target_quarter_mhz } => {
                        // The voltage is never below the target while the frequency moves.
                        assert!(hardware.core_voltage_mv >= target.core_voltage().millivolts());
                        let step = if target_quarter_mhz > hardware.frequency_quarter_mhz {
                            (hardware.frequency_quarter_mhz + 25).min(target_quarter_mhz)
                        } else {
                            hardware
                                .frequency_quarter_mhz
                                .saturating_sub(25)
                                .max(target_quarter_mhz)
                        };
                        hardware.frequency_quarter_mhz = step;
                    }
                    LiveHardwareAction::SetNonceSpace { quarter_mhz } => {
                        hardware.nonce_space_quarter_mhz = quarter_mhz;
                    }
                    LiveHardwareAction::LowerVoltage { millivolts } => {
                        // The voltage only falls once the frequency and nonce space are done.
                        assert_eq!(
                            hardware.frequency_quarter_mhz,
                            u32::from(target.frequency().mhz()) * 4
                        );
                        assert_eq!(
                            hardware.nonce_space_quarter_mhz,
                            hardware.frequency_quarter_mhz
                        );
                        hardware.core_voltage_mv = millivolts;
                    }
                }
            }
            assert_eq!(next_live_hardware_action(hardware, target), None);
        }
    }
}
