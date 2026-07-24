//! Explicit-input, read-only classification of immutable Attempt 31 companions.

use std::collections::BTreeSet;
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use thiserror::Error;

use crate::operator_evidence::{
    publish_phase36_generation, read_phase36_public_checklist, Phase36GenerationDocuments,
    Phase36PublicationOptions,
};
use crate::phase35_evidence::sha256_hex;
use crate::phase36_evidence::effects::{
    classify_independent_effect_document, ValidatedIndependentEffectInterval,
};
use crate::phase36_evidence::runtime_identity::{
    validate_observed_runtime_identity_documents, ValidatedObservedRuntimeIdentity,
};
use crate::phase36_evidence::{
    validate_substantive_snapshot_components, ComponentInsufficiency, SubstantiveSnapshotJoin,
    ValidatedRuntimeHealthSubstance, ValidatedSensorSubstance,
};
use crate::phase36_promotion::{
    current_phase36_evaluator_digest, evaluate_phase36_promotion, Phase36ChecklistSnapshot,
    Phase36ClaimPrerequisites, ValidatedHostnameDurabilityFacts,
};
use crate::protected_input::{ProtectedFile, ProtectedRoot};
use crate::ReevaluatePhase36Attempt31Args;

const PHASE35_ROOT: &str =
    "docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion";
const PHASE36_ROOT: &str =
    "docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion";
const CHECKLIST: &str = "docs/parity/checklist.md";
const PHASE36_STAGING: &str =
    "docs/parity/evidence/.phase-36-substantive-evidence-admission-and-exact-re-promotion.staging";

#[derive(Debug)]
pub(crate) struct Phase36OfflineRequest {
    workspace_root: Utf8PathBuf,
    maybe_protected_root: Option<Utf8PathBuf>,
    companion_paths: CompanionPaths,
}

impl Phase36OfflineRequest {
    pub(crate) fn from_args(args: &ReevaluatePhase36Attempt31Args) -> Self {
        Self {
            workspace_root: args.workspace_root.clone(),
            maybe_protected_root: args.maybe_protected_root.clone(),
            companion_paths: CompanionPaths {
                maybe_api: args.maybe_api_document.clone(),
                maybe_websocket: args.maybe_websocket_document.clone(),
                maybe_retained: args.maybe_retained_document.clone(),
                maybe_exact_package: args.maybe_exact_package_document.clone(),
                maybe_request: args.maybe_request_document.clone(),
                maybe_event_ledger: args.maybe_event_ledger_document.clone(),
                maybe_private_result: args.maybe_private_result_document.clone(),
                maybe_public_projection: args.maybe_public_projection_document.clone(),
                maybe_independent_effect: args.maybe_independent_effect_document.clone(),
            },
        }
    }
}

#[derive(Debug)]
struct CompanionPaths {
    maybe_api: Option<Utf8PathBuf>,
    maybe_websocket: Option<Utf8PathBuf>,
    maybe_retained: Option<Utf8PathBuf>,
    maybe_exact_package: Option<Utf8PathBuf>,
    maybe_request: Option<Utf8PathBuf>,
    maybe_event_ledger: Option<Utf8PathBuf>,
    maybe_private_result: Option<Utf8PathBuf>,
    maybe_public_projection: Option<Utf8PathBuf>,
    maybe_independent_effect: Option<Utf8PathBuf>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Phase36OfflineOutcome {
    status: &'static str,
    component_insufficiencies: Vec<ComponentInsufficiency>,
    supported_rows: Vec<String>,
    authoritative_generation_digest: String,
}

#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum Phase36OfflineError {
    #[error("offline_public_inputs_invalid")]
    PublicInputsInvalid,
    #[error("offline_companion_invalid")]
    CompanionInvalid,
    #[error("offline_protected_snapshot_changed")]
    ProtectedSnapshotChanged,
    #[error("offline_publication_failed")]
    PublicationFailed,
}

