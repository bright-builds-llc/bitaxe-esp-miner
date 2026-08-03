use super::*;

#[test]
fn phase34_package_admission_manifest_uses_stamped_provenance() {
    // Arrange
    let required_manifest_markers = [
        "BuildProvenance::parse_stamp",
        "schema_version: 3",
        "app_elf_sha256",
        "validate_package_manifest_v3(&manifest)?",
    ];
    let required_validation_markers = [
        "validate_firmware_elf_app_sha_relationship",
        "firmware_elf_app_sha_mismatch",
    ];
    let forbidden_manifest_markers = ["Command::new", "git describe"];
    let forbidden_xtask_markers = ["fn firmware_commit", "fn reference_commit"];

    // Act
    let manifest_builder = source_between(
        PACKAGE_MANIFEST_SOURCE,
        "pub(crate) fn build_manifest",
        "pub(crate) fn validate_default_flash_image",
    );

    // Assert
    assert_contains_all(
        manifest_builder,
        &required_manifest_markers,
        "package manifest provenance",
    );
    assert_contains_all(
        PACKAGE_MANIFEST_SOURCE,
        &required_validation_markers,
        "firmware ELF application identity validation",
    );
    assert_excludes_all(
        &[manifest_builder],
        &forbidden_manifest_markers,
        "package manifest runtime discovery",
    );
    assert_excludes_all(
        &[XTASK_SOURCE],
        &forbidden_xtask_markers,
        "xtask runtime identity derivation",
    );
}

#[test]
fn phase34_package_admission_snapshots_bytes_before_device_effects() {
    // Arrange
    let preparation_order = [
        "resolve_flash_image",
        "create_admitted_execution_snapshot",
        "resolve_port",
    ];
    let execution_markers = [
        "_execution_snapshot",
        "environment.begin_usb_session",
        "environment.execute(&execution_command)",
    ];
    let execution_order = [
        "environment.begin_usb_session",
        "environment.execute(&execution_command)",
    ];

    // Act
    let flash_preparation = source_between(
        FLASH_PACKAGE_SOURCE,
        "fn prepare_flash",
        "fn flash_command_for_admitted_image",
    );
    let flash_execution = source_between(FLASH_EXECUTION_SOURCE, "fn run_flash", "fn run_monitor");

    // Assert
    assert_markers_in_order(flash_preparation, &preparation_order, "flash preparation");
    assert_contains_all(
        flash_execution,
        &execution_markers,
        "admitted snapshot execution",
    );
    assert_markers_in_order(flash_execution, &execution_order, "USB effect execution");
}

#[test]
fn phase34_package_admission_snapshot_is_private_and_durable() {
    // Arrange
    let required_snapshot_markers = [
        "NamedTempFile",
        "write_all",
        "flush",
        "sync_all",
        "set_mode(0o600)",
    ];

    // Act
    let snapshot_materialization = source_between(
        FLASH_EXECUTION_SNAPSHOT_SOURCE,
        "impl AdmittedExecutionSnapshot",
        "pub(crate) fn path",
    );

    // Assert
    assert_contains_all(
        snapshot_materialization,
        &required_snapshot_markers,
        "immutable snapshot",
    );
}

#[test]
fn phase34_flash_model_stays_independent_of_effectful_resource_owners() {
    // Arrange
    let prohibited_effect_tokens = [
        "tempfile",
        "NamedTempFile",
        "TempDir",
        "std::fs",
        "std::process",
        "std::net",
        "std::time",
        "std::io",
    ];

    // Act / Assert
    assert_excludes_all(
        &[FLASH_MODEL_SOURCE],
        &prohibited_effect_tokens,
        "pure flash model effect imports",
    );
}

