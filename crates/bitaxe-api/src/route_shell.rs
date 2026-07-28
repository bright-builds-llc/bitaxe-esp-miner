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

macro_rules! axeos_route {
    ($path:literal, $method:ident, $kind:expr) => {
        AxeosRoute {
            path: $path,
            method: RouteMethod::$method,
            kind: $kind,
        }
    };
}

const PHASE05_ROUTES: &[AxeosRoute] = &[
    axeos_route!("/api/system/info", Get, RouteKind::Http),
    axeos_route!("/api/system", Patch, RouteKind::Http),
    axeos_route!("/api/system/logs", Get, RouteKind::Http),
    axeos_route!("/api/system/asic", Get, RouteKind::Http),
    axeos_route!("/api/system/statistics", Get, RouteKind::Http),
    axeos_route!("/api/system/scoreboard", Get, RouteKind::Http),
    axeos_route!("/api/system/pause", Post, RouteKind::Http),
    axeos_route!("/api/system/resume", Post, RouteKind::Http),
    axeos_route!("/api/system/restart", Post, RouteKind::Http),
    axeos_route!("/api/system/identify", Post, RouteKind::Http),
    axeos_route!("/api/system/blockFound/dismiss", Post, RouteKind::Http),
    axeos_route!("/api/system/OTA", Post, RouteKind::SafeUnsupportedUpdate),
    axeos_route!("/api/system/OTAWWW", Post, RouteKind::SafeUnsupportedUpdate),
    axeos_route!(
        "/api/ws",
        Get,
        RouteKind::WebSocket(WebSocketRouteKind::Logs)
    ),
    axeos_route!(
        "/api/ws/live",
        Get,
        RouteKind::WebSocket(WebSocketRouteKind::LiveTelemetry)
    ),
];

const PHASE07_ROUTES: &[AxeosRoute] = &[
    axeos_route!("/api/system/info", Get, RouteKind::Http),
    axeos_route!("/api/system", Patch, RouteKind::Http),
    axeos_route!("/api/system/logs", Get, RouteKind::Http),
    axeos_route!("/api/system/asic", Get, RouteKind::Http),
    axeos_route!("/api/system/statistics", Get, RouteKind::Http),
    axeos_route!("/api/system/scoreboard", Get, RouteKind::Http),
    axeos_route!("/api/system/pause", Post, RouteKind::Http),
    axeos_route!("/api/system/resume", Post, RouteKind::Http),
    axeos_route!("/api/system/restart", Post, RouteKind::Http),
    axeos_route!("/api/system/identify", Post, RouteKind::Http),
    axeos_route!("/api/system/blockFound/dismiss", Post, RouteKind::Http),
    axeos_route!("/api/system/OTA", Post, RouteKind::FirmwareUpdate),
    axeos_route!("/api/system/OTAWWW", Post, RouteKind::AxeOsStaticUpdateGap),
    axeos_route!(
        "/api/ws",
        Get,
        RouteKind::WebSocket(WebSocketRouteKind::Logs)
    ),
    axeos_route!(
        "/api/ws/live",
        Get,
        RouteKind::WebSocket(WebSocketRouteKind::LiveTelemetry)
    ),
    axeos_route!("/recovery", Get, RouteKind::Recovery),
    axeos_route!("/*", Get, RouteKind::StaticFiles),
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
    /// Phase 7 firmware OTA route owner.
    FirmwareUpdate,
    /// Phase 7 AxeOS static OTAWWW gap owner.
    AxeOsStaticUpdateGap,
    /// Phase 7 embedded recovery route owner.
    Recovery,
    /// Phase 7 static wildcard route owner.
    StaticFiles,
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

/// Manifest-derived Phase 7 route ownership counts for firmware startup logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phase07RouteReport {
    /// Number of routes declared in the Phase 7 route manifest.
    pub total_routes: usize,
    /// Number of firmware OTA routes declared in the Phase 7 route manifest.
    pub firmware_update_routes: usize,
    /// Number of OTAWWW static update gap routes declared in the Phase 7 route manifest.
    pub otawww_gap_routes: usize,
    /// Number of recovery routes declared in the Phase 7 route manifest.
    pub recovery_routes: usize,
    /// Number of static file wildcard routes declared in the Phase 7 route manifest.
    pub static_file_routes: usize,
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
    /// Client peer IPv4 address, or `0.0.0.0` when the firmware platform cannot expose it.
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

/// Result of normalizing the raw ESP-IDF peer IPv4 value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerIpv4Normalization {
    /// The raw value decoded as a network-order address.
    NetworkOrder(Ipv4Addr),
    /// The network-order candidate was public while the host-order candidate
    /// was RFC1918, so the private host-order candidate was selected.
    HostOrderFallback {
        /// Selected RFC1918 peer address.
        address: Ipv4Addr,
        /// Rejected public network-order candidate retained for diagnostics.
        network_order_address: Ipv4Addr,
    },
}

