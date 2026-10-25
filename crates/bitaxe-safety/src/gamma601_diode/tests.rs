use super::*;
use DiodeStage::{Initialized, PoweredUninitialized, Unpowered};

const STAGES: [DiodeStage; 3] = [Unpowered, PoweredUninitialized, Initialized];

#[test]
fn a_confirmed_off_is_unpowered_from_every_stage() {
    // Act
    let next = STAGES.map(|prior| next_stage(prior, true, true, true));
    // Assert
    assert_eq!(next, [Unpowered; 3]);
}

#[test]
fn the_regulator_on_without_this_power_on_s_init_is_powered_uninitialized() {
    // Act
    let next = [Unpowered, PoweredUninitialized].map(|prior| next_stage(prior, false, true, false));
    // Assert
    assert_eq!(next, [PoweredUninitialized; 2]);
}

#[test]
fn the_regulator_on_with_this_power_on_s_init_is_initialized() {
    // Act
    let next = [Unpowered, PoweredUninitialized].map(|prior| next_stage(prior, false, true, true));
    // Assert
    assert_eq!(next, [Initialized; 2]);
}

#[test]
fn initialized_stays_latched_until_the_off_is_confirmed() {
    // Act
    let next = [(true, false), (false, false), (false, true)]
        .map(|(regulator_on, init)| next_stage(Initialized, false, regulator_on, init));
    // Assert
    assert_eq!(next, [Initialized; 3]);
}

#[test]
fn an_unconfirmed_off_holds_the_prior_stage() {
    // Act
    let next = STAGES.map(|prior| next_stage(prior, false, false, false));
    // Assert
    assert_eq!(next, STAGES);
}

#[test]
fn an_unpowered_chip_has_no_reading() {
    // Act
    let outcome = chip_temperature_outcome(Unpowered, true, AcquisitionOutcome::Success(40.0));
    // Assert
    assert_eq!(
        outcome,
        AcquisitionOutcome::Unavailable(UnavailableReason::AsicUnpowered)
    );
}

#[test]
fn an_uninitialized_chip_has_no_reading() {
    // Act
    let outcome = chip_temperature_outcome(
        PoweredUninitialized,
        true,
        AcquisitionOutcome::Success(BM1370_DIODE_UNCONFIGURED_C),
    );
    // Assert
    assert_eq!(
        outcome,
        AcquisitionOutcome::Unavailable(UnavailableReason::AsicNotInitialized)
    );
}

#[test]
fn an_initialized_chip_reading_the_unconfigured_value_is_invalid() {
    // Act
    let outcome = chip_temperature_outcome(
        Initialized,
        true,
        AcquisitionOutcome::Success(BM1370_DIODE_UNCONFIGURED_C),
    );
    // Assert
    assert_eq!(outcome, AcquisitionOutcome::InvalidSample);
}

#[test]
fn an_initialized_chip_passes_every_other_outcome_through() {
    // Arrange
    let raws = [
        AcquisitionOutcome::Success(127.5),
        AcquisitionOutcome::Success(80.0),
        AcquisitionOutcome::ReadFailed,
        AcquisitionOutcome::InvalidSample,
        AcquisitionOutcome::Unavailable(UnavailableReason::ThermalReadingUnavailable),
    ];
    // Act
    let outcomes = raws.map(|raw| chip_temperature_outcome(Initialized, true, raw));
    // Assert
    assert_eq!(outcomes, raws);
}

#[test]
fn the_uninitialized_window_holds_up_to_its_cap() {
    // Arrange
    let mut window = UninitializedWindow::default();
    window.observe(PoweredUninitialized, 1_000);
    // Act
    let fault = window.observe(PoweredUninitialized, 1_000 + POWERED_UNINITIALIZED_CAP_MS);
    // Assert
    assert_eq!(fault, None);
}

#[test]
fn the_uninitialized_window_faults_past_its_cap() {
    // Arrange
    let mut window = UninitializedWindow::default();
    window.observe(PoweredUninitialized, 1_000);
    // Act
    let fault = window.observe(PoweredUninitialized, 1_001 + POWERED_UNINITIALIZED_CAP_MS);
    // Assert
    assert_eq!(fault, Some(FaultReason::Bm1370InitOverdue));
}

#[test]
fn leaving_the_stage_restarts_the_window() {
    // Arrange
    let mut window = UninitializedWindow::default();
    window.observe(PoweredUninitialized, 1_000);
    window.observe(Initialized, 15_000);
    window.observe(PoweredUninitialized, 16_000);
    // Act
    let fault = window.observe(PoweredUninitialized, 16_000 + POWERED_UNINITIALIZED_CAP_MS);
    // Assert
    assert_eq!(fault, None);
}

#[test]
fn the_uninitialized_window_never_faults_another_stage() {
    // Arrange
    let mut window = UninitializedWindow::default();
    // Act
    let faults = [Unpowered, Initialized].map(|stage| window.observe(stage, u64::MAX));
    // Assert
    assert_eq!(faults, [None, None]);
}

#[test]
fn a_powering_down_chip_s_unconfigured_reading_is_neither_a_fault_nor_unpowered() {
    // Act
    let outcome = chip_temperature_outcome(
        Initialized,
        false,
        AcquisitionOutcome::Success(BM1370_DIODE_UNCONFIGURED_C),
    );
    // Assert
    assert_eq!(
        outcome,
        AcquisitionOutcome::Unavailable(UnavailableReason::AsicNotInitialized)
    );
}

#[test]
fn a_powering_down_chip_s_real_reading_still_counts() {
    // Act
    let outcome = chip_temperature_outcome(Initialized, false, AcquisitionOutcome::Success(80.0));
    // Assert
    assert_eq!(outcome, AcquisitionOutcome::Success(80.0));
}

#[test]
fn only_an_initialized_chip_with_the_regulator_on_is_loaded() {
    // Act
    let unloaded = [
        chip_unloaded(Initialized, false, true),
        chip_unloaded(Initialized, false, false),
        chip_unloaded(Initialized, true, true),
        chip_unloaded(PoweredUninitialized, false, true),
        chip_unloaded(Unpowered, true, false),
    ];
    // Assert
    assert_eq!(unloaded, [false, true, true, true, true]);
}
