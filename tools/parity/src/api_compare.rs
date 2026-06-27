//! Phase 05 API/static compatibility comparison checks.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/openapi.yaml`
//! - `reference/esp-miner/main/http_server/axe-os/src/app/services/system.service.ts`
//! - `reference/esp-miner/main/http_server/axe-os/src/app/services/live-data.service.ts`
//! - `reference/esp-miner/main/http_server/axe-os/src/app/services/web-socket.service.ts`
//! - `reference/esp-miner/main/filesystem.c`

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use bitaxe_api::{phase05_routes, RouteMethod};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Borrowed input strings for the API comparison run.
pub struct ApiCompareRequest<'a> {
    /// Pinned upstream OpenAPI YAML text.
    pub openapi_yaml: &'a str,
    /// Structured Phase 05 route/property assertion manifest.
    pub route_manifest_json: &'a str,
    /// Structured AxeOS static route usage fixture.
    pub static_usage_json: &'a str,
}

/// Filesystem-backed fixture loader for captured response fixtures.
#[derive(Debug)]
pub struct WorkspaceFixtureLoader {
    workspace_dir: Utf8PathBuf,
}

impl WorkspaceFixtureLoader {
    /// Creates a workspace-backed loader.
    #[must_use]
    pub fn new(workspace_dir: Utf8PathBuf) -> Self {
        Self { workspace_dir }
    }
}

/// Loads JSON fixture data referenced by the API compare manifest.
pub trait JsonFixtureLoader {
    /// Loads a fixture path relative to the workspace root.
    fn load_json_fixture(&self, path: &str) -> Result<Value>;
}

impl JsonFixtureLoader for WorkspaceFixtureLoader {
    fn load_json_fixture(&self, path: &str) -> Result<Value> {
        let fixture_path = self.workspace_dir.join(path);
        let contents = std::fs::read_to_string(fixture_path.as_std_path())
            .with_context(|| format!("failed to read captured response fixture {fixture_path}"))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("fixture {fixture_path} was not valid JSON"))
    }
}

/// API compare report rendered by the CLI.
#[derive(Debug, Serialize)]
pub struct ApiCompareReport {
    /// Evidence labels that were evaluated.
    pub evidence: Vec<ApiCompareEvidence>,
    /// Validation errors. Empty means comparison passed.
    pub validation_errors: Vec<String>,
}

impl ApiCompareReport {
    /// Returns true when any comparison check failed.
    #[must_use]
    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }
}

/// Result for one evidence class.
#[derive(Debug, Serialize)]
pub struct ApiCompareEvidence {
    /// Evidence type label.
    pub evidence_type: &'static str,
    /// `passed`, `failed`, or `not-run`.
    pub status: &'static str,
    /// Number of checks in this evidence class.
    pub checked: usize,
    /// Human-readable note.
    pub note: String,
}

/// Runs Phase 05 API comparison checks.
pub fn run_api_compare(
    request: &ApiCompareRequest<'_>,
    loader: &impl JsonFixtureLoader,
) -> Result<ApiCompareReport> {
    let route_manifest: RouteManifest = serde_json::from_str(request.route_manifest_json)
        .context("failed to parse Phase 05 route/property manifest")?;
    let static_usage: StaticRouteUsageFixture = serde_json::from_str(request.static_usage_json)
        .context("failed to parse AxeOS route usage fixture")?;

    let mut validation_errors = Vec::new();
    let schema_checked = validate_schema_evidence(
        request.openapi_yaml,
        &route_manifest,
        &mut validation_errors,
    );
    let captured_checked =
        validate_captured_response_evidence(&route_manifest, loader, &mut validation_errors)?;
    let static_checked = validate_static_route_evidence(&static_usage, &mut validation_errors);

    let has_errors = !validation_errors.is_empty();

    Ok(ApiCompareReport {
        evidence: vec![
            ApiCompareEvidence {
                evidence_type: "schema",
                status: status_for_errors(has_errors),
                checked: schema_checked,
                note: "OpenAPI route/property coverage and Rust route-shell manifest".to_owned(),
            },
            ApiCompareEvidence {
                evidence_type: "captured-response",
                status: status_for_errors(has_errors),
                checked: captured_checked,
                note: "Representative checked-in JSON response fixtures".to_owned(),
            },
            ApiCompareEvidence {
                evidence_type: "static-route",
                status: status_for_errors(has_errors),
                checked: static_checked,
                note: "Existing AxeOS service route usage plus recovery/static boundaries"
                    .to_owned(),
            },
            ApiCompareEvidence {
                evidence_type: "firmware-smoke",
                status: "not-run",
                checked: 0,
                note: route_manifest.firmware_smoke.reason,
            },
        ],
        validation_errors,
    })
}