impl PeerIpv4Normalization {
    /// Returns the normalized peer IPv4 address.
    #[must_use]
    pub const fn address(self) -> Ipv4Addr {
        match self {
            Self::NetworkOrder(address) | Self::HostOrderFallback { address, .. } => address,
        }
    }
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

/// Returns Phase 7 firmware API, update, recovery, and static routes.
#[must_use]
pub const fn phase07_routes() -> &'static [AxeosRoute] {
    PHASE07_ROUTES
}

/// Returns manifest-derived Phase 7 route ownership counts.
#[must_use]
pub fn phase07_route_report() -> Phase07RouteReport {
    let mut report = Phase07RouteReport {
        total_routes: phase07_routes().len(),
        firmware_update_routes: 0,
        otawww_gap_routes: 0,
        recovery_routes: 0,
        static_file_routes: 0,
    };

    for route in phase07_routes() {
        match route.kind {
            RouteKind::FirmwareUpdate => report.firmware_update_routes += 1,
            RouteKind::AxeOsStaticUpdateGap => report.otawww_gap_routes += 1,
            RouteKind::Recovery => report.recovery_routes += 1,
            RouteKind::StaticFiles => report.static_file_routes += 1,
            RouteKind::Http | RouteKind::SafeUnsupportedUpdate | RouteKind::WebSocket(_) => {}
        }
    }

    report
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

/// Normalizes an ESP-IDF `sockaddr_in.sin_addr.s_addr` value.
///
/// ESP-IDF supplies network-order bytes, but supported bindings have exposed
/// both network- and host-order integer interpretations. Prefer the
/// network-order candidate unless only the host-order candidate is RFC1918.
#[must_use]
pub fn normalize_peer_ipv4(raw_addr: u32) -> PeerIpv4Normalization {
    let network_order_address = Ipv4Addr::from(u32::from_be(raw_addr));
    if is_rfc1918_ipv4(network_order_address) {
        return PeerIpv4Normalization::NetworkOrder(network_order_address);
    }

    let host_order_address = Ipv4Addr::from(raw_addr);
    if is_rfc1918_ipv4(host_order_address) {
        return PeerIpv4Normalization::HostOrderFallback {
            address: host_order_address,
            network_order_address,
        };
    }

    PeerIpv4Normalization::NetworkOrder(network_order_address)
}

fn is_access_allowed(input: RouteAccessInput) -> bool {
    if input.ap_mode_enabled {
        return true;
    }

    // ESP-IDF HTTPD can report 0.0.0.0 for the peer after accepting a STA socket.
    // Keep upstream-like local curl behavior while still rejecting hostile origins.
    if input.request_ip.is_unspecified() {
        return is_origin_allowed(input.origin);
    }

    if !is_rfc1918_ipv4(input.request_ip) {
        return false;
    }

    is_origin_allowed(input.origin)
}

fn is_origin_allowed(origin: OriginGate) -> bool {
    match origin {
        OriginGate::Missing => true,
        OriginGate::Parsed(origin_ip) => is_rfc1918_ipv4(origin_ip),
        OriginGate::Invalid => false,
    }
}

fn is_rfc1918_ipv4(ip: Ipv4Addr) -> bool {
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
mod tests;
