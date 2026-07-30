use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use camino::Utf8PathBuf;
use serde::Deserialize;

use super::*;

static NEXT_ROOT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CatalogCase {
    name: String,
    expected: String,
}

#[derive(Clone, Copy)]
enum ExpectedOutcome {
    Error(Phase36EvidenceError),
    Insufficient(ComponentInsufficiency),
}

struct MutationCase {
    name: &'static str,
    expected: ExpectedOutcome,
    mutate: fn(&mut Phase36EvidenceEnvelope),
}

fn reseal(input: &mut Phase36EvidenceEnvelope) {
    input.shareable_facts.claim_digests =
        computed_claim_digests(&input.shareable_facts).expect("mutated claims should reseal");
}

fn mutations() -> Vec<MutationCase> {
    vec![
        MutationCase {
            name: "unsupported_schema",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::UnsupportedSchema),
            mutate: |input| input.schema_version = "phase36-evidence-v2".to_owned(),
        },
        MutationCase {
            name: "changed_evaluator_identity",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::EvaluatorIdentityMismatch),
            mutate: |input| {
                input.evaluation_identity.successor_contract_digest = digest('8');
            },
        },
        MutationCase {
            name: "missing_artifact_role",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::MissingArtifactRole),
            mutate: |input| {
                input.immutable_artifacts.pop();
            },
        },
        MutationCase {
            name: "extra_artifact_role",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::ExtraArtifactRole),
            mutate: |input| {
                input
                    .immutable_artifacts
                    .push(input.immutable_artifacts[0].clone());
            },
        },
        MutationCase {
            name: "duplicate_artifact_role",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::DuplicateArtifactRole),
            mutate: |input| {
                input.immutable_artifacts[1].role = input.immutable_artifacts[0].role;
            },
        },
        MutationCase {
            name: "unsafe_artifact_path",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::UnsafeArtifactPath),
            mutate: |input| {
                input.immutable_artifacts[0].relative_path = "../opaque".to_owned();
            },
        },
        MutationCase {
            name: "mixed_evidence_source_commits",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::MixedEvidenceSourceCommits),
            mutate: |input| input.immutable_artifacts[2].evidence_source_commit = commit('c'),
        },
        MutationCase {
            name: "package_derived_runtime_observation",
            expected: ExpectedOutcome::Insufficient(
                ComponentInsufficiency::RuntimeIdentityObservation,
            ),
            mutate: |input| {
                input.shareable_facts.runtime_identity.observation_source =
                    RuntimeIdentityObservationSource::PackageDerived;
                input.attempt31_sufficiency.runtime_identity_observation =
                    SufficiencyResult::Insufficient {
                        category: ComponentInsufficiency::RuntimeIdentityObservation,
                    };
                reseal(input);
            },
        },
        MutationCase {
            name: "supervisor_authored_no_actuation",
            expected: ExpectedOutcome::Insufficient(
                ComponentInsufficiency::IndependentEffectObservation,
            ),
            mutate: |input| {
                input.shareable_facts.independent_effects.observation_source =
                    EffectObservationSource::SupervisorAuthored;
                input.attempt31_sufficiency.independent_effect_observation =
                    SufficiencyResult::Insufficient {
                        category: ComponentInsufficiency::IndependentEffectObservation,
                    };
                reseal(input);
            },
        },
        MutationCase {
            name: "incomplete_effect_interval",
            expected: ExpectedOutcome::Insufficient(
                ComponentInsufficiency::IndependentEffectObservation,
            ),
            mutate: |input| {
                input.shareable_facts.independent_effects.interval_state =
                    EffectIntervalState::Incomplete;
                input.attempt31_sufficiency.independent_effect_observation =
                    SufficiencyResult::Insufficient {
                        category: ComponentInsufficiency::IndependentEffectObservation,
                    };
                reseal(input);
            },
        },
        MutationCase {
            name: "contradictory_sensor_state",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::ContradictorySensorState),
            mutate: |input| input.shareable_facts.power.maybe_current_milliamps = None,
        },
        MutationCase {
            name: "contradictory_runtime_health_state",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::ContradictoryRuntimeHealthState),
            mutate: |input| {
                input.shareable_facts.runtime_health.supervisor_availability =
                    SupervisorAvailability::Unavailable;
            },
        },
        MutationCase {
            name: "missing_provenance_join",
            expected: ExpectedOutcome::Error(Phase36EvidenceError::MissingProvenanceJoin),
            mutate: |input| {
                input
                    .shareable_facts
                    .provenance_join
                    .boot_session_digest
                    .clear();
            },
        },
        MutationCase {
            name: "missing_snapshot_join",
            expected: ExpectedOutcome::Insufficient(ComponentInsufficiency::SnapshotSubstance),
            mutate: |input| {
                input.shareable_facts.provenance_join.sensor_snapshot_joined = false;
                input.attempt31_sufficiency.snapshot_substance = SufficiencyResult::Insufficient {
                    category: ComponentInsufficiency::SnapshotSubstance,
                };
                reseal(input);
            },
        },
    ]
}

fn rendered_expected(expected: ExpectedOutcome) -> String {
    match expected {
        ExpectedOutcome::Error(error) => error.to_string(),
        ExpectedOutcome::Insufficient(category) => serde_json::to_value(category)
            .expect("typed insufficiency should serialize")
            .as_str()
            .expect("typed insufficiency should be a string")
            .to_owned(),
    }
}

