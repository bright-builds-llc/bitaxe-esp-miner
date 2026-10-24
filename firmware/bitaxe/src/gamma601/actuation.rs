//! The 601's safety actuation owner (ADR-0041). The safe-boot thread owns the bus, so it applies
//! the queued commands between its sweeps: fan commands by the authority the 205's owner uses,
//! and, in the mining variant only, the TPS546 regulator's.

use crate::safety_adapter::{
    self, gamma601 as hardware, FanDecision, RuntimeI2cOwner, SafetyActuationCommand,
    SafetyActuationReply,
};
use bitaxe_safety::gamma601_safe_boot::SafeBootReport;

/// What the owner's commands left behind.
#[derive(Debug, Default)]
pub(super) struct ActuationState {
    /// The last fan duty an owner commanded.
    pub(super) maybe_fan_duty: Option<u8>,
    /// The output is deliberately on: the safe boot neither commands it off nor holds the fan
    /// at full for it.
    pub(super) regulator_on: bool,
    /// The permit the output was turned on under; the output goes off once it no longer holds.
    #[cfg(feature = "gamma601-mining")]
    maybe_permit: Option<crate::production_mining_session::revocation::WorkPermit>,
    /// The enable awaiting `VerifyRegulator`: its target, its baseline and when it was made.
    #[cfg(feature = "gamma601-mining")]
    maybe_pending_verify: Option<PendingVerify>,
}

/// An enable not yet judged settled.
#[cfg(feature = "gamma601-mining")]
#[derive(Debug, Clone, Copy)]
struct PendingVerify {
    millivolts: u16,
    baseline: bitaxe_safety::tps546::StatusWord,
    enabled_at_ms: u64,
}

/// While the output is on, the thread checks its authority at least this often, so a lapsed
/// heartbeat (2.8 s) starts the off within the 3 s shutdown bound.
pub(super) const ON_AUTHORITY_CHECK_MS: u64 = 100;

/// An enable not verified within this long is turned off: its owner has gone.
#[cfg(feature = "gamma601-mining")]
const VERIFY_DEADLINE_MS: u64 = 5_000;

impl ActuationState {
    /// The duty the sweeps keep: the owner's, within the safe boot's rule.
    pub(super) fn fan_duty(&self, report: &SafeBootReport) -> u8 {
        report.fan_duty_with(self.maybe_fan_duty, self.regulator_on)
    }

    /// Whether the output is on and judged settled, so its `OFF` bit is a fault.
    pub(super) fn output_verified_on(&self) -> bool {
        #[cfg(feature = "gamma601-mining")]
        return self.regulator_on && self.maybe_pending_verify.is_none();
        #[cfg(not(feature = "gamma601-mining"))]
        false
    }
}

/// Turns the output off once its authority lapses: the permit it was turned on under is
/// revoked (the Worker's heartbeat deadline passed, or an unsafe observation revoked it), or its
/// enable was never verified. Called on every pass of the thread, at least every sweep, so the
/// output never outlives its owner.
pub(super) fn enforce_authority(
    owner: &mut RuntimeI2cOwner<'_>,
    state: &mut ActuationState,
    report: SafeBootReport,
) {
    #[cfg(feature = "gamma601-mining")]
    {
        use crate::production_mining_session::revocation;
        if !state.regulator_on {
            return;
        }
        let now_ms = crate::runtime_uptime::millis();
        revocation::check_deadline(now_ms);
        let permitted = state.maybe_permit.is_some_and(revocation::permits_work);
        let verify_overdue = state.maybe_pending_verify.is_some_and(|pending| {
            now_ms.saturating_sub(pending.enabled_at_ms) > VERIFY_DEADLINE_MS
        });
        if !permitted || verify_overdue {
            log::warn!(
                "gamma601_regulator=off reason={}",
                if permitted {
                    "verify_overdue"
                } else {
                    "permit_revoked"
                }
            );
            let _ = turn_off(owner, state, report);
        }
    }
    #[cfg(not(feature = "gamma601-mining"))]
    let _ = (owner, state, report);
}

/// Whether `command` drives the regulator, so the caller refreshes the observations around it.
pub(super) fn drives_regulator(command: SafetyActuationCommand) -> bool {
    matches!(
        safety_adapter::decide_fan_duty(command),
        FanDecision::NotFan
    )
}

