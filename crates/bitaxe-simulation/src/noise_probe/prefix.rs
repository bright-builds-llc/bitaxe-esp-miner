//! Diagnostic prefixes stop before certificate verification and never qualify a handshake.
use super::{NoisePhase, NoiseProbeError};
use crate::v2::exchange::{SyntheticRng, PRIVATE, PUBLIC};
use bitaxe_stratum::v2::noise::NoiseInitiator;
use noise_sv2::Responder;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u32)]
pub enum NoisePrefix {
    Entry = 101,
    BeforeInitiator = 102,
    AfterInitiator = 103,
    AfterActOne = 105,
    BeforeResponder = 106,
    AfterResponder = 107,
    AfterActTwo = 109,
    BeforeCompletion = 110,
}
impl TryFrom<u32> for NoisePrefix {
    type Error = NoiseProbeError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            101 => Ok(Self::Entry),
            102 => Ok(Self::BeforeInitiator),
            103 => Ok(Self::AfterInitiator),
            105 => Ok(Self::AfterActOne),
            106 => Ok(Self::BeforeResponder),
            107 => Ok(Self::AfterResponder),
            109 => Ok(Self::AfterActTwo),
            110 => Ok(Self::BeforeCompletion),
            _ => Err(NoiseProbeError::PrefixUnsupported),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoisePrefixEvent {
    Checkpoint(NoisePhase),
    BoundaryReached(NoisePrefix),
    Released,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NoisePrefixResult {
    pub schema: &'static str,
    pub seed: u64,
    pub selected_stop: u32,
    pub boundary_reached: bool,
    pub resources_released: bool,
    pub subset_only: bool,
    pub authenticated: bool,
    pub frame_round_trip: bool,
    pub full_probe_qualified: bool,
    pub hardware_qualified: bool,
}

/// All stops share this compiled path; only the requested runtime boundary changes.
#[inline(never)]
pub fn run_until(
    seed: u64,
    prefix: NoisePrefix,
    observe: &mut dyn FnMut(NoisePrefixEvent),
) -> Result<NoisePrefixResult, NoiseProbeError> {
    let outcome = handshake_until_prefix(seed, prefix, observe);
    observe(NoisePrefixEvent::Released);
    outcome?;
    Ok(NoisePrefixResult {
        schema: "bitaxe-noise-prefix-v1",
        seed,
        selected_stop: prefix as u32,
        boundary_reached: true,
        resources_released: true,
        subset_only: true,
        authenticated: false,
        frame_round_trip: false,
        full_probe_qualified: false,
        hardware_qualified: false,
    })
}

fn stop_at(
    prefix: NoisePrefix,
    phase: NoisePhase,
    observe: &mut dyn FnMut(NoisePrefixEvent),
) -> bool {
    observe(NoisePrefixEvent::Checkpoint(phase));
    if prefix as u32 != phase as u32 {
        return false;
    }
    // Owners remain in the caller until this observer returns and the caller exits.
    observe(NoisePrefixEvent::BoundaryReached(prefix));
    true
}

#[inline(never)]
fn handshake_until_prefix(
    seed: u64,
    prefix: NoisePrefix,
    observe: &mut dyn FnMut(NoisePrefixEvent),
) -> Result<(), NoiseProbeError> {
    if stop_at(prefix, NoisePhase::Entry, observe) {
        return Ok(());
    }
    let mut client_rng = SyntheticRng::new(seed);
    let mut responder_rng = SyntheticRng::new(seed ^ 0xaaccee);
    if stop_at(prefix, NoisePhase::BeforeInitiator, observe) {
        return Ok(());
    }
    let mut client = NoiseInitiator::new(Some(PUBLIC), &mut client_rng)
        .map_err(|_| NoiseProbeError::Initiator)?;
    if stop_at(prefix, NoisePhase::AfterInitiator, observe) {
        return Ok(());
    }
    observe(NoisePrefixEvent::Checkpoint(NoisePhase::BeforeActOne));
    let act_one = client.act_one().map_err(|_| NoiseProbeError::ActOne)?;
    if stop_at(prefix, NoisePhase::AfterActOne, observe) {
        return Ok(());
    }
    if stop_at(prefix, NoisePhase::BeforeResponder, observe) {
        return Ok(());
    }
    let mut responder = Responder::from_authority_kp_with_rng(
        &PUBLIC,
        &PRIVATE,
        Duration::from_secs(3600),
        &mut responder_rng,
    )
    .map_err(|_| NoiseProbeError::Responder)?;
    if stop_at(prefix, NoisePhase::AfterResponder, observe) {
        return Ok(());
    }
    observe(NoisePrefixEvent::Checkpoint(NoisePhase::BeforeActTwo));
    let (_act_two, _server) = responder
        .step_1_with_now_rng(act_one, 100, &mut responder_rng)
        .map_err(|_| NoiseProbeError::ActTwo)?;
    if stop_at(prefix, NoisePhase::AfterActTwo, observe) {
        return Ok(());
    }
    if stop_at(prefix, NoisePhase::BeforeCompletion, observe) {
        return Ok(());
    }
    Err(NoiseProbeError::PrefixUnsupported)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn each_prefix_stops_before_completion_and_separates_release() {
        // Arrange
        for stop in [101, 102, 103, 105, 106, 107, 109, 110] {
            let prefix = NoisePrefix::try_from(stop).expect("admitted prefix");
            let mut events = Vec::new();
            // Act
            let result =
                run_until(1, prefix, &mut |event| events.push(event)).expect("real crypto prefix");
            // Assert
            let checkpoints: Vec<_> = events
                .iter()
                .filter_map(|event| match event {
                    NoisePrefixEvent::Checkpoint(phase) => Some(*phase as u32),
                    _ => None,
                })
                .collect();
            assert_eq!(checkpoints, (101..=stop).collect::<Vec<_>>());
            assert_eq!(
                events[events.len() - 2],
                NoisePrefixEvent::BoundaryReached(prefix)
            );
            assert_eq!(events.last(), Some(&NoisePrefixEvent::Released));
            assert!(result.boundary_reached && result.resources_released && result.subset_only);
            assert!(
                !result.authenticated
                    && !result.frame_round_trip
                    && !result.full_probe_qualified
                    && !result.hardware_qualified
            );
        }
    }
    #[test]
    fn completion_and_unknown_stops_are_never_admitted() {
        // Arrange
        let rejected = [0, 100, 104, 108, 111, 112, 113, 114, u32::MAX];
        // Act / Assert
        for stop in rejected {
            assert_eq!(
                NoisePrefix::try_from(stop),
                Err(NoiseProbeError::PrefixUnsupported)
            );
        }
    }
}
