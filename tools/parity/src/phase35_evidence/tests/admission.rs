use super::*;

type AdmissionMutation = fn(&mut RootAdmissionFacts);
type AdmissionCase = (&'static str, AdmissionMutation, Phase35EvidenceError);
type FixtureMutation = fn(&mut EligibleFixture);
type FixtureCase = (&'static str, FixtureMutation, Phase35EvidenceError);

#[test]
fn phase35_evidence_binds_package_artifact_identities() {
    let cases = [
        ("package_manifest", Phase35EvidenceError::PackageMismatch),
        (
            "executable_image",
            Phase35EvidenceError::ExecutableImageMismatch,
        ),
        ("factory_image", Phase35EvidenceError::FactoryImageMismatch),
        ("package", Phase35EvidenceError::PackageMismatch),
        (
            "runtime_identity",
            Phase35EvidenceError::RuntimeIdentityMismatch,
        ),
    ];

    for (role, expected) in cases {
        // Arrange
        let mut fixture = EligibleFixture::new();
        let path = fixture
            .input
            .inventory
            .iter()
            .find(|entry| entry.role == role)
            .expect("fixture contains every package artifact role")
            .path
            .clone();
        fixture
            .artifacts
            .insert(path, InventoryArtifact::regular(b"drifted artifact"));

        // Act
        let error = fixture.validate().expect_err("identity drift must fail");

        // Assert
        assert_eq!(error, expected, "{role}");
    }
}

#[test]
fn phase35_evidence_requires_every_admission_fact() {
    let cases: [AdmissionCase; 10] = [
        (
            "current head",
            |facts| facts.current_head_rechecked = false,
            Phase35EvidenceError::StaleCurrentHead,
        ),
        (
            "reference cleanliness",
            |facts| facts.reference_cleanliness_rechecked = false,
            Phase35EvidenceError::DirtyReference,
        ),
        (
            "lifecycle",
            |facts| facts.lifecycle_verified = false,
            Phase35EvidenceError::LifecycleMismatch,
        ),
        (
            "runtime identity",
            |facts| facts.runtime_identity_rechecked = false,
            Phase35EvidenceError::RuntimeIdentityMismatch,
        ),
        (
            "no actuation",
            |facts| facts.no_actuation_verified = false,
            Phase35EvidenceError::NoActuationFailure,
        ),
        (
            "restoration",
            |facts| facts.restoration_verified = false,
            Phase35EvidenceError::RestorationFailure,
        ),
        (
            "cleanup",
            |facts| facts.cleanup_verified = false,
            Phase35EvidenceError::CleanupFailure,
        ),
        (
            "inventory",
            |facts| facts.inventory_verified = false,
            Phase35EvidenceError::InventoryMismatch,
        ),
        (
            "chronology",
            |facts| facts.chronology_verified = false,
            Phase35EvidenceError::ChronologyViolation,
        ),
        (
            "redaction",
            |facts| facts.redaction_verified = false,
            Phase35EvidenceError::ForbiddenProjectionField,
        ),
    ];

    for (name, mutate, expected) in cases {
        // Arrange
        let mut fixture = EligibleFixture::new();
        mutate(&mut fixture.input.admission_facts);

        // Act
        let error = fixture.validate().expect_err("false fact must fail");

        // Assert
        assert_eq!(error, expected, "{name}");
    }
}

#[test]
fn phase35_evidence_rejects_incomplete_or_ambiguous_inventory() {
    let cases: [FixtureCase; 5] = [
        (
            "absolute path",
            |fixture| fixture.input.inventory[0].path = "/manifest.json".to_owned(),
            Phase35EvidenceError::UnsafePath,
        ),
        (
            "duplicate role",
            |fixture| fixture.input.inventory[1].role = fixture.input.inventory[0].role.clone(),
            Phase35EvidenceError::InventoryMismatch,
        ),
        (
            "missing role",
            |fixture| {
                fixture.input.inventory.pop();
            },
            Phase35EvidenceError::InventoryMismatch,
        ),
        (
            "extra role",
            |fixture| {
                let mut extra = fixture.input.inventory[0].clone();
                extra.role = "unexpected".to_owned();
                fixture.input.inventory.push(extra);
            },
            Phase35EvidenceError::InventoryMismatch,
        ),
        (
            "digest mismatch",
            |fixture| fixture.input.inventory[0].sha256 = "f".repeat(64),
            Phase35EvidenceError::PackageMismatch,
        ),
    ];

    for (name, mutate, expected) in cases {
        // Arrange
        let mut fixture = EligibleFixture::new();
        mutate(&mut fixture);

        // Act
        let error = fixture
            .validate()
            .expect_err("inventory mutation must fail");

        // Assert
        assert_eq!(error, expected, "{name}");
    }
}

#[test]
fn phase35_evidence_binds_event_payload_and_predecessor_digests() {
    // Arrange
    let mut payload_fixture = EligibleFixture::new();
    payload_fixture.input.events[3].payload_digest = "a".repeat(64);
    let mut predecessor_fixture = EligibleFixture::new();
    predecessor_fixture.input.events[3].predecessor_event_digest = "b".repeat(64);

    // Act
    let payload_error = payload_fixture
        .validate()
        .expect_err("payload drift must fail");
    let predecessor_error = predecessor_fixture
        .validate()
        .expect_err("predecessor drift must fail");

    // Assert
    assert_eq!(payload_error, Phase35EvidenceError::PredecessorViolation);
    assert_eq!(
        predecessor_error,
        Phase35EvidenceError::PredecessorViolation
    );
}

#[test]
fn phase35_cli_contract_never_renders_raw_paths_or_values() {
    // Arrange
    let fixture = EligibleFixture::new();
    let projection = fixture
        .validate()
        .expect("eligible fixture validates")
        .shareable_projection()
        .expect("eligible fixture projects");
    let sentinel_path = "private/device/path";
    let sentinel_value = "private-input-value";

    // Act
    let rendered = serde_json::to_string(&projection).expect("projection serializes");
    let rendered_error = Phase35EvidenceError::InventoryMismatch.to_string();

    // Assert
    for sentinel in [sentinel_path, sentinel_value] {
        assert!(!rendered.contains(sentinel));
        assert!(!rendered_error.contains(sentinel));
    }
}
