use super::*;

#[test]
fn real_noise_probe_authenticates_and_round_trips_opaque_frame() {
    // Arrange
    let mut phases = Vec::new();

    // Act
    let result = run(1, NoiseFault::None, &mut |phase| phases.push(phase))
        .expect("synthetic real Noise probe");

    // Assert
    assert!(result.authenticated && result.frame_round_trip && result.resources_released);
    assert_eq!(result.payload_bytes, 32);
    assert!(!result.hardware_qualified);
    assert_eq!(
        phases.iter().map(|phase| *phase as u32).collect::<Vec<_>>(),
        (101..=114).collect::<Vec<_>>()
    );
}

#[test]
fn wrong_synthetic_authority_rejects_completion() {
    // Arrange
    let mut phases = Vec::new();

    // Act
    let result = run(1, NoiseFault::WrongAuthority, &mut |phase| {
        phases.push(phase)
    });

    // Assert
    assert_eq!(
        result,
        Err(NoiseProbeError::Completion(
            NoiseCompletionFailure::CertificateSignature
        ))
    );
    assert_eq!(phases.last(), Some(&NoisePhase::Released));
    assert!(!phases.contains(&NoisePhase::AfterCompletion));
}

#[test]
fn tampered_handshake_rejects_authentication() {
    // Arrange
    let mut phases = Vec::new();

    // Act
    let result = run(1, NoiseFault::TamperedActTwo, &mut |phase| {
        phases.push(phase)
    });

    // Assert
    assert_eq!(
        result,
        Err(NoiseProbeError::Completion(NoiseCompletionFailure::Decrypt))
    );
    assert_eq!(phases.last(), Some(&NoisePhase::Released));
}

#[test]
fn truncated_handshake_rejects_before_completion_crypto() {
    // Arrange
    let mut phases = Vec::new();

    // Act
    let result = run(1, NoiseFault::TruncatedActTwo, &mut |phase| {
        phases.push(phase)
    });

    // Assert
    assert_eq!(
        result,
        Err(NoiseProbeError::Completion(
            NoiseCompletionFailure::MessageLength
        ))
    );
    assert_eq!(phases.last(), Some(&NoisePhase::Released));
}

#[test]
fn tampered_opaque_frame_rejects_mac() {
    // Arrange
    let mut phases = Vec::new();

    // Act
    let result = run(1, NoiseFault::TamperedFrame, &mut |phase| {
        phases.push(phase)
    });

    // Assert
    assert_eq!(result, Err(NoiseProbeError::FrameAuthentication));
    assert!(!phases.contains(&NoisePhase::AfterFrame));
    assert_eq!(phases.last(), Some(&NoisePhase::Released));
}

#[test]
fn truncated_opaque_frame_rejects_exact_ciphertext_bound() {
    // Arrange
    let mut phases = Vec::new();

    // Act
    let result = run(1, NoiseFault::TruncatedFrame, &mut |phase| {
        phases.push(phase)
    });

    // Assert
    assert_eq!(result, Err(NoiseProbeError::FrameLength));
    assert_eq!(phases.last(), Some(&NoisePhase::Released));
}

#[test]
fn identical_seed_repeats_semantic_result() {
    // Arrange
    let mut observe = |_| {};

    // Act
    let first = run(1, NoiseFault::None, &mut observe);
    let second = run(1, NoiseFault::None, &mut observe);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn failed_real_owner_reservation_stops_before_crypto_and_still_releases() {
    // Arrange
    let mut phases = Vec::new();

    // Act
    let result = run_with_owner_capacity(
        1,
        NoiseFault::None,
        &mut |phase| phases.push(phase),
        usize::MAX,
    );

    // Assert
    assert_eq!(result, Err(NoiseProbeError::OwnerReservation));
    assert_eq!(phases, vec![NoisePhase::Entry, NoisePhase::Released]);
}

#[test]
fn unreserved_owner_slots_cannot_enter_preparation() {
    // Arrange
    let mut act_two = [0_u8; ACT_TWO_LEN];
    let mut server_slot = Vec::new();
    let mut phases = Vec::new();

    // Act
    let result = prepare_handshake(
        1,
        NoiseFault::None,
        &mut act_two,
        &mut server_slot,
        &mut |phase| phases.push(phase),
    );

    // Assert
    assert!(matches!(result, Err(NoiseProbeError::OwnerReservation)));
    assert!(phases.is_empty());
    assert!(server_slot.is_empty());
}