#[test]
fn phase34_package_admission_flash_command_requires_typed_images() {
    // Arrange
    let required_builder_markers = [
        "AdmittedFlashImage::Factory",
        "AdmittedFlashImage::DeveloperDryRun",
    ];
    let forbidden_builder_markers = ["file_name", "FACTORY_IMAGE_NAME"];
    let required_model_markers = [
        "struct AdmittedFactoryImage",
        "enum AdmittedFlashImage",
        "struct AdmittedExecutionSnapshot",
        "<admitted-factory-snapshot>",
    ];
    let required_package_markers = [
        "explicit_image_not_admitted_factory",
        "read_validated_artifact",
    ];
    let forbidden_bypass_markers = [
        "require_manifest_artifact_for_path",
        "validate_artifact_digest_for_path",
        "resolve_manifest_flash_image",
        "environment.read_bytes(&factory_path)",
    ];

    // Act
    let admitted_command_builder = source_between(
        FLASH_PACKAGE_SOURCE,
        "fn flash_command_for_admitted_image",
        "fn resolve_flash_image",
    );
    let admitted_image_sources = [
        FLASH_MODEL_SOURCE,
        FLASH_EXECUTION_SNAPSHOT_SOURCE,
        FLASH_PACKAGE_SOURCE,
    ];
    let bypass_sources = [
        FLASH_PACKAGE_SOURCE,
        FLASH_EXECUTION_SOURCE,
        FLASH_MODEL_SOURCE,
    ];

    // Assert
    assert_contains_all(
        admitted_command_builder,
        &required_builder_markers,
        "typed admitted-image command",
    );
    assert_excludes_all(
        &[admitted_command_builder],
        &forbidden_builder_markers,
        "filename-based image selection",
    );
    assert_contains_in_any(
        &admitted_image_sources,
        &required_model_markers,
        "admitted-image model",
    );
    assert_contains_all(
        FLASH_PACKAGE_SOURCE,
        &required_package_markers,
        "exact admitted-image package gate",
    );
    assert_excludes_all(
        &bypass_sources,
        &forbidden_bypass_markers,
        "admission bypass",
    );
}

#[test]
fn phase34_package_admission_rejects_dirty_or_mismatched_workspaces() {
    // Arrange
    let required_workspace_gates = [
        "package_source_dirty",
        "current_workspace_dirty",
        "package_workspace_identity_mismatch",
    ];

    // Act
    let package_source = FLASH_PACKAGE_SOURCE;

    // Assert
    assert_contains_all(
        package_source,
        &required_workspace_gates,
        "workspace identity admission",
    );
}

#[test]
fn phase34_package_admission_binds_factory_and_ota_layouts() {
    // Arrange
    let required_admission_markers = [
        "validate_factory_ota_identity",
        "PartitionTable::try_from_bytes",
        "factory_ota_image_mismatch",
        "package_admission_rejects_non_drom_descriptor_in_ota_and_factory",
        "package_admission_rejects_destination_overlap_in_ota_and_factory",
        "package_admission_rejects_alias_overlap_in_ota_and_factory",
        "package_admission_accepts_exact_destination_and_alias_adjacency",
        "package_admission_accepts_range_free_zero_length_segment",
    ];

    // Act
    let admission_sources = [
        FLASH_PACKAGE_ADMISSION_SOURCE,
        FLASH_PACKAGE_ADMISSION_TEST_SOURCE,
    ];

    // Assert
    assert_contains_in_any(
        &admission_sources,
        &required_admission_markers,
        "factory and OTA package admission",
    );
}

#[test]
fn phase34_package_admission_esp32s3_layout_policy_is_closed() {
    // Arrange
    let required_structural_markers = [
        "ESP_APP_DESCRIPTOR_MAGIC",
        "ESP32_S3_CHIP_ID",
        "SPI_MODE_DIO",
        "SPI_SPEED_80MHZ_SIZE_16MB",
        "APP_MMU_PAGE_SIZE_LOG2",
        "MappedSegmentMisaligned",
        "EntryAddressUnsupported",
        "ota_chip_id_mismatch",
        "ota_header_policy_unsupported",
        "ota_segment_load_address_unsupported",
        "ota_entry_address_unsupported",
        "ota_segment_checksum_mismatch",
        "ota_alignment_padding_invalid",
        "ota_appended_sha256_mismatch",
        "ota_appended_sha256_truncated",
        "ota_trailing_data",
        "embedded_source_commit_mismatch",
        "app_descriptor_version_mismatch",
        "app_descriptor_sha_mismatch",
        "app_descriptor_mmu_page_size_mismatch",
        "app_descriptor_segment_empty",
        "app_descriptor_segment_not_drom",
        "ota_segment_destination_overlap",
        "ota_segment_alias_overlap",
        "ValidatedSegmentLayout",
        "SOC_I_D_OFFSET",
        "0x006f_0000",
    ];
    let layout_validation_order = [
        "validate_descriptor_segment",
        "validate_destination_disjointness",
        "validate_alias_disjointness",
    ];
    let required_layout_signatures = [
        "fn validate_entry_address(\n    entry_address: u32,\n    layout: &ValidatedSegmentLayout,",
        "fn validate_descriptor(\n    image: &[u8],\n    layout: &ValidatedSegmentLayout,",
    ];

    // Act
    let layout_constructor = source_between(
        FLASH_ESP32S3_IMAGE_SOURCE,
        "impl ValidatedSegmentLayout",
        "fn validate_header",
    );

    // Assert
    assert_contains_all(
        FLASH_ESP32S3_IMAGE_SOURCE,
        &required_structural_markers,
        "ESP32-S3 structural admission",
    );
    assert_markers_in_order(
        layout_constructor,
        &layout_validation_order,
        "validated segment layout",
    );
    assert_contains_all(
        FLASH_ESP32S3_IMAGE_SOURCE,
        &required_layout_signatures,
        "ESP32-S3 layout-aware validation signatures",
    );
}

