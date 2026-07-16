const BUILD_SCRIPT_SOURCE: &str = include_str!("../../../firmware/bitaxe/build.rs");
const MAIN_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/main.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/runtime_snapshot.rs");
const HTTP_API_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/http_api.rs");
const SNAPSHOT_PUBLICATION_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/operator_snapshot_publication.rs");
const SNAPSHOT_EVIDENCE_SOURCE: &str = include_str!("operator_snapshot_evidence.rs");
const SNAPSHOT_RETENTION_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/operator_snapshot_retention.rs");
const LOG_BUFFER_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/log_buffer.rs");
const RUNTIME_HEALTH_ADAPTER_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/runtime_health_adapter.rs");
const RUNTIME_HEALTH_CORE_SOURCE: &str =
    include_str!("../../../crates/bitaxe-core/src/runtime_health.rs");
const WATCHDOG_ADAPTER_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/safety_adapter/watchdog.rs");
const PLATFORM_IDENTITY_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/platform_identity.rs");
const CORE_SOURCE: &str = include_str!("../../../crates/bitaxe-core/src/lib.rs");
const API_WIRE_SOURCE: &str = include_str!("../../../crates/bitaxe-api/src/wire.rs");
const BUILD_IDENTITY_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/build_identity.rs");
const XTASK_SOURCE: &str = include_str!("../../xtask/src/main.rs");
const PACKAGE_MANIFEST_SOURCE: &str = include_str!("../../xtask/src/package_manifest.rs");
const FLASH_SOURCE: &str = include_str!("../../flash/src/main.rs");
const FLASH_ESP32S3_IMAGE_SOURCE: &str = include_str!("../../flash/src/esp32s3_image.rs");
const FLASH_PACKAGE_ADMISSION_SOURCE: &str = include_str!("../../flash/src/package_admission.rs");
const PACKAGE_SCRIPT_SOURCE: &str = include_str!("../../../scripts/package-firmware.sh");

#[test]
fn phase34_identity_runtime_source_guard() {
    // Arrange
    let lcd_identity = source_between(CORE_SOURCE, "fn startup_debug_build_label", "#[cfg(test)]");
    let retained_identity =
        source_between(MAIN_SOURCE, "fn retain_build_identity", "fn info_retained");
    let platform_identity = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn collect_platform_snapshot",
        "fn compatibility_string",
    );

    // Act / Assert
    assert!(BUILD_SCRIPT_SOURCE.contains("required_build_provenance"));
    assert!(!BUILD_SCRIPT_SOURCE.contains("Command::new"));
    assert!(!BUILD_SCRIPT_SOURCE.contains("git describe"));

    assert!(lcd_identity.contains("build_label.to_owned()"));
    assert!(!lcd_identity.contains(".take("));
    assert!(!lcd_identity.contains("source_commit"));

    for marker in [
        "firmware_commit={}",
        "reference_commit={}",
        "app_elf_sha256={}",
        "BITAXE_RUNTIME_BUILD_IDENTITY",
    ] {
        assert!(retained_identity.contains(marker), "missing {marker}");
    }
    assert!(BUILD_IDENTITY_SOURCE.contains(
        "runtime_build_identity semantic_version={} label={} channel={} source_dirty={} release_tag={} redacted=true"
    ));

    for assignment in [
        "platform.version = crate::build_label()",
        "platform.semantic_version = crate::semantic_version()",
        "platform.source_commit = crate::firmware_commit()",
        "platform.reference_commit = crate::reference_commit()",
        "platform.app_elf_sha256 = crate::app_elf_sha256()",
        "platform.build_channel = crate::build_channel()",
        "platform.source_dirty = crate::source_dirty()",
        "platform.maybe_release_tag = crate::maybe_release_tag()",
    ] {
        assert!(
            platform_identity.contains(assignment),
            "missing {assignment}"
        );
    }

    for field in [
        "semanticVersion",
        "sourceCommit",
        "referenceCommit",
        "appElfSha256",
        "buildChannel",
        "sourceDirty",
        "releaseTag",
    ] {
        assert!(API_WIRE_SOURCE.contains(field), "missing API field {field}");
    }
}

