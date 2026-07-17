use serde::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InvalidFixtureDescriptor {
    category: String,
    expected_error: String,
}

fn assert_descriptor(document: &str, category: &str, expected: Phase35EvidenceError) {
    let descriptor: InvalidFixtureDescriptor =
        serde_json::from_str(document).expect("invalid fixture descriptor must parse");
    assert_eq!(descriptor.category, category);
    assert_eq!(descriptor.expected_error, format!("{expected:?}"));
}

fn drift_artifact(fixture: &mut EligibleFixture, role: &str) {
    let path = fixture
        .input
        .inventory
        .iter()
        .find(|entry| entry.role == role)
        .expect("eligible fixture contains every role")
        .path
        .clone();
    fixture
        .artifacts
        .insert(path, InventoryArtifact::regular(b"synthetic drift"));
}

macro_rules! invalid_evidence_case {
    ($test_name:ident, $file_name:literal, $category:literal, $expected:expr, $mutate:expr) => {
        #[test]
        fn $test_name() {
            // Arrange
            let descriptor =
                include_str!(concat!("../../../fixtures/phase35/invalid/", $file_name));
            let mut fixture = EligibleFixture::new();
            ($mutate)(&mut fixture);

            // Act
            let error = fixture
                .validate()
                .expect_err("invalid fixture mutation must fail closed");

            // Assert
            assert_descriptor(descriptor, $category, $expected);
            assert_eq!(error, $expected);
        }
    };
}