#[test]
fn phase34_package_admission_orders_digest_and_structural_binding() {
    // Arrange
    let factory_binding_order = [
        "read_validated_artifact(factory_artifact",
        "validate_factory_ota_identity",
    ];
    let elf_binding_order = [
        "read_validated_artifact(elf_artifact",
        "firmware_elf_app_sha_mismatch",
        "read_validated_artifact(ota_artifact",
    ];
    let forbidden_byte_searches = ["contains_bytes(&ota_bytes", "contains_bytes(&factory_bytes"];

    // Act
    let identity_admission = source_between(
        FLASH_PACKAGE_SOURCE,
        "fn validate_identity_admission",
        "fn require_artifact",
    );

    // Assert
    assert_markers_in_order(
        identity_admission,
        &factory_binding_order,
        "factory digest and structural binding",
    );
    assert_markers_in_order(
        identity_admission,
        &elf_binding_order,
        "firmware ELF and OTA identity binding",
    );
    assert_excludes_all(
        &[FLASH_PACKAGE_SOURCE],
        &forbidden_byte_searches,
        "unstructured factory and OTA byte search",
    );
}

#[test]
fn phase34_package_admission_layout_failures_stop_before_effects() {
    // Arrange
    let required_layout_regressions = [
        "identity_admission_rejects_all_layout_classes_in_parsed_dry_run_before_effects",
        "identity_admission_rejects_all_layout_classes_in_parsed_non_dry_run_before_effects",
        "assert_parsed_layout_rejected_before_effects",
    ];
    let required_effect_counters = [
        "list_ports_calls",
        "read_string_paths",
        "generated_nvs_partitions",
        "created_snapshot_paths",
        "captured_commands",
        "executed_commands",
        "observed_flashes",
    ];

    // Act
    let layout_regression_sources = [
        FLASH_ADMISSION_LAYOUT_TEST_SOURCE,
        FLASH_ADMISSION_FIXTURE_SOURCE,
    ];

    // Assert
    assert_contains_in_any(
        &layout_regression_sources,
        &required_layout_regressions,
        "parsed pre-effect layout regression",
    );
    assert_contains_all(
        FLASH_FAKE_ENVIRONMENT_SOURCE,
        &required_effect_counters,
        "pre-effect fake-environment observation",
    );
}

#[test]
fn phase34_package_admission_uses_managed_tools_and_provenance() {
    // Arrange
    let required_script_markers = [
        "image_info",
        "--elf-sha256-offset",
        "partitionTableBin",
        "--build-provenance-stamp",
    ];
    let forbidden_script_markers = ["save-image"];

    // Act
    let package_script = include_str!("../../../automation/src/package.ts");

    // Assert
    assert_contains_all(
        package_script,
        &required_script_markers,
        "package script managed tooling",
    );
    assert_excludes_all(
        &[package_script],
        &forbidden_script_markers,
        "legacy espflash image generation",
    );
}

fn assert_contains_all(source: &str, markers: &[&str], context: &str) {
    for marker in markers {
        assert!(source.contains(marker), "missing {context} marker {marker}");
    }
}

fn assert_contains_in_any(sources: &[&str], markers: &[&str], context: &str) {
    for marker in markers {
        assert!(
            sources.iter().any(|source| source.contains(marker)),
            "missing {context} marker {marker}"
        );
    }
}

fn assert_excludes_all(sources: &[&str], markers: &[&str], context: &str) {
    for marker in markers {
        assert!(
            sources.iter().all(|source| !source.contains(marker)),
            "forbidden {context} marker remains: {marker}"
        );
    }
}

fn assert_markers_in_order(source: &str, markers: &[&str], context: &str) {
    let mut maybe_previous_index = None;
    for marker in markers {
        let Some(index) = source.find(marker) else {
            panic!("missing ordered {context} marker {marker}");
        };
        if let Some(previous_index) = maybe_previous_index {
            assert!(
                previous_index < index,
                "misordered {context} marker {marker}"
            );
        }
        maybe_previous_index = Some(index);
    }
}