#[test]
fn phase34_source_guard_rejects_platform_substitution_and_effects() {
    // Arrange
    let production_identity_sources = [PLATFORM_IDENTITY_SOURCE, RUNTIME_SNAPSHOT_SOURCE];
    let completed_snapshot = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn complete_operator_snapshot",
        "/// Returns the current command-visible mining state.",
    );
    let candidate_collection = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn collect_operator_snapshot_candidate",
        "fn runtime_projection_for_api_views",
    );

    // Act / Assert
    assert!(
        PLATFORM_IDENTITY_SOURCE.contains("include_str!(\"../static/www/assets/release.json\")")
    );
    assert!(PLATFORM_IDENTITY_SOURCE.contains("sys::esp_get_idf_version()"));
    assert!(PLATFORM_IDENTITY_SOURCE.contains("PlatformBoard::Ultra205"));
    assert!(PLATFORM_IDENTITY_SOURCE.contains("PlatformAsic::Bm1366"));
    assert!(PLATFORM_IDENTITY_SOURCE.contains("sys::esp_ota_get_running_partition()"));
    assert!(PLATFORM_IDENTITY_SOURCE.contains("PlatformResetReason::decode"));
    assert!(PLATFORM_IDENTITY_SOURCE.contains("sys::esp_timer_get_time()"));
    assert!(candidate_collection.contains("crate::platform_identity::collect()"));
    let identity_assignment = completed_snapshot
        .find("snapshot.operator_snapshot_identity = operator_snapshot_identity")
        .expect("capture identity assignment");
    let platform_attachment = completed_snapshot
        .find("snapshot.platform_identity = candidate.platform_identity")
        .expect("platform candidate attachment");
    assert!(identity_assignment < platform_attachment);
    assert_eq!(
        candidate_collection
            .matches("crate::platform_identity::collect()")
            .count(),
        1
    );

    for source in production_identity_sources {
        for forbidden in [
            "fixtures/",
            "safe-fixture",
            "placeholder",
            "std::process",
            "Command::new",
            "git rev-parse",
            "esp_restart",
            "esp_ota_begin",
            "esp_ota_write",
            "esp_ota_end",
            "esp_ota_set_boot_partition",
            "esp_task_wdt",
            "uart_",
            "gpio_set",
            "credential",
            "BM1370",
            "Gamma601",
        ] {
            assert!(
                !source.contains(forbidden),
                "production platform identity contains prohibited token {forbidden}"
            );
        }
    }

    for request_time_mutation in [
        "static mut",
        "Atomic",
        "Mutex",
        "OnceLock",
        "fn set",
        "fn write",
    ] {
        assert!(
            !PLATFORM_IDENTITY_SOURCE.contains(request_time_mutation),
            "platform adapter contains request-time mutation token {request_time_mutation}"
        );
    }
}

