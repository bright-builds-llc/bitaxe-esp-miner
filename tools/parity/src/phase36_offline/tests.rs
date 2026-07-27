use std::os::unix::fs::symlink;
use std::os::unix::fs::PermissionsExt;
use std::process::Command as ChildCommand;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use super::*;
use crate::phase36_evidence::tests::runtime_identity;

const PHASE35_MANIFEST: &str = include_str!(
    "../../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/.phase35-generation-manifest.json"
    );
const PHASE35_PROJECTION: &str = include_str!(
    "../../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/projection.json"
    );
const PHASE35_MATRIX: &str = include_str!(
    "../../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/decision-matrix.json"
    );
const PHASE35_VERDICT: &str = include_str!(
    "../../../../docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/admitted.json"
    );
const CHECKLIST_DOCUMENT: &str = include_str!("../../../../docs/parity/checklist.md");
const SUBSTANCE: &str = include_str!("../../fixtures/phase36/substance-eligible.json");
const EFFECTS: &str = include_str!("../../fixtures/phase36/independent-effects-eligible.json");
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
        let outcome =
            reevaluate_attempt31(&request).expect("insufficient root should publish correction");

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
        let checklist_before =
            fs::read(fixture.workspace.join(CHECKLIST).as_std_path()).expect("checklist must read");

        // Act
        let result = reevaluate_attempt31(&request);

        // Assert
        assert!(matches!(result, Err(Phase36OfflineError::CompanionInvalid)));
        assert_eq!(
            fs::read(fixture.workspace.join(CHECKLIST).as_std_path()).expect("checklist must read"),
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
                    serde_json::to_string(&projection).expect("public projection must serialize"),
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
        let workspace = Utf8PathBuf::from(format!("/tmp/bitaxe-phase36-offline-{name}-{nonce}"));
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
                maybe_public_projection: (!matches!(missing, MissingComponent::RuntimeIdentity))
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
