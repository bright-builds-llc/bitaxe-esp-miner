use super::*;

pub(super) fn validate_shareable_facts(
    facts: &ShareablePhase36FactsV1,
) -> Result<(), Phase36EvidenceError> {
    validate_power(&facts.power)?;
    validate_scalar_sensor(
        facts.temperature.state,
        facts.temperature.maybe_millicelsius.is_some(),
        facts.temperature.producer_sequence,
        facts.temperature.acquisition_millis,
        facts.temperature.reason,
    )?;
    validate_scalar_sensor(
        facts.tachometer.state,
        facts.tachometer.maybe_rpm.is_some(),
        facts.tachometer.producer_sequence,
        facts.tachometer.acquisition_millis,
        facts.tachometer.reason,
    )?;
    validate_runtime_health(&facts.runtime_health)?;
    validate_provenance_join(&facts.provenance_join)?;
    if [
        &facts.claim_digests.snapshot_substance,
        &facts.claim_digests.runtime_health,
        &facts.claim_digests.runtime_identity,
        &facts.claim_digests.independent_no_actuation,
    ]
    .into_iter()
    .any(|digest| !is_lower_hex(digest, 64))
    {
        return Err(Phase36EvidenceError::PartialPublicOutput);
    }
    Ok(())
}

fn validate_power(power: &PowerSensorFacts) -> Result<(), Phase36EvidenceError> {
    let all_values = power.maybe_current_milliamps.is_some()
        && power.maybe_bus_millivolts.is_some()
        && power.maybe_power_milliwatts.is_some();
    let no_values = power.maybe_current_milliamps.is_none()
        && power.maybe_bus_millivolts.is_none()
        && power.maybe_power_milliwatts.is_none();
    validate_sensor_state(
        power.state,
        all_values,
        no_values,
        power.producer_sequence,
        power.acquisition_millis,
        power.reason,
    )
}

fn validate_scalar_sensor(
    state: SensorTruthState,
    has_value: bool,
    producer_sequence: u64,
    acquisition_millis: u64,
    reason: SensorReason,
) -> Result<(), Phase36EvidenceError> {
    validate_sensor_state(
        state,
        has_value,
        !has_value,
        producer_sequence,
        acquisition_millis,
        reason,
    )
}

fn validate_sensor_state(
    state: SensorTruthState,
    all_values: bool,
    no_values: bool,
    producer_sequence: u64,
    acquisition_millis: u64,
    reason: SensorReason,
) -> Result<(), Phase36EvidenceError> {
    let legal = match state {
        SensorTruthState::Fresh => {
            all_values
                && producer_sequence > 0
                && acquisition_millis > 0
                && reason == SensorReason::None
        }
        SensorTruthState::Stale => {
            all_values
                && producer_sequence > 0
                && acquisition_millis > 0
                && reason == SensorReason::ObservationExpired
        }
        SensorTruthState::Unavailable => {
            no_values
                && producer_sequence == 0
                && acquisition_millis == 0
                && reason == SensorReason::NeverObserved
        }
        SensorTruthState::Fault => {
            no_values
                && producer_sequence > 0
                && acquisition_millis > 0
                && reason == SensorReason::AcquisitionFailed
        }
    };
    if legal {
        return Ok(());
    }
    Err(Phase36EvidenceError::ContradictorySensorState)
}

fn validate_runtime_health(health: &RuntimeHealthFacts) -> Result<(), Phase36EvidenceError> {
    let legal = match health.health_category {
        RuntimeHealthCategory::Healthy => {
            health.supervisor_availability == contract::SupervisorAvailability::Available
                && health.checkpoint_sequence > 0
                && health.checkpoint_age_millis <= 5_000
        }
        RuntimeHealthCategory::Stale => {
            health.supervisor_availability == contract::SupervisorAvailability::Available
                && health.checkpoint_sequence > 0
                && health.checkpoint_age_millis > 5_000
        }
        RuntimeHealthCategory::Unavailable => {
            health.supervisor_availability == contract::SupervisorAvailability::Unavailable
                && health.checkpoint_sequence == 0
        }
    };
    if legal {
        return Ok(());
    }
    Err(Phase36EvidenceError::ContradictoryRuntimeHealthState)
}

fn validate_provenance_join(provenance: &ProvenanceJoinFacts) -> Result<(), Phase36EvidenceError> {
    if is_lower_hex(&provenance.boot_session_digest, 64)
        && provenance.operator_snapshot_revision > 0
    {
        return Ok(());
    }
    Err(Phase36EvidenceError::MissingProvenanceJoin)
}