#[test]
fn phase34_runtime_health_is_passive_correlated_and_effect_free() {
    // Arrange
    let completed_snapshot = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn complete_operator_snapshot",
        "/// Returns the current command-visible mining state.",
    );
    let candidate_collection = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn collect_operator_snapshot_candidate",
        "fn runtime_projection_for_api_views",
    );
    let retained_projection = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn publish_operator_snapshot",
        "fn collect_operator_snapshot_candidate",
    );
    let passive_sources = [RUNTIME_HEALTH_CORE_SOURCE, RUNTIME_HEALTH_ADAPTER_SOURCE];

    // Act / Assert
    assert!(RUNTIME_HEALTH_ADAPTER_SOURCE.contains("RuntimeHealthSnapshot::evaluate"));
    assert!(RUNTIME_HEALTH_ADAPTER_SOURCE.contains("supervisor_checkpoint_history"));
    assert_eq!(
        candidate_collection
            .matches("runtime_health_adapter::collect")
            .count(),
        1
    );
    let identity_assignment = completed_snapshot
        .find("snapshot.operator_snapshot_identity = operator_snapshot_identity")
        .expect("capture identity assignment");
    let health_attachment = completed_snapshot
        .find("snapshot.runtime_health = candidate.runtime_health")
        .expect("runtime health candidate attachment");
    assert!(identity_assignment < health_attachment);

    assert!(retained_projection.contains("retained_runtime_health_record"));
    for marker in [
        "boot_session={boot_session}",
        "operator_snapshot_revision={}",
        "self_test={}",
        "supervisor={}",
        "checkpoint_category={checkpoint_category}",
        "checkpoint_sequence={checkpoint_sequence}",
        "checkpoint_age_millis={checkpoint_age_millis}",
        "checkpoint_health={}",
        "task_watchdog_participation={}",
        "task_watchdog_reason={task_watchdog_reason}",
        "redacted=true",
    ] {
        assert!(
            API_WIRE_SOURCE.contains(marker),
            "missing retained health marker {marker}"
        );
    }
    for field in [
        "runtimeHealth",
        "selfTestState",
        "supervisorAvailability",
        "checkpointCategory",
        "checkpointSequence",
        "checkpointAgeMillis",
        "checkpointHealth",
        "taskWatchdogParticipation",
        "taskWatchdogReason",
    ] {
        assert!(API_WIRE_SOURCE.contains(field), "missing API field {field}");
    }

    for source in passive_sources {
        for forbidden in [
            "SelfTestLifecycle::apply",
            "SelfTestCommand::",
            "start_safety_supervisor",
            "esp_task_wdt_",
            "std::thread",
            "thread::sleep",
            "gpio",
            "i2c",
            "reset",
            "power",
            "fan",
            "voltage",
            "asic",
            "mining",
            "load",
            "fault",
        ] {
            assert!(
                !source
                    .to_ascii_lowercase()
                    .contains(&forbidden.to_ascii_lowercase()),
                "passive runtime-health source contains prohibited token {forbidden}"
            );
        }
    }

    let supervisor_transition = source_between(
        WATCHDOG_ADAPTER_SOURCE,
        "fn transition_supervisor_step",
        "/// Returns a read-only copy",
    );
    let decision_handling = supervisor_transition
        .find("let maybe_log = match decision")
        .expect("supervisor decision handling");
    let checkpoint_publication = supervisor_transition
        .find("record_supervisor_checkpoint(checkpoints, observed_at_millis)")
        .expect("recurring checkpoint publication");
    assert!(decision_handling < checkpoint_publication);
    assert!(!supervisor_transition.contains("return SupervisorStepOutcome::default()"));
    assert!(supervisor_transition.contains("if *logged_yield"));

    for forbidden in [
        "esp_task_wdt_",
        "esp_restart",
        "gpio_set",
        "i2c_master",
        "uart_",
        "credential",
        "std::net",
    ] {
        assert!(
            !WATCHDOG_ADAPTER_SOURCE.contains(forbidden),
            "supervisor checkpoint adapter contains prohibited effect {forbidden}"
        );
    }
}

#[test]
fn phase34_snapshot_publication_orders_real_retention_and_issuance() {
    // Arrange
    let publication = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn publish_operator_snapshot",
        "fn collect_operator_snapshot_candidate",
    );
    let system_info = source_between(
        HTTP_API_SOURCE,
        "fn handle_system_info",
        "fn handle_settings_patch",
    );
    let live_cadence = source_between(
        HTTP_API_SOURCE,
        "fn broadcast_live_telemetry_cadence",
        "fn broadcast_raw_log_chunks",
    );
    let live_connect = source_between(
        HTTP_API_SOURCE,
        "fn send_websocket_connect_frames",
        "fn send_websocket_text_frame(",
    );
    let adversarial_regression = source_between(
        SNAPSHOT_EVIDENCE_SOURCE,
        "fn operator_snapshot_publication_reverse_completion_preserves_direct_chronology",
        "#[test]\n    fn phase34_operator_snapshot_runtime_source_guard",
    );

    // Act / Assert
    assert_eq!(
        RUNTIME_SNAPSHOT_SOURCE
            .matches("static OPERATOR_SNAPSHOT_PUBLISHER:")
            .count(),
        1
    );
    assert!(!RUNTIME_SNAPSHOT_SOURCE.contains("OPERATOR_SNAPSHOT_SEQUENCE"));
    let collect = publication
        .find("collect_operator_snapshot_candidate")
        .expect("collection adapter");
    let complete = publication
        .find("|candidate, identity|")
        .expect("completion adapter");
    let retain = publication
        .find("operator_snapshot_retention::retain_completed_operator_snapshot")
        .expect("retention adapter");
    let issue = publication
        .find("issue(publication.output)")
        .expect("issuance adapter");
    assert!(collect < complete && complete < retain && retain < issue);
    assert!(SNAPSHOT_PUBLICATION_SOURCE.contains("let candidate = collect();"));
    assert!(SNAPSHOT_PUBLICATION_SOURCE.contains("issue(publication).map_err"));
    assert!(SNAPSHOT_PUBLICATION_SOURCE.contains("RetentionError, IssueError"));
    assert!(SNAPSHOT_RETENTION_SOURCE.contains("retain_operator_snapshot_pair"));
    assert!(LOG_BUFFER_SOURCE.contains("pub fn retain_operator_snapshot_pair"));
    assert!(!publication.contains("Ok::<(), E>(())"));
    assert!(!SNAPSHOT_RETENTION_SOURCE.contains("append_runtime_log_line"));

    assert!(system_info.contains("publish_projected_system_info"));
    assert!(system_info.contains("send_json(request, &system_info)"));
    assert!(live_cadence.contains("publish_projected_live_telemetry_payload"));
    assert!(live_cadence.contains("websocket_api::live_cadence_frame(current)"));
    assert!(live_cadence.contains("broadcast_websocket_text_frame("));
    assert!(live_connect.contains("publish_projected_live_telemetry_payload"));
    assert!(live_connect.contains("websocket_api::live_connect_frame(current)"));
    assert!(live_connect.contains("send_websocket_text_frame(request, &body)"));
    assert!(HTTP_API_SOURCE.contains("send_websocket_text_frame_async(server, session, body)"));
    assert!(!RUNTIME_SNAPSHOT_SOURCE.contains("pub fn projected_system_info"));
    assert!(!RUNTIME_SNAPSHOT_SOURCE.contains("pub fn projected_live_telemetry_payload"));

    assert!(adversarial_regression.contains("OperatorSnapshotPublisher::new()"));
    assert!(adversarial_regression.contains("IssuedPayload::Http"));
    assert!(adversarial_regression.contains("IssuedPayload::LiveWebSocket"));
    assert!(adversarial_regression.contains("assert_eq!(issued_revisions, [1, 2])"));
    assert!(!adversarial_regression.contains(".sort"));
}

