use std::net::Ipv4Addr;

use super::{
    is_rfc1918_ipv4, maybe_origin_ip_from_header, normalize_peer_ipv4, origin_gate_from_header,
    phase05_routes, phase07_route_report, phase07_routes, plan_http_access,
    plan_settings_patch_body_size, plan_websocket_upgrade, unknown_api_route_response,
    HttpAccessDecision, OriginGate, PeerIpv4Normalization, RouteAccessInput, RouteKind,
    RouteMethod, SettingsPatchBodyDecision, WebSocketRouteKind, WebSocketUpgradeDecision,
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

fn network_order_raw(address: Ipv4Addr) -> u32 {
    u32::from(address).to_be()
}

#[test]
fn peer_ipv4_normalization_decodes_network_order_private_address() {
    // Arrange
    let address = Ipv4Addr::new(192, 168, 1, 25);

    // Act
    let normalized = normalize_peer_ipv4(network_order_raw(address));

    // Assert
    assert_eq!(normalized, PeerIpv4Normalization::NetworkOrder(address));
    assert_eq!(normalized.address(), address);
}

#[test]
fn peer_ipv4_normalization_falls_back_to_host_order_private_address() {
    // Arrange
    let address = Ipv4Addr::new(192, 168, 1, 25);
    let raw_addr = u32::from(address);
    let network_order_address = Ipv4Addr::from(u32::from_be(raw_addr));

    // Act
    let normalized = normalize_peer_ipv4(raw_addr);

    // Assert
    assert_eq!(
        normalized,
        PeerIpv4Normalization::HostOrderFallback {
            address,
            network_order_address,
        }
    );
    assert_eq!(normalized.address(), address);
}

#[test]
fn peer_ipv4_normalization_preserves_unspecified_address() {
    // Arrange
    let raw_addr = network_order_raw(Ipv4Addr::UNSPECIFIED);

    // Act
    let normalized = normalize_peer_ipv4(raw_addr);

    // Assert
    assert_eq!(
        normalized,
        PeerIpv4Normalization::NetworkOrder(Ipv4Addr::UNSPECIFIED)
    );
}

#[test]
fn peer_ipv4_normalization_preserves_network_order_public_address() {
    // Arrange
    let address = Ipv4Addr::new(8, 8, 8, 8);

    // Act
    let normalized = normalize_peer_ipv4(network_order_raw(address));

    // Assert
    assert_eq!(normalized, PeerIpv4Normalization::NetworkOrder(address));
}

#[test]
fn rfc1918_classifier_accepts_all_three_private_ranges() {
    // Arrange
    let private_addresses = [
        Ipv4Addr::new(10, 0, 0, 1),
        Ipv4Addr::new(172, 16, 0, 1),
        Ipv4Addr::new(172, 31, 255, 254),
        Ipv4Addr::new(192, 168, 1, 1),
    ];

    // Act
    let classifications = private_addresses.map(is_rfc1918_ipv4);

    // Assert
    assert_eq!(classifications, [true, true, true, true]);
}

#[test]
fn rfc1918_classifier_rejects_public_and_adjacent_addresses() {
    // Arrange
    let non_private_addresses = [
        Ipv4Addr::new(8, 8, 8, 8),
        Ipv4Addr::new(172, 15, 255, 255),
        Ipv4Addr::new(172, 32, 0, 0),
        Ipv4Addr::new(192, 167, 255, 255),
    ];

    // Act
    let classifications = non_private_addresses.map(is_rfc1918_ipv4);

    // Assert
    assert_eq!(classifications, [false, false, false, false]);
}

#[test]
fn route_shell_lists_all_phase05_http_and_websocket_routes() {
    // Arrange
    let expected = [
        ("/api/system/info", RouteMethod::Get),
        ("/api/system/wifi/scan", RouteMethod::Get),
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
fn theme_get_and_post_are_owned_http_routes_in_both_manifests() {
    // Arrange
    let expected = [
        ("/api/theme", RouteMethod::Get),
        ("/api/theme", RouteMethod::Post),
    ];

    // Act and assert
    for routes in [super::phase05_routes(), super::phase07_routes()] {
        for (path, method) in expected {
            let route = routes
                .iter()
                .find(|route| route.path == path && route.method == method)
                .expect("theme route must be present");
            assert_eq!(route.kind, RouteKind::Http);
        }
    }
}

#[test]
fn command_status_is_a_phase07_read_only_extension() {
    // Arrange
    let phase05 = super::phase05_routes();

    // Act
    let maybe_route = super::phase07_routes()
        .iter()
        .find(|route| route.path == "/api/system/command-status");

    // Assert
    assert!(phase05
        .iter()
        .all(|route| route.path != "/api/system/command-status"));
    let route = maybe_route.expect("command status extension must be registered");
    assert_eq!(route.method, RouteMethod::Get);
    assert_eq!(route.kind, RouteKind::Http);
}

#[test]
fn phase05_update_routes_keep_safe_unsupported_owner_for_api_compare() {
    // Arrange
    let routes = phase05_routes();

    // Act
    let firmware_ota = routes
        .iter()
        .find(|route| route.path == "/api/system/OTA")
        .expect("Phase 05 route manifest should include firmware OTA");
    let otawww = routes
        .iter()
        .find(|route| route.path == "/api/system/OTAWWW")
        .expect("Phase 05 route manifest should include OTAWWW");

    // Assert
    assert_eq!(firmware_ota.kind, RouteKind::SafeUnsupportedUpdate);
    assert_eq!(otawww.kind, RouteKind::SafeUnsupportedUpdate);
}

#[test]
fn phase07_routes_assign_update_recovery_and_static_owners() {
    // Arrange
    let routes = phase07_routes();

    // Act
    let firmware_ota = routes
        .iter()
        .find(|route| route.path == "/api/system/OTA")
        .expect("Phase 7 route manifest should include firmware OTA");
    let otawww = routes
        .iter()
        .find(|route| route.path == "/api/system/OTAWWW")
        .expect("Phase 7 route manifest should include OTAWWW");
    let recovery = routes
        .iter()
        .find(|route| route.path == "/recovery")
        .expect("Phase 7 route manifest should include recovery");
    let static_files = routes
        .iter()
        .find(|route| route.path == "/*")
        .expect("Phase 7 route manifest should include static wildcard");

    // Assert
    assert_eq!(firmware_ota.method, RouteMethod::Post);
    assert_eq!(firmware_ota.kind, RouteKind::FirmwareUpdate);
    assert_eq!(otawww.method, RouteMethod::Post);
    assert_eq!(otawww.kind, RouteKind::AxeOsStaticUpdateGap);
    assert_eq!(recovery.method, RouteMethod::Get);
    assert_eq!(recovery.kind, RouteKind::Recovery);
    assert_eq!(static_files.method, RouteMethod::Get);
    assert_eq!(static_files.kind, RouteKind::StaticFiles);
    assert_ne!(firmware_ota.kind, RouteKind::SafeUnsupportedUpdate);
    assert_ne!(otawww.kind, RouteKind::SafeUnsupportedUpdate);
}

#[test]
fn phase07_route_report_counts_manifest_and_phase7_owned_routes() {
    // Arrange
    let routes = phase07_routes();

    // Act
    let report = phase07_route_report();

    // Assert
    assert_eq!(report.total_routes, routes.len());
    assert_eq!(report.firmware_update_routes, 1);
    assert_eq!(report.otawww_gap_routes, 1);
    assert_eq!(report.recovery_routes, 1);
    assert_eq!(report.static_file_routes, 1);
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
fn unspecified_peer_ip_from_firmware_allows_missing_origin_for_http_and_websocket() {
    // Arrange
    let input = RouteAccessInput {
        ap_mode_enabled: false,
        request_ip: Ipv4Addr::UNSPECIFIED,
        origin: OriginGate::Missing,
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
fn unspecified_peer_ip_from_firmware_still_applies_origin_gate() {
    // Arrange
    let private_origin_input = RouteAccessInput {
        ap_mode_enabled: false,
        request_ip: Ipv4Addr::UNSPECIFIED,
        origin: OriginGate::Parsed(Ipv4Addr::new(192, 168, 86, 1)),
    };
    let public_origin_input = RouteAccessInput {
        ap_mode_enabled: false,
        request_ip: Ipv4Addr::UNSPECIFIED,
        origin: OriginGate::Parsed(Ipv4Addr::new(203, 0, 113, 10)),
    };
    let invalid_origin_input = RouteAccessInput {
        ap_mode_enabled: false,
        request_ip: Ipv4Addr::UNSPECIFIED,
        origin: OriginGate::Invalid,
    };

    // Act
    let private_origin_decision = plan_http_access(private_origin_input);
    let public_origin_decision = plan_http_access(public_origin_input);
    let invalid_origin_decision =
        plan_websocket_upgrade(invalid_origin_input, WebSocketRouteKind::LiveTelemetry);

    // Assert
    assert_eq!(private_origin_decision, HttpAccessDecision::Allow);
    assert!(matches!(
        public_origin_decision,
        HttpAccessDecision::Deny(_)
    ));
    assert!(matches!(
        invalid_origin_decision,
        WebSocketUpgradeDecision::Reject(_)
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
