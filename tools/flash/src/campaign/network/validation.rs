use bitaxe_api::{ObservationStateWire, SystemInfoWire};

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

fn validate_identity(
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
        && sample.voltage.is_finite()
        && (4.5..=5.5).contains(&sample.voltage)
        && sample.current.is_finite()
        && sample.current >= 0.0
        && sample.temp.is_finite()
        && sample.temp < 75.0
        && sample.fan_rpm > 0
}

pub(super) fn watchdog_valid(sample: &SystemInfoWire) -> bool {
    sample.runtime_health.supervisor_availability == "available"
        && sample.runtime_health.checkpoint_health == "healthy"
        && sample.runtime_health.maybe_checkpoint_sequence.is_some()
        && sample.runtime_health.task_watchdog_participation == "participating"
        && sample.runtime_health.maybe_task_watchdog_reason.as_deref() == Some("feed_fresh")
        && sample
            .runtime_health
            .maybe_task_watchdog_feed_sequence
            .is_some()
        && sample
            .runtime_health
            .maybe_task_watchdog_feed_age_millis
            .is_some_and(|age| age <= 2_000)
}

pub(super) fn window_index(active_ms: u64) -> usize {
    usize::try_from(active_ms / WINDOW_MILLIS)
        .unwrap_or(REQUIRED_WINDOWS - 1)
        .min(REQUIRED_WINDOWS - 1)
}

pub(super) fn advances(first: Option<u64>, last: Option<u64>) -> bool {
    match (first, last) {
        (None, Some(_)) => true,
        (Some(first), Some(last)) => last > first,
        _ => false,
    }
}

pub(super) fn regresses(previous: Option<(u64, u64)>, current: (u64, u64)) -> bool {
    previous.is_some_and(|previous| current.0 < previous.0 || current.1 < previous.1)
}

pub(super) fn update_gap(maximum: &mut u64, previous: &mut Option<u64>, current: u64) {
    if let Some(previous) = previous.replace(current) {
        *maximum = (*maximum).max(current.saturating_sub(previous));
    }
}
