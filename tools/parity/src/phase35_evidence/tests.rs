use super::*;

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const REFERENCE_COMMIT: &str = "89abcdef0123456789abcdef0123456789abcdef";
const SESSION_A: &str = "0123456789abcdef0011223344556677";
const SESSION_B: &str = "fedcba9876543210ffeeddccbbaa9988";

type InputMutation = fn(&mut Phase35EvidenceRootInput);
type InputCase = (&'static str, InputMutation, Phase35EvidenceError);

struct EligibleFixture {
    input: Phase35EvidenceRootInput,
    artifacts: BTreeMap<String, InventoryArtifact>,
}

impl EligibleFixture {
    fn new() -> Self {
        let artifact_bytes = BTreeMap::from([
            ("package_manifest", b"synthetic manifest-v3".to_vec()),
            ("executable_image", b"synthetic executable".to_vec()),
            ("factory_image", b"synthetic factory".to_vec()),
            ("package", b"synthetic package".to_vec()),
            ("runtime_identity", b"synthetic runtime identity".to_vec()),
            ("target_lock", b"synthetic target lock".to_vec()),
            (
                "detector_capability",
                b"synthetic detector capability".to_vec(),
            ),
            ("no_actuation", b"no_actuation_verified=true\n".to_vec()),
        ]);
        let mut exact_package = ExactPackageCapability {
            source_commit: SOURCE_COMMIT.to_owned(),
            reference_commit: REFERENCE_COMMIT.to_owned(),
            reference_clean: true,
            manifest_schema: "manifest-v3".to_owned(),
            manifest_digest: digest_for(&artifact_bytes, "package_manifest"),
            executable_image_digest: digest_for(&artifact_bytes, "executable_image"),
            factory_image_digest: digest_for(&artifact_bytes, "factory_image"),
            package_digest: digest_for(&artifact_bytes, "package"),
            runtime_identity_digest: digest_for(&artifact_bytes, "runtime_identity"),
            current_head_verified: true,
            capability_digest: String::new(),
        };
        exact_package.capability_digest = exact_package_capability_digest(&exact_package);

        let mut detector_run = DetectorRunCapability {
            board_category: "205".to_owned(),
            detector_capability_digest: digest_for(&artifact_bytes, "detector_capability"),
            physical_identity_digest: sha256_hex(b"synthetic stable physical identity"),
            board_info_verified: true,
            single_candidate_verified: true,
            run_id_digest: sha256_hex(b"synthetic run identifier"),
            capability_digest: String::new(),
        };
        detector_run.capability_digest = detector_run_capability_digest(&detector_run);

        let target_lock_digest = digest_for(&artifact_bytes, "target_lock");
        let boot_a = epoch(EpochFixtureSpec {
            ordinal: 41,
            session: SESSION_A,
            started_millis: 1_000,
            ended_millis: 2_000,
            revision: 7,
            reset_category: "setup",
            package: &exact_package,
            detector: &detector_run,
            target_lock_digest: &target_lock_digest,
        });
        let boot_b = epoch(EpochFixtureSpec {
            ordinal: 42,
            session: SESSION_B,
            started_millis: 6_000,
            ended_millis: 7_000,
            revision: 9,
            reset_category: "software_cpu",
            package: &exact_package,
            detector: &detector_run,
            target_lock_digest: &target_lock_digest,
        });
        let mut input = Phase35EvidenceRootInput {
            schema_version: PHASE35_SCHEMA.to_owned(),
            exact_package,
            detector_run,
            admission_facts: RootAdmissionFacts {
                root_contract_digest: "0".repeat(64),
                target_lock_digest,
                lifecycle_id: PHASE35_LIFECYCLE_ID.to_owned(),
                lifecycle_verified: true,
                current_head_rechecked: true,
                reference_cleanliness_rechecked: true,
                runtime_identity_rechecked: true,
                no_actuation_verified: true,
                inventory_verified: true,
                chronology_verified: true,
                restoration_verified: true,
                cleanup_verified: true,
                redaction_verified: true,
            },
            events: Vec::new(),
            inventory: Vec::new(),
            boot_a,
            boot_b,
        };

        let documents = [
            ("boot_a_api", input.boot_a.system_info_document.as_bytes()),
            (
                "boot_a_websocket",
                input.boot_a.websocket_document.as_bytes(),
            ),
            (
                "boot_a_retained_log",
                input.boot_a.retained_log_document.as_bytes(),
            ),
            ("boot_b_api", input.boot_b.system_info_document.as_bytes()),
            (
                "boot_b_websocket",
                input.boot_b.websocket_document.as_bytes(),
            ),
            (
                "boot_b_retained_log",
                input.boot_b.retained_log_document.as_bytes(),
            ),
        ];
        let mut artifacts = BTreeMap::new();
        for role in INVENTORY_ROLES {
            let path = format!("artifacts/{role}.txt");
            let bytes = artifact_bytes
                .get(role)
                .cloned()
                .or_else(|| {
                    documents.iter().find_map(|(document_role, bytes)| {
                        (*document_role == role).then(|| bytes.to_vec())
                    })
                })
                .expect("every exact inventory role has synthetic bytes");
            input.inventory.push(InventoryEntryInput {
                role: role.to_owned(),
                path: path.clone(),
                sha256: sha256_hex(&bytes),
            });
            artifacts.insert(path, InventoryArtifact::regular(bytes));
        }

        let inventory_digest = inventory_contract_digest(&input.inventory);
        let root_digest = phase35_root_contract_digest(&input, &inventory_digest);
        input.admission_facts.root_contract_digest = root_digest.clone();
        input.boot_a.root_contract_digest = root_digest.clone();
        input.boot_b.root_contract_digest = root_digest;
        input.events = events(&input, &artifacts);
        Self { input, artifacts }
    }

    fn validate(&self) -> Result<ValidatedPhase35Evidence, Phase35EvidenceError> {
        validate_phase35_evidence(&self.input, &self.artifacts)
    }

    fn reseal(&mut self) {
        for entry in &mut self.input.inventory {
            let artifact = self
                .artifacts
                .get(&entry.path)
                .expect("every inventory entry must retain its artifact");
            entry.sha256 = sha256_hex(&artifact.bytes);
        }
        let inventory_digest = inventory_contract_digest(&self.input.inventory);
        let root_digest = phase35_root_contract_digest(&self.input, &inventory_digest);
        self.input.admission_facts.root_contract_digest = root_digest.clone();
        self.input.boot_a.root_contract_digest = root_digest.clone();
        self.input.boot_b.root_contract_digest = root_digest;
        self.input.events = events(&self.input, &self.artifacts);
    }
}

fn digest_for(artifacts: &BTreeMap<&str, Vec<u8>>, role: &str) -> String {
    sha256_hex(
        artifacts
            .get(role)
            .expect("synthetic artifact role must be present"),
    )
}

struct EpochFixtureSpec<'a> {
    ordinal: u64,
    session: &'a str,
    started_millis: u64,
    ended_millis: u64,
    revision: u64,
    reset_category: &'a str,
    package: &'a ExactPackageCapability,
    detector: &'a DetectorRunCapability,
    target_lock_digest: &'a str,
}

