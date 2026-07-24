use super::contract::{
    CheckpointCategory, ImmutableArtifactReference, ImmutableArtifactStatus,
    IndependentEffectFacts, Phase35RootReference, Phase36EvaluationIdentity, RuntimeIdentityFacts,
    RuntimeLifecycleState, SupervisorAvailability, TachometerSensorFacts, TemperatureSensorFacts,
    WatchdogParticipation,
};

type SufficiencyMutation = (ComponentInsufficiency, fn(&mut Phase36EvidenceEnvelope));
use super::*;

mod capture;

fn digest(seed: char) -> String {
    std::iter::repeat_n(seed, 64).collect()
}

fn commit(seed: char) -> String {
    std::iter::repeat_n(seed, 40).collect()
}

fn sufficient_results() -> Attempt31Sufficiency {
    Attempt31Sufficiency {
        snapshot_substance: SufficiencyResult::Sufficient,
        runtime_health: SufficiencyResult::Sufficient,
        runtime_identity_observation: SufficiencyResult::Sufficient,
        independent_effect_observation: SufficiencyResult::Sufficient,
    }
}

fn shareable_facts() -> ShareablePhase36FactsV1 {
    let mut facts = ShareablePhase36FactsV1 {
        schema_version: SHAREABLE_PHASE36_FACTS_SCHEMA.to_owned(),
        power: PowerSensorFacts {
            state: SensorTruthState::Fresh,
            maybe_current_milliamps: Some(1_250),
            maybe_bus_millivolts: Some(5_100),
            maybe_power_milliwatts: Some(6_375),
            producer_sequence: 11,
            acquisition_millis: 1_100,
            reason: SensorReason::None,
        },
        temperature: TemperatureSensorFacts {
            state: SensorTruthState::Fresh,
            maybe_millicelsius: Some(55_250),
            producer_sequence: 12,
            acquisition_millis: 1_200,
            reason: SensorReason::None,
        },
        tachometer: TachometerSensorFacts {
            state: SensorTruthState::Fresh,
            maybe_rpm: Some(4_800),
            producer_sequence: 13,
            acquisition_millis: 1_300,
            reason: SensorReason::None,
        },
        runtime_health: RuntimeHealthFacts {
            lifecycle_state: RuntimeLifecycleState::Ready,
            supervisor_availability: SupervisorAvailability::Available,
            checkpoint_category: CheckpointCategory::ServiceLoop,
            checkpoint_sequence: 14,
            checkpoint_age_millis: 250,
            health_category: RuntimeHealthCategory::Healthy,
            watchdog_participation: WatchdogParticipation::Unproved,
        },
        provenance_join: ProvenanceJoinFacts {
            boot_session_digest: digest('7'),
            operator_snapshot_revision: 15,
            sensor_snapshot_joined: true,
            runtime_health_snapshot_joined: true,
            api_websocket_retained_joined: true,
        },
        runtime_identity: RuntimeIdentityFacts {
            observation_source: RuntimeIdentityObservationSource::DeviceSessionReplay,
            same_physical_device: true,
            source_commit_observed: true,
            reference_commit_observed: true,
            application_elf_observed: true,
            exact_package_joined: true,
        },
        independent_effects: IndependentEffectFacts {
            observation_source: EffectObservationSource::IndependentLedger,
            interval_state: EffectIntervalState::Complete,
            all_effect_paths_covered: true,
            prohibited_effect_observed: false,
        },
        claim_digests: Phase36ClaimDigests {
            snapshot_substance: String::new(),
            runtime_health: String::new(),
            runtime_identity: String::new(),
            independent_no_actuation: String::new(),
        },
    };
    facts.claim_digests =
        computed_claim_digests(&facts).expect("synthetic claim digests should compute");
    facts
}

