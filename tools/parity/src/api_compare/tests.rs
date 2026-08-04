use anyhow::anyhow;
use serde_json::json;

use super::*;

const OPENAPI: &str = r#"
components:
  schemas:
    GenericResponse:
      properties:
        message:
    SystemInfo:
      properties:
        ASICModel:
        hashRate_1m:
        fanspeed:
        fanrpm:
        miningPaused:
        sharesRejectedReasons:
        poolDifficulty:
        responseTime:
    SystemASIC:
      properties:
        ASICModel:
        deviceModel:
        asicCount:
        defaultFrequency:
        frequencyOptions:
        defaultVoltage:
        voltageOptions:
    SystemStatistics:
      properties:
        currentTimestamp:
        labels:
        statistics:
    SystemScoreboardEntry:
      properties:
        difficulty:
        job_id:
        extranonce2:
        ntime:
        nonce:
        version_bits:
    BlockFoundDismiss:
      properties:
        blockFound:
        showNewBlock:
        message:
    Settings:
      properties:
        frequency:
        coreVoltage:
        hostname:
        fanspeed:
        autofanspeed:
        statsFrequency:
paths:
  /api/system/info:
    get:
  /api/system/logs:
    get:
  /api/system/asic:
    get:
  /api/system/statistics:
    get:
  /api/system/scoreboard:
    get:
  /api/system/pause:
    post:
  /api/system/resume:
    post:
  /api/system/restart:
    post:
  /api/system/identify:
    post:
  /api/system/blockFound/dismiss:
    post:
  /api/system:
    patch:
  /api/system/OTA:
    post:
  /api/system/OTAWWW:
    post:
"#;
const ROUTE_MANIFEST: &str = include_str!("../../fixtures/api/phase05-required-routes.json");
const STATIC_USAGE: &str = include_str!("../../fixtures/api/axeos-route-usage.json");

#[test]
fn api_compare_passes_phase05_route_property_and_fixture_checks() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let request = ApiCompareRequest {
        openapi_yaml: OPENAPI,
        route_manifest_json: ROUTE_MANIFEST,
        static_usage_json: STATIC_USAGE,
    };

    // Act
    let report = run_api_compare(&request, &loader).expect("api compare should run");

    // Assert
    assert!(report.validation_errors.is_empty(), "{report:#?}");
    assert!(render_api_compare_report(&report).contains("schema"));
    assert!(render_api_compare_report(&report).contains("captured-response"));
    assert!(render_api_compare_report(&report).contains("static-route"));
    assert!(render_api_compare_report(&report).contains("firmware-smoke"));
}

#[test]
fn api_compare_with_phase07_routes_preserves_schema_and_response_evidence() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let request = ApiCompareRequest {
        openapi_yaml: OPENAPI,
        route_manifest_json: ROUTE_MANIFEST,
        static_usage_json: STATIC_USAGE,
    };

    // Act
    let report = run_api_compare_with_routes(&request, &loader, bitaxe_api::phase07_routes())
        .expect("api compare should run");
    let rendered = render_api_compare_report(&report);

    // Assert
    assert!(report.validation_errors.is_empty(), "{report:#?}");
    assert!(rendered.contains("- schema | status=passed"));
    assert!(rendered.contains("- captured-response | status=passed"));
    assert!(rendered.contains("- static-route | status=passed"));
}

#[test]
fn api_compare_fails_when_phase07_route_is_missing_from_rust_manifest() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let request = default_request(STATIC_USAGE);
    let routes = bitaxe_api::phase07_routes()
        .iter()
        .copied()
        .filter(|route| route.method != RouteMethod::Get || route.path != "/recovery")
        .collect::<Vec<_>>();

    // Act
    let report =
        run_api_compare_with_routes(&request, &loader, &routes).expect("api compare should run");

    // Assert
    assert_validation_error_contains(&report, &["missing", "GET /recovery"]);
}

#[test]
fn api_compare_fails_when_firmware_ota_route_kind_is_downgraded() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let request = default_request(STATIC_USAGE);
    let mut routes = bitaxe_api::phase07_routes().to_vec();
    downgrade_route_kind(
        &mut routes,
        RouteMethod::Post,
        "/api/system/OTA",
        RouteKind::SafeUnsupportedUpdate,
    );

    // Act
    let report =
        run_api_compare_with_routes(&request, &loader, &routes).expect("api compare should run");

    // Assert
    assert_validation_error_contains(
        &report,
        &["POST /api/system/OTA", "expected RouteKind::FirmwareUpdate"],
    );
}

#[test]
fn api_compare_fails_when_otawww_route_kind_is_downgraded() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let request = default_request(STATIC_USAGE);
    let mut routes = bitaxe_api::phase07_routes().to_vec();
    downgrade_route_kind(
        &mut routes,
        RouteMethod::Post,
        "/api/system/OTAWWW",
        RouteKind::SafeUnsupportedUpdate,
    );

    // Act
    let report =
        run_api_compare_with_routes(&request, &loader, &routes).expect("api compare should run");

    // Assert
    assert_validation_error_contains(
        &report,
        &[
            "POST /api/system/OTAWWW",
            "expected RouteKind::AxeOsStaticUpdateGap",
        ],
    );
}

