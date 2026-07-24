use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{SystemTime, UNIX_EPOCH};

use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;

use super::runtime_identity;
use super::*;
use crate::phase35_evidence::tests::EligibleFixture as Phase35Fixture;
use crate::phase35_evidence::{exact_package_capability_digest, sha256_hex};
use crate::phase36_evidence::effects::IndependentEffectAdmission;
use crate::phase36_evidence::runtime_identity::ObservedRuntimeIdentityAdmission;

const SUBSTANCE: &str = include_str!("../../../fixtures/phase36/substance-eligible.json");
const EFFECTS: &str = include_str!("../../../fixtures/phase36/independent-effects-eligible.json");

pub(crate) struct CompleteAuthorityFixture {
    parent: Utf8PathBuf,
    pub(crate) root: Utf8PathBuf,
    authority: Phase36Authority,
    envelope: Phase36EvidenceEnvelope,
    phase35_root_path: Utf8PathBuf,
    phase35_root_bytes: Vec<u8>,
}

impl CompleteAuthorityFixture {
    pub(crate) fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow epoch")
            .as_nanos();
        let parent = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "phase36-authority-{name}-{}-{nonce}",
            std::process::id()
        )))
        .expect("temporary path should be UTF-8");
        let root = parent.join("protected");
        private_directory(&parent);
        private_directory(&root);
        private_directory(&root.join("immutable"));
        let phase35_directory = root.join("immutable/phase35");
        let artifacts_directory = phase35_directory.join("artifacts");
        private_directory(&phase35_directory);
        private_directory(&artifacts_directory);
        let generation_directory = root.join("immutable/generation");
        private_directory(&generation_directory);
        let phase36_directory = root.join("immutable/phase36");
        private_directory(&phase36_directory);

        let (api, websocket, retained) = substantive_documents();
        let snapshot_artifact = serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "phase36-snapshot-substance-artifact-v1",
            "api_document": api,
            "websocket_document": websocket,
            "retained_document": retained,
        }))
        .expect("snapshot artifact should serialize");
        let health_artifact = serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "phase36-runtime-health-artifact-v1",
            "api_document": api,
            "websocket_document": websocket,
            "retained_document": retained,
        }))
        .expect("health artifact should serialize");
        let runtime_documents = runtime_identity::documents();
        let runtime_artifact = serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "phase36-runtime-identity-artifact-v1",
            "exact_package_document": runtime_documents.package,
            "request_document": runtime_documents.request,
            "event_ledger_document": runtime_documents.ledger,
            "private_result_document": runtime_documents.private_result,
            "public_projection_document": runtime_documents.public_projection,
        }))
        .expect("runtime artifact should serialize");
        private_file(
            &phase36_directory.join("snapshot-substance.json"),
            &snapshot_artifact,
        );
        private_file(
            &phase36_directory.join("runtime-health.json"),
            &health_artifact,
        );
        private_file(
            &phase36_directory.join("runtime-identity.json"),
            &runtime_artifact,
        );
        private_file(
            &phase36_directory.join("independent-effects.json"),
            EFFECTS.as_bytes(),
        );

        let mut phase35 = Phase35Fixture::new();
        phase35.input.exact_package.source_commit = "1".repeat(40);
        phase35.input.exact_package.capability_digest =
            exact_package_capability_digest(&phase35.input.exact_package);
        for epoch in [&mut phase35.input.boot_a, &mut phase35.input.boot_b] {
            epoch.package_capability_digest = phase35.input.exact_package.capability_digest.clone();
        }
        phase35.reseal();
        let validated = phase35
            .validate()
            .expect("complete Phase 35 root validates");
        let phase35_root_digest = validated.root_digest().to_owned();
        let phase35_root_bytes =
            serde_json::to_vec_pretty(&phase35.input).expect("Phase 35 root should serialize");
        let phase35_root_path = phase35_directory.join("eligible.json");
        private_file(&phase35_root_path, &phase35_root_bytes);
        for entry in &phase35.input.inventory {
            let artifact = phase35
                .artifacts
                .get(&entry.path)
                .expect("every inventory artifact exists");
            private_file(&phase35_directory.join(&entry.path), artifact.bytes());
        }

        let projection = serde_json::to_vec_pretty(
            &validated
                .shareable_projection()
                .expect("Phase 35 projection should validate"),
        )
        .expect("projection should serialize");
        let matrix = serde_json::to_vec_pretty(&serde_json::json!({
            "evidence_root_digest": phase35_root_digest,
            "scope_decisions": [[
                "passive_hostname_durability",
                {
                    "decision": "promote",
                    "row_id": "V12-HOSTNAME-205",
                    "evidence_root_digest": phase35_root_digest
                }
            ]]
        }))
        .expect("matrix should serialize");
        let verdict = serde_json::to_vec_pretty(&serde_json::json!({
            "admitted": true,
            "evidence_root_digest": phase35_root_digest
        }))
        .expect("verdict should serialize");
        let checklist = b"# Synthetic Phase 35 checklist\n".to_vec();
        let manifest = serde_json::to_vec_pretty(&serde_json::json!({
            "schema": "phase35-generation-v1",
            "root_digest": phase35_root_digest,
            "checklist_sha256": sha256_hex(&checklist),
            "matrix_sha256": sha256_hex(&matrix),
            "projection_sha256": sha256_hex(&projection),
        }))
        .expect("manifest should serialize");
        for (name, bytes) in [
            ("projection.json", projection.as_slice()),
            ("decision-matrix.json", matrix.as_slice()),
            ("admitted.json", verdict.as_slice()),
            ("checklist.md", checklist.as_slice()),
            (".phase35-generation-manifest.json", manifest.as_slice()),
        ] {
            private_file(&generation_directory.join(name), bytes);
        }
        let generation_digest = sha256_hex(&manifest);

        let components = validate_substantive_snapshot_components(&api, &websocket, &retained)
            .expect("substantive documents should validate");
        let sensors = components
            .maybe_sensors
            .expect("synthetic snapshot contains sensors");
        let health = components
            .maybe_runtime_health
            .expect("synthetic snapshot contains runtime health");
        let runtime_admission = validate_observed_runtime_identity_documents(
            &runtime_documents.package,
            Some(&runtime_documents.request),
            Some(&runtime_documents.ledger),
            Some(&runtime_documents.private_result),
            Some(&runtime_documents.public_projection),
        )
        .expect("runtime identity should validate");
        let ObservedRuntimeIdentityAdmission::Validated { identity } = runtime_admission else {
            panic!("runtime identity should be complete");
        };
        let effect_admission = classify_independent_effect_document(Some(EFFECTS), None)
            .expect("effect interval should validate");
        let IndependentEffectAdmission::Validated { interval } = effect_admission else {
            panic!("effect interval should be complete");
        };

        let mut facts = shareable_facts();
        facts.provenance_join.boot_session_digest =
            components.join.operator_boot_session_digest.clone();
        facts.claim_digests = Phase36ClaimDigests {
            snapshot_substance: sensors.claim_fact_digest,
            runtime_health: health.claim_fact_digest,
            runtime_identity: identity.claim_fact_digest,
            independent_no_actuation: interval.claim_fact_digest,
        };
        let source_commit = phase35.input.exact_package.source_commit.clone();
        let role_reference = |role: Phase36ArtifactRole, name: &str, bytes: &[u8]| {
            contract::ImmutableArtifactReference {
                role,
                relative_path: format!("immutable/phase36/{name}"),
                sha256: sha256_hex(bytes),
                evidence_source_commit: source_commit.clone(),
            }
        };
        let envelope = Phase36EvidenceEnvelope {
            schema_version: PHASE36_SCHEMA.to_owned(),
            phase35_root_reference: contract::Phase35RootReference {
                root_digest: phase35_root_digest.clone(),
                evidence_source_commit: source_commit.clone(),
                phase35_generation_digest: generation_digest.clone(),
            },
            evaluation_identity: contract::Phase36EvaluationIdentity {
                evaluator_digest: current_phase36_evidence_evaluator_digest(),
                successor_contract_digest: current_phase36_evidence_contract_digest(),
            },
            immutable_artifacts: vec![
                contract::ImmutableArtifactReference {
                    role: Phase36ArtifactRole::Phase35Root,
                    relative_path: "immutable/phase35/eligible.json".to_owned(),
                    sha256: sha256_hex(&phase35_root_bytes),
                    evidence_source_commit: source_commit.clone(),
                },
                contract::ImmutableArtifactReference {
                    role: Phase36ArtifactRole::Phase35Generation,
                    relative_path: "immutable/generation/.phase35-generation-manifest.json"
                        .to_owned(),
                    sha256: generation_digest.clone(),
                    evidence_source_commit: source_commit.clone(),
                },
                role_reference(
                    Phase36ArtifactRole::SnapshotSubstance,
                    "snapshot-substance.json",
                    &snapshot_artifact,
                ),
                role_reference(
                    Phase36ArtifactRole::RuntimeHealth,
                    "runtime-health.json",
                    &health_artifact,
                ),
                role_reference(
                    Phase36ArtifactRole::RuntimeIdentityObservation,
                    "runtime-identity.json",
                    &runtime_artifact,
                ),
                role_reference(
                    Phase36ArtifactRole::IndependentEffectObservation,
                    "independent-effects.json",
                    EFFECTS.as_bytes(),
                ),
            ],
            attempt31_sufficiency: sufficient_results(),
            shareable_facts: facts,
        };
        private_file(
            &root.join(PHASE36_INPUT_DOCUMENT),
            &serde_json::to_vec_pretty(&envelope).expect("envelope should serialize"),
        );
        let authority = Phase36Authority::synthetic(
            phase35_root_digest,
            generation_digest,
            [
                sha256_hex(&snapshot_artifact),
                sha256_hex(&health_artifact),
                sha256_hex(&runtime_artifact),
                sha256_hex(EFFECTS.as_bytes()),
            ],
        );
        Self {
            parent,
            root,
            authority,
            envelope,
            phase35_root_path,
            phase35_root_bytes,
        }
    }

    pub(crate) fn classify(&self) -> Result<Phase36Classification, Phase36EvidenceError> {
        load_and_classify_with_authority(&self.root, &self.authority)
    }

    fn rewrite_envelope(&self, envelope: &Phase36EvidenceEnvelope) {
        private_file(
            &self.root.join(PHASE36_INPUT_DOCUMENT),
            &serde_json::to_vec_pretty(envelope).expect("envelope should serialize"),
        );
    }

    fn referenced_path(&self, index: usize) -> Utf8PathBuf {
        self.root
            .join(&self.envelope.immutable_artifacts[index].relative_path)
    }
}