fn envelope() -> Phase36EvidenceEnvelope {
    let evidence_source_commit = commit('a');
    let phase35_root_digest = digest('1');
    let phase35_generation_digest = digest('2');
    let artifact = |role, relative_path: &str, sha256: String| ImmutableArtifactReference {
        role,
        relative_path: relative_path.to_owned(),
        sha256,
        evidence_source_commit: evidence_source_commit.clone(),
    };
    Phase36EvidenceEnvelope {
        schema_version: PHASE36_SCHEMA.to_owned(),
        phase35_root_reference: Phase35RootReference {
            root_digest: phase35_root_digest.clone(),
            evidence_source_commit: evidence_source_commit.clone(),
            phase35_generation_digest: phase35_generation_digest.clone(),
        },
        evaluation_identity: Phase36EvaluationIdentity {
            evaluator_digest: current_phase36_evidence_evaluator_digest(),
            successor_contract_digest: current_phase36_evidence_contract_digest(),
        },
        immutable_artifacts: vec![
            artifact(
                Phase36ArtifactRole::Phase35Root,
                "immutable/phase35-root.bin",
                phase35_root_digest,
            ),
            artifact(
                Phase36ArtifactRole::Phase35Generation,
                "immutable/phase35-generation.json",
                phase35_generation_digest,
            ),
            artifact(
                Phase36ArtifactRole::SnapshotSubstance,
                "immutable/snapshot.json",
                digest('3'),
            ),
            artifact(
                Phase36ArtifactRole::RuntimeHealth,
                "immutable/runtime-health.json",
                digest('4'),
            ),
            artifact(
                Phase36ArtifactRole::RuntimeIdentityObservation,
                "immutable/runtime-identity.json",
                digest('5'),
            ),
            artifact(
                Phase36ArtifactRole::IndependentEffectObservation,
                "immutable/effect-ledger.json",
                digest('6'),
            ),
        ],
        attempt31_sufficiency: sufficient_results(),
        shareable_facts: shareable_facts(),
    }
}

#[test]
fn phase36_contract_exposes_exact_insufficiency_vocabulary() {
    // Arrange
    let categories = [
        ComponentInsufficiency::SnapshotSubstance,
        ComponentInsufficiency::RuntimeHealth,
        ComponentInsufficiency::RuntimeIdentityObservation,
        ComponentInsufficiency::IndependentEffectObservation,
    ];

    // Act
    let rendered = categories.map(|category| {
        serde_json::to_value(category).expect("insufficiency category should serialize")
    });

    // Assert
    assert_eq!(
        rendered,
        [
            serde_json::json!("snapshot_substance_insufficient"),
            serde_json::json!("runtime_health_insufficient"),
            serde_json::json!("runtime_identity_observation_insufficient"),
            serde_json::json!("independent_effect_observation_insufficient"),
        ]
    );
}

#[test]
fn phase36_contract_aggregates_each_component_insufficiency_in_order() {
    // Arrange
    let categories = [
        ComponentInsufficiency::SnapshotSubstance,
        ComponentInsufficiency::RuntimeHealth,
        ComponentInsufficiency::RuntimeIdentityObservation,
        ComponentInsufficiency::IndependentEffectObservation,
    ];

    // Act and Assert
    for (index, category) in categories.into_iter().enumerate() {
        let mut sufficiency = sufficient_results();
        match index {
            0 => {
                sufficiency.snapshot_substance = SufficiencyResult::Insufficient { category };
            }
            1 => sufficiency.runtime_health = SufficiencyResult::Insufficient { category },
            2 => {
                sufficiency.runtime_identity_observation =
                    SufficiencyResult::Insufficient { category };
            }
            3 => {
                sufficiency.independent_effect_observation =
                    SufficiencyResult::Insufficient { category };
            }
            _ => unreachable!("fixed category table has four members"),
        }
        let assessment = ImmutableArtifactAssessment::from_sufficiency(&sufficiency);
        assert_eq!(
            assessment.status,
            ImmutableArtifactStatus::ImmutableArtifactsInsufficient
        );
        assert_eq!(assessment.component_insufficiencies, vec![category]);
    }
}

