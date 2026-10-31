use std::net::{Ipv4Addr, Ipv6Addr};

use super::super::{
    maybe_mapped_peer_ipv4, plan_http_access, plan_websocket_upgrade, HttpAccessDecision,
    OriginGate, RouteAccessInput, WebSocketRouteKind, WebSocketUpgradeDecision,
};

fn input_from_peer_octets(peer_octets: [u8; 16], origin: OriginGate) -> RouteAccessInput {
    RouteAccessInput {
        ap_mode_enabled: false,
        maybe_request_ip: maybe_mapped_peer_ipv4(peer_octets),
        origin,
    }
}

fn assert_http_and_websocket_denied(input: RouteAccessInput) {
    assert!(matches!(
        plan_http_access(input),
        HttpAccessDecision::Deny(_)
    ));
    assert!(matches!(
        plan_websocket_upgrade(input, WebSocketRouteKind::LiveTelemetry),
        WebSocketUpgradeDecision::Reject(_)
    ));
}

fn assert_http_and_websocket_allowed(input: RouteAccessInput) {
    assert_eq!(plan_http_access(input), HttpAccessDecision::Allow);
    assert!(matches!(
        plan_websocket_upgrade(input, WebSocketRouteKind::LiveTelemetry),
        WebSocketUpgradeDecision::Accept(_)
    ));
}

#[test]
fn mapped_peer_address_yields_its_ipv4_address() {
    // Arrange
    let address = Ipv4Addr::new(192, 168, 1, 25);

    // Act
    let maybe_peer = maybe_mapped_peer_ipv4(address.to_ipv6_mapped().octets());

    // Assert
    assert_eq!(maybe_peer, Some(address));
}

#[test]
fn peer_that_is_not_ipv4_mapped_has_no_ipv4_address() {
    // Arrange
    let peers: [Ipv6Addr; 6] = [
        Ipv6Addr::UNSPECIFIED,
        Ipv6Addr::LOCALHOST,
        "fe80::1".parse().expect("link-local literal"),
        "fd00::1".parse().expect("unique local literal"),
        "2001:db8::1".parse().expect("global literal"),
        // IPv4-compatible, the form `Ipv6Addr::to_ipv4` would wrongly accept.
        Ipv4Addr::new(192, 168, 1, 25).to_ipv6_compatible(),
    ];

    for peer in peers {
        // Act
        let maybe_peer = maybe_mapped_peer_ipv4(peer.octets());

        // Assert
        assert_eq!(maybe_peer, None, "{peer}");
    }
}

#[test]
fn private_mapped_peer_is_allowed_without_origin() {
    // Arrange
    let peer = Ipv4Addr::new(192, 168, 1, 25).to_ipv6_mapped().octets();

    // Act
    let input = input_from_peer_octets(peer, OriginGate::Missing);

    // Assert
    assert_http_and_websocket_allowed(input);
}

#[test]
fn private_mapped_peer_is_allowed_with_private_origin() {
    // Arrange
    let peer = Ipv4Addr::new(10, 0, 0, 7).to_ipv6_mapped().octets();
    let origin = OriginGate::Parsed(Ipv4Addr::new(172, 16, 0, 2));

    // Act
    let input = input_from_peer_octets(peer, origin);

    // Assert
    assert_http_and_websocket_allowed(input);
}

#[test]
fn private_mapped_peer_is_denied_with_public_origin() {
    // Arrange
    let peer = Ipv4Addr::new(192, 168, 1, 25).to_ipv6_mapped().octets();
    let origin = OriginGate::Parsed(Ipv4Addr::new(8, 8, 8, 8));

    // Act
    let input = input_from_peer_octets(peer, origin);

    // Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn public_mapped_peer_is_denied_without_origin() {
    // Arrange
    let peer = Ipv4Addr::new(8, 8, 8, 8).to_ipv6_mapped().octets();

    // Act
    let input = input_from_peer_octets(peer, OriginGate::Missing);

    // Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn public_mapped_peer_is_denied_with_private_origin() {
    // Arrange
    let peer = Ipv4Addr::new(8, 8, 8, 8).to_ipv6_mapped().octets();
    let origin = OriginGate::Parsed(Ipv4Addr::new(192, 168, 1, 2));

    // Act
    let input = input_from_peer_octets(peer, origin);

    // Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn public_peer_whose_reversed_octets_are_private_is_denied() {
    // Arrange
    let peer = Ipv4Addr::new(1, 2, 168, 192).to_ipv6_mapped().octets();

    // Act
    let input = input_from_peer_octets(peer, OriginGate::Missing);

    // Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn real_ipv6_peer_is_denied_even_with_private_origin() {
    // Arrange
    let peer: Ipv6Addr = "fe80::1".parse().expect("link-local literal");
    let origin = OriginGate::Parsed(Ipv4Addr::new(192, 168, 1, 2));

    // Act
    let input = input_from_peer_octets(peer.octets(), origin);

    // Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn unreadable_peer_is_denied_without_origin() {
    // Arrange
    let input = RouteAccessInput {
        ap_mode_enabled: false,
        maybe_request_ip: None,
        origin: OriginGate::Missing,
    };

    // Act / Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn unreadable_peer_is_denied_with_private_origin() {
    // Arrange
    let input = RouteAccessInput {
        ap_mode_enabled: false,
        maybe_request_ip: None,
        origin: OriginGate::Parsed(Ipv4Addr::new(192, 168, 1, 2)),
    };

    // Act / Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn zero_peer_address_is_denied_with_private_origin() {
    // Arrange
    let peer = Ipv4Addr::UNSPECIFIED.to_ipv6_mapped().octets();
    let origin = OriginGate::Parsed(Ipv4Addr::new(192, 168, 1, 2));

    // Act
    let input = input_from_peer_octets(peer, origin);

    // Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn zero_peer_address_is_denied_without_origin() {
    // Arrange
    let peer = Ipv4Addr::UNSPECIFIED.to_ipv6_mapped().octets();

    // Act
    let input = input_from_peer_octets(peer, OriginGate::Missing);

    // Assert
    assert_http_and_websocket_denied(input);
}

#[test]
fn unreadable_peer_keeps_the_ap_mode_allowance() {
    // Arrange
    let input = RouteAccessInput {
        ap_mode_enabled: true,
        maybe_request_ip: None,
        origin: OriginGate::Missing,
    };

    // Act / Assert
    assert_http_and_websocket_allowed(input);
}
