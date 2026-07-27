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
use bitaxe_api::{phase05_routes, phase07_routes, AxeosRoute, RouteKind, RouteMethod};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod openapi;
mod static_routes;

use openapi::{openapi_has_path_method, openapi_route_schema_has_property};
use static_routes::validate_static_route_evidence;

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
    run_api_compare_with_routes(request, loader, phase07_routes())
}

fn run_api_compare_with_routes(
    request: &ApiCompareRequest<'_>,
    loader: &impl JsonFixtureLoader,
    rust_routes: &[AxeosRoute],
) -> Result<ApiCompareReport> {
    let route_manifest: RouteManifest = serde_json::from_str(request.route_manifest_json)
        .context("failed to parse Phase 05 route/property manifest")?;
    let static_usage: StaticRouteUsageFixture = serde_json::from_str(request.static_usage_json)
        .context("failed to parse AxeOS route usage fixture")?;

    let mut validation_errors = Vec::new();
    let schema_checked = validate_schema_evidence(
        request.openapi_yaml,
        &route_manifest,
        rust_routes,
        &mut validation_errors,
    ) + validate_phase07_route_policy(rust_routes, &mut validation_errors);
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
    #[serde(default)]
    verified_claim: Option<VerifiedClaim>,
}

#[derive(Debug, Deserialize)]
struct VerifiedClaim {
    status: String,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StaticPackaging {
    phase05_success_claim: bool,
    phase07_owner: bool,
}

#[derive(Debug, Clone, Copy)]
struct Phase07RoutePolicy {
    method: RouteMethod,
    path: &'static str,
    kind: RouteKind,
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

const REQUIRED_PHASE07_ROUTE_POLICY: &[Phase07RoutePolicy] = &[
    Phase07RoutePolicy {
        method: RouteMethod::Post,
        path: "/api/system/OTA",
        kind: RouteKind::FirmwareUpdate,
    },
    Phase07RoutePolicy {
        method: RouteMethod::Post,
        path: "/api/system/OTAWWW",
        kind: RouteKind::AxeOsStaticUpdateGap,
    },
    Phase07RoutePolicy {
        method: RouteMethod::Get,
        path: "/recovery",
        kind: RouteKind::Recovery,
    },
    Phase07RoutePolicy {
        method: RouteMethod::Get,
        path: "/*",
        kind: RouteKind::StaticFiles,
    },
];

const WEAK_VERIFIED_EVIDENCE_LABELS: &[&str] =
    &["unit", "workflow", "package", "api-compare", "static-route"];
const STRONG_VERIFIED_EVIDENCE_LABELS: &[&str] =
    &["hardware-smoke", "hardware-regression", "release-gate"];

fn validate_schema_evidence(
    openapi_yaml: &str,
    route_manifest: &RouteManifest,
    rust_routes: &[AxeosRoute],
    validation_errors: &mut Vec<String>,
) -> usize {
    let mut checked = 0;
    let manifest_routes = route_set(&route_manifest.rust_route_manifest_routes);
    let rust_route_keys = rust_route_set(rust_routes);

    for (method, path) in REQUIRED_PHASE05_ROUTES {
        checked += 1;
        if !manifest_routes.contains(&route_key(method, path)) {
            validation_errors.push(format!("required route fixture missing {method} {path}"));
        }
    }

    for route in &route_manifest.rust_route_manifest_routes {
        checked += 1;
        if !rust_route_keys.contains(&route_key(&route.method, &route.path)) {
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
            if !openapi_route_schema_has_property(openapi_yaml, schema_route, property) {
                validation_errors.push(format!(
                    "OpenAPI schema {} for {} {} missing property {property}",
                    schema_route.schema, schema_route.method, schema_route.path
                ));
            }
        }
    }

    checked
}

fn validate_phase07_route_policy(
    rust_routes: &[AxeosRoute],
    validation_errors: &mut Vec<String>,
) -> usize {
    let mut checked = 0;

    for policy in REQUIRED_PHASE07_ROUTE_POLICY {
        checked += 1;
        let method = route_method_label(policy.method);
        let maybe_route = rust_routes
            .iter()
            .find(|route| route.method == policy.method && route.path == policy.path);
        let Some(route) = maybe_route else {
            validation_errors.push(format!(
                "Phase 7 Rust route manifest missing {method} {}",
                policy.path
            ));
            continue;
        };

        if route.kind != policy.kind {
            validation_errors.push(format!(
                "Phase 7 Rust route manifest {method} {} expected {}, got {}",
                policy.path,
                route_kind_label(policy.kind),
                route_kind_label(route.kind)
            ));
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

fn rust_route_set(routes: &[AxeosRoute]) -> BTreeSet<String> {
    routes
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

fn route_kind_label(kind: RouteKind) -> &'static str {
    match kind {
        RouteKind::Http => "RouteKind::Http",
        RouteKind::WebSocket(_) => "RouteKind::WebSocket",
        RouteKind::SafeUnsupportedUpdate => "RouteKind::SafeUnsupportedUpdate",
        RouteKind::FirmwareUpdate => "RouteKind::FirmwareUpdate",
        RouteKind::AxeOsStaticUpdateGap => "RouteKind::AxeOsStaticUpdateGap",
        RouteKind::Recovery => "RouteKind::Recovery",
        RouteKind::StaticFiles => "RouteKind::StaticFiles",
    }
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
mod tests;
