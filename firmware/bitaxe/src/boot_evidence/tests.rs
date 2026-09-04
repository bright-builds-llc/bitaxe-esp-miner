use super::{evidence_marker, BootEvidenceState, BootSessionNonce, ConnectedOriginReplay};

#[test]
fn boot_evidence_marker_is_fixed_width_and_redacted() {
    // Arrange
    let nonce = BootSessionNonce([0, 1, u32::MAX, 0x1234_abcd]);

    // Act
    let marker = evidence_marker(nonce, BootEvidenceState::Booted);

    // Assert
    assert_eq!(
        marker,
        "plan13_boot_evidence session=0000000000000001ffffffff1234abcd state=booted redacted=true"
    );
}

#[test]
fn connected_origin_remains_due_after_an_unbounded_human_delay() {
    // Arrange
    let mut replay = ConnectedOriginReplay::new("http://private-device".to_owned(), 1_000);

    // Act
    let observed = replay.maybe_take_due(24 * 60 * 60 * 1_000);

    // Assert
    assert_eq!(observed.as_deref(), Some("http://private-device"));
    assert!(replay.next_deadline_ms > 24 * 60 * 60 * 1_000);
}