fn epoch(spec: EpochFixtureSpec<'_>) -> EvidenceEpochInput {
    let EpochFixtureSpec {
        ordinal,
        session,
        started_millis,
        ended_millis,
        revision,
        reset_category,
        package,
        detector,
        target_lock_digest,
    } = spec;
    let json = serde_json::json!({
        "bootSession": session,
        "operatorSnapshotRevision": revision,
    })
    .to_string();
    EvidenceEpochInput {
        boot_ordinal: ordinal,
        boot_session_digest: sha256_hex(session.as_bytes()),
        started_millis,
        ended_millis,
        system_info_document: format!(
            "system_info_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
        ),
        websocket_document: format!(
            "live_websocket_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
        ),
        retained_log_document: format!(
            "operator_snapshot session={session} revision={revision} redacted=true\n"
        ),
        storage_revision: revision,
        storage_value_digest: sha256_hex(b"synthetic persisted setting"),
        reset_category: reset_category.to_owned(),
        package_capability_digest: package.capability_digest.clone(),
        detector_capability_digest: detector.capability_digest.clone(),
        root_contract_digest: "0".repeat(64),
        target_lock_digest: target_lock_digest.to_owned(),
        run_id_digest: detector.run_id_digest.clone(),
        runtime_identity_digest: package.runtime_identity_digest.clone(),
        physical_identity_digest: detector.physical_identity_digest.clone(),
    }
}

fn events(
    input: &Phase35EvidenceRootInput,
    artifacts: &BTreeMap<String, InventoryArtifact>,
) -> Vec<EvidenceEventInput> {
    let mut events = Vec::new();
    let mut predecessor = input.admission_facts.root_contract_digest.clone();
    for (index, category) in EVENT_CATEGORIES.into_iter().enumerate() {
        let payload_digest = expected_event_payload(input, artifacts, category);
        let event = EvidenceEventInput {
            sequence: index as u64 + 1,
            category,
            monotonic_millis: 100 + index as u64 * 100,
            payload_digest,
            predecessor_event_digest: predecessor,
        };
        predecessor = evidence_event_digest(&event);
        events.push(event);
    }
    events
}

#[test]
fn phase35_evidence_accepts_one_exact_two_epoch_root() {
    // Arrange
    let fixture = EligibleFixture::new();

    // Act
    let validated = fixture.validate().expect("eligible fixture must pass");
    let projection = validated
        .shareable_projection()
        .expect("eligible projection must remain redacted");

    // Assert
    assert_eq!(
        validated.root_digest(),
        fixture.input.admission_facts.root_contract_digest
    );
    assert_eq!(projection.schema, PHASE35_SCHEMA);
    assert_eq!(projection.event_count, EVENT_CATEGORIES.len() as u64);
    assert_eq!(projection.inventory_count, INVENTORY_ROLES.len() as u64);
}

