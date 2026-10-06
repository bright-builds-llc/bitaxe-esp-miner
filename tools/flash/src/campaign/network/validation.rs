use bitaxe_api::{ObservationStateWire, SystemInfoWire};
pub(super) use bitaxe_http_transport::continuity::{advances, regresses, update_gap};

use super::model::{TrustedNetworkTarget, REQUIRED_WINDOWS, WINDOW_MILLIS};

pub(super) enum SampleValidationFailure {
    Identity,
    MiningState,
    Safety,
}

pub(super) fn validate_active_prerequisites(
    sample: &SystemInfoWire,
    target: &TrustedNetworkTarget,
) -> Result<(), SampleValidationFailure> {
    validate_identity(sample, target)?;
    if !safety_valid(sample) {
        return Err(SampleValidationFailure::Safety);
    }
    Ok(())
}

pub(super) fn active_mining_state_valid(sample: &SystemInfoWire) -> bool {
    !sample.mining_paused && sample.mining_activity == "active"
}

pub(super) fn validate_sample(
    sample: &SystemInfoWire,
    target: &TrustedNetworkTarget,
    terminal: bool,
) -> Result<(), SampleValidationFailure> {
    validate_identity(sample, target)?;
    let state_valid = if terminal {
        sample.mining_paused && sample.mining_activity == "paused" && !sample.start_mining_on_boot
    } else {
        active_mining_state_valid(sample)
    };
    if !state_valid {
        return Err(SampleValidationFailure::MiningState);
    }
    if !safety_valid(sample) {
        return Err(SampleValidationFailure::Safety);
    }
    Ok(())
}

pub(super) fn validate_identity(
    sample: &SystemInfoWire,
    target: &TrustedNetworkTarget,
) -> Result<(), SampleValidationFailure> {
    if sample.boot_session.to_string() != target.boot_session
        || sample.boot_ordinal != target.boot_ordinal
        || sample.source_commit != target.expected.firmware_commit
        || sample.reference_commit != target.expected.reference_commit
        || sample.app_elf_sha256 != target.expected.app_elf_sha256
        || sample.source_dirty
    {
        return Err(SampleValidationFailure::Identity);
    }
    Ok(())
}

pub(super) fn validate_identity_and_safety(
    sample: &SystemInfoWire,
    target: &TrustedNetworkTarget,
) -> Result<(), SampleValidationFailure> {
    validate_identity(sample, target)?;
    if !safety_valid(sample) {
        return Err(SampleValidationFailure::Safety);
    }
    Ok(())
}

fn safety_valid(sample: &SystemInfoWire) -> bool {
    [
        sample.power_status.state,
        sample.voltage_status.state,
        sample.current_status.state,
        sample.chip_temp_status.state,
        sample.fan_rpm_status.state,
    ]
    .into_iter()
    .all(|state| state == ObservationStateWire::Fresh)
        && sample.power.is_finite()
        && (0.0..=15.0).contains(&sample.power)
        && sample.voltage_millivolts.is_finite()
        && (4_500.0..=5_500.0).contains(&sample.voltage_millivolts)
        && sample.current_milliamps.is_finite()
        && sample.current_milliamps >= 0.0
        && sample.temp.is_finite()
        && sample.temp < 75.0
        && sample.fan_rpm > 0
}

pub(super) fn window_index(active_ms: u64) -> usize {
    usize::try_from(active_ms / WINDOW_MILLIS)
        .unwrap_or(REQUIRED_WINDOWS - 1)
        .min(REQUIRED_WINDOWS - 1)
}
