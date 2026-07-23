use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;

use super::super::phase36::{
    publish_phase36_generation, Phase36GenerationDocuments, Phase36PublicationFailurePoint,
    Phase36PublicationOptions,
};
use super::super::GenerationError;
use super::support::{create_workspace, snapshot};
use crate::phase35_evidence::sha256_hex;
use crate::phase36_promotion::{
    current_phase36_evaluator_digest, evaluate_phase36_promotion, synthetic_phase36_prerequisites,
    Phase36ChecklistSnapshot,
};

const CHECKLIST: &str = include_str!("../../../../../../docs/parity/checklist.md");

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn phase36_publication_owns_exact_redacted_successor_inventory() {
    // Arrange
    let fixture = PublicationFixture::new("success");

    // Act
    fixture.publish(Phase36PublicationOptions::default());

    // Assert
    let destination = fixture.workspace.join("destination");
    let inventory = fs::read_dir(destination.as_std_path())
        .expect("destination must be readable")
        .map(|entry| {
            entry
                .expect("entry must be readable")
                .file_name()
                .into_string()
                .expect("entry must be UTF-8")
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        inventory,
        [
            "typed-fact-projection.json",
            "decision-matrix.json",
            "verdict.json",
            "manifest.json",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    let admitted = snapshot(&destination);
    assert!(admitted.contains("\"schema_version\": \"phase36-generation-v1\""));
    assert!(admitted.contains("\"complete_matrix\": true"));
    assert!(admitted.contains("\"hostname_durability\""));
    assert!(admitted.contains("\"sensor_substance\""));
    assert!(admitted.contains("\"runtime_health\""));
    assert!(admitted.contains("\"runtime_identity\""));
    assert!(admitted.contains("\"independent_effect\""));
    for forbidden in [
        "synthetic stable physical identity",
        "synthetic persisted setting",
        "device_url",
        "request_body",
        "response_body",
    ] {
        assert!(!admitted.contains(forbidden));
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn phase36_every_injected_boundary_restores_both_outputs_byte_identically() {
    for failure_point in Phase36PublicationFailurePoint::ALL {
        // Arrange
        let fixture = PublicationFixture::new(&format!("rollback-{failure_point:?}"));
        fixture.publish(Phase36PublicationOptions::default());
        let destination_before = snapshot(&fixture.workspace.join("destination"));
        let checklist_before =
            fs::read_to_string(fixture.workspace.join("checklist.md").as_std_path())
                .expect("checklist must read");

        // Act
        let error = publish_phase36_generation(
            &fixture.workspace,
            Utf8Path::new("staging"),
            Utf8Path::new("destination"),
            Utf8Path::new("checklist.md"),
            Utf8Path::new("phase35-manifest.json"),
            &fixture.documents_for_current_checklist(),
            Phase36PublicationOptions {
                maybe_failure: Some(failure_point),
            },
        )
        .expect_err("injected failure must fail");

        // Assert
        assert!(matches!(error, GenerationError::Phase36Injected(point) if point == failure_point));
        assert_eq!(
            snapshot(&fixture.workspace.join("destination")),
            destination_before
        );
        assert_eq!(
            fs::read_to_string(fixture.workspace.join("checklist.md").as_std_path())
                .expect("checklist must read"),
            checklist_before
        );
        assert!(!fixture.workspace.join("staging").exists());
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn phase36_publisher_rejects_prior_evaluator_matrix_and_destination_drift() {
    type Mutation = fn(&mut PublicationFixture);
    let cases: [Mutation; 4] = [
        |fixture| fixture.documents.prerequisites.evaluator_digest = "f".repeat(64),
        |fixture| fixture.documents.prerequisites.phase35_root_digest = "e".repeat(64),
        |fixture| {
            fixture.documents.matrix.checklist_fingerprint_after = "d".repeat(64);
        },
        |fixture| {
            fs::create_dir_all(fixture.workspace.join("destination").as_std_path())
                .expect("destination must be created");
            fs::write(
                fixture.workspace.join("destination/unowned").as_std_path(),
                "unowned",
            )
            .expect("unowned marker must write");
        },
    ];
    for (index, mutate) in cases.into_iter().enumerate() {
        // Arrange
        let mut fixture = PublicationFixture::new(&format!("drift-{index}"));
        mutate(&mut fixture);
        let checklist_before =
            fs::read_to_string(fixture.workspace.join("checklist.md").as_std_path())
                .expect("checklist must read");

        // Act
        let result = publish_phase36_generation(
            &fixture.workspace,
            Utf8Path::new("staging"),
            Utf8Path::new("destination"),
            Utf8Path::new("checklist.md"),
            Utf8Path::new("phase35-manifest.json"),
            &fixture.documents,
            Phase36PublicationOptions::default(),
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(fixture.workspace.join("checklist.md").as_std_path())
                .expect("checklist must read"),
            checklist_before
        );
    }
}

#[test]
fn phase36_typed_projection_contains_every_evaluator_fact_leaf() {
    // Arrange
    let fixture = PublicationFixture::new("projection-leaves");
    let expected = serde_json::to_value(super::super::phase36::typed_projection(
        &fixture.documents.prerequisites,
    ))
    .expect("projection must serialize");
    let mut leaf_paths = Vec::new();

    // Act
    collect_leaf_paths("", &expected, &mut leaf_paths);

    // Assert
    for required in [
        "/hostname_durability/storage_confirmed",
        "/hostname_durability/reload_confirmed",
        "/hostname_durability/exactly_once_reboot_confirmed",
        "/sensor_substance/power",
        "/sensor_substance/temperature",
        "/sensor_substance/tachometer",
        "/snapshot_join",
        "/runtime_health",
        "/runtime_identity/observation_source",
        "/runtime_identity/exact_package",
        "/independent_effect/observation_source",
        "/independent_effect/effect_count",
        "/phase35_root_digest",
        "/superseded_phase35_generation_digest",
        "/evaluator_digest",
    ] {
        assert!(
            leaf_paths.iter().any(|path| path.starts_with(required)),
            "missing typed evaluator fact {required}"
        );
    }
    let baseline_digest = sha256_hex(
        serde_json::to_vec(&expected)
            .expect("projection must serialize")
            .as_slice(),
    );
    for path in leaf_paths {
        let mut mutation = expected.clone();
        mutate_leaf(&mut mutation, &path);
        assert_ne!(
            sha256_hex(
                serde_json::to_vec(&mutation)
                    .expect("mutation must serialize")
                    .as_slice()
            ),
            baseline_digest,
            "leaf mutation must change projection digest: {path}"
        );
    }
}

struct PublicationFixture {
    workspace: Utf8PathBuf,
    documents: Phase36GenerationDocuments,
}

impl PublicationFixture {
    fn new(name: &str) -> Self {
        let workspace = create_workspace(&format!("phase36-{name}"));
        fs::write(workspace.join("checklist.md").as_std_path(), CHECKLIST)
            .expect("checklist must write");
        let mut prerequisites = synthetic_phase36_prerequisites();
        prerequisites.evaluator_digest = current_phase36_evaluator_digest();
        let prior_manifest = serde_json::to_string_pretty(&serde_json::json!({
            "schema": "phase35-generation-v1",
            "root_digest": prerequisites.phase35_root_digest,
        }))
        .expect("prior manifest must serialize");
        prerequisites.superseded_phase35_generation_digest = sha256_hex(prior_manifest.as_bytes());
        fs::write(
            workspace.join("phase35-manifest.json").as_std_path(),
            &prior_manifest,
        )
        .expect("prior manifest must write");
        let checklist =
            Phase36ChecklistSnapshot::capture(CHECKLIST.to_owned()).expect("checklist must parse");
        let matrix =
            evaluate_phase36_promotion(&prerequisites, &checklist).expect("matrix must evaluate");
        Self {
            workspace,
            documents: Phase36GenerationDocuments::new(prerequisites, matrix),
        }
    }

    fn publish(&self, options: Phase36PublicationOptions) {
        publish_phase36_generation(
            &self.workspace,
            Utf8Path::new("staging"),
            Utf8Path::new("destination"),
            Utf8Path::new("checklist.md"),
            Utf8Path::new("phase35-manifest.json"),
            &self.documents,
            options,
        )
        .expect("Phase 36 publication must succeed");
    }

    fn documents_for_current_checklist(&self) -> Phase36GenerationDocuments {
        let current = fs::read_to_string(self.workspace.join("checklist.md").as_std_path())
            .expect("checklist must read");
        let checklist =
            Phase36ChecklistSnapshot::capture(current).expect("current checklist must parse");
        let matrix = evaluate_phase36_promotion(&self.documents.prerequisites, &checklist)
            .expect("current matrix must evaluate");
        Phase36GenerationDocuments::new(self.documents.prerequisites.clone(), matrix)
    }
}

fn collect_leaf_paths(prefix: &str, value: &Value, paths: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                collect_leaf_paths(&format!("{prefix}/{key}"), value, paths);
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_leaf_paths(&format!("{prefix}/{index}"), value, paths);
            }
        }
        _ => paths.push(prefix.to_owned()),
    }
}

fn mutate_leaf(value: &mut Value, path: &str) {
    let mut current = value;
    for segment in path.trim_start_matches('/').split('/') {
        current = match current {
            Value::Object(object) => object.get_mut(segment).expect("object leaf must exist"),
            Value::Array(values) => {
                let index = segment.parse::<usize>().expect("array index must parse");
                values.get_mut(index).expect("array leaf must exist")
            }
            _ => panic!("leaf path cannot descend through scalar"),
        };
    }
    *current = match current {
        Value::Bool(value) => Value::Bool(!*value),
        Value::Number(value) => Value::Number((value.as_u64().unwrap_or_default() + 1).into()),
        Value::String(value) => Value::String(format!("{value}x")),
        Value::Null => Value::Bool(true),
        Value::Array(_) | Value::Object(_) => panic!("path must identify a scalar leaf"),
    };
}