pub(crate) fn reevaluate_attempt31(
    request: &Phase36OfflineRequest,
) -> Result<Phase36OfflineOutcome, Phase36OfflineError> {
    let checklist = read_phase36_public_checklist(
        &request.workspace_root,
        Utf8Path::new(PHASE36_STAGING),
        Utf8Path::new(PHASE36_ROOT),
        Utf8Path::new(CHECKLIST),
        Utf8Path::new(&format!("{PHASE35_ROOT}/.phase35-generation-manifest.json")),
    )
    .map_err(|_| Phase36OfflineError::PublicInputsInvalid)?;
    let public = PublicGeneration::load(&request.workspace_root, checklist)?;
    let companions = CompanionSnapshot::load(
        request.maybe_protected_root.as_deref(),
        &request.companion_paths,
    );
    let facts = classify_companions(&companions)?;
    companions.verify_unchanged()?;
    let component_insufficiencies = facts.component_insufficiencies();
    let prerequisites = Phase36ClaimPrerequisites {
        phase35_root_digest: public.hostname.phase35_root_digest.clone(),
        superseded_phase35_generation_digest: public.generation_digest,
        evaluator_digest: current_phase36_evaluator_digest(),
        maybe_hostname: Some(public.hostname),
        maybe_sensors: facts.maybe_sensors,
        maybe_snapshot_join: facts.maybe_join,
        maybe_runtime_health: facts.maybe_runtime_health,
        maybe_runtime_identity: facts.maybe_runtime_identity,
        maybe_independent_effect: facts.maybe_independent_effect,
    };
    let checklist = Phase36ChecklistSnapshot::capture(public.checklist)
        .map_err(|_| Phase36OfflineError::PublicInputsInvalid)?;
    let matrix = evaluate_phase36_promotion(&prerequisites, &checklist)
        .map_err(|_| Phase36OfflineError::PublicInputsInvalid)?;
    let supported_rows = matrix
        .supported_row_ids()
        .into_iter()
        .map(str::to_owned)
        .collect();
    let authoritative_generation_digest = matrix
        .resolver
        .authoritative_phase36_generation_digest
        .clone();
    publish_phase36_generation(
        &request.workspace_root,
        Utf8Path::new(PHASE36_STAGING),
        Utf8Path::new(PHASE36_ROOT),
        Utf8Path::new(CHECKLIST),
        Utf8Path::new(&format!("{PHASE35_ROOT}/.phase35-generation-manifest.json")),
        &Phase36GenerationDocuments::new(prerequisites, matrix),
        Phase36PublicationOptions::default(),
    )
    .map_err(|_| Phase36OfflineError::PublicationFailed)?;
    let status = if component_insufficiencies.is_empty() {
        "immutable_artifacts_sufficient"
    } else {
        "immutable_artifacts_insufficient"
    };
    Ok(Phase36OfflineOutcome {
        status,
        component_insufficiencies,
        supported_rows,
        authoritative_generation_digest,
    })
}

struct PublicGeneration {
    hostname: ValidatedHostnameDurabilityFacts,
    generation_digest: String,
    checklist: String,
}

impl PublicGeneration {
    fn load(workspace: &Utf8Path, checklist: String) -> Result<Self, Phase36OfflineError> {
        let manifest = read_public(
            workspace,
            &format!("{PHASE35_ROOT}/.phase35-generation-manifest.json"),
        )?;
        let projection = read_public(workspace, &format!("{PHASE35_ROOT}/projection.json"))?;
        let matrix = read_public(workspace, &format!("{PHASE35_ROOT}/decision-matrix.json"))?;
        let verdict = read_public(workspace, &format!("{PHASE35_ROOT}/admitted.json"))?;
        let hostname = ValidatedHostnameDurabilityFacts::from_public_generation(
            &manifest,
            &projection,
            &matrix,
            &verdict,
        )
        .map_err(|_| Phase36OfflineError::PublicInputsInvalid)?;
        Ok(Self {
            hostname,
            generation_digest: sha256_hex(manifest.as_bytes()),
            checklist,
        })
    }
}

fn read_public(workspace: &Utf8Path, relative: &str) -> Result<String, Phase36OfflineError> {
    fs::read_to_string(workspace.join(relative).as_std_path())
        .map_err(|_| Phase36OfflineError::PublicInputsInvalid)
}

