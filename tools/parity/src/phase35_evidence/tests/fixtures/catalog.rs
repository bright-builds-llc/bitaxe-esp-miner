use super::*;

const ELIGIBLE_FIXTURE: &str = include_str!("../../../../fixtures/phase35/eligible.json");

#[test]
fn eligible_json_parses_and_passes_the_validator() {
    // Arrange
    let input: Phase35EvidenceRootInput =
        serde_json::from_str(ELIGIBLE_FIXTURE).expect("eligible fixture parses");
    let mut fixture = EligibleFixture::new();
    fixture.input = input;

    // Act
    let result = fixture.validate();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn invalid_fixture_taxonomy_is_complete_and_synthetic() {
    // Arrange
    let expected_categories = [
        "source_drift",
        "reference_drift",
        "manifest_v3_drift",
        "executable_image_drift",
        "factory_image_drift",
        "package_drift",
        "runtime_identity_drift",
        "package_capability_drift",
        "detector_capability_drift",
        "root_contract_drift",
        "target_lock_drift",
        "current_head_false",
        "reference_cleanliness_false",
        "lifecycle_false",
        "no_actuation_false",
        "wrong_board",
        "invalid_digest",
        "missing_inventory_role",
        "extra_inventory_role",
        "duplicate_inventory_role",
        "path_traversal",
        "symlink",
        "digest_mismatch",
        "chronology_inversion",
        "sequence_inversion",
        "broken_predecessor",
        "duplicate_checkpoint",
        "mixed_boot_session",
        "skipped_reboot",
        "additional_reboot",
        "wrong_reset_category",
        "physical_identity_drift",
        "storage_revision_mismatch",
        "hostname_digest_mismatch",
        "restoration_false",
        "process_tree_cleanup_false",
        "unexpected_serial_holder",
        "forbidden_raw_key",
        "raw_value_canary",
    ];
    let descriptors = [
        include_str!("../../../../fixtures/phase35/invalid/source-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/reference-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/manifest-v3-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/executable-image-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/factory-image-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/package-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/runtime-identity-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/package-capability-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/detector-capability-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/root-contract-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/target-lock-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/current-head-false.json"),
        include_str!("../../../../fixtures/phase35/invalid/reference-cleanliness-false.json"),
        include_str!("../../../../fixtures/phase35/invalid/lifecycle-false.json"),
        include_str!("../../../../fixtures/phase35/invalid/no-actuation-false.json"),
        include_str!("../../../../fixtures/phase35/invalid/wrong-board.json"),
        include_str!("../../../../fixtures/phase35/invalid/invalid-digest.json"),
        include_str!("../../../../fixtures/phase35/invalid/missing-inventory-role.json"),
        include_str!("../../../../fixtures/phase35/invalid/extra-inventory-role.json"),
        include_str!("../../../../fixtures/phase35/invalid/duplicate-inventory-role.json"),
        include_str!("../../../../fixtures/phase35/invalid/path-traversal.json"),
        include_str!("../../../../fixtures/phase35/invalid/symlink.json"),
        include_str!("../../../../fixtures/phase35/invalid/digest-mismatch.json"),
        include_str!("../../../../fixtures/phase35/invalid/chronology-inversion.json"),
        include_str!("../../../../fixtures/phase35/invalid/sequence-inversion.json"),
        include_str!("../../../../fixtures/phase35/invalid/broken-predecessor.json"),
        include_str!("../../../../fixtures/phase35/invalid/duplicate-checkpoint.json"),
        include_str!("../../../../fixtures/phase35/invalid/mixed-boot-session.json"),
        include_str!("../../../../fixtures/phase35/invalid/skipped-reboot.json"),
        include_str!("../../../../fixtures/phase35/invalid/additional-reboot.json"),
        include_str!("../../../../fixtures/phase35/invalid/wrong-reset-category.json"),
        include_str!("../../../../fixtures/phase35/invalid/physical-identity-drift.json"),
        include_str!("../../../../fixtures/phase35/invalid/storage-revision-mismatch.json"),
        include_str!("../../../../fixtures/phase35/invalid/hostname-digest-mismatch.json"),
        include_str!("../../../../fixtures/phase35/invalid/restoration-false.json"),
        include_str!("../../../../fixtures/phase35/invalid/process-tree-cleanup-false.json"),
        include_str!("../../../../fixtures/phase35/invalid/unexpected-serial-holder.json"),
        include_str!("../../../../fixtures/phase35/invalid/forbidden-raw-key.json"),
        include_str!("../../../../fixtures/phase35/invalid/raw-value-canary.json"),
    ];

    // Act
    let actual_categories = descriptors
        .iter()
        .map(|document| {
            serde_json::from_str::<InvalidFixtureDescriptor>(document)
                .expect("every invalid descriptor parses")
                .category
        })
        .collect::<Vec<_>>();
    let fixture_text = format!("{ELIGIBLE_FIXTURE}\n{}", descriptors.join("\n"));

    // Assert
    assert_eq!(actual_categories, expected_categories);
    for forbidden in [
        "http://",
        "https://",
        "/dev/",
        "ssid",
        "btc",
        "worker",
        "pool_endpoint",
        "credential",
        "secret",
    ] {
        assert!(!fixture_text.to_ascii_lowercase().contains(forbidden));
    }
}
