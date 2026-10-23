use super::*;

#[test]
fn phase34_source_guard_rejects_platform_substitution_and_effects() {
    // Arrange
    let production_identity_sources = [
        PLATFORM_IDENTITY_SOURCE,
        INSTALLED_ASSET_VERSION_SOURCE,
        RUNTIME_SNAPSHOT_SOURCE,
    ];
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
    assert!(PLATFORM_IDENTITY_SOURCE
        .contains("INSTALLED_ASSET_VERSION.observe(|| fs::read(STATIC_ASSET_VERSION_PATH).ok())"));
    assert!(INSTALLED_ASSET_VERSION_SOURCE.contains("parse_static_asset_version(&bytes)"));
    assert!(PLATFORM_IDENTITY_SOURCE.contains("sys::esp_get_idf_version()"));
    // The board and ASIC are the built board's fixed facts (ADR-0041), never a probe.
    assert!(PLATFORM_IDENTITY_SOURCE
        .contains("board: PlatformFact::available(crate::board_identity::PLATFORM_BOARD),"));
    assert!(PLATFORM_IDENTITY_SOURCE
        .contains("asic: PlatformFact::available(crate::board_identity::PLATFORM_ASIC),"));
    let ultra_205_build = source_between(
        BOARD_IDENTITY_SOURCE,
        "#[cfg(not(feature = \"board-gamma-601\"))]",
        "#[cfg(feature = \"board-gamma-601\")]",
    );
    for fact in [
        "PLATFORM_BOARD: PlatformBoard = PlatformBoard::Ultra205;",
        "PLATFORM_ASIC: PlatformAsic = PlatformAsic::Bm1366;",
        "FAMILY: AsicFamily = AsicFamily::Bm1366;",
    ] {
        assert!(
            ultra_205_build.contains(fact),
            "the Ultra 205 build lacks {fact}"
        );
    }
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
    for collection in [
        source_between(
            RUNTIME_SNAPSHOT_SOURCE,
            "fn collect_operator_snapshot_candidate(",
            "fn collect_operator_snapshot_candidate_profiled(",
        ),
        source_between(
            RUNTIME_SNAPSHOT_SOURCE,
            "fn collect_operator_snapshot_candidate_profiled(",
            "fn runtime_projection_for_api_views",
        ),
    ] {
        assert_eq!(
            collection
                .matches("crate::platform_identity::collect()")
                .count(),
            1
        );
    }

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