#[derive(Default)]
struct ClassifiedFacts {
    maybe_sensors: Option<ValidatedSensorSubstance>,
    maybe_join: Option<SubstantiveSnapshotJoin>,
    maybe_runtime_health: Option<ValidatedRuntimeHealthSubstance>,
    maybe_runtime_identity: Option<ValidatedObservedRuntimeIdentity>,
    maybe_independent_effect: Option<ValidatedIndependentEffectInterval>,
}

impl ClassifiedFacts {
    fn component_insufficiencies(&self) -> Vec<ComponentInsufficiency> {
        let mut categories = BTreeSet::new();
        if self.maybe_sensors.is_none() || self.maybe_join.is_none() {
            categories.insert(ComponentInsufficiency::SnapshotSubstance);
        }
        if self.maybe_runtime_health.is_none() || self.maybe_join.is_none() {
            categories.insert(ComponentInsufficiency::RuntimeHealth);
        }
        if self.maybe_runtime_identity.is_none() {
            categories.insert(ComponentInsufficiency::RuntimeIdentityObservation);
        }
        if self.maybe_independent_effect.is_none() {
            categories.insert(ComponentInsufficiency::IndependentEffectObservation);
        }
        categories.into_iter().collect()
    }
}

fn classify_companions(
    companions: &CompanionSnapshot,
) -> Result<ClassifiedFacts, Phase36OfflineError> {
    if let (Some(api), Some(websocket), Some(retained)) = (
        companions.text(&companions.api),
        companions.text(&companions.websocket),
        companions.text(&companions.retained),
    ) {
        validate_substantive_snapshot_components(api, websocket, retained)
            .map_err(|_| Phase36OfflineError::CompanionInvalid)?;
    }
    if let Some(package) = companions.text(&companions.exact_package) {
        validate_observed_runtime_identity_documents(
            package,
            companions.text(&companions.request),
            companions.text(&companions.event_ledger),
            companions.text(&companions.private_result),
            companions.text(&companions.public_projection),
        )
        .map_err(|_| Phase36OfflineError::CompanionInvalid)?;
    } else if [
        &companions.request,
        &companions.event_ledger,
        &companions.private_result,
        &companions.public_projection,
    ]
    .into_iter()
    .any(Option::is_some)
    {
        return Err(Phase36OfflineError::CompanionInvalid);
    }
    classify_independent_effect_document(companions.text(&companions.independent_effect), None)
        .map_err(|_| Phase36OfflineError::CompanionInvalid)?;

    // The immutable Phase 35 generation does not anchor any of these companion
    // roles or their digests. Caller-supplied documents cannot create evidence
    // authority and therefore remain insufficient.
    Ok(ClassifiedFacts::default())
}

#[derive(Default)]
struct CompanionSnapshot {
    _protected_root: Option<ProtectedRoot>,
    api: Option<ProtectedFile>,
    websocket: Option<ProtectedFile>,
    retained: Option<ProtectedFile>,
    exact_package: Option<ProtectedFile>,
    request: Option<ProtectedFile>,
    event_ledger: Option<ProtectedFile>,
    private_result: Option<ProtectedFile>,
    public_projection: Option<ProtectedFile>,
    independent_effect: Option<ProtectedFile>,
}

impl CompanionSnapshot {
    fn load(maybe_root: Option<&Utf8Path>, paths: &CompanionPaths) -> Self {
        let Some(root) = maybe_root.and_then(|root| ProtectedRoot::open(root).ok()) else {
            return Self::default();
        };
        Self {
            api: snapshot_optional(&root, paths.maybe_api.as_deref()),
            websocket: snapshot_optional(&root, paths.maybe_websocket.as_deref()),
            retained: snapshot_optional(&root, paths.maybe_retained.as_deref()),
            exact_package: snapshot_optional(&root, paths.maybe_exact_package.as_deref()),
            request: snapshot_optional(&root, paths.maybe_request.as_deref()),
            event_ledger: snapshot_optional(&root, paths.maybe_event_ledger.as_deref()),
            private_result: snapshot_optional(&root, paths.maybe_private_result.as_deref()),
            public_projection: snapshot_optional(&root, paths.maybe_public_projection.as_deref()),
            independent_effect: snapshot_optional(&root, paths.maybe_independent_effect.as_deref()),
            _protected_root: Some(root),
        }
    }