/// Applies one command. A fan command writes the owner's duty within the safe boot's rule, so
/// while the output is neither deliberately on nor confirmed off the fan stays full.
pub(super) fn apply(
    owner: &mut RuntimeI2cOwner<'_>,
    state: &mut ActuationState,
    report: SafeBootReport,
    command: SafetyActuationCommand,
) -> SafetyActuationReply {
    match safety_adapter::decide_fan_duty(command) {
        FanDecision::Write(duty) => {
            state.maybe_fan_duty = Some(duty);
            match hardware::write_fan_duty(owner, state.fan_duty(&report)) {
                Ok(()) => SafetyActuationReply::Applied,
                Err(_) => SafetyActuationReply::HardwareWriteFailed,
            }
        }
        FanDecision::Reply(reply) => reply,
        FanDecision::NotFan => apply_regulator(owner, state, report, command),
    }
}

#[cfg(not(feature = "gamma601-mining"))]
fn apply_regulator(
    _owner: &mut RuntimeI2cOwner<'_>,
    _state: &mut ActuationState,
    _report: SafeBootReport,
    _command: SafetyActuationCommand,
) -> SafetyActuationReply {
    // A production 601 drives no regulator, and the 601 has no DS4432U.
    SafetyActuationReply::Refused
}

#[cfg(feature = "gamma601-mining")]
fn apply_regulator(
    owner: &mut RuntimeI2cOwner<'_>,
    state: &mut ActuationState,
    report: SafeBootReport,
    command: SafetyActuationCommand,
) -> SafetyActuationReply {
    use super::regulator;
    use crate::production_mining_session::revocation::permits_work;
    let outcome = match command {
        SafetyActuationCommand::PowerOnRegulator { millivolts, permit } => {
            // Only a Worker generation's permit lapses with its heartbeat, so only one may turn
            // the output on.
            if permit.maybe_generation().is_none() || !permits_work(permit) {
                return refuse(owner, state, report);
            }
            regulator::power_on(owner, report, millivolts).map(|baseline| {
                state.regulator_on = true;
                state.maybe_permit = Some(permit);
                // An owner's earlier duty does not carry over to the newly enabled output.
                state.maybe_fan_duty = None;
                state.maybe_pending_verify = Some(PendingVerify {
                    millivolts,
                    baseline,
                    enabled_at_ms: crate::runtime_uptime::millis(),
                });
            })
        }
        // A verify with no enable pending, such as a repeated one, turns the output off.
        SafetyActuationCommand::VerifyRegulator => match state.maybe_pending_verify.take() {
            Some(pending) if state.regulator_on => {
                regulator::verify_on(owner, report, pending.millivolts, pending.baseline)
            }
            _ => Err("no_enable_to_verify"),
        },
        SafetyActuationCommand::SetRegulatorVoltage { millivolts, permit } => {
            if !state.regulator_on {
                return SafetyActuationReply::HardwareWriteFailed;
            }
            if !permits_work(permit) {
                return refuse(owner, state, report);
            }
            regulator::set_vout(owner, millivolts)
        }
        SafetyActuationCommand::TurnRegulatorOff => {
            return turn_off(owner, state, report);
        }
        // The 205's DS4432U commands: the 601 has none.
        _ => return SafetyActuationReply::Refused,
    };
    match outcome {
        Ok(()) => SafetyActuationReply::Applied,
        Err(reason) => {
            log::warn!("gamma601_regulator=fault category=command reason={reason}");
            // Any failed step may have left the output on: turn it off.
            let _ = turn_off(owner, state, report);
            SafetyActuationReply::HardwareWriteFailed
        }
    }
}

/// A command whose permit no longer holds: nothing it asked for is done, and an output still on
/// under a lapsed authority goes off.
#[cfg(feature = "gamma601-mining")]
fn refuse(
    owner: &mut RuntimeI2cOwner<'_>,
    state: &mut ActuationState,
    report: SafeBootReport,
) -> SafetyActuationReply {
    if state.regulator_on {
        let _ = turn_off(owner, state, report);
    }
    SafetyActuationReply::Refused
}

/// Turns the output off; the safe boot re-asserts off from now on, confirmed or not.
#[cfg(feature = "gamma601-mining")]
fn turn_off(
    owner: &mut RuntimeI2cOwner<'_>,
    state: &mut ActuationState,
    report: SafeBootReport,
) -> SafetyActuationReply {
    state.regulator_on = false;
    state.maybe_permit = None;
    state.maybe_fan_duty = None;
    state.maybe_pending_verify = None;
    match super::regulator::turn_off_and_confirm(owner, report) {
        Some(bitaxe_safety::tps546::OffState::Confirmed) => SafetyActuationReply::Applied,
        _ => SafetyActuationReply::HardwareWriteFailed,
    }
}
