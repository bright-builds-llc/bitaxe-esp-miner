//! Pure route-shell decisions for the firmware HTTP/WebSocket adapter.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `reference/esp-miner/main/http_server/websocket.c`

use std::net::Ipv4Addr;

use crate::settings::SettingsPatchPublicError;

/// Public denial body used by HTTP and WebSocket upgrade rejection.
pub const UNAUTHORIZED_BODY: &str = "Unauthorized";
/// Unknown API route body expected by AxeOS clients.
pub const UNKNOWN_API_ROUTE_BODY: &str = r#"{"error":"unknown route"}"#;
/// Upstream scratch buffer accepts at most 10 KiB minus one null terminator.
pub const MAX_SETTINGS_PATCH_BODY_BYTES: usize = (10 * 1024) - 1;
const APPLICATION_JSON: &str = "application/json";
const TEXT_PLAIN: &str = "text/plain";

const PHASE05_ROUTES: &[AxeosRoute] = &[
    AxeosRoute {
        path: "/api/system/info",
        method: RouteMethod::Get,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system",
        method: RouteMethod::Patch,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/logs",
        method: RouteMethod::Get,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/asic",
        method: RouteMethod::Get,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/statistics",
        method: RouteMethod::Get,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/scoreboard",
        method: RouteMethod::Get,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/pause",
        method: RouteMethod::Post,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/resume",
        method: RouteMethod::Post,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/restart",
        method: RouteMethod::Post,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/identify",
        method: RouteMethod::Post,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/blockFound/dismiss",
        method: RouteMethod::Post,
        kind: RouteKind::Http,
    },
    AxeosRoute {
        path: "/api/system/OTA",
        method: RouteMethod::Post,
        kind: RouteKind::SafeUnsupportedUpdate,
    },
    AxeosRoute {
        path: "/api/system/OTAWWW",
        method: RouteMethod::Post,
        kind: RouteKind::SafeUnsupportedUpdate,
    },
    AxeosRoute {
        path: "/api/ws",
        method: RouteMethod::Get,
        kind: RouteKind::WebSocket(WebSocketRouteKind::Logs),
    },
    AxeosRoute {
        path: "/api/ws/live",
        method: RouteMethod::Get,
        kind: RouteKind::WebSocket(WebSocketRouteKind::LiveTelemetry),
    },
];

/// Firmware-visible HTTP method for route registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMethod {
    /// HTTP GET.
    Get,
    /// HTTP PATCH.
    Patch,
    /// HTTP POST.
    Post,
}

/// Firmware-visible route kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteKind {
    /// Normal HTTP API route.
    Http,
    /// WebSocket upgrade route.
    WebSocket(WebSocketRouteKind),
    /// OTA/OTAWWW route that must not apply updates in Phase 5.
    SafeUnsupportedUpdate,
}

/// WebSocket route type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketRouteKind {
    /// Raw retained-log stream.
    Logs,
    /// Live telemetry stream.
    LiveTelemetry,
}

/// Compile-visible AxeOS route registration entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxeosRoute {
    /// Route path.
    pub path: &'static str,
    /// Route method.
    pub method: RouteMethod,
    /// Route behavior owner.
    pub kind: RouteKind,
}

/// Public HTTP response decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicHttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Public response body.
    pub body: &'static str,
    /// Optional content type.
    pub content_type: Option<&'static str>,
}

/// Access-check input shared by HTTP and WebSocket route shells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteAccessInput {
    /// Upstream AP-mode bypass for captive/local setup.
    pub ap_mode_enabled: bool,
    /// Client peer IPv4 address.
    pub request_ip: Ipv4Addr,
    /// Origin header state.
    pub origin: OriginGate,
}

/// Parsed request Origin state for the private-network access gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginGate {
    /// The request did not include an Origin header.
    Missing,
    /// The Origin header host parsed as an IPv4 address.
    Parsed(Ipv4Addr),
    /// The Origin header was present but could not be accepted.
    Invalid,
}

/// HTTP access decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpAccessDecision {
    /// Request may continue to the route handler.
    Allow,
    /// Request is denied with a public response.
    Deny(PublicHttpResponse),
}

/// WebSocket registration plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebSocketClientRegistrationPlan {
    /// WebSocket route type to register after a successful upgrade gate.
    pub route: WebSocketRouteKind,
}

/// WebSocket upgrade decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketUpgradeDecision {
    /// Upgrade may continue and register a client.
    Accept(WebSocketClientRegistrationPlan),
    /// Upgrade must be rejected before client registration.
    Reject(PublicHttpResponse),
}

/// Settings PATCH body-size decision before JSON parsing or persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPatchBodyDecision {
    /// Body length is within the bounded scratch-buffer contract.
    Accept,
    /// Body length must be rejected before reading/parsing side effects.
    Reject(PublicHttpResponse),
}