    fn text<'a>(&self, maybe_file: &'a Option<ProtectedFile>) -> Option<&'a str> {
        maybe_file.as_ref().and_then(|file| file.text().ok())
    }

    fn verify_unchanged(&self) -> Result<(), Phase36OfflineError> {
        for file in [
            &self.api,
            &self.websocket,
            &self.retained,
            &self.exact_package,
            &self.request,
            &self.event_ledger,
            &self.private_result,
            &self.public_projection,
            &self.independent_effect,
        ]
        .into_iter()
        .flatten()
        {
            file.verify_unchanged()
                .map_err(|_| Phase36OfflineError::ProtectedSnapshotChanged)?;
        }
        Ok(())
    }
}

fn snapshot_optional(
    root: &ProtectedRoot,
    maybe_relative: Option<&Utf8Path>,
) -> Option<ProtectedFile> {
    maybe_relative.and_then(|relative| root.open_file(relative).ok())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command as ChildCommand;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::Value;

    use super::*;
    use crate::phase36_evidence::tests::runtime_identity;

    const PHASE35_MANIFEST: &str = include_str!(
        "../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/.phase35-generation-manifest.json"
    );
    const PHASE35_PROJECTION: &str = include_str!(
        "../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/projection.json"
    );
    const PHASE35_MATRIX: &str = include_str!(
        "../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/decision-matrix.json"
    );
    const PHASE35_VERDICT: &str = include_str!(
        "../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/admitted.json"
    );
    const CHECKLIST_DOCUMENT: &str = include_str!("../../../docs/parity/checklist.md");
    const SUBSTANCE: &str = include_str!("../fixtures/phase36/substance-eligible.json");
    const EFFECTS: &str = include_str!("../fixtures/phase36/independent-effects-eligible.json");
    const PROTECTED_HELPER_SCENARIO: &str = "BITAXE_PHASE36_PROTECTED_HELPER_SCENARIO";
    const PROTECTED_HELPER_ROOT: &str = "BITAXE_PHASE36_PROTECTED_HELPER_ROOT";
    const PROTECTED_HELPER_TEST: &str = "phase36_offline::tests::protected_snapshot_process_helper";

    #[test]
    fn phase36_offline_unanchored_companion_sets_remain_fully_insufficient() {
        for (name, missing) in [
            ("missing-sensor", MissingComponent::Sensor),
            ("missing-health", MissingComponent::Health),
            (
                "missing-runtime-identity",
                MissingComponent::RuntimeIdentity,
            ),
            ("missing-effect", MissingComponent::Effect),
        ] {
            // Arrange
            let fixture = OfflineFixture::new(name);
            let request = fixture.request(missing);

            // Act
            let outcome = reevaluate_attempt31(&request)
                .expect("insufficient root should publish correction");

            // Assert
            assert_eq!(outcome.status, "immutable_artifacts_insufficient");
            assert_eq!(
                outcome.component_insufficiencies,
                vec![
                    ComponentInsufficiency::SnapshotSubstance,
                    ComponentInsufficiency::RuntimeHealth,
                    ComponentInsufficiency::RuntimeIdentityObservation,
                    ComponentInsufficiency::IndependentEffectObservation,
                ]
            );
        }
    }

    #[test]
    fn phase36_offline_caller_authored_companions_cannot_create_authority() {
        // Arrange
        let fixture = OfflineFixture::new("unauthenticated-companions");
        let request = fixture.request(MissingComponent::None);

        // Act
        let outcome = reevaluate_attempt31(&request).expect("insufficient root should publish");

        // Assert
        assert_eq!(outcome.status, "immutable_artifacts_insufficient");
        assert_eq!(
            outcome.component_insufficiencies,
            vec![
                ComponentInsufficiency::SnapshotSubstance,
                ComponentInsufficiency::RuntimeHealth,
                ComponentInsufficiency::RuntimeIdentityObservation,
                ComponentInsufficiency::IndependentEffectObservation,
            ]
        );
        assert_eq!(outcome.supported_rows, vec!["V12-HOSTNAME-205"]);
        for name in [
            "typed-fact-projection.json",
            "decision-matrix.json",
            "verdict.json",
            "manifest.json",
        ] {
            assert!(fixture.workspace.join(PHASE36_ROOT).join(name).exists());
        }
    }

    #[test]
    fn phase36_offline_invalid_companions_leave_publication_unchanged() {
        for (name, mutation) in [
            ("malformed", InvalidCompanion::Malformed),
            ("contradictory", InvalidCompanion::Contradictory),
            ("prohibited", InvalidCompanion::Prohibited),
            (
                "public-private-disagreement",
                InvalidCompanion::PublicPrivateDisagreement,
            ),
        ] {
            // Arrange
            let fixture = OfflineFixture::new(name);
            let request = fixture.request(MissingComponent::None);
            mutation.apply(&fixture);
            let checklist_before = fs::read(fixture.workspace.join(CHECKLIST).as_std_path())
                .expect("checklist must read");

            // Act
            let result = reevaluate_attempt31(&request);

            // Assert
            assert!(matches!(result, Err(Phase36OfflineError::CompanionInvalid)));
            assert_eq!(
                fs::read(fixture.workspace.join(CHECKLIST).as_std_path())
                    .expect("checklist must read"),
                checklist_before
            );
            assert!(!fixture.workspace.join(PHASE36_ROOT).exists());
        }
    }

    #[test]
    fn protected_snapshot_rejects_ancestor_symlink_escape() {
        // Arrange
        let fixture = OfflineFixture::new("ancestor-symlink");
        let external = fixture.workspace.join("external");
        fs::create_dir(&external).expect("external directory must be created");
        fs::set_permissions(external.as_std_path(), fs::Permissions::from_mode(0o700))
            .expect("external directory mode must be set");
        let escaped = external.join("api.md");
        fs::write(escaped.as_std_path(), "escaped").expect("escaped file must write");
        fs::set_permissions(escaped.as_std_path(), fs::Permissions::from_mode(0o600))
            .expect("escaped file mode must be set");
        symlink(
            external.as_std_path(),
            fixture.protected.join("linked").as_std_path(),
        )
        .expect("ancestor symlink must be created");

        // Act
        let status = run_protected_snapshot_helper(&fixture, "ancestor-symlink");

        // Assert
        assert!(status.success());
    }

    #[test]
    fn protected_snapshot_verifies_the_admitted_descriptor_after_path_swap() {
        // Arrange
        let fixture = OfflineFixture::new("path-swap");
        // Act
        let status = run_protected_snapshot_helper(&fixture, "path-swap");

        // Assert
        assert!(status.success());
    }

    #[test]
    fn protected_snapshot_process_helper() {
        let Ok(scenario) = std::env::var(PROTECTED_HELPER_SCENARIO) else {
            return;
        };
        let root = Utf8PathBuf::from(
            std::env::var(PROTECTED_HELPER_ROOT).expect("protected helper root must be supplied"),
        );
        let paths = CompanionPaths {
            maybe_api: Some(
                if scenario == "ancestor-symlink" {
                    "linked/api.md"
                } else {
                    "api.md"
                }
                .into(),
            ),
            maybe_websocket: (scenario == "ancestor-symlink").then(|| "websocket.md".into()),
            maybe_retained: None,
            maybe_exact_package: None,
            maybe_request: None,
            maybe_event_ledger: None,
            maybe_private_result: None,
            maybe_public_projection: None,
            maybe_independent_effect: None,
        };
        let snapshot = CompanionSnapshot::load(Some(&root), &paths);
        if scenario == "ancestor-symlink" {
            assert!(snapshot.api.is_none());
            assert!(
                snapshot.websocket.is_some(),
                "direct companion must still be admitted"
            );
            return;
        }

        assert_eq!(scenario, "path-swap");
        assert!(snapshot.api.is_some(), "api companion must be admitted");
        let admitted_path = root.join("api.md");
        fs::rename(
            admitted_path.as_std_path(),
            root.join("api-admitted.md").as_std_path(),
        )
        .expect("admitted file must be renamed");
        fs::write(admitted_path.as_std_path(), "replacement").expect("replacement must write");
        fs::set_permissions(
            admitted_path.as_std_path(),
            fs::Permissions::from_mode(0o600),
        )
        .expect("replacement mode must be set");
        assert!(snapshot.verify_unchanged().is_ok());
    }

    fn run_protected_snapshot_helper(
        fixture: &OfflineFixture,
        scenario: &str,
    ) -> std::process::ExitStatus {
        ChildCommand::new(std::env::current_exe().expect("test executable path must resolve"))
            .args(["--exact", PROTECTED_HELPER_TEST, "--nocapture"])
            .env(PROTECTED_HELPER_SCENARIO, scenario)
            .env(PROTECTED_HELPER_ROOT, fixture.protected.as_str())
            .status()
            .expect("protected snapshot helper must launch")
    }

    #[test]
    fn phase36_offline_rejects_phase35_generation_without_hostname_promotion() {
        // Arrange
        let fixture = OfflineFixture::new("hostname-not-promoted");
        let matrix_path = fixture
            .workspace
            .join(PHASE35_ROOT)
            .join("decision-matrix.json");
        let mut matrix: Value =
            serde_json::from_str(PHASE35_MATRIX).expect("matrix fixture must parse");
        let hostname = matrix["scope_decisions"]
            .as_array_mut()
            .expect("scope decisions must be an array")
            .iter_mut()
            .find(|entry| entry[0] == "passive_hostname_durability")
            .expect("hostname decision must exist");
        hostname[1]["decision"] = Value::String("exclude".to_owned());
        let matrix_document =
            serde_json::to_string_pretty(&matrix).expect("matrix fixture must serialize");
        fs::write(matrix_path.as_std_path(), &matrix_document).expect("matrix fixture must update");
        let manifest_path = fixture
            .workspace
            .join(PHASE35_ROOT)
            .join(".phase35-generation-manifest.json");
        let mut manifest: Value =
            serde_json::from_str(PHASE35_MANIFEST).expect("manifest fixture must parse");
        manifest["matrix_sha256"] = Value::String(sha256_hex(matrix_document.as_bytes()));
        fs::write(
            manifest_path.as_std_path(),
            serde_json::to_string_pretty(&manifest).expect("manifest fixture must serialize"),
        )
        .expect("manifest fixture must update");
        let request = fixture.request(MissingComponent::None);

        // Act
        let result = reevaluate_attempt31(&request);

        // Assert
        assert!(matches!(
            result,
            Err(Phase36OfflineError::PublicInputsInvalid)
        ));
    }

    #[derive(Clone, Copy)]
    enum MissingComponent {
        None,
        Sensor,
        Health,
        RuntimeIdentity,
        Effect,
    }

    enum InvalidCompanion {
        Malformed,
        Contradictory,
        Prohibited,
        PublicPrivateDisagreement,
    }

    impl InvalidCompanion {
        fn apply(self, fixture: &OfflineFixture) {
            let (name, contents) = match self {
                Self::Malformed => ("effects.json", "{".to_owned()),
                Self::Contradictory => ("api.md", "not-an-operator-snapshot".to_owned()),
                Self::Prohibited => (
                    "effects.json",
                    EFFECTS.replacen("\"package_probe\"", "\"active_control\"", 1),
                ),
                Self::PublicPrivateDisagreement => {
                    let documents = runtime_identity::documents();
                    let mut projection: Value = serde_json::from_str(&documents.public_projection)
                        .expect("public projection must parse");
                    projection["same_physical_device"] = Value::Bool(false);
                    (
                        "public-projection.json",
                        serde_json::to_string(&projection)
                            .expect("public projection must serialize"),
                    )
                }
            };
            fs::write(fixture.protected.join(name).as_std_path(), contents)
                .expect("invalid companion must write");
        }
    }

    struct OfflineFixture {
        workspace: Utf8PathBuf,
        protected: Utf8PathBuf,
    }

    impl OfflineFixture {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos();
            let workspace =
                Utf8PathBuf::from(format!("/tmp/bitaxe-phase36-offline-{name}-{nonce}"));
            let protected = workspace.join("protected");
            fs::create_dir_all(workspace.join(PHASE35_ROOT).as_std_path())
                .expect("Phase 35 root must be created");
            fs::create_dir_all(workspace.join("docs/parity").as_std_path())
                .expect("parity root must be created");
            fs::create_dir(&protected).expect("protected root must be created");
            fs::set_permissions(protected.as_std_path(), fs::Permissions::from_mode(0o700))
                .expect("protected root mode must be set");
            for (relative, contents) in [
                (
                    format!("{PHASE35_ROOT}/.phase35-generation-manifest.json"),
                    PHASE35_MANIFEST,
                ),
                (
                    format!("{PHASE35_ROOT}/projection.json"),
                    PHASE35_PROJECTION,
                ),
                (
                    format!("{PHASE35_ROOT}/decision-matrix.json"),
                    PHASE35_MATRIX,
                ),
                (format!("{PHASE35_ROOT}/admitted.json"), PHASE35_VERDICT),
                (CHECKLIST.to_owned(), CHECKLIST_DOCUMENT),
            ] {
                fs::write(workspace.join(relative).as_std_path(), contents)
                    .expect("public fixture must write");
            }
            let fixture = Self {
                workspace,
                protected,
            };
            fixture.write_companions(MissingComponent::None);
            fixture
        }

        fn request(&self, missing: MissingComponent) -> Phase36OfflineRequest {
            self.write_companions(missing);
            Phase36OfflineRequest {
                workspace_root: self.workspace.clone(),
                maybe_protected_root: Some(self.protected.clone()),
                companion_paths: CompanionPaths {
                    maybe_api: Some("api.md".into()),
                    maybe_websocket: Some("websocket.md".into()),
                    maybe_retained: Some("retained.md".into()),
                    maybe_exact_package: (!matches!(missing, MissingComponent::RuntimeIdentity))
                        .then(|| "package.json".into()),
                    maybe_request: (!matches!(missing, MissingComponent::RuntimeIdentity))
                        .then(|| "request.json".into()),
                    maybe_event_ledger: (!matches!(missing, MissingComponent::RuntimeIdentity))
                        .then(|| "ledger.jsonl".into()),
                    maybe_private_result: (!matches!(missing, MissingComponent::RuntimeIdentity))
                        .then(|| "private-result.json".into()),
                    maybe_public_projection: (!matches!(
                        missing,
                        MissingComponent::RuntimeIdentity
                    ))
                    .then(|| "public-projection.json".into()),
                    maybe_independent_effect: (!matches!(missing, MissingComponent::Effect))
                        .then(|| "effects.json".into()),
                },
            }
        }

        fn write_companions(&self, missing: MissingComponent) {
            let mut substance: Value =
                serde_json::from_str(SUBSTANCE).expect("substance fixture must parse");
            if matches!(missing, MissingComponent::Sensor) {
                substance
                    .as_object_mut()
                    .expect("substance must be an object")
                    .remove("current");
            }
            if matches!(missing, MissingComponent::Health) {
                substance
                    .as_object_mut()
                    .expect("substance must be an object")
                    .remove("runtimeHealth");
            }
            let json = serde_json::to_string(&substance).expect("substance must serialize");
            let session = substance["bootSession"]
                .as_str()
                .expect("session must be textual");
            let revision = substance["operatorSnapshotRevision"]
                .as_u64()
                .expect("revision must be numeric");
            let identity = runtime_identity::documents();
            let documents = [
                (
                    "api.md",
                    format!(
                        "system_info_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
                    ),
                ),
                (
                    "websocket.md",
                    format!(
                        "live_websocket_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
                    ),
                ),
                (
                    "retained.md",
                    format!(
                        "operator_snapshot session={session} revision={revision} redacted=true\nsubstantive_snapshot_json: {json}\n"
                    ),
                ),
                ("package.json", identity.package),
                ("request.json", identity.request),
                ("ledger.jsonl", identity.ledger),
                ("private-result.json", identity.private_result),
                ("public-projection.json", identity.public_projection),
                ("effects.json", EFFECTS.to_owned()),
            ];
            for (name, contents) in documents {
                let path = self.protected.join(name);
                fs::write(path.as_std_path(), contents).expect("companion must write");
                fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o600))
                    .expect("companion mode must be set");
            }
        }
    }

    impl Drop for OfflineFixture {
        fn drop(&mut self) {
            if self
                .workspace
                .as_str()
                .starts_with("/tmp/bitaxe-phase36-offline-")
            {
                fs::remove_dir_all(self.workspace.as_std_path())
                    .expect("offline fixture must clean up");
            }
        }
    }
}
