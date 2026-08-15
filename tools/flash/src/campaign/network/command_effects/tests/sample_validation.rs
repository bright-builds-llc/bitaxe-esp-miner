use bitaxe_api::{
    ApiSnapshot, ExpectedRuntimeAttestationIdentity, ObservationStateWire, SystemInfoWire,
};

use super::super::validate_command_sample;
use super::*;

fn target_for_sample(sample: &SystemInfoWire) -> TrustedNetworkTarget {
    TrustedNetworkTarget {
        origin: "http://127.0.0.1:9".to_owned(),
        boot_session: sample.boot_session.to_string(),
        boot_ordinal: sample.boot_ordinal,
        expected: ExpectedRuntimeAttestationIdentity {
            firmware_commit: sample.source_commit.clone(),
            reference_commit: sample.reference_commit.clone(),
            app_elf_sha256: sample.app_elf_sha256.clone(),
        },
    }
}

fn stale_sensor_sample() -> SystemInfoWire {
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.power_status.state = ObservationStateWire::Stale;
    sample.voltage_status.state = ObservationStateWire::Stale;
    sample.current_status.state = ObservationStateWire::Stale;
    sample.chip_temp_status.state = ObservationStateWire::Stale;
    sample.fan_rpm_status.state = ObservationStateWire::Stale;
    sample
}

#[test]
fn stopped_command_phase_accepts_identity_with_stale_sensors() {
    // Arrange
    let sample = stale_sensor_sample();
    let target = target_for_sample(&sample);

    // Act
    let result = validate_command_sample(
        CommandPhase::ProgrammaticPause(PauseJoinState::new(std::time::Instant::now())),
        &sample,
        &target,
    );

    // Assert
    assert!(result.is_ok());
}

#[test]
fn active_command_phase_rejects_stale_sensors() {
    // Arrange
    let mut sample = stale_sensor_sample();
    sample.mining_paused = false;
    sample.mining_activity = "active".to_owned();
    let target = target_for_sample(&sample);

    // Act
    let result = validate_command_sample(
        CommandPhase::ProgrammaticPause(PauseJoinState::new(std::time::Instant::now())),
        &sample,
        &target,
    );

    // Assert
    assert!(result.is_err());
}
