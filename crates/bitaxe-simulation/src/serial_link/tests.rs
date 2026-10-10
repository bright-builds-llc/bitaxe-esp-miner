use super::*;
use bitaxe_virtual_board::{BoardConfig, VirtualBoard};
use std::rc::Rc;

fn actuation(seed: u64) -> RuntimeBoardAdapter {
    let board = Rc::new(RefCell::new(
        VirtualBoard::new(BoardConfig {
            seed,
            ..BoardConfig::default()
        })
        .expect("board"),
    ));
    RuntimeBoardAdapter::new(board, "healthy-lifecycle").expect("adapter")
}

fn admitted(seed: u64) -> (SerialHost, RuntimeBoardAdapter) {
    let mut actuation = actuation(seed);
    let mut host = SerialHost::new(seed);
    host.hello(&mut actuation).expect("hello admitted");
    host.authenticate();
    (host, actuation)
}

#[test]
fn hello_binding_repeats_within_a_seed() {
    // Arrange
    let mut first = actuation(19);
    let mut second = actuation(19);
    // Act
    let a = SerialHost::new(19).hello(&mut first).expect("hello");
    let b = SerialHost::new(19).hello(&mut second).expect("hello");
    // Assert
    assert_eq!(
        (a.session_id, a.host_nonce, a.device_nonce),
        (b.session_id, b.host_nonce, b.device_nonce)
    );
}

#[test]
fn hello_binding_differs_across_seeds() {
    // Arrange
    let mut first = actuation(1);
    let mut second = actuation(205);
    // Act
    let a = SerialHost::new(1).hello(&mut first).expect("hello");
    let b = SerialHost::new(205).hello(&mut second).expect("hello");
    // Assert
    assert_ne!(a.session_id, b.session_id);
}

#[test]
fn delivered_control_reaches_the_owner_with_one_lf() {
    // Arrange
    let (mut host, mut actuation) = admitted(1);
    // Act
    let frame = host
        .deliver(&mut actuation, br#"{"command":"status"}"#)
        .expect("delivered");
    // Assert
    assert_eq!(frame, b"{\"command\":\"status\"}\n");
}

#[test]
fn delivery_without_a_session_is_a_boundary_error() {
    // Arrange
    let mut actuation = actuation(1);
    let mut host = SerialHost::new(1);
    // Act
    let result = host.deliver(&mut actuation, b"{}");
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("serial_session_missing"))
    ));
}

#[test]
fn replayed_heartbeats_keep_the_link_open_across_long_operations() {
    // Arrange
    let (mut host, mut actuation) = admitted(1);
    actuation.advance(10_000, true).expect("advance");
    // Act
    let check = host.close_check(&mut actuation);
    // Assert
    assert_eq!(check.status, CheckStatus::Passed, "{}", check.detail);
}

#[test]
fn host_heartbeat_loss_closes_the_link_at_the_deadline() {
    // Arrange
    let (mut host, mut actuation) = admitted(1);
    actuation.host_heartbeats = false;
    actuation.advance(2_800, false).expect("advance");
    // Act
    host.supervise(&mut actuation);
    // Assert
    assert_eq!(
        host.maybe_failure(),
        Some(SerialLinkFailure::HeartbeatTimeout)
    );
}

#[test]
fn link_survives_just_before_the_deadline_without_heartbeats() {
    // Arrange
    let (mut host, mut actuation) = admitted(1);
    actuation.host_heartbeats = false;
    actuation.advance(2_750, false).expect("advance");
    // Act
    host.supervise(&mut actuation);
    // Assert
    assert_eq!(host.maybe_failure(), None);
}

#[test]
fn a_full_owner_queue_closes_the_link() {
    // Arrange
    let (mut host, mut actuation) = admitted(1);
    for _ in 0..OWNER_CAPACITY {
        host.state
            .owner
            .borrow_mut()
            .push_back(OwnerEvent::Frame(Vec::new()));
    }
    // Act
    let _ = host.deliver(&mut actuation, b"{}");
    // Assert
    assert_eq!(host.maybe_failure(), Some(SerialLinkFailure::ControlQueue));
}
