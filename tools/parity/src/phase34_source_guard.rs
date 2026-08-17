mod package_admission;

const BUILD_SCRIPT_SOURCE: &str = include_str!("../../../firmware/bitaxe/build.rs");
const MAIN_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/main.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/runtime_snapshot.rs");
const HTTP_HANDLER_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/http_api/handlers.rs");
const HTTP_ACCESS_ADAPTER_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/http_api/access.rs");
const HTTP_WEBSOCKET_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/http_api/websocket.rs");
const HTTP_ACCESS_POLICY_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/route_shell.rs");
const SNAPSHOT_PUBLICATION_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/operator_snapshot_publication.rs");
const OPERATOR_SNAPSHOT_MODEL_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/operator_snapshot.rs");
const SNAPSHOT_PUBLICATION_MODEL_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/operator_snapshot_publication.rs");
const CONFIRMED_SNAPSHOT_MODEL_SOURCE: &str =
    include_str!("../../../crates/bitaxe-config/src/confirmed_snapshot.rs");
const SETTINGS_SNAPSHOT_STORE_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/settings_snapshot_store.rs");
const DEFERRED_EFFECT_MODEL_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/deferred_effect.rs");
const DEFERRED_EFFECT_QUEUE_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/http_api/deferred_effect_queue.rs");
const SNAPSHOT_EVIDENCE_TEST_SOURCE: &str = include_str!("operator_snapshot_evidence/tests.rs");
const SNAPSHOT_RETENTION_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/operator_snapshot_retention.rs");
const LOG_BUFFER_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/log_buffer.rs");
const RUNTIME_HEALTH_ADAPTER_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/runtime_health_adapter.rs");
const TASK_WATCHDOG_OBSERVATION_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/task_watchdog_observation.rs");
const RUNTIME_HEALTH_CORE_SOURCE: &str =
    include_str!("../../../crates/bitaxe-core/src/runtime_health.rs");
const RUNTIME_HEALTH_WAIT_SOURCE: &str =
    include_str!("../../../crates/bitaxe-core/src/runtime_health/wait.rs");
const WATCHDOG_ADAPTER_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/safety_adapter/watchdog.rs");
const PRODUCTION_TASK_WATCHDOG_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/production_mining_session/watchdog.rs");
const PRODUCTION_OWNER_LOOP_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/production_mining_session/owner_loop.rs");
const PRODUCTION_OWNER_PROGRESS_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/production_mining_session/owner_progress.rs");
const PLATFORM_IDENTITY_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/platform_identity.rs");
const CORE_SOURCE: &str = include_str!("../../../crates/bitaxe-core/src/lib.rs");
const API_WIRE_SOURCE: &str = include_str!("../../../crates/bitaxe-api/src/wire.rs");
const RUNTIME_HEALTH_WIRE_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/wire/runtime_health.rs");
const BUILD_IDENTITY_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/build_identity.rs");
const XTASK_SOURCE: &str = include_str!("../../xtask/src/main.rs");
const PACKAGE_MANIFEST_SOURCE: &str = include_str!("../../xtask/src/package_manifest.rs");
const FLASH_EXECUTION_SOURCE: &str = include_str!("../../flash/src/commands/flash.rs");
const FLASH_EXECUTION_SNAPSHOT_SOURCE: &str = include_str!("../../flash/src/execution_snapshot.rs");
const FLASH_MODEL_SOURCE: &str = include_str!("../../flash/src/model.rs");
const FLASH_PACKAGE_SOURCE: &str = include_str!("../../flash/src/package.rs");
const FLASH_PACKAGE_ADMISSION_TEST_SOURCE: &str =
    include_str!("../../flash/src/package_admission/tests.rs");
const FLASH_ADMISSION_LAYOUT_TEST_SOURCE: &str =
    include_str!("../../flash/src/tests/admission_layout.rs");
const FLASH_ADMISSION_FIXTURE_SOURCE: &str = include_str!("../../flash/src/tests/fixtures.rs");
const FLASH_FAKE_ENVIRONMENT_SOURCE: &str =
    include_str!("../../flash/src/tests/fake_environment.rs");