#[test]
fn phase35_evidence_rejects_unsupported_schema_manifest_and_board() {
    let cases: [InputCase; 3] = [
        (
            "schema",
            |input: &mut Phase35EvidenceRootInput| input.schema_version = "future".to_owned(),
            Phase35EvidenceError::UnsupportedSchema,
        ),
        (
            "manifest",
            |input: &mut Phase35EvidenceRootInput| {
                input.exact_package.manifest_schema = "manifest-v2".to_owned()
            },
            Phase35EvidenceError::ManifestV3Mismatch,
        ),
        (
            "board",
            |input: &mut Phase35EvidenceRootInput| {
                input.detector_run.board_category = "601".to_owned()
            },
            Phase35EvidenceError::WrongBoard,
        ),
    ];

    for (name, mutate, expected) in cases {
        // Arrange
        let mut fixture = EligibleFixture::new();
        mutate(&mut fixture.input);

        // Act
        let error = fixture.validate().expect_err("mutation must fail closed");

        // Assert
        assert_eq!(error, expected, "{name}");
    }
}

#[test]
fn phase35_evidence_recomputes_capability_root_and_predecessor_digests() {
    let cases: [InputCase; 4] = [
        (
            "package capability",
            |input: &mut Phase35EvidenceRootInput| {
                input.exact_package.capability_digest = "a".repeat(64)
            },
            Phase35EvidenceError::InvalidPackageCapability,
        ),
        (
            "detector capability",
            |input: &mut Phase35EvidenceRootInput| {
                input.detector_run.capability_digest = "b".repeat(64)
            },
            Phase35EvidenceError::InvalidDetectorCapability,
        ),
        (
            "root contract",
            |input: &mut Phase35EvidenceRootInput| {
                input.admission_facts.root_contract_digest = "c".repeat(64)
            },
            Phase35EvidenceError::RootContractMismatch,
        ),
        (
            "event predecessor",
            |input: &mut Phase35EvidenceRootInput| {
                input.events[1].predecessor_event_digest = "d".repeat(64)
            },
            Phase35EvidenceError::PredecessorViolation,
        ),
    ];

    for (name, mutate, expected) in cases {
        // Arrange
        let mut fixture = EligibleFixture::new();
        mutate(&mut fixture.input);

        // Act
        let error = fixture.validate().expect_err("mutation must fail closed");

        // Assert
        assert_eq!(error, expected, "{name}");
    }
}

#[test]
fn phase35_evidence_rejects_mixed_boot_local_snapshot_session() {
    // Arrange
    let mut fixture = EligibleFixture::new();
    fixture.input.boot_a.websocket_document = fixture
        .input
        .boot_a
        .websocket_document
        .replace(SESSION_A, SESSION_B);
    let path = fixture.input.inventory[9].path.clone();
    fixture.artifacts.insert(
        path,
        InventoryArtifact::regular(fixture.input.boot_a.websocket_document.as_bytes()),
    );
    fixture.input.inventory[9].sha256 =
        sha256_hex(fixture.input.boot_a.websocket_document.as_bytes());
    fixture.reseal();

    // Act
    let error = fixture.validate().expect_err("mixed session must fail");

    // Assert
    assert_eq!(error, Phase35EvidenceError::MixedSession);
}

#[test]
fn phase35_evidence_rejects_unsafe_inventory_paths_and_symlinks() {
    // Arrange
    let mut unsafe_fixture = EligibleFixture::new();
    unsafe_fixture.input.inventory[0].path = "../manifest.json".to_owned();
    let mut symlink_fixture = EligibleFixture::new();
    let symlink_path = symlink_fixture.input.inventory[0].path.clone();
    symlink_fixture
        .artifacts
        .insert(symlink_path, InventoryArtifact::symlink());

    // Act
    let unsafe_error = unsafe_fixture
        .validate()
        .expect_err("traversal path must fail");
    let symlink_error = symlink_fixture.validate().expect_err("symlink must fail");

    // Assert
    assert_eq!(unsafe_error, Phase35EvidenceError::UnsafePath);
    assert_eq!(symlink_error, Phase35EvidenceError::Symlink);
}

#[test]
fn phase35_projection_rejects_forbidden_keys_and_raw_canaries() {
    // Arrange
    let forbidden = serde_json::json!({"device_path": "redacted"});
    let raw_canary = serde_json::json!({"session_digest": "sentinel-private"});

    // Act
    let forbidden_error =
        validate_projection_value(&forbidden, &[]).expect_err("raw key must fail");
    let canary_error = validate_projection_value(&raw_canary, &["sentinel-private".to_owned()])
        .expect_err("raw canary must fail");

    // Assert
    assert_eq!(
        forbidden_error,
        Phase35EvidenceError::ForbiddenProjectionField
    );
    assert_eq!(canary_error, Phase35EvidenceError::ForbiddenProjectionField);
}

mod admission;
mod fixtures;