/// Renders API compare output in the parity tool text style.
#[must_use]
pub fn render_api_compare_report(report: &ApiCompareReport) -> String {
    let mut output = String::new();
    output.push_str("api_compare:\n");

    for evidence in &report.evidence {
        output.push_str(&format!(
            "- {} | status={} | checked={}\n  note: {}\n",
            evidence.evidence_type, evidence.status, evidence.checked, evidence.note
        ));
    }

    if report.validation_errors.is_empty() {
        output.push_str("validation_errors: none\n");
    } else {
        output.push_str("validation_errors:\n");
        for error in &report.validation_errors {
            output.push_str(&format!("- {error}\n"));
        }
    }

    output
}

#[derive(Debug, Deserialize)]
struct RouteManifest {
    rust_route_manifest_routes: Vec<RouteAssertion>,
    schema_routes: Vec<SchemaRouteAssertion>,
    captured_response_checks: Vec<CapturedResponseCheck>,
    firmware_smoke: FirmwareSmoke,
}

#[derive(Debug, Deserialize)]
struct RouteAssertion {
    method: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct SchemaRouteAssertion {
    method: String,
    path: String,
    schema: String,
    #[serde(default)]
    required_properties: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CapturedResponseCheck {
    name: String,
    evidence_type: String,
    fixture: String,
    json_pointer: String,
    expected_kind: String,
    #[serde(default)]
    required_properties: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FirmwareSmoke {
    reason: String,
}

#[derive(Debug, Deserialize)]
struct StaticRouteUsageFixture {
    service_calls: Vec<StaticRouteUsage>,
    static_packaging: StaticPackaging,
}

#[derive(Debug, Deserialize)]
struct StaticRouteUsage {
    surface: String,
    method: String,
    path: String,
    evidence_type: String,
    category: String,
    phase_owner: String,
    phase05_behavior: String,
    counts_as_phase05_success: bool,
}

#[derive(Debug, Deserialize)]
struct StaticPackaging {
    phase05_success_claim: bool,
    phase07_owner: bool,
}

const REQUIRED_PHASE05_ROUTES: &[(&str, &str)] = &[
    ("GET", "/api/system/info"),
    ("PATCH", "/api/system"),
    ("GET", "/api/system/logs"),
    ("GET", "/api/system/asic"),
    ("GET", "/api/system/statistics"),
    ("GET", "/api/system/scoreboard"),
    ("POST", "/api/system/pause"),
    ("POST", "/api/system/resume"),
    ("POST", "/api/system/restart"),
    ("POST", "/api/system/identify"),
    ("POST", "/api/system/blockFound/dismiss"),
    ("POST", "/api/system/OTA"),
    ("POST", "/api/system/OTAWWW"),
    ("GET", "/api/ws"),
    ("GET", "/api/ws/live"),
];

const REQUIRED_STATIC_USAGE_ROUTES: &[(&str, &str)] = &[
    ("GET", "/api/system/info"),
    ("GET", "/api/system/asic"),
    ("GET", "/api/system/statistics"),
    ("GET", "/api/system/scoreboard"),
    ("GET", "/api/system/logs"),
    ("PATCH", "/api/system"),
    ("POST", "/api/system/pause"),
    ("POST", "/api/system/resume"),
    ("POST", "/api/system/restart"),
    ("POST", "/api/system/identify"),
    ("POST", "/api/system/blockFound/dismiss"),
    ("GET", "/api/ws"),
    ("GET", "/api/ws/live"),
    ("POST", "/api/system/OTA"),
    ("POST", "/api/system/OTAWWW"),
    ("GET", "/recovery"),
    ("GET", "/*"),
];

fn validate_schema_evidence(
    openapi_yaml: &str,
    route_manifest: &RouteManifest,
    validation_errors: &mut Vec<String>,
) -> usize {
    let mut checked = 0;
    let manifest_routes = route_set(&route_manifest.rust_route_manifest_routes);
    let rust_routes = rust_route_set();

    for (method, path) in REQUIRED_PHASE05_ROUTES {
        checked += 1;
        if !manifest_routes.contains(&route_key(method, path)) {
            validation_errors.push(format!("required route fixture missing {method} {path}"));
        }
    }

    for route in &route_manifest.rust_route_manifest_routes {
        checked += 1;
        if !rust_routes.contains(&route_key(&route.method, &route.path)) {
            validation_errors.push(format!(
                "Rust route shell missing {} {} from fixture",
                route.method, route.path
            ));
        }
    }

    for route in phase05_routes() {
        checked += 1;
        let method = route_method_label(route.method);
        if !manifest_routes.contains(&route_key(method, route.path)) {
            validation_errors.push(format!(
                "required route fixture missing {method} {}",
                route.path
            ));
        }
    }

    for schema_route in &route_manifest.schema_routes {
        checked += 1;
        if !manifest_routes.contains(&route_key(&schema_route.method, &schema_route.path)) {
            validation_errors.push(format!(
                "schema route {} {} is missing from Rust route fixture",
                schema_route.method, schema_route.path
            ));
        }

        if !openapi_has_path_method(openapi_yaml, &schema_route.path, &schema_route.method) {
            validation_errors.push(format!(
                "OpenAPI contract missing {} {}",
                schema_route.method, schema_route.path
            ));
        }

        for property in &schema_route.required_properties {
            checked += 1;
            if !openapi_has_property(openapi_yaml, property) {
                validation_errors.push(format!(
                    "OpenAPI schema {} for {} {} missing property {property}",
                    schema_route.schema, schema_route.method, schema_route.path
                ));
            }
        }
    }

    checked
}

fn validate_captured_response_evidence(
    route_manifest: &RouteManifest,
    loader: &impl JsonFixtureLoader,
    validation_errors: &mut Vec<String>,
) -> Result<usize> {
    let mut checked = 0;

    for check in &route_manifest.captured_response_checks {
        checked += 1;
        if check.evidence_type != "captured-response" {
            validation_errors.push(format!(
                "captured response check {} has wrong evidence label {}",
                check.name, check.evidence_type
            ));
        }

        let fixture = loader.load_json_fixture(&check.fixture)?;
        let maybe_value = if check.json_pointer.is_empty() {
            Some(&fixture)
        } else {
            fixture.pointer(&check.json_pointer)
        };
        let Some(value) = maybe_value else {
            validation_errors.push(format!(
                "captured response check {} missing JSON pointer {} in {}",
                check.name, check.json_pointer, check.fixture
            ));
            continue;
        };

        if !value_matches_kind(value, &check.expected_kind) {
            validation_errors.push(format!(
                "captured response check {} expected {} at {} in {}",
                check.name, check.expected_kind, check.json_pointer, check.fixture
            ));
        }

        for property in &check.required_properties {
            checked += 1;
            if value.get(property).is_none() {
                validation_errors.push(format!(
                    "captured response check {} missing property {property} in {}",
                    check.name, check.fixture
                ));
            }
        }
    }

    Ok(checked)
}

fn validate_static_route_evidence(
    static_usage: &StaticRouteUsageFixture,
    validation_errors: &mut Vec<String>,
) -> usize {
    let mut checked = 0;
    let static_routes = static_route_set(&static_usage.service_calls);

    for (method, path) in REQUIRED_STATIC_USAGE_ROUTES {
        checked += 1;
        if !static_routes.contains(&route_key(method, path)) {
            validation_errors.push(format!(
                "AxeOS static route usage fixture missing {method} {path}"
            ));
        }
    }

    for call in &static_usage.service_calls {
        checked += 1;
        if call.evidence_type != "static-route" {
            validation_errors.push(format!(
                "AxeOS route usage {} ({} {}) has wrong evidence label {}",
                call.surface, call.method, call.path, call.evidence_type
            ));
        }

        if matches!(call.path.as_str(), "/api/system/OTA" | "/api/system/OTAWWW") {
            validate_phase7_update_route(call, validation_errors);
        }

        if call.path == "/recovery" {
            validate_phase7_static_boundary(call, "recovery-route", validation_errors);
        }

        if call.path == "/*" {
            validate_phase7_static_boundary(call, "static-fallback", validation_errors);
        }
    }

    checked += 1;
    if static_usage.static_packaging.phase05_success_claim {
        validation_errors
            .push("static/recovery packaging fixture must not claim Phase 05 success".to_owned());
    }

    checked += 1;
    if !static_usage.static_packaging.phase07_owner {
        validation_errors
            .push("static/recovery packaging fixture must mark Phase 7 ownership".to_owned());
    }

    checked
}

fn validate_phase7_update_route(call: &StaticRouteUsage, validation_errors: &mut Vec<String>) {
    if call.phase_owner != "phase07" {
        validation_errors.push(format!(
            "{} ({}) must remain Phase 7-owned, got {}",
            call.surface, call.path, call.phase_owner
        ));
    }

    if call.phase05_behavior != "unsafe-success-blocked" {
        validation_errors.push(format!(
            "{} ({}) must be unsafe-success-blocked in Phase 05, got {}",
            call.surface, call.path, call.phase05_behavior
        ));
    }

    if call.counts_as_phase05_success {
        validation_errors.push(format!(
            "{} ({}) must not count as Phase 05 update success",
            call.surface, call.path
        ));
    }
}

fn validate_phase7_static_boundary(
    call: &StaticRouteUsage,
    expected_category: &str,
    validation_errors: &mut Vec<String>,
) {
    if call.category != expected_category {
        validation_errors.push(format!(
            "{} ({}) must be category {expected_category}, got {}",
            call.surface, call.path, call.category
        ));
    }

    if call.phase_owner != "phase07" {
        validation_errors.push(format!(
            "{} ({}) must remain Phase 7-owned, got {}",
            call.surface, call.path, call.phase_owner
        ));
    }

    if call.counts_as_phase05_success {
        validation_errors.push(format!(
            "{} ({}) must not count as Phase 05 static/recovery packaging success",
            call.surface, call.path
        ));
    }
}

fn route_set(routes: &[RouteAssertion]) -> BTreeSet<String> {
    routes
        .iter()
        .map(|route| route_key(&route.method, &route.path))
        .collect()
}

fn static_route_set(routes: &[StaticRouteUsage]) -> BTreeSet<String> {
    routes
        .iter()
        .map(|route| route_key(&route.method, &route.path))
        .collect()
}

fn rust_route_set() -> BTreeSet<String> {
    phase05_routes()
        .iter()
        .map(|route| route_key(route_method_label(route.method), route.path))
        .collect()
}

fn route_key(method: &str, path: &str) -> String {
    format!("{} {}", method.to_ascii_uppercase(), path)
}

fn route_method_label(method: RouteMethod) -> &'static str {
    match method {
        RouteMethod::Get => "GET",
        RouteMethod::Patch => "PATCH",
        RouteMethod::Post => "POST",
    }
}

fn openapi_has_path_method(openapi_yaml: &str, path: &str, method: &str) -> bool {
    let path_marker = format!("  {path}:");
    let Some(start) = openapi_yaml.find(&path_marker) else {
        return false;
    };
    let path_block = &openapi_yaml[start + path_marker.len()..];
    let path_block_end = path_block.find("\n  /").unwrap_or(path_block.len());
    let path_block = &path_block[..path_block_end];
    let method_marker = format!("\n    {}:", method.to_ascii_lowercase());

    path_block.contains(&method_marker)
}

fn openapi_has_property(openapi_yaml: &str, property: &str) -> bool {
    let property_marker = format!("{property}:");
    openapi_yaml
        .lines()
        .any(|line| line.trim_start().starts_with(&property_marker))
}

fn value_matches_kind(value: &Value, expected_kind: &str) -> bool {
    match expected_kind {
        "array" => value.is_array(),
        "boolean" => value.is_boolean(),
        "number" => value.is_number(),
        "object" => value.is_object(),
        "string" => value.is_string(),
        _ => false,
    }
}

fn status_for_errors(has_errors: bool) -> &'static str {
    if has_errors {
        return "failed";
    }

    "passed"
}

#[cfg(test)]
mod tests {
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
    Settings:
      properties:
        frequency:
        coreVoltage:
        hostname:
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
    const ROUTE_MANIFEST: &str = include_str!("../fixtures/api/phase05-required-routes.json");
    const STATIC_USAGE: &str = include_str!("../fixtures/api/axeos-route-usage.json");

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
                _ => return Err(anyhow!("missing test fixture {path}")),
            };

            Ok(value)
        }
    }
}