const FLASH_ESP32S3_IMAGE_SOURCE: &str = include_str!("../../flash/src/esp32s3_image.rs");
const FLASH_PACKAGE_ADMISSION_SOURCE: &str = include_str!("../../flash/src/package_admission.rs");

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
    assert!(PLATFORM_IDENTITY_SOURCE.contains("/www/version.txt"));
    assert!(PLATFORM_IDENTITY_SOURCE.contains("parse_static_asset_version"));
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
fn http_access_policy_stays_in_pure_route_shell() {
    // Arrange
    let required_policy_markers = [
        "pub fn normalize_peer_ipv4",
        "enum PeerIpv4Normalization",
        "fn is_rfc1918_ipv4",
        "PeerIpv4Normalization::HostOrderFallback",
    ];
    let required_adapter_markers = [
        "sys::lwip_getpeername",
        "normalize_peer_ipv4(raw_addr)",
        "PeerIpv4Normalization::HostOrderFallback",
        "axeos_access_gate_peer_ip_byte_order=host_order",
    ];

    // Act / Assert
    for marker in required_policy_markers {
        assert!(
            HTTP_ACCESS_POLICY_SOURCE.contains(marker),
            "pure HTTP access policy is missing {marker}"
        );
    }
    for forbidden in ["esp_idf_svc", "sys::", "log::", "unsafe"] {
        assert!(
            !HTTP_ACCESS_POLICY_SOURCE.contains(forbidden),
            "pure HTTP access policy contains effect token {forbidden}"
        );
    }
    for marker in required_adapter_markers {
        assert!(
            HTTP_ACCESS_ADAPTER_SOURCE.contains(marker),
            "HTTP access adapter is missing {marker}"
        );
    }
    for forbidden in [
        "fn peer_ipv4_from_s_addr",
        "fn is_rfc1918_ipv4",
        "u32::from_be",
        "(16..=31)",
    ] {
        assert!(
            !HTTP_ACCESS_ADAPTER_SOURCE.contains(forbidden),
            "HTTP access adapter retained policy token {forbidden}"
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
    let passive_sources = [
        RUNTIME_HEALTH_CORE_SOURCE,
        RUNTIME_HEALTH_WAIT_SOURCE,
        RUNTIME_HEALTH_ADAPTER_SOURCE,
    ];

    // Act / Assert
    assert!(RUNTIME_HEALTH_ADAPTER_SOURCE.contains("RuntimeHealthSnapshot::evaluate"));
    assert!(RUNTIME_HEALTH_ADAPTER_SOURCE.contains("supervisor_checkpoint_history"));
    assert!(RUNTIME_HEALTH_ADAPTER_SOURCE.contains("CONFIG_ESP_TASK_WDT_TIMEOUT_S"));
    assert!(RUNTIME_HEALTH_ADAPTER_SOURCE.contains("checked_mul(MILLIS_PER_SECOND)"));
    assert!(!RUNTIME_HEALTH_CORE_SOURCE.contains("TASK_WATCHDOG_FRESH_AFTER_MILLIS"));
    let watchdog_observation = RUNTIME_HEALTH_ADAPTER_SOURCE
        .find("task_watchdog_observation::coherent_observation()")
        .expect("coherent watchdog observation read");
    let evaluation_time = RUNTIME_HEALTH_ADAPTER_SOURCE
        .find("let current_monotonic_millis = crate::runtime_uptime::millis();")
        .expect("evaluation time read");
    assert!(watchdog_observation < evaluation_time);
    assert!(TASK_WATCHDOG_OBSERVATION_SOURCE.contains("COHERENT_READ_ATTEMPTS"));
    assert!(TASK_WATCHDOG_OBSERVATION_SOURCE.contains("publication_sequence: AtomicU32"));
    assert!(TASK_WATCHDOG_OBSERVATION_SOURCE
        .contains("start_sequence == end_sequence && end_sequence & 1 == 0"));
    assert!(candidate_collection.contains("runtime_health_adapter::collect()"));
    assert!(!candidate_collection.contains("collect(crate::runtime_uptime::millis())"));
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
        "task_watchdog_feed_sequence={task_watchdog_feed_sequence}",
        "task_watchdog_feed_age_millis={task_watchdog_feed_age_millis}",
        "task_watchdog_read_outcome={}",
        "task_watchdog_owner_phase={}",
        "task_watchdog_wait_state={}",
        "redacted=true",
    ] {
        assert!(
            RUNTIME_HEALTH_WIRE_SOURCE.contains(marker),
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
        "taskWatchdogFeedSequence",
        "taskWatchdogFeedAgeMillis",
        "taskWatchdogReadOutcome",
        "taskWatchdogOwnerPhase",
        "taskWatchdogWaitState",
    ] {
        let source = if field == "runtimeHealth" {
            API_WIRE_SOURCE
        } else {
            RUNTIME_HEALTH_WIRE_SOURCE
        };
        assert!(source.contains(field), "missing API field {field}");
    }

    for source in passive_sources {
        for forbidden in [
            "SelfTestLifecycle::apply",
            "SelfTestCommand::",
            "start_safety_supervisor",
            "esp_task_wdt_add",
            "esp_task_wdt_reset",
            "esp_task_wdt_delete",
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
                !contains_prohibited_token(source, forbidden),
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
        .find("maybe_record_supervisor_checkpoint(")
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

fn contains_prohibited_token(source: &str, forbidden: &str) -> bool {
    let source = source.to_ascii_lowercase();
    let forbidden = forbidden.to_ascii_lowercase();
    if forbidden.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return source
            .split(|character: char| !character.is_ascii_alphanumeric())
            .any(|token| token == forbidden);
    }
    source.contains(&forbidden)
}

#[test]
fn production_task_watchdog_participates_without_global_reconfiguration() {
    // Arrange
    let required_calls = [
        "esp_task_wdt_add(ptr::null_mut())",
        "esp_task_wdt_reset()",
        "esp_task_wdt_delete(ptr::null_mut())",
    ];

    // Act / Assert
    for required in required_calls {
        assert!(
            PRODUCTION_TASK_WATCHDOG_SOURCE.contains(required),
            "production task watchdog is missing {required}"
        );
    }
    for prohibited in ["esp_task_wdt_init", "esp_task_wdt_reconfigure"] {
        assert!(
            !PRODUCTION_TASK_WATCHDOG_SOURCE.contains(prohibited),
            "production task watchdog contains prohibited global operation {prohibited}"
        );
    }
}

#[test]
fn production_task_watchdog_tracks_completed_owner_progress() {
    // Arrange
    let execute = PRODUCTION_OWNER_PROGRESS_SOURCE
        .find("let maybe_feedback = execute(effect);")
        .expect("effect execution boundary");
    let completed = PRODUCTION_OWNER_PROGRESS_SOURCE
        .find("progress(OwnerProgressBoundary::EffectCompleted);")
        .expect("completed effect boundary");

    // Act / Assert
    assert!(execute < completed);
    assert!(PRODUCTION_OWNER_LOOP_SOURCE.contains("drive_feedback("));
    assert!(PRODUCTION_OWNER_LOOP_SOURCE.contains("|_| task_watchdog.feed("));
    assert!(!PRODUCTION_OWNER_PROGRESS_SOURCE.contains("esp_task_wdt_"));
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
        HTTP_HANDLER_SOURCE,
        "fn handle_system_info",
        "fn handle_logs_download",
    );
    let live_cadence = source_between(
        HTTP_WEBSOCKET_SOURCE,
        "fn broadcast_live_telemetry_cadence",
        "fn broadcast_raw_log_chunks",
    );
    let live_connect = source_between(
        HTTP_WEBSOCKET_SOURCE,
        "fn send_websocket_connect_frames",
        "fn send_websocket_text_frame(",
    );
    let adversarial_regression = source_between(
        SNAPSHOT_EVIDENCE_TEST_SOURCE,
        "fn operator_snapshot_publication_reverse_completion_preserves_direct_chronology",
        "#[test]\nfn phase34_operator_snapshot_runtime_source_guard",
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
    assert!(live_cadence.contains("websocket_api::maybe_live_cadence_frame(current)"));
    assert!(live_cadence.contains("broadcast_websocket_text_frame("));
    assert!(live_connect.contains("publish_projected_live_telemetry_payload"));
    assert!(live_connect.contains("websocket_api::maybe_live_connect_frame(current)"));
    assert!(live_connect.contains("send_websocket_text_frame(request, &body)"));
    assert!(HTTP_WEBSOCKET_SOURCE.contains("send_websocket_text_frame_async(server, lease, body)"));
    assert!(!RUNTIME_SNAPSHOT_SOURCE.contains("pub fn projected_system_info"));
    assert!(!RUNTIME_SNAPSHOT_SOURCE.contains("pub fn projected_live_telemetry_payload"));

    assert!(adversarial_regression.contains("OperatorSnapshotPublisher::new()"));
    assert!(adversarial_regression.contains("IssuedPayload::Http"));
    assert!(adversarial_regression.contains("IssuedPayload::LiveWebSocket"));
    assert!(adversarial_regression.contains("assert_eq!(issued_revisions, [1, 2])"));
    assert!(!adversarial_regression.contains(".sort"));
}

#[test]
fn reusable_crates_keep_models_while_firmware_owns_concurrency() {
    // Arrange
    let reusable_sources = [
        OPERATOR_SNAPSHOT_MODEL_SOURCE,
        SNAPSHOT_PUBLICATION_MODEL_SOURCE,
        CONFIRMED_SNAPSHOT_MODEL_SOURCE,
        DEFERRED_EFFECT_MODEL_SOURCE,
    ];
    // Act / Assert
    for source in reusable_sources {
        for forbidden in [
            "std::sync",
            "Mutex",
            "mpsc",
            "thread_local!",
            "thread::spawn",
        ] {
            assert!(
                !source.contains(forbidden),
                "reusable model source retained concurrency token {forbidden}"
            );
        }
    }
    for required in [
        "OperatorSnapshotPublishError",
        "ConfirmedSnapshotPublicationFailure",
        "DeferredEffectQueueUnavailable",
    ] {
        assert!(
            reusable_sources
                .iter()
                .any(|source| source.contains(required)),
            "reusable model source is missing {required}"
        );
    }
    assert!(SNAPSHOT_PUBLICATION_SOURCE.contains("Mutex<OperatorSnapshotSequence>"));
    assert!(SNAPSHOT_PUBLICATION_SOURCE.contains("thread_local!"));
    assert!(SETTINGS_SNAPSHOT_STORE_SOURCE.contains("Mutex<NvsSnapshot>"));
    assert!(DEFERRED_EFFECT_QUEUE_SOURCE.contains("mpsc::sync_channel"));
    assert!(DEFERRED_EFFECT_QUEUE_SOURCE.contains("release_after_response"));
    assert!(SETTINGS_SNAPSHOT_STORE_SOURCE.contains("PoisonRecovered"));
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source.find(start).expect("start marker should exist");
    let tail = &source[start_index..];
    let end_index = tail.find(end).expect("end marker should exist");
    &tail[..end_index]
}