#[test]
fn phase36_mutation_catalog_matches_executable_expectations() {
    // Arrange
    let catalog = serde_json::from_str::<Vec<CatalogCase>>(include_str!(
        "../../../fixtures/phase36/mutation-catalog.json"
    ))
    .expect("mutation catalog should parse");
    let mut expected = mutations()
        .into_iter()
        .map(|case| CatalogCase {
            name: case.name.to_owned(),
            expected: rendered_expected(case.expected),
        })
        .collect::<Vec<_>>();
    expected.splice(
        1..1,
        [
            CatalogCase {
                name: "changed_phase35_root".to_owned(),
                expected: Phase36EvidenceError::Phase35RootReferenceMismatch.to_string(),
            },
            CatalogCase {
                name: "changed_phase35_generation".to_owned(),
                expected: Phase36EvidenceError::Phase35GenerationReferenceMismatch.to_string(),
            },
            CatalogCase {
                name: "partial_public_output".to_owned(),
                expected: Phase36EvidenceError::PartialPublicOutput.to_string(),
            },
        ],
    );
    expected.extend([
        CatalogCase {
            name: "protected_root_symlink".to_owned(),
            expected: Phase36EvidenceError::ProtectedRootSymlink.to_string(),
        },
        CatalogCase {
            name: "protected_input_symlink".to_owned(),
            expected: Phase36EvidenceError::ProtectedInputSymlink.to_string(),
        },
        CatalogCase {
            name: "protected_root_wrong_permissions".to_owned(),
            expected: Phase36EvidenceError::WrongPermissions.to_string(),
        },
        CatalogCase {
            name: "protected_input_wrong_permissions".to_owned(),
            expected: Phase36EvidenceError::WrongPermissions.to_string(),
        },
    ]);

    // Act and Assert
    assert_eq!(catalog, expected);
}

#[test]
fn phase36_mutation_table_classifies_every_typed_boundary_exactly() {
    // Arrange, Act, Assert
    for case in mutations() {
        let mut input = envelope();
        (case.mutate)(&mut input);
        match case.expected {
            ExpectedOutcome::Error(expected) => assert_eq!(
                classify_phase36_envelope(&input),
                Err(expected),
                "mutation {}",
                case.name
            ),
            ExpectedOutcome::Insufficient(expected) => {
                let result = classify_phase36_envelope(&input)
                    .unwrap_or_else(|error| panic!("mutation {} failed: {error}", case.name));
                assert_eq!(
                    result
                        .immutable_artifact_assessment
                        .component_insufficiencies,
                    vec![expected],
                    "mutation {}",
                    case.name
                );
            }
        }
    }
}

struct RootFixture {
    parent: Utf8PathBuf,
    root: Utf8PathBuf,
}

impl RootFixture {
    fn create() -> Self {
        let fixture_id = NEXT_ROOT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow epoch")
            .as_nanos();
        let parent = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "phase36-mutation-{}-{nonce}-{fixture_id}",
            std::process::id()
        )))
        .expect("temporary path should be UTF-8");
        let root = parent.join("protected");
        fs::create_dir(&parent).expect("parent should be created");
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700))
            .expect("parent mode should be private");
        fs::create_dir(&root).expect("protected root should be created");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .expect("protected root mode should be private");
        Self { parent, root }
    }

    fn write_input(&self) -> Utf8PathBuf {
        let input = self.root.join(PHASE36_INPUT_DOCUMENT);
        fs::write(
            &input,
            include_bytes!("../../../fixtures/phase36/envelope-only.json"),
        )
        .expect("input should be written");
        fs::set_permissions(&input, fs::Permissions::from_mode(0o600))
            .expect("input mode should be private");
        input
    }
}

impl Drop for RootFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.parent).expect("fixture should be removed");
    }
}

#[test]
fn phase36_protected_root_and_input_symlinks_fail_closed() {
    // Arrange
    let root_case = RootFixture::create();
    let real_root = root_case.parent.join("real");
    fs::create_dir(&real_root).expect("real root should be created");
    fs::set_permissions(&real_root, fs::Permissions::from_mode(0o700))
        .expect("real root mode should be private");
    let linked_root = root_case.parent.join("linked");
    symlink(&real_root, &linked_root).expect("root symlink should be created");

    // Act
    let root_error = load_and_classify_phase36_root(&linked_root);

    // Assert
    assert_eq!(root_error, Err(Phase36EvidenceError::ProtectedRootSymlink));

    // Arrange
    let input_case = RootFixture::create();
    let target = input_case.parent.join("target.json");
    fs::write(
        &target,
        include_bytes!("../../../fixtures/phase36/envelope-only.json"),
    )
    .expect("target should be written");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600))
        .expect("target mode should be private");
    symlink(&target, input_case.root.join(PHASE36_INPUT_DOCUMENT))
        .expect("input symlink should be created");

    // Act
    let input_error = load_and_classify_phase36_root(&input_case.root);

    // Assert
    assert_eq!(
        input_error,
        Err(Phase36EvidenceError::ProtectedInputSymlink)
    );
}

#[test]
fn phase36_protected_root_and_input_permissions_fail_closed() {
    // Arrange
    let root_case = RootFixture::create();
    root_case.write_input();
    fs::set_permissions(&root_case.root, fs::Permissions::from_mode(0o755))
        .expect("root mode should change");

    // Act
    let root_error = load_and_classify_phase36_root(&root_case.root);

    // Assert
    assert_eq!(root_error, Err(Phase36EvidenceError::WrongPermissions));

    // Arrange
    let input_case = RootFixture::create();
    let input = input_case.write_input();
    fs::set_permissions(&input, fs::Permissions::from_mode(0o644))
        .expect("input mode should change");

    // Act
    let input_error = load_and_classify_phase36_root(&input_case.root);

    // Assert
    assert_eq!(input_error, Err(Phase36EvidenceError::WrongPermissions));
}
