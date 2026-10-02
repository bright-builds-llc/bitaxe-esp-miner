//! Synthetic Noise-only diagnostic. No controller, authority grant, job or ASIC exists here.
use bitaxe_stratum::v2::{
    frame::{Frame, FrameHeader, FRAME_HEADER_LEN},
    noise::{
        NoiseCompletionFailure, NoiseInitiator, NoiseTransport, ACT_TWO_LEN, ENCRYPTED_HEADER_LEN,
    },
};
use noise_sv2::{NoiseCodec, Responder};
use serde::Serialize;
use std::time::Duration;

use crate::v2::exchange::{SyntheticRng, OTHER_PUBLIC, PRIVATE, PUBLIC};

pub mod prefix;

#[cfg(test)]
mod tests;

/// Fixed diagnostic checkpoints. Observer adapters must neither allocate nor alter decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NoisePhase {
    Entry = 101,
    BeforeInitiator = 102,
    AfterInitiator = 103,
    BeforeActOne = 104,
    AfterActOne = 105,
    BeforeResponder = 106,
    AfterResponder = 107,
    BeforeActTwo = 108,
    AfterActTwo = 109,
    BeforeCompletion = 110,
    AfterCompletion = 111,
    BeforeFrame = 112,
    AfterFrame = 113,
    Released = 114,
}

/// One alteration per diagnostic; synthetic randomness and trust must never reach a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseFault {
    None,
    WrongAuthority,
    TamperedActTwo,
    TruncatedActTwo,
    TamperedFrame,
    TruncatedFrame,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NoiseProbeError {
    #[error("noise_prefix_unsupported")]
    PrefixUnsupported,
    #[error("noise_owner_reservation")]
    OwnerReservation,
    #[error("noise_initiator")]
    Initiator,
    #[error("noise_act_one")]
    ActOne,
    #[error("noise_responder")]
    Responder,
    #[error("noise_act_two")]
    ActTwo,
    #[error("noise_completion_{0:?}")]
    Completion(NoiseCompletionFailure),
    #[error("noise_frame")]
    Frame,
    #[error("noise_frame_authentication")]
    FrameAuthentication,
    #[error("noise_frame_length")]
    FrameLength,
    #[error("noise_frame_mismatch")]
    FrameMismatch,
}

/// Small public facts, serialized only after all cipher and frame resources have dropped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NoiseProbeResult {
    pub schema: &'static str,
    pub seed: u64,
    pub authenticated: bool,
    pub frame_round_trip: bool,
    pub payload_bytes: usize,
    pub resources_released: bool,
    pub hardware_qualified: bool,
}

/// Executes exactly one real handshake and one 32-byte opaque encrypted frame round trip.
/// Time, synthetic trust and seed match the composed fixture handshake.
#[inline(never)]
pub fn run(
    seed: u64,
    fault: NoiseFault,
    observe: &mut dyn FnMut(NoisePhase),
) -> Result<NoiseProbeResult, NoiseProbeError> {
    run_with_owner_capacity(seed, fault, observe, 1)
}

fn run_with_owner_capacity(
    seed: u64,
    fault: NoiseFault,
    observe: &mut dyn FnMut(NoisePhase),
    owner_capacity: usize,
) -> Result<NoiseProbeResult, NoiseProbeError> {
    observe(NoisePhase::Entry);
    let outcome = handshake_and_frame(seed, fault, observe, owner_capacity);
    // The helper's local owners have dropped on both success and ordinary rejection.
    observe(NoisePhase::Released);
    outcome?;
    Ok(NoiseProbeResult {
        schema: "bitaxe-noise-probe-v1",
        seed,
        authenticated: true,
        frame_round_trip: true,
        payload_bytes: 32,
        resources_released: true,
        hardware_qualified: false,
    })
}

#[inline(never)]
fn handshake_and_frame(
    seed: u64,
    fault: NoiseFault,
    observe: &mut dyn FnMut(NoisePhase),
    owner_capacity: usize,
) -> Result<(), NoiseProbeError> {
    let mut server_slot = Vec::new();
    let mut client_slot = Vec::new();
    server_slot
        .try_reserve_exact(owner_capacity)
        .map_err(|_| NoiseProbeError::OwnerReservation)?;
    client_slot
        .try_reserve_exact(owner_capacity)
        .map_err(|_| NoiseProbeError::OwnerReservation)?;
    if server_slot.capacity() == 0 || client_slot.capacity() == 0 {
        return Err(NoiseProbeError::OwnerReservation);
    }
    let mut act_two = [0_u8; ACT_TWO_LEN];
    let client = prepare_handshake(seed, fault, &mut act_two, &mut server_slot, observe)?;
    if fault == NoiseFault::TamperedActTwo {
        act_two[80] ^= 1;
    }
    observe(NoisePhase::BeforeCompletion);
    if fault == NoiseFault::TruncatedActTwo {
        return reject_truncated_response(client, &act_two[..ACT_TWO_LEN - 1]);
    }
    client
        .complete_diagnostic_into(&act_two, 100, &mut client_slot)
        .map_err(NoiseProbeError::Completion)?;
    observe(NoisePhase::AfterCompletion);
    observe(NoisePhase::BeforeFrame);
    let client = client_slot
        .first_mut()
        .ok_or(NoiseProbeError::Completion(NoiseCompletionFailure::State))?;
    let server = server_slot
        .first_mut()
        .ok_or(NoiseProbeError::Completion(NoiseCompletionFailure::State))?;
    frame_round_trip(client, server, fault)?;
    observe(NoisePhase::AfterFrame);
    Ok(())
}