pub(super) fn validate_sensor_projection(
    facts: &ShareablePhase36FactsV1,
    sensors: &ValidatedSensorSubstance,
    join: &SubstantiveSnapshotJoin,
) -> Result<(), Phase36EvidenceError> {
    let power_stamp = fresh_stamp(&sensors.power.state)?;
    let temperature_stamp = fresh_stamp(&sensors.temperature.state)?;
    let tachometer_stamp = fresh_stamp(&sensors.tachometer.state)?;
    let tachometer_milliunits = sensors
        .tachometer
        .maybe_value_milliunits
        .filter(|value| value % 1_000 == 0)
        .ok_or(Phase36EvidenceError::ArtifactInvalid)?;
    let matches = facts.power.state == SensorTruthState::Fresh
        && facts.power.maybe_current_milliamps == sensors.power.maybe_current_milliamps
        && facts.power.maybe_bus_millivolts == sensors.power.maybe_bus_millivolts
        && facts.power.maybe_power_milliwatts == sensors.power.maybe_power_milliwatts
        && facts.power.producer_sequence == power_stamp.sequence
        && facts.power.acquisition_millis == power_stamp.acquired_at_ms
        && facts.temperature.state == SensorTruthState::Fresh
        && facts.temperature.maybe_millicelsius == sensors.temperature.maybe_value_milliunits
        && facts.temperature.producer_sequence == temperature_stamp.sequence
        && facts.temperature.acquisition_millis == temperature_stamp.acquired_at_ms
        && facts.tachometer.state == SensorTruthState::Fresh
        && facts.tachometer.maybe_rpm == u64::try_from(tachometer_milliunits / 1_000).ok()
        && facts.tachometer.producer_sequence == tachometer_stamp.sequence
        && facts.tachometer.acquisition_millis == tachometer_stamp.acquired_at_ms
        && facts.provenance_join.boot_session_digest == join.operator_boot_session_digest
        && facts.provenance_join.operator_snapshot_revision == join.operator_snapshot_revision
        && facts.provenance_join.sensor_snapshot_joined
        && facts.provenance_join.api_websocket_retained_joined;
    if matches {
        Ok(())
    } else {
        Err(Phase36EvidenceError::ArtifactInvalid)
    }
}

fn fresh_stamp(
    state: &ObservationState,
) -> Result<&substance::ObservationStamp, Phase36EvidenceError> {
    let ObservationState::Fresh { stamp } = state else {
        return Err(Phase36EvidenceError::ArtifactInvalid);
    };
    Ok(stamp)
}

pub(super) fn validate_health_projection(
    facts: &ShareablePhase36FactsV1,
    health: &ValidatedRuntimeHealthSubstance,
    join: &SubstantiveSnapshotJoin,
) -> Result<(), Phase36EvidenceError> {
    let lifecycle_matches = matches!(
        health.lifecycle_state,
        substance::RuntimeLifecycleState::Idle | substance::RuntimeLifecycleState::Passed
    ) && facts.runtime_health.lifecycle_state
        == contract::RuntimeLifecycleState::Ready;
    let checkpoint_matches = health
        .maybe_checkpoint_category
        .as_ref()
        .is_some_and(|category| category.as_str() == "telemetry")
        && facts.runtime_health.checkpoint_category == contract::CheckpointCategory::ServiceLoop;
    let matches = lifecycle_matches
        && checkpoint_matches
        && health.supervisor_availability == substance::SupervisorAvailability::Available
        && facts.runtime_health.supervisor_availability
            == contract::SupervisorAvailability::Available
        && health.checkpoint_health == substance::CheckpointHealth::Healthy
        && facts.runtime_health.health_category == RuntimeHealthCategory::Healthy
        && health.maybe_checkpoint_sequence == Some(facts.runtime_health.checkpoint_sequence)
        && health.maybe_checkpoint_age_millis == Some(facts.runtime_health.checkpoint_age_millis)
        && facts.runtime_health.watchdog_participation == contract::WatchdogParticipation::Unproved
        && facts.provenance_join.boot_session_digest == join.operator_boot_session_digest
        && facts.provenance_join.operator_snapshot_revision == join.operator_snapshot_revision
        && facts.provenance_join.runtime_health_snapshot_joined
        && facts.provenance_join.api_websocket_retained_joined;
    if matches {
        Ok(())
    } else {
        Err(Phase36EvidenceError::ArtifactInvalid)
    }
}

pub(super) fn derive_sufficiency(facts: &ShareablePhase36FactsV1) -> Attempt31Sufficiency {
    let snapshot_sufficient = facts.provenance_join.sensor_snapshot_joined
        && facts.provenance_join.api_websocket_retained_joined;
    let health_sufficient = facts.provenance_join.runtime_health_snapshot_joined
        && facts.provenance_join.api_websocket_retained_joined;
    let identity_sufficient = facts.runtime_identity.observation_source
        != RuntimeIdentityObservationSource::PackageDerived
        && facts.runtime_identity.same_physical_device
        && facts.runtime_identity.source_commit_observed
        && facts.runtime_identity.reference_commit_observed
        && facts.runtime_identity.application_elf_observed
        && facts.runtime_identity.exact_package_joined;
    let effects_sufficient = facts.independent_effects.observation_source
        == EffectObservationSource::IndependentLedger
        && facts.independent_effects.interval_state == EffectIntervalState::Complete
        && facts.independent_effects.all_effect_paths_covered
        && !facts.independent_effects.prohibited_effect_observed;
    Attempt31Sufficiency {
        snapshot_substance: sufficiency(
            snapshot_sufficient,
            ComponentInsufficiency::SnapshotSubstance,
        ),
        runtime_health: sufficiency(health_sufficient, ComponentInsufficiency::RuntimeHealth),
        runtime_identity_observation: sufficiency(
            identity_sufficient,
            ComponentInsufficiency::RuntimeIdentityObservation,
        ),
        independent_effect_observation: sufficiency(
            effects_sufficient,
            ComponentInsufficiency::IndependentEffectObservation,
        ),
    }
}

fn sufficiency(sufficient: bool, category: ComponentInsufficiency) -> SufficiencyResult {
    if sufficient {
        SufficiencyResult::Sufficient
    } else {
        SufficiencyResult::Insufficient { category }
    }
}