#[test]
fn api_compare_fails_when_recovery_or_static_route_kind_is_downgraded() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let request = default_request(STATIC_USAGE);
    let cases = [
        (
            RouteMethod::Get,
            "/recovery",
            RouteKind::Http,
            "expected RouteKind::Recovery",
        ),
        (
            RouteMethod::Get,
            "/*",
            RouteKind::Http,
            "expected RouteKind::StaticFiles",
        ),
    ];

    for (method, path, replacement_kind, expected_error) in cases {
        let mut routes = bitaxe_api::phase07_routes().to_vec();
        downgrade_route_kind(&mut routes, method, path, replacement_kind);

        // Act
        let report = run_api_compare_with_routes(&request, &loader, &routes)
            .expect("api compare should run");

        // Assert
        assert_validation_error_contains(
            &report,
            &[&route_key(route_method_label(method), path), expected_error],
        );
    }
}

#[test]
fn api_compare_fails_when_release_sensitive_route_claims_verified_from_weak_evidence() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let weak_evidence = ["unit", "workflow", "package", "api-compare", "static-route"];

    for (method, path) in [
        ("POST", "/api/system/OTA"),
        ("POST", "/api/system/OTAWWW"),
        ("GET", "/recovery"),
        ("GET", "/*"),
    ] {
        let static_usage =
            static_usage_with_verified_claim(method, path, "verified", &weak_evidence);
        let request = default_request(&static_usage);

        // Act
        let report = run_api_compare_with_routes(&request, &loader, bitaxe_api::phase07_routes())
            .expect("api compare should run");

        // Assert
        assert_validation_error_contains(
            &report,
            &[&route_key(method, path), "insufficient verified evidence"],
        );
    }
}

#[test]
fn api_compare_fails_when_release_sensitive_route_claims_verified_from_unknown_evidence() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let unknown_evidence = ["hardwar-smoke"];
    let static_usage =
        static_usage_with_verified_claim("POST", "/api/system/OTA", "verified", &unknown_evidence);
    let request = default_request(&static_usage);

    // Act
    let report = run_api_compare_with_routes(&request, &loader, bitaxe_api::phase07_routes())
        .expect("api compare should run");

    // Assert
    assert_validation_error_contains(
        &report,
        &[
            "POST /api/system/OTA",
            "insufficient verified evidence",
            "hardwar-smoke",
        ],
    );
}

#[test]
fn api_compare_fails_when_required_route_is_removed_from_fixture() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let route_manifest =
        ROUTE_MANIFEST.replace(r#"{"method": "GET", "path": "/api/system/info"},"#, "");
    let request = ApiCompareRequest {
        openapi_yaml: OPENAPI,
        route_manifest_json: &route_manifest,
        static_usage_json: STATIC_USAGE,
    };

    // Act
    let report = run_api_compare(&request, &loader).expect("api compare should run");

    // Assert
    assert!(report
        .validation_errors
        .iter()
        .any(|error| error.contains("GET /api/system/info")));
}

#[test]
fn api_compare_fails_when_required_property_only_exists_in_another_schema() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let openapi = OPENAPI
        .replace(
            "        responseTime:\n    SystemASIC:",
            "        responseTime:\n        frequency:\n    SystemASIC:",
        )
        .replace(
            "        frequency:\n        coreVoltage:",
            "        coreVoltage:",
        );
    let request = ApiCompareRequest {
        openapi_yaml: &openapi,
        route_manifest_json: ROUTE_MANIFEST,
        static_usage_json: STATIC_USAGE,
    };

    // Act
    let report = run_api_compare(&request, &loader).expect("api compare should run");

    // Assert
    assert!(report.validation_errors.iter().any(|error| {
        error.contains("OpenAPI schema Settings")
            && error.contains("PATCH /api/system")
            && error.contains("frequency")
    }));
}

#[test]
fn api_compare_fails_when_ota_route_is_marked_phase05_success() {
    // Arrange
    let loader = MemoryFixtureLoader;
    let static_usage = STATIC_USAGE.replace(
        r#""surface": "firmware_ota",
      "source": "system.service.ts performOTAUpdate",
      "method": "POST",
      "path": "/api/system/OTA",
      "evidence_type": "static-route",
      "category": "update-route",
      "phase_owner": "phase07",
      "phase05_behavior": "unsafe-success-blocked",
      "counts_as_phase05_success": false"#,
        r#""surface": "firmware_ota",
      "source": "system.service.ts performOTAUpdate",
      "method": "POST",
      "path": "/api/system/OTA",
      "evidence_type": "static-route",
      "category": "update-route",
      "phase_owner": "phase05",
      "phase05_behavior": "administrable",
      "counts_as_phase05_success": true"#,
    );
    let request = ApiCompareRequest {
        openapi_yaml: OPENAPI,
        route_manifest_json: ROUTE_MANIFEST,
        static_usage_json: &static_usage,
    };

    // Act
    let report = run_api_compare(&request, &loader).expect("api compare should run");

    // Assert
    assert!(report
        .validation_errors
        .iter()
        .any(|error| error.contains("Phase 7-owned")));
    assert!(report
        .validation_errors
        .iter()
        .any(|error| error.contains("Phase 05 update success")));
}