#[test]
fn phase36_contract_omits_aggregate_insufficiency_only_when_all_components_are_sufficient() {
    // Arrange
    let sufficiency = sufficient_results();

    // Act
    let assessment = ImmutableArtifactAssessment::from_sufficiency(&sufficiency);

    // Assert
    assert_eq!(
        assessment.status,
        ImmutableArtifactStatus::ImmutableArtifactsSufficient
    );
    assert!(assessment.component_insufficiencies.is_empty());
}

#[test]
fn phase36_contract_shareable_facts_round_trip_without_protected_fields() {
    // Arrange
    let facts = shareable_facts();

    // Act
    let encoded = serde_json::to_vec(&facts).expect("shareable facts should encode");
    let decoded = serde_json::from_slice::<ShareablePhase36FactsV1>(&encoded)
        .expect("shareable facts should decode");
    let text = String::from_utf8(encoded).expect("JSON should be UTF-8");

    // Assert
    assert_eq!(decoded, facts);
    for forbidden in [
        "relative_path",
        "device_url",
        "ssid",
        "mac_address",
        "credential",
        "token",
        "worker",
    ] {
        assert!(!text.contains(forbidden), "forbidden field {forbidden}");
    }
}

#[test]
fn phase36_contract_each_claim_field_changes_its_claim_digest_or_eligibility() {
    // Arrange
    let original = shareable_facts();
    let original_digests =
        computed_claim_digests(&original).expect("original claim digests should compute");
    let mut mutations: Vec<(&str, ShareablePhase36FactsV1, &str)> = Vec::new();
    let mut add =
        |name: &'static str, target: &'static str, mutate: fn(&mut ShareablePhase36FactsV1)| {
            let mut changed = original.clone();
            mutate(&mut changed);
            mutations.push((name, changed, target));
        };
    add("power value", "snapshot", |facts| {
        facts.power.maybe_current_milliamps = Some(1_251);
    });
    add("power voltage", "snapshot", |facts| {
        facts.power.maybe_bus_millivolts = Some(5_101);
    });
    add("power wattage", "snapshot", |facts| {
        facts.power.maybe_power_milliwatts = Some(6_376);
    });
    add("power state", "snapshot", |facts| {
        facts.power.state = SensorTruthState::Stale;
    });
    add("power sequence", "snapshot", |facts| {
        facts.power.producer_sequence += 1;
    });
    add("power stamp", "snapshot", |facts| {
        facts.power.acquisition_millis += 1;
    });
    add("power reason", "snapshot", |facts| {
        facts.power.reason = SensorReason::ObservationExpired;
    });
    add("temperature value", "snapshot", |facts| {
        facts.temperature.maybe_millicelsius = Some(55_251);
    });
    add("temperature state", "snapshot", |facts| {
        facts.temperature.state = SensorTruthState::Stale;
    });
    add("temperature sequence", "snapshot", |facts| {
        facts.temperature.producer_sequence += 1;
    });
    add("temperature stamp", "snapshot", |facts| {
        facts.temperature.acquisition_millis += 1;
    });
    add("temperature reason", "snapshot", |facts| {
        facts.temperature.reason = SensorReason::ObservationExpired;
    });
    add("tachometer value", "snapshot", |facts| {
        facts.tachometer.maybe_rpm = Some(4_801);
    });
    add("tachometer state", "snapshot", |facts| {
        facts.tachometer.state = SensorTruthState::Stale;
    });
    add("tachometer sequence", "snapshot", |facts| {
        facts.tachometer.producer_sequence += 1;
    });
    add("tachometer stamp", "snapshot", |facts| {
        facts.tachometer.acquisition_millis += 1;
    });
    add("tachometer reason", "snapshot", |facts| {
        facts.tachometer.reason = SensorReason::ObservationExpired;
    });
    add("snapshot revision", "snapshot_and_health", |facts| {
        facts.provenance_join.operator_snapshot_revision += 1;
    });
    add("boot session digest", "snapshot_and_health", |facts| {
        facts.provenance_join.boot_session_digest = digest('8');
    });
    add("sensor provenance join", "snapshot_and_health", |facts| {
        facts.provenance_join.sensor_snapshot_joined = false;
    });
    add("health provenance join", "snapshot_and_health", |facts| {
        facts.provenance_join.runtime_health_snapshot_joined = false;
    });
    add(
        "three-surface provenance join",
        "snapshot_and_health",
        |facts| {
            facts.provenance_join.api_websocket_retained_joined = false;
        },
    );
    add("lifecycle state", "health", |facts| {
        facts.runtime_health.lifecycle_state = RuntimeLifecycleState::Degraded;
    });
    add("supervisor availability", "health", |facts| {
        facts.runtime_health.supervisor_availability = SupervisorAvailability::Unavailable;
    });
    add("checkpoint category", "health", |facts| {
        facts.runtime_health.checkpoint_category = CheckpointCategory::Degraded;
    });
    add("checkpoint sequence", "health", |facts| {
        facts.runtime_health.checkpoint_sequence += 1;
    });
    add("checkpoint age", "health", |facts| {
        facts.runtime_health.checkpoint_age_millis += 1;
    });
    add("health category", "health", |facts| {
        facts.runtime_health.health_category = RuntimeHealthCategory::Stale;
    });
    add("watchdog category", "health", |facts| {
        facts.runtime_health.watchdog_participation = WatchdogParticipation::Participating;
    });
    add("runtime observation source", "identity", |facts| {
        facts.runtime_identity.observation_source =
            RuntimeIdentityObservationSource::TerminalResultProjection;
    });
    add("runtime exact package join", "identity", |facts| {
        facts.runtime_identity.exact_package_joined = false;
    });
    add("runtime physical-device join", "identity", |facts| {
        facts.runtime_identity.same_physical_device = false;
    });
    add("runtime source observation", "identity", |facts| {
        facts.runtime_identity.source_commit_observed = false;
    });
    add("runtime reference observation", "identity", |facts| {
        facts.runtime_identity.reference_commit_observed = false;
    });
    add("runtime ELF observation", "identity", |facts| {
        facts.runtime_identity.application_elf_observed = false;
    });
    add("effect observation source", "effects", |facts| {
        facts.independent_effects.observation_source = EffectObservationSource::SupervisorAuthored;
    });
    add("effect interval state", "effects", |facts| {
        facts.independent_effects.interval_state = EffectIntervalState::Incomplete;
    });
    add("effect interval", "effects", |facts| {
        facts.independent_effects.all_effect_paths_covered = false;
    });
    add("effect prohibited category", "effects", |facts| {
        facts.independent_effects.prohibited_effect_observed = true;
    });

    // Act and Assert
    for (name, changed, target) in mutations {
        let changed_digests =
            computed_claim_digests(&changed).expect("mutated claim digests should compute");
        let changed_expected_digest = match target {
            "snapshot" => changed_digests.snapshot_substance != original_digests.snapshot_substance,
            "snapshot_and_health" => {
                changed_digests.snapshot_substance != original_digests.snapshot_substance
                    && changed_digests.runtime_health != original_digests.runtime_health
            }
            "health" => changed_digests.runtime_health != original_digests.runtime_health,
            "identity" => changed_digests.runtime_identity != original_digests.runtime_identity,
            "effects" => {
                changed_digests.independent_no_actuation
                    != original_digests.independent_no_actuation
            }
            _ => false,
        };
        assert!(changed_expected_digest, "mutation {name} was not bound");
    }
}