/// Returns every Phase 05 firmware API/WebSocket route.
#[must_use]
pub const fn phase05_routes() -> &'static [AxeosRoute] {
    PHASE05_ROUTES
}

/// Applies the private-network/AP-origin gate to an HTTP route.
#[must_use]
pub fn plan_http_access(input: RouteAccessInput) -> HttpAccessDecision {
    if is_access_allowed(input) {
        return HttpAccessDecision::Allow;
    }

    HttpAccessDecision::Deny(unauthorized_response())
}

/// Applies the private-network/AP-origin gate to a WebSocket route.
#[must_use]
pub fn plan_websocket_upgrade(
    input: RouteAccessInput,
    route: WebSocketRouteKind,
) -> WebSocketUpgradeDecision {
    if !is_access_allowed(input) {
        return WebSocketUpgradeDecision::Reject(unauthorized_response());
    }

    WebSocketUpgradeDecision::Accept(WebSocketClientRegistrationPlan { route })
}

/// Applies the settings PATCH body cap before JSON parsing or NVS access.
#[must_use]
pub const fn plan_settings_patch_body_size(body_len: usize) -> SettingsPatchBodyDecision {
    if body_len > MAX_SETTINGS_PATCH_BODY_BYTES {
        return SettingsPatchBodyDecision::Reject(settings_patch_body_too_large_response());
    }

    SettingsPatchBodyDecision::Accept
}

/// Returns the public 404 shape for unknown `/api/*` routes.
#[must_use]
pub const fn unknown_api_route_response() -> PublicHttpResponse {
    PublicHttpResponse {
        status: 404,
        body: UNKNOWN_API_ROUTE_BODY,
        content_type: Some(APPLICATION_JSON),
    }
}

/// Returns a safe unsupported response for Phase 7-owned update routes.
#[must_use]
pub const fn unsupported_update_response() -> PublicHttpResponse {
    PublicHttpResponse {
        status: 400,
        body: "Wrong API input",
        content_type: Some(TEXT_PLAIN),
    }
}

const fn settings_patch_body_too_large_response() -> PublicHttpResponse {
    PublicHttpResponse {
        status: 400,
        body: SettingsPatchPublicError::WrongApiInput.body(),
        content_type: Some(TEXT_PLAIN),
    }
}

/// Parses a request Origin header into an IPv4 address when the host is an IPv4 literal.
#[must_use]
pub fn maybe_origin_ip_from_header(origin: &str) -> Option<Ipv4Addr> {
    let without_scheme = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .unwrap_or(origin);
    let host_with_maybe_port = without_scheme.split('/').next().unwrap_or(without_scheme);
    let host = host_with_maybe_port
        .split(':')
        .next()
        .unwrap_or(host_with_maybe_port);

    host.parse().ok()
}

/// Classifies a present request Origin header for access-gate decisions.
#[must_use]
pub fn origin_gate_from_header(origin: &str) -> OriginGate {
    let Some(origin_ip) = maybe_origin_ip_from_header(origin) else {
        return OriginGate::Invalid;
    };

    OriginGate::Parsed(origin_ip)
}

fn is_access_allowed(input: RouteAccessInput) -> bool {
    if input.ap_mode_enabled {
        return true;
    }

    if !is_private_ipv4(input.request_ip) {
        return false;
    }

    match input.origin {
        OriginGate::Missing => true,
        OriginGate::Parsed(origin_ip) => is_private_ipv4(origin_ip),
        OriginGate::Invalid => false,
    }
}

fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    let [first, second, _, _] = ip.octets();

    first == 10 || (first == 172 && (16..=31).contains(&second)) || (first == 192 && second == 168)
}