// Construction and response scratch leave the call stack before certificate verification.
#[inline(never)]
fn prepare_handshake(
    seed: u64,
    fault: NoiseFault,
    act_two: &mut [u8; ACT_TWO_LEN],
    server_slot: &mut Vec<NoiseCodec>,
    observe: &mut dyn FnMut(NoisePhase),
) -> Result<NoiseInitiator, NoiseProbeError> {
    if !server_slot.is_empty() || server_slot.capacity() == 0 {
        return Err(NoiseProbeError::OwnerReservation);
    }
    let mut client_rng = SyntheticRng::new(seed);
    let mut responder_rng = SyntheticRng::new(seed ^ 0xaaccee);
    observe(NoisePhase::BeforeInitiator);
    let mut client = NoiseInitiator::new(
        Some(if fault == NoiseFault::WrongAuthority {
            OTHER_PUBLIC
        } else {
            PUBLIC
        }),
        &mut client_rng,
    )
    .map_err(|_| NoiseProbeError::Initiator)?;
    observe(NoisePhase::AfterInitiator);
    observe(NoisePhase::BeforeActOne);
    let act_one = client.act_one().map_err(|_| NoiseProbeError::ActOne)?;
    observe(NoisePhase::AfterActOne);
    observe(NoisePhase::BeforeResponder);
    let mut responder = Responder::from_authority_kp_with_rng(
        &PUBLIC,
        &PRIVATE,
        Duration::from_secs(3600),
        &mut responder_rng,
    )
    .map_err(|_| NoiseProbeError::Responder)?;
    observe(NoisePhase::AfterResponder);
    observe(NoisePhase::BeforeActTwo);
    let (response, server) = responder
        .step_1_with_now_rng(act_one, 100, &mut responder_rng)
        .map_err(|_| NoiseProbeError::ActTwo)?;
    *act_two = response;
    // Capacity was reserved fallibly before opaque crypto; this push cannot allocate.
    server_slot.push(server);
    observe(NoisePhase::AfterActTwo);
    Ok(client)
}

// The malformed fixture still enters the production length guard, without retaining
// its large by-value return scratch on the valid certificate-verification path.
#[inline(never)]
fn reject_truncated_response(
    client: NoiseInitiator,
    response: &[u8],
) -> Result<(), NoiseProbeError> {
    client
        .complete_diagnostic(response, 100)
        .map(|_| ())
        .map_err(NoiseProbeError::Completion)
}

#[inline(never)]
fn frame_round_trip(
    client: &mut NoiseTransport,
    server: &mut NoiseCodec,
    fault: NoiseFault,
) -> Result<(), NoiseProbeError> {
    let frame = Frame::new(0, 0xfe, vec![0x53; 32]).map_err(|_| NoiseProbeError::Frame)?;
    let mut ciphertext = client
        .encrypt_frame(&frame)
        .map_err(|_| NoiseProbeError::FrameAuthentication)?;
    if fault == NoiseFault::TamperedFrame {
        let last = ciphertext.last_mut().ok_or(NoiseProbeError::FrameLength)?;
        *last ^= 1;
    }
    if fault == NoiseFault::TruncatedFrame {
        ciphertext.pop();
    }
    if ciphertext.len() != ENCRYPTED_HEADER_LEN + 32 + noise_sv2::AEAD_MAC_LEN {
        return Err(NoiseProbeError::FrameLength);
    }
    let mut header = ciphertext[..ENCRYPTED_HEADER_LEN].to_vec();
    let mut payload = ciphertext[ENCRYPTED_HEADER_LEN..].to_vec();
    server
        .decrypt(&mut header)
        .map_err(|_| NoiseProbeError::FrameAuthentication)?;
    server
        .decrypt(&mut payload)
        .map_err(|_| NoiseProbeError::FrameAuthentication)?;
    if header.len() != FRAME_HEADER_LEN
        || FrameHeader::parse(&header).map_err(|_| NoiseProbeError::Frame)? != frame.header
        || payload != frame.payload()
    {
        return Err(NoiseProbeError::FrameMismatch);
    }
    server
        .encrypt(&mut header)
        .map_err(|_| NoiseProbeError::FrameAuthentication)?;
    server
        .encrypt(&mut payload)
        .map_err(|_| NoiseProbeError::FrameAuthentication)?;
    let returned = client
        .decrypt_frame(&header, &payload)
        .map_err(|_| NoiseProbeError::FrameAuthentication)?;
    if returned != frame {
        return Err(NoiseProbeError::FrameMismatch);
    }
    Ok(())
}