invalid_evidence_case!(
    source_drift_is_rejected,
    "source-drift.json",
    "source_drift",
    Phase35EvidenceError::InvalidPackageCapability,
    |fixture: &mut EligibleFixture| fixture.input.exact_package.source_commit =
        "1123456789abcdef0123456789abcdef01234567".to_owned()
);
invalid_evidence_case!(
    reference_drift_is_rejected,
    "reference-drift.json",
    "reference_drift",
    Phase35EvidenceError::InvalidPackageCapability,
    |fixture: &mut EligibleFixture| fixture.input.exact_package.reference_commit =
        "99abcdef0123456789abcdef0123456789abcdef".to_owned()
);
invalid_evidence_case!(
    manifest_v3_drift_is_rejected,
    "manifest-v3-drift.json",
    "manifest_v3_drift",
    Phase35EvidenceError::ManifestV3Mismatch,
    |fixture: &mut EligibleFixture| fixture.input.exact_package.manifest_schema =
        "manifest-v2".to_owned()
);
invalid_evidence_case!(
    executable_image_drift_is_rejected,
    "executable-image-drift.json",
    "executable_image_drift",
    Phase35EvidenceError::ExecutableImageMismatch,
    |fixture: &mut EligibleFixture| drift_artifact(fixture, "executable_image")
);
invalid_evidence_case!(
    factory_image_drift_is_rejected,
    "factory-image-drift.json",
    "factory_image_drift",
    Phase35EvidenceError::FactoryImageMismatch,
    |fixture: &mut EligibleFixture| drift_artifact(fixture, "factory_image")
);
invalid_evidence_case!(
    package_drift_is_rejected,
    "package-drift.json",
    "package_drift",
    Phase35EvidenceError::PackageMismatch,
    |fixture: &mut EligibleFixture| drift_artifact(fixture, "package")
);
invalid_evidence_case!(
    runtime_identity_drift_is_rejected,
    "runtime-identity-drift.json",
    "runtime_identity_drift",
    Phase35EvidenceError::RuntimeIdentityMismatch,
    |fixture: &mut EligibleFixture| drift_artifact(fixture, "runtime_identity")
);
invalid_evidence_case!(
    package_capability_drift_is_rejected,
    "package-capability-drift.json",
    "package_capability_drift",
    Phase35EvidenceError::InvalidPackageCapability,
    |fixture: &mut EligibleFixture| fixture.input.exact_package.capability_digest = "a".repeat(64)
);
invalid_evidence_case!(
    detector_capability_drift_is_rejected,
    "detector-capability-drift.json",
    "detector_capability_drift",
    Phase35EvidenceError::InvalidDetectorCapability,
    |fixture: &mut EligibleFixture| fixture.input.detector_run.capability_digest = "b".repeat(64)
);
invalid_evidence_case!(
    root_contract_drift_is_rejected,
    "root-contract-drift.json",
    "root_contract_drift",
    Phase35EvidenceError::RootContractMismatch,
    |fixture: &mut EligibleFixture| fixture.input.admission_facts.root_contract_digest =
        "c".repeat(64)
);
invalid_evidence_case!(
    target_lock_drift_is_rejected,
    "target-lock-drift.json",
    "target_lock_drift",
    Phase35EvidenceError::RootContractMismatch,
    |fixture: &mut EligibleFixture| fixture.input.boot_b.target_lock_digest = "d".repeat(64)
);
invalid_evidence_case!(
    current_head_false_is_rejected,
    "current-head-false.json",
    "current_head_false",
    Phase35EvidenceError::StaleCurrentHead,
    |fixture: &mut EligibleFixture| fixture.input.exact_package.current_head_verified = false
);
invalid_evidence_case!(
    reference_cleanliness_false_is_rejected,
    "reference-cleanliness-false.json",
    "reference_cleanliness_false",
    Phase35EvidenceError::DirtyReference,
    |fixture: &mut EligibleFixture| fixture.input.exact_package.reference_clean = false
);
invalid_evidence_case!(
    lifecycle_false_is_rejected,
    "lifecycle-false.json",
    "lifecycle_false",
    Phase35EvidenceError::LifecycleMismatch,
    |fixture: &mut EligibleFixture| fixture.input.admission_facts.lifecycle_verified = false
);
invalid_evidence_case!(
    no_actuation_false_is_rejected,
    "no-actuation-false.json",
    "no_actuation_false",
    Phase35EvidenceError::NoActuationFailure,
    |fixture: &mut EligibleFixture| fixture.input.admission_facts.no_actuation_verified = false
);
invalid_evidence_case!(
    wrong_board_is_rejected,
    "wrong-board.json",
    "wrong_board",
    Phase35EvidenceError::WrongBoard,
    |fixture: &mut EligibleFixture| fixture.input.detector_run.board_category = "other".to_owned()
);
invalid_evidence_case!(
    invalid_digest_is_rejected,
    "invalid-digest.json",
    "invalid_digest",
    Phase35EvidenceError::InvalidDigest,
    |fixture: &mut EligibleFixture| fixture.input.admission_facts.target_lock_digest =
        "A".repeat(64)
);
invalid_evidence_case!(
    missing_inventory_role_is_rejected,
    "missing-inventory-role.json",
    "missing_inventory_role",
    Phase35EvidenceError::InventoryMismatch,
    |fixture: &mut EligibleFixture| {
        fixture.input.inventory.pop();
    }
);
invalid_evidence_case!(
    extra_inventory_role_is_rejected,
    "extra-inventory-role.json",
    "extra_inventory_role",
    Phase35EvidenceError::InventoryMismatch,
    |fixture: &mut EligibleFixture| {
        let mut extra = fixture.input.inventory[0].clone();
        extra.role = "synthetic_extra".to_owned();
        fixture.input.inventory.push(extra);
    }
);
invalid_evidence_case!(
    duplicate_inventory_role_is_rejected,
    "duplicate-inventory-role.json",
    "duplicate_inventory_role",
    Phase35EvidenceError::InventoryMismatch,
    |fixture: &mut EligibleFixture| fixture.input.inventory[1].role =
        fixture.input.inventory[0].role.clone()
);
invalid_evidence_case!(
    path_traversal_is_rejected,
    "path-traversal.json",
    "path_traversal",
    Phase35EvidenceError::UnsafePath,
    |fixture: &mut EligibleFixture| fixture.input.inventory[0].path = "../synthetic".to_owned()
);
invalid_evidence_case!(
    symlink_is_rejected,
    "symlink.json",
    "symlink",
    Phase35EvidenceError::Symlink,
    |fixture: &mut EligibleFixture| {
        let path = fixture.input.inventory[0].path.clone();
        fixture.artifacts.insert(path, InventoryArtifact::symlink());
    }
);
invalid_evidence_case!(
    digest_mismatch_is_rejected,
    "digest-mismatch.json",
    "digest_mismatch",
    Phase35EvidenceError::PackageMismatch,
    |fixture: &mut EligibleFixture| fixture.input.inventory[0].sha256 = "e".repeat(64)
);
invalid_evidence_case!(
    chronology_inversion_is_rejected,
    "chronology-inversion.json",
    "chronology_inversion",
    Phase35EvidenceError::ChronologyViolation,
    |fixture: &mut EligibleFixture| fixture.input.boot_a.ended_millis =
        fixture.input.boot_a.started_millis
);
invalid_evidence_case!(
    sequence_inversion_is_rejected,
    "sequence-inversion.json",
    "sequence_inversion",
    Phase35EvidenceError::ChronologyViolation,
    |fixture: &mut EligibleFixture| fixture.input.events[2].sequence = 4
);
invalid_evidence_case!(
    broken_predecessor_is_rejected,
    "broken-predecessor.json",
    "broken_predecessor",
    Phase35EvidenceError::PredecessorViolation,
    |fixture: &mut EligibleFixture| fixture.input.events[2].predecessor_event_digest =
        "f".repeat(64)
);
invalid_evidence_case!(
    duplicate_checkpoint_is_rejected,
    "duplicate-checkpoint.json",
    "duplicate_checkpoint",
    Phase35EvidenceError::DuplicateCheckpoint,
    |fixture: &mut EligibleFixture| fixture.input.events[2].category =
        EvidenceEventCategory::BootAObserved
);
invalid_evidence_case!(
    mixed_boot_session_is_rejected,
    "mixed-boot-session.json",
    "mixed_boot_session",
    Phase35EvidenceError::MixedSession,
    |fixture: &mut EligibleFixture| {
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
        fixture.reseal();
    }
);
invalid_evidence_case!(
    skipped_reboot_is_rejected,
    "skipped-reboot.json",
    "skipped_reboot",
    Phase35EvidenceError::BootOrdinalMismatch,
    |fixture: &mut EligibleFixture| {
        fixture.input.boot_b.boot_ordinal = fixture.input.boot_a.boot_ordinal;
        fixture.reseal();
    }
);
invalid_evidence_case!(
    additional_reboot_is_rejected,
    "additional-reboot.json",
    "additional_reboot",
    Phase35EvidenceError::BootOrdinalMismatch,
    |fixture: &mut EligibleFixture| {
        fixture.input.boot_b.boot_ordinal = fixture.input.boot_a.boot_ordinal + 2;
        fixture.reseal();
    }
);
invalid_evidence_case!(
    wrong_reset_category_is_rejected,
    "wrong-reset-category.json",
    "wrong_reset_category",
    Phase35EvidenceError::UnapprovedReset,
    |fixture: &mut EligibleFixture| fixture.input.boot_b.reset_category = "other".to_owned()
);
invalid_evidence_case!(
    physical_identity_drift_is_rejected,
    "physical-identity-drift.json",
    "physical_identity_drift",
    Phase35EvidenceError::PhysicalIdentityDrift,
    |fixture: &mut EligibleFixture| fixture.input.boot_b.physical_identity_digest = "1".repeat(64)
);
invalid_evidence_case!(
    storage_revision_mismatch_is_rejected,
    "storage-revision-mismatch.json",
    "storage_revision_mismatch",
    Phase35EvidenceError::ValueRevisionMismatch,
    |fixture: &mut EligibleFixture| fixture.input.boot_b.storage_revision += 1
);
invalid_evidence_case!(
    hostname_digest_mismatch_is_rejected,
    "hostname-digest-mismatch.json",
    "hostname_digest_mismatch",
    Phase35EvidenceError::RuntimeIdentityMismatch,
    |fixture: &mut EligibleFixture| fixture.input.boot_b.runtime_identity_digest = "2".repeat(64)
);
invalid_evidence_case!(
    restoration_false_is_rejected,
    "restoration-false.json",
    "restoration_false",
    Phase35EvidenceError::RestorationFailure,
    |fixture: &mut EligibleFixture| fixture.input.admission_facts.restoration_verified = false
);
invalid_evidence_case!(
    process_tree_cleanup_false_is_rejected,
    "process-tree-cleanup-false.json",
    "process_tree_cleanup_false",
    Phase35EvidenceError::CleanupFailure,
    |fixture: &mut EligibleFixture| fixture.input.admission_facts.cleanup_verified = false
);
invalid_evidence_case!(
    unexpected_serial_holder_is_rejected,
    "unexpected-serial-holder.json",
    "unexpected_serial_holder",
    Phase35EvidenceError::CleanupFailure,
    |fixture: &mut EligibleFixture| fixture.input.admission_facts.cleanup_verified = false
);

macro_rules! invalid_projection_case {
    ($test_name:ident, $file_name:literal, $category:literal, $value:expr, $canaries:expr) => {
        #[test]
        fn $test_name() {
            // Arrange
            let descriptor =
                include_str!(concat!("../../../fixtures/phase35/invalid/", $file_name));
            let value = $value;
            let canaries = $canaries;

            // Act
            let error = validate_projection_value(&value, &canaries)
                .expect_err("unsafe projection must fail closed");

            // Assert
            assert_descriptor(
                descriptor,
                $category,
                Phase35EvidenceError::ForbiddenProjectionField,
            );
            assert_eq!(error, Phase35EvidenceError::ForbiddenProjectionField);
        }
    };
}

invalid_projection_case!(
    forbidden_raw_key_is_rejected,
    "forbidden-raw-key.json",
    "forbidden_raw_key",
    serde_json::json!({"device_path": "synthetic"}),
    Vec::<String>::new()
);
invalid_projection_case!(
    raw_value_canary_is_rejected,
    "raw-value-canary.json",
    "raw_value_canary",
    serde_json::json!({"session_digest": "synthetic-private-canary"}),
    vec!["synthetic-private-canary".to_owned()]
);

mod catalog;