impl Drop for CompleteAuthorityFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.parent).expect("fixture should be removed");
    }
}

#[test]
fn complete_authenticated_tree_classifies_and_preserves_phase35_bytes() {
    // Arrange
    let fixture = CompleteAuthorityFixture::new("eligible");
    let before = fs::read(&fixture.phase35_root_path).expect("Phase 35 root reads");

    // Act
    let classification = fixture.classify().expect("complete tree should classify");
    let after = fs::read(&fixture.phase35_root_path).expect("Phase 35 root still reads");

    // Assert
    assert_eq!(classification.schema_version, PHASE36_SCHEMA);
    assert_eq!(before, fixture.phase35_root_bytes);
    assert_eq!(after, before);
}

#[test]
fn complete_caller_tree_without_role_authority_fails_closed() {
    // Arrange
    let fixture = CompleteAuthorityFixture::new("no-role-authority");
    let authority = Phase36Authority {
        phase35_root_digest: fixture.authority.phase35_root_digest.clone(),
        phase35_generation_digest: fixture.authority.phase35_generation_digest.clone(),
        maybe_role_digests: None,
    };

    // Act
    let result = load_and_classify_with_authority(&fixture.root, &authority);

    // Assert
    assert_eq!(result, Err(Phase36EvidenceError::ArtifactInvalid));
}

