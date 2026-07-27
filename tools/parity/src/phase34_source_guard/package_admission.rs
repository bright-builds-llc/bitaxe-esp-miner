use super::*;

#[test]
fn phase34_package_and_hardware_admission_source_guard() {
    // Arrange
    let flash_preparation = source_between(
        FLASH_PACKAGE_SOURCE,
        "fn prepare_flash",
        "fn flash_command_for_admitted_image",
    );
    let flash_execution = source_between(FLASH_EXECUTION_SOURCE, "fn run_flash", "fn run_monitor");
    let snapshot_materialization = source_between(
        FLASH_MODEL_SOURCE,
        "impl AdmittedExecutionSnapshot",
        "pub(crate) struct NvsSeedOutcome",
    );
    let admitted_command_builder = source_between(
        FLASH_PACKAGE_SOURCE,
        "fn flash_command_for_admitted_image",
        "fn resolve_flash_image",
    );
    let manifest_builder = source_between(
        PACKAGE_MANIFEST_SOURCE,
        "pub(crate) fn build_manifest",
        "pub(crate) fn validate_default_flash_image",
    );

    // Act / Assert
    assert!(manifest_builder.contains("BuildProvenance::parse_stamp"));
    assert!(manifest_builder.contains("schema_version: 3"));
    assert!(manifest_builder.contains("app_elf_sha256"));
    assert!(manifest_builder.contains("validate_package_manifest_v3(&manifest)?"));
    assert!(PACKAGE_MANIFEST_SOURCE.contains("validate_firmware_elf_app_sha_relationship"));
    assert!(PACKAGE_MANIFEST_SOURCE.contains("firmware_elf_app_sha_mismatch"));
    assert!(!manifest_builder.contains("Command::new"));
    assert!(!manifest_builder.contains("git describe"));
    assert!(!XTASK_SOURCE.contains("fn firmware_commit"));
    assert!(!XTASK_SOURCE.contains("fn reference_commit"));

    let image_resolution = flash_preparation
        .find("resolve_flash_image")
        .expect("identity admission must resolve the image");
    let port_resolution = flash_preparation
        .find("resolve_port")
        .expect("port resolution must remain explicit");
    let snapshot_creation = flash_preparation
        .find("create_admitted_execution_snapshot")
        .expect("the admitted bytes must be snapshotted before external effects");
    assert!(image_resolution < snapshot_creation && snapshot_creation < port_resolution);
    assert!(flash_execution.contains("_execution_snapshot"));
    assert!(flash_execution.contains("environment.begin_usb_session"));
    assert!(flash_execution.contains("environment.execute(&execution_command)"));
    let session_admission = flash_execution
        .find("environment.begin_usb_session")
        .expect("device session must begin before the effect");
    let child_execution = flash_execution
        .find("environment.execute(&execution_command)")
        .expect("admitted image must execute");
    assert!(session_admission < child_execution);
    for marker in [
        "NamedTempFile",
        "write_all",
        "flush",
        "sync_all",
        "set_mode(0o600)",
    ] {
        assert!(
            snapshot_materialization.contains(marker),
            "missing immutable snapshot marker {marker}"
        );
    }
    assert!(admitted_command_builder.contains("AdmittedFlashImage::Factory"));
    assert!(admitted_command_builder.contains("AdmittedFlashImage::DeveloperDryRun"));
    assert!(!admitted_command_builder.contains("file_name"));
    assert!(!admitted_command_builder.contains("FACTORY_IMAGE_NAME"));
    for marker in [
        "struct AdmittedFactoryImage",
        "enum AdmittedFlashImage",
        "struct AdmittedExecutionSnapshot",
        "<admitted-factory-snapshot>",
    ] {
        assert!(
            FLASH_MODEL_SOURCE.contains(marker) || FLASH_PACKAGE_SOURCE.contains(marker),
            "missing exact admitted-image marker {marker}"
        );
    }
    for marker in [
        "explicit_image_not_admitted_factory",
        "read_validated_artifact",
    ] {
        assert!(
            FLASH_PACKAGE_SOURCE.contains(marker),
            "missing exact admitted-image marker {marker}"
        );
    }
    for forbidden in [
        "require_manifest_artifact_for_path",
        "validate_artifact_digest_for_path",
        "resolve_manifest_flash_image",
        "environment.read_bytes(&factory_path)",
    ] {
        assert!(
            !FLASH_PACKAGE_SOURCE.contains(forbidden)
                && !FLASH_EXECUTION_SOURCE.contains(forbidden)
                && !FLASH_MODEL_SOURCE.contains(forbidden),
            "forbidden admission bypass remains: {forbidden}"
        );
    }
    for marker in [
        "package_source_dirty",
        "current_workspace_dirty",
        "package_workspace_identity_mismatch",
    ] {
        assert!(
            FLASH_PACKAGE_SOURCE.contains(marker),
            "missing admission gate {marker}"
        );
    }
    for marker in [
        "validate_factory_ota_identity",
        "PartitionTable::try_from_bytes",
        "factory_ota_image_mismatch",
        "package_admission_rejects_non_drom_descriptor_in_ota_and_factory",
        "package_admission_rejects_destination_overlap_in_ota_and_factory",
        "package_admission_rejects_alias_overlap_in_ota_and_factory",
        "package_admission_accepts_exact_destination_and_alias_adjacency",
        "package_admission_accepts_range_free_zero_length_segment",
    ] {
        assert!(
            FLASH_PACKAGE_ADMISSION_SOURCE.contains(marker)
                || FLASH_PACKAGE_ADMISSION_TEST_SOURCE.contains(marker),
            "missing package admission marker {marker}"
        );
    }
    for marker in [
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
    ] {
        assert!(
            FLASH_ESP32S3_IMAGE_SOURCE.contains(marker),
            "missing structural admission marker {marker}"
        );
    }
    let layout_constructor = source_between(
        FLASH_ESP32S3_IMAGE_SOURCE,
        "impl ValidatedSegmentLayout",
        "fn validate_header",
    );
    let descriptor_segment_gate = layout_constructor
        .find("validate_descriptor_segment")
        .expect("descriptor segment gate");
    let direct_overlap_gate = layout_constructor
        .find("validate_destination_disjointness")
        .expect("direct destination overlap gate");
    let alias_overlap_gate = layout_constructor
        .find("validate_alias_disjointness")
        .expect("D/IRAM alias overlap gate");
    assert!(descriptor_segment_gate < direct_overlap_gate);
    assert!(direct_overlap_gate < alias_overlap_gate);
    assert!(FLASH_ESP32S3_IMAGE_SOURCE.contains(
        "fn validate_entry_address(\n    entry_address: u32,\n    layout: &ValidatedSegmentLayout,"
    ));
    assert!(FLASH_ESP32S3_IMAGE_SOURCE.contains(
        "fn validate_descriptor(\n    image: &[u8],\n    layout: &ValidatedSegmentLayout,"
    ));
    let identity_admission = source_between(
        FLASH_PACKAGE_SOURCE,
        "fn validate_identity_admission",
        "fn require_artifact",
    );
    let factory_digest = identity_admission
        .find("read_validated_artifact(factory_artifact")
        .expect("factory digest admission");
    let elf_digest = identity_admission
        .find("read_validated_artifact(elf_artifact")
        .expect("firmware ELF digest admission");
    let elf_app_binding = identity_admission
        .find("firmware_elf_app_sha_mismatch")
        .expect("firmware ELF application SHA binding");
    let ota_digest = identity_admission
        .find("read_validated_artifact(ota_artifact")
        .expect("OTA digest admission");
    let factory_binding = identity_admission
        .find("validate_factory_ota_identity")
        .expect("factory and OTA structural binding");
    assert!(factory_digest < factory_binding);
    assert!(elf_digest < elf_app_binding && elf_app_binding < ota_digest);
    for marker in [
        "identity_admission_rejects_all_layout_classes_in_parsed_dry_run_before_effects",
        "identity_admission_rejects_all_layout_classes_in_parsed_non_dry_run_before_effects",
        "assert_parsed_layout_rejected_before_effects",
    ] {
        assert!(
            FLASH_ADMISSION_LAYOUT_TEST_SOURCE.contains(marker)
                || FLASH_ADMISSION_FIXTURE_SOURCE.contains(marker),
            "missing parsed pre-effect layout marker {marker}"
        );
    }
    for marker in [
        "list_ports_calls",
        "read_string_paths",
        "generated_nvs_partitions",
        "created_snapshot_paths",
        "captured_commands",
        "executed_commands",
        "observed_flashes",
    ] {
        assert!(
            FLASH_FAKE_ENVIRONMENT_SOURCE.contains(marker),
            "missing parsed pre-effect layout marker {marker}"
        );
    }
    assert!(!FLASH_PACKAGE_SOURCE.contains("contains_bytes(&ota_bytes"));
    assert!(!FLASH_PACKAGE_SOURCE.contains("contains_bytes(&factory_bytes"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("esptool\" image_info --version 2"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("--elf-sha256-offset"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("generated_partition_table"));
    assert!(!PACKAGE_SCRIPT_SOURCE.contains("espflash\n\tsave-image"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("--build-provenance-stamp"));
}