const fn unauthorized_response() -> PublicHttpResponse {
    PublicHttpResponse {
        status: 401,
        body: UNAUTHORIZED_BODY,
        content_type: Some(TEXT_PLAIN),
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::{
        maybe_origin_ip_from_header, origin_gate_from_header, phase05_routes, plan_http_access,
        plan_settings_patch_body_size, plan_websocket_upgrade, unknown_api_route_response,
        HttpAccessDecision, OriginGate, RouteAccessInput, RouteKind, RouteMethod,
        SettingsPatchBodyDecision, WebSocketRouteKind, WebSocketUpgradeDecision,
        MAX_SETTINGS_PATCH_BODY_BYTES, UNAUTHORIZED_BODY, UNKNOWN_API_ROUTE_BODY,
    };

    fn denied_public_client_input() -> RouteAccessInput {
        RouteAccessInput {
            ap_mode_enabled: false,
            request_ip: Ipv4Addr::new(8, 8, 8, 8),
            origin: OriginGate::Parsed(Ipv4Addr::new(203, 0, 113, 10)),
        }
    }

    fn private_client_input(origin: OriginGate) -> RouteAccessInput {
        RouteAccessInput {
            ap_mode_enabled: false,
            request_ip: Ipv4Addr::new(192, 168, 1, 25),
            origin,
        }
    }

    #[test]
    fn route_shell_lists_all_phase05_http_and_websocket_routes() {
        // Arrange
        let expected = [
            ("/api/system/info", RouteMethod::Get),
            ("/api/system", RouteMethod::Patch),
            ("/api/system/logs", RouteMethod::Get),
            ("/api/system/asic", RouteMethod::Get),
            ("/api/system/statistics", RouteMethod::Get),
            ("/api/system/scoreboard", RouteMethod::Get),
            ("/api/system/pause", RouteMethod::Post),
            ("/api/system/resume", RouteMethod::Post),
            ("/api/system/restart", RouteMethod::Post),
            ("/api/system/identify", RouteMethod::Post),
            ("/api/system/blockFound/dismiss", RouteMethod::Post),
            ("/api/system/OTA", RouteMethod::Post),
            ("/api/system/OTAWWW", RouteMethod::Post),
            ("/api/ws", RouteMethod::Get),
            ("/api/ws/live", RouteMethod::Get),
        ];

        // Act
        let routes = phase05_routes();

        // Assert
        for (path, method) in expected {
            assert!(
                routes
                    .iter()
                    .any(|route| route.path == path && route.method == method),
                "missing route {method:?} {path}"
            );
        }
        assert!(routes
            .iter()
            .any(|route| matches!(route.kind, RouteKind::SafeUnsupportedUpdate)));
    }

    #[test]
    fn http_access_gate_denies_public_request_with_generic_unauthorized_body() {
        // Arrange
        let input = denied_public_client_input();

        // Act
        let decision = plan_http_access(input);

        // Assert
        let HttpAccessDecision::Deny(response) = decision else {
            panic!("public client must be denied");
        };
        assert_eq!(response.status, 401);
        assert_eq!(response.body, UNAUTHORIZED_BODY);
        assert!(!response.body.contains("private"));
        assert!(!response.body.contains("origin"));
        assert!(!response.body.contains("8.8.8.8"));
    }

    #[test]
    fn websocket_access_gate_denies_upgrade_without_registration_plan() {
        // Arrange
        let input = denied_public_client_input();

        // Act
        let decision = plan_websocket_upgrade(input, WebSocketRouteKind::LiveTelemetry);

        // Assert
        let WebSocketUpgradeDecision::Reject(response) = decision else {
            panic!("denied upgrade must not return a registration plan");
        };
        assert_eq!(response.status, 401);
        assert_eq!(response.body, UNAUTHORIZED_BODY);
        assert!(!response.body.contains("websocket"));
        assert!(!response.body.contains("client"));
        assert!(!response.body.contains("origin"));
    }

    #[test]
    fn private_origin_and_request_ip_are_allowed_for_http_and_websocket() {
        // Arrange
        let input = RouteAccessInput {
            ap_mode_enabled: false,
            request_ip: Ipv4Addr::new(192, 168, 1, 25),
            origin: OriginGate::Parsed(Ipv4Addr::new(192, 168, 1, 2)),
        };

        // Act
        let http_decision = plan_http_access(input);
        let ws_decision = plan_websocket_upgrade(input, WebSocketRouteKind::Logs);

        // Assert
        assert_eq!(http_decision, HttpAccessDecision::Allow);
        assert_eq!(
            ws_decision,
            WebSocketUpgradeDecision::Accept(super::WebSocketClientRegistrationPlan {
                route: WebSocketRouteKind::Logs,
            })
        );
    }

    #[test]
    fn missing_origin_from_private_client_is_allowed_for_http_and_websocket() {
        // Arrange
        let input = private_client_input(OriginGate::Missing);

        // Act
        let http_decision = plan_http_access(input);
        let ws_decision = plan_websocket_upgrade(input, WebSocketRouteKind::LiveTelemetry);

        // Assert
        assert_eq!(http_decision, HttpAccessDecision::Allow);
        assert!(matches!(
            ws_decision,
            WebSocketUpgradeDecision::Accept(super::WebSocketClientRegistrationPlan {
                route: WebSocketRouteKind::LiveTelemetry
            })
        ));
    }

    #[test]
    fn public_named_origin_is_denied_for_http_and_websocket() {
        // Arrange
        let input = private_client_input(origin_gate_from_header("https://example.com"));

        // Act
        let http_decision = plan_http_access(input);
        let ws_decision = plan_websocket_upgrade(input, WebSocketRouteKind::Logs);

        // Assert
        assert!(matches!(http_decision, HttpAccessDecision::Deny(_)));
        assert!(matches!(ws_decision, WebSocketUpgradeDecision::Reject(_)));
    }

    #[test]
    fn public_ipv4_origin_is_denied_for_http_and_websocket() {
        // Arrange
        let input = private_client_input(origin_gate_from_header("https://203.0.113.10/dashboard"));

        // Act
        let http_decision = plan_http_access(input);
        let ws_decision = plan_websocket_upgrade(input, WebSocketRouteKind::Logs);

        // Assert
        assert!(matches!(http_decision, HttpAccessDecision::Deny(_)));
        assert!(matches!(ws_decision, WebSocketUpgradeDecision::Reject(_)));
    }

    #[test]
    fn invalid_or_overlong_origin_is_denied_for_http_and_websocket() {
        // Arrange
        let input = private_client_input(OriginGate::Invalid);

        // Act
        let http_decision = plan_http_access(input);
        let ws_decision = plan_websocket_upgrade(input, WebSocketRouteKind::LiveTelemetry);

        // Assert
        assert!(matches!(http_decision, HttpAccessDecision::Deny(_)));
        assert!(matches!(ws_decision, WebSocketUpgradeDecision::Reject(_)));
    }

    #[test]
    fn unknown_api_routes_map_to_json_404_body() {
        // Arrange
        let expected_body = UNKNOWN_API_ROUTE_BODY;

        // Act
        let response = unknown_api_route_response();

        // Assert
        assert_eq!(response.status, 404);
        assert_eq!(response.body, expected_body);
        assert_eq!(response.content_type, Some("application/json"));
    }

    #[test]
    fn origin_header_parser_accepts_ipv4_literal_hosts_without_rich_url_dependency() {
        // Arrange
        let origin = "http://192.168.1.2:8080/dashboard";

        // Act
        let maybe_origin_ip = maybe_origin_ip_from_header(origin);

        // Assert
        assert_eq!(maybe_origin_ip, Some(Ipv4Addr::new(192, 168, 1, 2)));
    }

    #[test]
    fn origin_gate_marks_non_ipv4_header_hosts_invalid() {
        // Arrange
        let origin = "https://example.com";

        // Act
        let gate = origin_gate_from_header(origin);

        // Assert
        assert_eq!(gate, OriginGate::Invalid);
    }

    #[test]
    fn settings_patch_body_cap_rejects_oversized_body_before_json_parse() {
        // Arrange
        let oversized_len = MAX_SETTINGS_PATCH_BODY_BYTES + 1;

        // Act
        let decision = plan_settings_patch_body_size(oversized_len);

        // Assert
        let SettingsPatchBodyDecision::Reject(response) = decision else {
            panic!("oversized settings PATCH body must be rejected before parsing");
        };
        assert_eq!(response.status, 400);
        assert_eq!(response.body, "Wrong API input");
        assert_eq!(response.content_type, Some("text/plain"));
        assert!(!response.body.contains("Invalid JSON"));
        assert!(!response.body.contains("content too long"));
        assert!(!response.body.contains(&oversized_len.to_string()));
    }

    #[test]
    fn settings_patch_body_cap_rejection_performs_zero_parser_or_persistence_calls() {
        // Arrange
        let body = "{".repeat(MAX_SETTINGS_PATCH_BODY_BYTES + 1);
        let mut counters = SettingsPatchPipelineCounters::default();

        // Act
        let response = run_counted_settings_patch_pipeline(&body, &mut counters);

        // Assert
        assert_eq!(response.status, 400);
        assert_eq!(response.body, "Wrong API input");
        assert_eq!(counters.parser_calls, 0);
        assert_eq!(counters.writes, 0);
        assert_eq!(counters.commits, 0);
        assert_eq!(counters.reloads, 0);
        assert!(!response.body.contains("parser"));
        assert!(!response.body.contains("size"));
        assert!(!response.body.contains("field"));
        assert!(!response.body.contains("adapter"));
    }

    #[derive(Default)]
    struct SettingsPatchPipelineCounters {
        parser_calls: usize,
        writes: usize,
        commits: usize,
        reloads: usize,
    }

    fn run_counted_settings_patch_pipeline(
        body: &str,
        counters: &mut SettingsPatchPipelineCounters,
    ) -> super::PublicHttpResponse {
        match plan_settings_patch_body_size(body.len()) {
            SettingsPatchBodyDecision::Accept => {
                counters.parser_calls += 1;
                counters.writes += 1;
                counters.commits += 1;
                counters.reloads += 1;
                super::PublicHttpResponse {
                    status: 200,
                    body: "",
                    content_type: None,
                }
            }
            SettingsPatchBodyDecision::Reject(response) => response,
        }
    }
}
