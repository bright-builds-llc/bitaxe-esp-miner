//! Real allocation failures inside the running virtual device: the shared receiver's
//! frame and control copies and the shared writer's encode fail typed, the device ends
//! the session instead of aborting, and PSRAM exhaustion alone falls back to internal.

use super::*;
use crate::real_allocation::{arm, Exhausted};
use bitaxe_worker_control::serial::link::CONTROL_COPY_SITE;
use bitaxe_worker_control::serial::{ENCODE_SITE, FRAME_COPY_SITE};

#[test]
fn an_exhausted_control_copy_ends_the_session_typed() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    let armed = arm(CONTROL_COPY_SITE, Exhausted::HEAP);
    // Act
    host.command("status");
    // Assert
    let attempts = armed.attempts();
    drop(armed);
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        host.device.outcome().maybe_link_failure.as_deref(),
        Some("control_allocation")
    );
    assert_eq!(
        host.replies("serial_status").len(),
        1,
        "only the admission status"
    );
}

#[test]
fn an_exhausted_frame_copy_ends_the_session_typed() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    let armed = arm(FRAME_COPY_SITE, Exhausted::HEAP);
    // Act
    host.command("status");
    // Assert
    drop(armed);
    assert_eq!(
        host.device.outcome().maybe_link_failure.as_deref(),
        Some("frame_allocation")
    );
}

#[test]
fn an_exhausted_reply_encode_revokes_without_a_reply() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    // The host encodes on this thread too, so the request is encoded before arming.
    let request = json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_status","command":"status"});
    let record = host.wire.borrow_mut().record(SerialKind::Control, &request);
    let armed = arm(ENCODE_SITE, Exhausted::HEAP);
    // Act
    host.step(&record);
    // Assert
    let attempts = armed.attempts();
    drop(armed);
    assert!(!attempts.is_empty());
    let outcome = host.device.outcome();
    assert!(outcome.maybe_output_failure.is_some(), "{outcome:?}");
    assert_eq!(
        host.replies("serial_status").len(),
        1,
        "only the admission status"
    );
}

#[test]
fn psram_exhaustion_alone_keeps_the_session() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    let armed = arm(CONTROL_COPY_SITE, Exhausted::PSRAM);
    // Act
    host.command("status");
    // Assert
    let attempts = armed.attempts();
    drop(armed);
    assert_eq!(
        attempts[0].maybe_served_by,
        Some(bitaxe_virtual_board::memory::Capability::Internal)
    );
    assert_eq!(host.replies("serial_status").len(), 2);
    assert_eq!(host.device.outcome().maybe_link_failure, None);
}

#[test]
fn the_device_closes_cleanly_after_an_allocation_failure() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    let armed = arm(CONTROL_COPY_SITE, Exhausted::HEAP);
    host.command("status");
    drop(armed);
    // Act
    host.device.input_closed(host.now);
    // Assert
    let outcome = host.device.outcome();
    assert!(outcome.hello_acknowledged && outcome.authenticated);
    assert!(!outcome.lease_pending);
}