#[test]
fn phase34_package_and_hardware_admission_source_guard() {
    // Arrange
    let flash_preparation = source_between(
        FLASH_SOURCE,
        "fn prepare_flash",
        "fn flash_command_for_admitted_image",
    );
    let flash_execution = source_between(FLASH_SOURCE, "fn run_flash", "fn run_monitor");
    let snapshot_materialization = source_between(
        FLASH_SOURCE,
        "impl AdmittedExecutionSnapshot",
        "#[derive(Debug)]\nstruct NvsSeedOutcome",
    );
    let admitted_command_builder = source_between(
        FLASH_SOURCE,
        "fn flash_command_for_admitted_image",
        "fn prepare_wifi_nvs_seed",
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
    assert!(flash_execution.contains("environment.execute(&execution_command)"));
    assert!(flash_execution.contains("admitted_image_child_failed"));
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
        "explicit_image_not_admitted_factory",
        "<admitted-factory-snapshot>",
        "read_validated_artifact",
    ] {
        assert!(
            FLASH_SOURCE.contains(marker),
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
            !FLASH_SOURCE.contains(forbidden),
            "forbidden admission bypass remains: {forbidden}"
        );
    }
    for marker in [
        "package_source_dirty",
        "current_workspace_dirty",
        "package_workspace_identity_mismatch",
    ] {
        assert!(
            FLASH_SOURCE.contains(marker),
            "missing admission gate {marker}"
        );
    }
    for marker in [
        "validate_factory_ota_identity",
        "PartitionTable::try_from_bytes",
        "factory_ota_image_mismatch",
    ] {
        assert!(
            FLASH_PACKAGE_ADMISSION_SOURCE.contains(marker),
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
    ] {
        assert!(
            FLASH_ESP32S3_IMAGE_SOURCE.contains(marker),
            "missing structural admission marker {marker}"
        );
    }
    let identity_admission = source_between(
        FLASH_SOURCE,
        "fn validate_identity_admission",
        "fn require_artifact",
    );
    let factory_digest = identity_admission
        .find("read_validated_artifact(factory_artifact")
        .expect("factory digest admission");
    let factory_binding = identity_admission
        .find("validate_factory_ota_identity")
        .expect("factory and OTA structural binding");
    assert!(factory_digest < factory_binding);
    assert!(!FLASH_SOURCE.contains("contains_bytes(&ota_bytes"));
    assert!(!FLASH_SOURCE.contains("contains_bytes(&factory_bytes"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("esptool\" image_info --version 2"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("--elf-sha256-offset"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("generated_partition_table"));
    assert!(!PACKAGE_SCRIPT_SOURCE.contains("espflash\n\tsave-image"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("--build-provenance-stamp"));
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source.find(start).expect("start marker should exist");
    let tail = &source[start_index..];
    let end_index = tail.find(end).expect("end marker should exist");
    &tail[..end_index]
}