#[test]
fn missing_file_and_byte_mutation_fail_closed() {
    for (name, mutation) in [
        (
            "missing",
            mutate_remove as fn(&CompleteAuthorityFixture, &Utf8Path),
        ),
        ("bytes", mutate_bytes),
    ] {
        // Arrange
        let fixture = CompleteAuthorityFixture::new(name);
        let path = fixture.referenced_path(5);
        mutation(&fixture, &path);

        // Act
        let result = fixture.classify();

        // Assert
        assert!(result.is_err(), "{name} must fail closed");
    }
}

#[test]
fn path_replacement_and_role_swap_fail_closed() {
    // Arrange
    let replacement = CompleteAuthorityFixture::new("path-replacement");
    let target = replacement.referenced_path(2);
    let moved = target.with_extension("admitted");
    fs::rename(&target, &moved).expect("admitted inode should move");
    private_file(&target, b"replacement bytes");

    // Act and Assert
    assert!(replacement.classify().is_err());

    // Arrange
    let role_swap = CompleteAuthorityFixture::new("role-swap");
    let mut envelope = role_swap.envelope.clone();
    envelope.immutable_artifacts.swap(2, 3);
    envelope.immutable_artifacts[2].role = Phase36ArtifactRole::SnapshotSubstance;
    envelope.immutable_artifacts[3].role = Phase36ArtifactRole::RuntimeHealth;
    role_swap.rewrite_envelope(&envelope);

    // Act and Assert
    assert_eq!(
        role_swap.classify(),
        Err(Phase36EvidenceError::ArtifactInvalid)
    );
}

