use bitaxe_worker_control::{LeaseAuthorizationError, PersistedWorkerEffectState};

#[test]
fn boot_baseline_advances_only_the_pending_effect_state() {
    // Arrange
    let states = [
        (None, PersistedWorkerEffectState::Clear),
        (Some(1), PersistedWorkerEffectState::EffectPending),
        (Some(2), PersistedWorkerEffectState::RebootBaselineConfirmed),
    ];

    // Act / Assert
    for (wire, state) in states {
        assert_eq!(PersistedWorkerEffectState::parse(wire), Ok(state));
    }
    assert_eq!(
        PersistedWorkerEffectState::Clear.after_boot_baseline(),
        PersistedWorkerEffectState::Clear
    );
    assert_eq!(
        PersistedWorkerEffectState::EffectPending.after_boot_baseline(),
        PersistedWorkerEffectState::RebootBaselineConfirmed
    );
    assert_eq!(
        PersistedWorkerEffectState::RebootBaselineConfirmed.after_boot_baseline(),
        PersistedWorkerEffectState::RebootBaselineConfirmed
    );
}

#[test]
fn unknown_marker_values_fail_closed() {
    // Arrange / Act
    let result = PersistedWorkerEffectState::parse(Some(0));

    // Assert
    assert_eq!(result, Err(LeaseAuthorizationError::Persistence));
}