#[test]
fn phase36_contract_classifies_each_incomplete_component_without_inference() {
    // Arrange
    let cases: [SufficiencyMutation; 4] = [
        (ComponentInsufficiency::SnapshotSubstance, |input| {
            input.shareable_facts.provenance_join.sensor_snapshot_joined = false;
            input.attempt31_sufficiency.snapshot_substance = SufficiencyResult::Insufficient {
                category: ComponentInsufficiency::SnapshotSubstance,
            };
        }),
        (ComponentInsufficiency::RuntimeHealth, |input| {
            input
                .shareable_facts
                .provenance_join
                .runtime_health_snapshot_joined = false;
            input.attempt31_sufficiency.runtime_health = SufficiencyResult::Insufficient {
                category: ComponentInsufficiency::RuntimeHealth,
            };
        }),
        (
            ComponentInsufficiency::RuntimeIdentityObservation,
            |input| {
                input.shareable_facts.runtime_identity.observation_source =
                    RuntimeIdentityObservationSource::PackageDerived;
                input.attempt31_sufficiency.runtime_identity_observation =
                    SufficiencyResult::Insufficient {
                        category: ComponentInsufficiency::RuntimeIdentityObservation,
                    };
            },
        ),
        (
            ComponentInsufficiency::IndependentEffectObservation,
            |input| {
                input.shareable_facts.independent_effects.observation_source =
                    EffectObservationSource::SupervisorAuthored;
                input.attempt31_sufficiency.independent_effect_observation =
                    SufficiencyResult::Insufficient {
                        category: ComponentInsufficiency::IndependentEffectObservation,
                    };
            },
        ),
    ];

    // Act and Assert
    for (expected, mutate) in cases {
        let mut input = envelope();
        mutate(&mut input);
        input.shareable_facts.claim_digests =
            computed_claim_digests(&input.shareable_facts).expect("claim digests should recompute");
        let classified =
            classify_phase36_envelope(&input).expect("typed insufficiency should classify");
        assert_eq!(
            classified.immutable_artifact_assessment.status,
            ImmutableArtifactStatus::ImmutableArtifactsInsufficient
        );
        assert_eq!(
            classified
                .immutable_artifact_assessment
                .component_insufficiencies,
            vec![expected]
        );
    }
}

