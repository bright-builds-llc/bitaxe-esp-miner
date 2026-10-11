//! Leases on the real-time device: Start while the link stays live, renewal,
//! restoration, Close with an active lease, and grants the device must refuse.

use super::*;
use crate::authorization::{signed_renewal, signed_start};

impl Host {
    /// Admits, then starts the synthetic lease with the host heartbeating throughout.
    fn start_lease(&mut self) {
        self.admit();
        self.wire.borrow_mut().auto_heartbeats = true;
        let grant = signed_start(&self.binding, 1).expect("grant");
        self.request("start_lease", Some(grant));
    }

    fn renew_lease(&mut self) {
        let renewal = signed_renewal(&self.binding, 2).expect("renewal");
        self.request("renew_lease", Some(renewal));
    }

    fn ok(&self, request_id: &str) -> bool {
        self.reply(request_id)
            .is_some_and(|reply| reply["ok"] == true)
    }
}

#[test]
fn a_signed_start_runs_the_lease() {
    // Arrange
    let mut host = Host::new();
    // Act
    host.start_lease();
    // Assert
    let reply = host.reply("serial_start_lease").expect("start reply");
    assert_eq!(reply["result"]["state"], "mining", "{reply}");
    assert_eq!(host.device.outcome().leases_started, 1);
}

#[test]
fn the_link_stays_live_while_the_start_prepares() {
    // Arrange
    let mut host = Host::new();
    // Act
    host.start_lease();
    // Assert
    assert!(host.now > 3_000, "preparation took {} ms", host.now);
    assert_eq!(host.device.outcome().maybe_link_failure, None);
    let check = host.peer_check();
    assert_eq!(check.status, CheckStatus::Passed, "{}", check.detail);
}

#[test]
fn a_host_that_stops_heartbeating_during_preparation_loses_the_start() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    // Act
    let grant = signed_start(&host.binding, 1).expect("grant");
    host.request("start_lease", Some(grant));
    // Assert
    let outcome = host.device.outcome();
    assert!(!host.ok("serial_start_lease"));
    assert!(outcome.maybe_link_failure.is_some(), "{outcome:?}");
    assert!(!outcome.lease_pending, "{outcome:?}");
}

#[test]
fn running_status_reports_the_attempt_and_owner_resources() {
    // Arrange
    let mut host = Host::new();
    host.start_lease();
    // Act
    host.command("status");
    // Assert
    let evidence = host.reply("serial_status").expect("status")["result"]["qualification"].clone();
    assert_eq!(evidence["attempt"]["ordinal"], 1, "{evidence}");
    assert_eq!(evidence["owner_resources"]["phase"], "active");
    assert_eq!(
        evidence["owner_resources"]["generation"],
        evidence["generation"]
    );
    assert!(evidence["owner_resources"]["stack_free_bytes"].as_u64() >= Some(4096));
}

#[test]
fn a_renewal_extends_the_lease() {
    // Arrange
    let mut host = Host::new();
    host.start_lease();
    // Act
    host.renew_lease();
    // Assert
    let reply = host.reply("serial_renew_lease").expect("renew reply");
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(host.device.outcome().renewals, 1);
}

#[test]
fn restore_returns_the_device_to_baseline() {
    // Arrange
    let mut host = Host::new();
    host.start_lease();
    // Act
    host.request("restore", Some(json!({"reason":"cancelled"})));
    // Assert
    let reply = host.reply("serial_restore").expect("restore reply");
    assert_eq!(reply["result"]["state"], "baseline", "{reply}");
    let outcome = host.device.outcome();
    assert_eq!(outcome.restorations, 1);
    assert!(!outcome.lease_pending);
}

#[test]
fn close_with_an_active_lease_restores_and_is_clean() {
    // Arrange
    let mut host = Host::new();
    host.start_lease();
    // Act
    host.close();
    host.idle_until(host.now + 3_000);
    // Assert
    let outcome = host.device.outcome();
    assert!(outcome.clean(), "{outcome:?}");
    assert_eq!(outcome.restorations, 1);
    let check = host.peer_check();
    assert_eq!(check.status, CheckStatus::Passed, "{}", check.detail);
}

#[test]
fn a_close_during_preparation_ends_the_start_and_restores() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    let grant = signed_start(&host.binding, 1).expect("grant");
    let request = json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_start_lease",
        "command":"start_lease","payload":grant});
    let start = {
        let mut wire = host.wire.borrow_mut();
        let start = wire.record(SerialKind::Control, &request);
        // The Close follows the Start on the wire and arrives while preparation runs;
        // the Gate sends nothing after it.
        wire.auto_heartbeats = false;
        let close = wire.record(
            SerialKind::Session,
            &json!({"op":"close","reason":"tab_closed"}),
        );
        wire.queued.push_back(close);
        start
    };
    // Act
    host.step(&start);
    host.idle_until(host.now + 3_000);
    // Assert
    let outcome = host.device.outcome();
    assert!(!host.ok("serial_start_lease"));
    assert_eq!((outcome.leases_started, outcome.restorations), (0, 1));
    assert!(!outcome.lease_pending, "{outcome:?}");
    assert_eq!(outcome.maybe_owner_failure, None);
}

#[test]
fn a_start_from_an_untrusted_authority_is_refused() {
    // Arrange
    let mut trust: Value =
        serde_json::from_str(&crate::authorization::synthetic_trust_json().expect("trust"))
            .expect("trust json");
    let other = ed25519_dalek::SigningKey::from_bytes(&[13; 32]);
    trust["workLeaseAuthority"]["keys"][0]["x"] = URL_SAFE_NO_PAD
        .encode(other.verifying_key().as_bytes())
        .into();
    let mut host = Host::with_trust(trust.to_string());
    // Act
    host.start_lease();
    // Assert
    assert!(!host.ok("serial_start_lease"));
    assert_eq!(host.device.outcome().leases_started, 0);
}

#[test]
fn a_rejected_restore_is_answered_and_ends_the_session() {
    // Arrange
    let mut host = Host::with_fault(StdioFault::RejectRestore);
    host.start_lease();
    // Act
    host.request("restore", Some(json!({"reason":"tab_closed"})));
    // Assert
    let reply = host.reply("serial_restore").expect("restore reply");
    assert_eq!(reply["ok"], false, "{reply}");
    assert_eq!(host.device.outcome().rejected_restorations, 1);
}

#[test]
fn after_a_rejected_restore_the_device_restores_without_the_host() {
    // Arrange
    let mut host = Host::with_fault(StdioFault::RejectRestore);
    host.start_lease();
    // Act
    host.request("restore", Some(json!({"reason":"tab_closed"})));
    host.idle_until(host.now + 1_000);
    // Assert
    let outcome = host.device.outcome();
    assert_eq!(
        (outcome.rejected_restorations, outcome.restorations),
        (1, 1)
    );
    assert!(!outcome.lease_pending, "{outcome:?}");
    assert!(!outcome.clean(), "a rejected restore is never clean");
}