#[test]
fn evaluator_and_contract_source_drift_fail_closed() {
    for evaluator_drift in [true, false] {
        // Arrange
        let fixture = CompleteAuthorityFixture::new("source-drift");
        let mut envelope = fixture.envelope.clone();
        if evaluator_drift {
            envelope.evaluation_identity.evaluator_digest = "a".repeat(64);
        } else {
            envelope.evaluation_identity.successor_contract_digest = "b".repeat(64);
        }
        fixture.rewrite_envelope(&envelope);

        // Act and Assert
        assert_eq!(
            fixture.classify(),
            Err(Phase36EvidenceError::EvaluatorIdentityMismatch)
        );
    }
}

#[test]
fn phase35_root_and_generation_reference_drift_fail_at_authenticated_boundary() {
    for root_drift in [true, false] {
        // Arrange
        let fixture = CompleteAuthorityFixture::new("phase35-reference-drift");
        let mut envelope = fixture.envelope.clone();
        if root_drift {
            envelope.phase35_root_reference.root_digest = "8".repeat(64);
        } else {
            envelope.phase35_root_reference.phase35_generation_digest = "8".repeat(64);
        }
        fixture.rewrite_envelope(&envelope);

        // Act
        let result = fixture.classify();

        // Assert
        let expected = if root_drift {
            Phase36EvidenceError::Phase35RootReferenceMismatch
        } else {
            Phase36EvidenceError::Phase35GenerationReferenceMismatch
        };
        assert_eq!(result, Err(expected));
    }
}

#[test]
fn well_formed_claim_digest_drift_fails_at_authenticated_boundary() {
    for claim_index in 0..4 {
        // Arrange
        let fixture = CompleteAuthorityFixture::new("claim-digest-drift");
        let mut envelope = fixture.envelope.clone();
        let changed = "8".repeat(64);
        match claim_index {
            0 => envelope.shareable_facts.claim_digests.snapshot_substance = changed,
            1 => envelope.shareable_facts.claim_digests.runtime_health = changed,
            2 => envelope.shareable_facts.claim_digests.runtime_identity = changed,
            3 => {
                envelope
                    .shareable_facts
                    .claim_digests
                    .independent_no_actuation = changed;
            }
            _ => unreachable!("fixed claim table"),
        }
        fixture.rewrite_envelope(&envelope);

        // Act
        let result = fixture.classify();

        // Assert
        assert_eq!(result, Err(Phase36EvidenceError::PartialPublicOutput));
    }
}

fn substantive_documents() -> (String, String, String) {
    let projection: Value = serde_json::from_str(SUBSTANCE).expect("substance JSON parses");
    let json = serde_json::to_string(&projection).expect("substance JSON serializes");
    let revision = projection["operatorSnapshotRevision"]
        .as_u64()
        .expect("revision is numeric");
    let session = projection["bootSession"]
        .as_str()
        .expect("session is textual");
    let marker = format!("operator_snapshot session={session} revision={revision} redacted=true");
    (
        format!(
            "system_info_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
        ),
        format!(
            "live_websocket_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
        ),
        format!("{marker}\nsubstantive_snapshot_json: {json}\n"),
    )
}

fn private_directory(path: &Utf8Path) {
    fs::create_dir(path).expect("private directory should be created");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("private directory mode should be set");
}

fn private_file(path: &Utf8Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("private file should be written");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .expect("private file mode should be set");
}

fn mutate_remove(_fixture: &CompleteAuthorityFixture, path: &Utf8Path) {
    fs::remove_file(path).expect("referenced file should be removed");
}

fn mutate_bytes(_fixture: &CompleteAuthorityFixture, path: &Utf8Path) {
    private_file(path, b"mutated bytes");
}