#[test]
fn phase36_contract_rejects_malformed_claim_digests_as_partial_public_output() {
    // Arrange
    let mut inputs = [envelope(), envelope(), envelope(), envelope()];
    inputs[0].shareable_facts.claim_digests.snapshot_substance = "8".repeat(63);
    inputs[1].shareable_facts.claim_digests.runtime_health = "A".repeat(64);
    inputs[2].shareable_facts.claim_digests.runtime_identity = String::new();
    inputs[3]
        .shareable_facts
        .claim_digests
        .independent_no_actuation = "z".repeat(64);

    // Act and Assert
    for input in inputs {
        assert_eq!(
            classify_phase36_envelope(&input),
            Err(Phase36EvidenceError::PartialPublicOutput)
        );
    }
}

#[test]
fn phase36_evaluator_inventory_binds_every_material_owned_validator() {
    // Arrange
    const HOSTNAME_VALIDATOR_DECLARATION: &str = "pub(crate) fn from_public_generation(";
    const SESSION_REQUEST_VALIDATOR_DECLARATION: &str = "pub fn schema_is_valid(&self) -> bool {";
    const SESSION_STATE_VALIDATOR_DECLARATION: &str =
        "pub fn apply(&mut self, event: SessionEvent) {";
    const HARDWARE_TRANSACTION_DECLARATION: &str =
        "pub(super) fn run_phase36_hardware_transaction_with(";
    let expected_sources = [
        "phase36_evidence.rs",
        "phase36_evidence/substance.rs",
        "phase36_evidence/substance/types.rs",
        "phase36_evidence/runtime_identity.rs",
        "phase36_evidence/runtime_identity/ledger.rs",
        "phase36_evidence/capture.rs",
        "phase36_evidence/capture/filesystem.rs",
        "phase36_evidence/capture/hardware.rs",
        "phase36_broker/contract.rs",
        "phase36_broker/ledger.rs",
        "phase36_broker/hardware.rs",
        "phase36_broker/hardware_process.rs",
        "scripts/phase36-substantive-evidence.sh",
        "scripts/phase36-hardware-effect.sh",
        "tools/device-session/src/model.rs",
        "phase36_evidence/effects.rs",
        "operator_snapshot_evidence.rs",
        "crates/bitaxe-api/src/operator_snapshot.rs",
        "phase35_evidence.rs",
        "phase35_evidence/contract.rs",
        "phase35_evidence/digests.rs",
        "phase35_evidence/inventory.rs",
        "phase35_evidence/projection.rs",
        "phase36_promotion/types.rs",
        "protected_input.rs",
    ];
    let inventory_sources = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .map(|(path, _)| *path)
        .collect::<Vec<_>>();
    let hostname_validator_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "phase36_promotion/types.rs")
        .map(|(_, source)| *source)
        .expect("hostname generation validator source should be inventoried");
    let session_validator_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "tools/device-session/src/model.rs")
        .map(|(_, source)| *source)
        .expect("device-session replay validator source should be inventoried");
    let hardware_transaction_source = PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
        .iter()
        .find(|(path, _)| *path == "phase36_broker/hardware.rs")
        .map(|(_, source)| *source)
        .expect("hardware transaction source should be inventoried");
    let drift_identity = |target_path: &str, declaration: &str, replacement: &str| {
        let drifted_inventory =
            PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
                .iter()
                .map(|(path, source)| {
                    let source = if *path == target_path {
                        std::borrow::Cow::Owned(source.replacen(declaration, replacement, 1))
                    } else {
                        std::borrow::Cow::Borrowed(*source)
                    };
                    (*path, source)
                });
        let evaluator = phase36_evidence_evaluator_digest_from_inventory(drifted_inventory);
        let contract = phase36_evidence_contract_digest_for_evaluator(&evaluator);
        (evaluator, contract)
    };

    // Act
    let hostname_drift = drift_identity(
        "phase36_promotion/types.rs",
        HOSTNAME_VALIDATOR_DECLARATION,
        "pub(crate) fn from_public_generation_drift(",
    );
    let session_state_drift = drift_identity(
        "tools/device-session/src/model.rs",
        SESSION_STATE_VALIDATOR_DECLARATION,
        "pub fn apply_drift(&mut self, event: SessionEvent) {",
    );
    let hardware_transaction_drift = drift_identity(
        "phase36_broker/hardware.rs",
        HARDWARE_TRANSACTION_DECLARATION,
        "pub(super) fn run_phase36_hardware_transaction_with_drift(",
    );
    let path_drifted_inventory =
        PHASE36_EVIDENCE_EVALUATOR_SOURCE_INVENTORY
            .iter()
            .map(|(path, source)| {
                let path = if *path == "tools/device-session/src/model.rs" {
                    "tools/device-session/src/model-drift.rs"
                } else {
                    *path
                };
                (path, *source)
            });
    let path_drifted_evaluator =
        phase36_evidence_evaluator_digest_from_inventory(path_drifted_inventory);
    let path_drift = (
        path_drifted_evaluator.clone(),
        phase36_evidence_contract_digest_for_evaluator(&path_drifted_evaluator),
    );

    // Assert
    assert_eq!(inventory_sources, expected_sources);
    assert!(hostname_validator_source.contains(HOSTNAME_VALIDATOR_DECLARATION));
    assert!(session_validator_source.contains(SESSION_REQUEST_VALIDATOR_DECLARATION));
    assert!(session_validator_source.contains(SESSION_STATE_VALIDATOR_DECLARATION));
    assert!(hardware_transaction_source.contains(HARDWARE_TRANSACTION_DECLARATION));
    for (drifted_evaluator, drifted_contract) in [
        hostname_drift,
        session_state_drift,
        hardware_transaction_drift,
        path_drift,
    ] {
        assert_ne!(
            drifted_evaluator,
            current_phase36_evidence_evaluator_digest()
        );
        assert_ne!(drifted_contract, current_phase36_evidence_contract_digest());
    }
}

mod authority;
mod effects;
mod mutations;
pub(crate) mod runtime_identity;
mod substance;