fn default_request(static_usage_json: &str) -> ApiCompareRequest<'_> {
    ApiCompareRequest {
        openapi_yaml: OPENAPI,
        route_manifest_json: ROUTE_MANIFEST,
        static_usage_json,
    }
}

fn downgrade_route_kind(
    routes: &mut [AxeosRoute],
    method: RouteMethod,
    path: &str,
    replacement_kind: RouteKind,
) {
    let route = routes
        .iter_mut()
        .find(|route| route.method == method && route.path == path)
        .expect("test route should exist");
    route.kind = replacement_kind;
}

fn static_usage_with_verified_claim(
    method: &str,
    path: &str,
    status: &str,
    evidence: &[&str],
) -> String {
    let mut fixture: Value =
        serde_json::from_str(STATIC_USAGE).expect("static usage fixture should parse");
    let calls = fixture
        .get_mut("service_calls")
        .and_then(Value::as_array_mut)
        .expect("static usage fixture should have service calls");
    let call = calls
        .iter_mut()
        .find(|call| call["method"] == method && call["path"] == path)
        .expect("static usage route should exist");
    let verified_claim = json!({
        "status": status,
        "evidence": evidence,
    });
    call.as_object_mut()
        .expect("static usage call should be an object")
        .insert("verified_claim".to_owned(), verified_claim);

    serde_json::to_string(&fixture).expect("static usage fixture should serialize")
}

fn assert_validation_error_contains(report: &ApiCompareReport, parts: &[&str]) {
    assert!(
        report
            .validation_errors
            .iter()
            .any(|error| parts.iter().all(|part| error.contains(part))),
        "expected validation error containing {parts:?}, got {:#?}",
        report.validation_errors
    );
}

struct MemoryFixtureLoader;

impl JsonFixtureLoader for MemoryFixtureLoader {
    fn load_json_fixture(&self, path: &str) -> Result<Value> {
        let value = match path {
            "crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json" => json!({
                "ASICModel": "BM1366",
                "hashRate_1m": 0,
                "fanspeed": 0,
                "fanrpm": 0,
                "miningPaused": true,
                "sharesRejectedReasons": [],
                "poolDifficulty": 0,
                "responseTime": 0
            }),
            "crates/bitaxe-api/fixtures/api/asic-settings-ultra205.json" => json!({
                "ASICModel": "BM1366",
                "deviceModel": "Ultra",
                "asicCount": 1,
                "defaultFrequency": 485,
                "frequencyOptions": [485],
                "defaultVoltage": 1200,
                "voltageOptions": [1200]
            }),
            "crates/bitaxe-api/fixtures/api/statistics-empty-compatible.json" => json!({
                "currentTimestamp": 0,
                "labels": ["timestamp"],
                "statistics": []
            }),
            "crates/bitaxe-api/fixtures/api/scoreboard-empty.json" => json!([]),
            "crates/bitaxe-api/fixtures/api/settings-patch-cases.json" => json!({
                "valid": {},
                "unknown_only": {},
                "invalid_known": {},
                "invalid_json_public_error": "Invalid JSON",
                "wrong_input_public_error": "Wrong API input"
            }),
            "crates/bitaxe-api/fixtures/api/log-buffer-cases.json" => json!({
                "download_headers": {
                    "content_type": "text/plain",
                    "content_disposition": "attachment; filename=\"bitaxe-logs.txt\""
                },
                "raw_stream": {
                    "payload": "I (123) bitaxe: live log line\n",
                    "json_enveloped": false
                }
            }),
            "crates/bitaxe-api/fixtures/api/live-telemetry-cases.json" => json!({
                "expected_connect_frame": {
                    "event": "update",
                    "data": {}
                },
                "expected_diff_frame": {
                    "event": "update",
                    "data": {}
                }
            }),
            "crates/bitaxe-api/fixtures/api/command-responses.json" => json!({
                "pause": {},
                "resume": {},
                "restart": {},
                "identify_on": {},
                "identify_off": {},
                "block_found_dismiss": {}
            }),
            "crates/bitaxe-api/fixtures/api/theme-cases.json" => json!({
                "metadata": {},
                "default_get": {},
                "post_success": {"status": "ok"}
            }),
            _ => return Err(anyhow!("missing test fixture {path}")),
        };

        Ok(value)
    }
}
