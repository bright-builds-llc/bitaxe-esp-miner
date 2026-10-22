use super::*;

fn classify(built: KnownBoard, maybe_stored: Option<&str>) -> BoardIdentity {
    BoardIdentity::classify(built, StoredBoard::from_stored(maybe_stored))
}

#[test]
fn a_stored_value_naming_the_built_board_matches() {
    // Act
    let identity = classify(KnownBoard::Ultra205, Some("205"));
    // Assert
    assert_eq!(identity, BoardIdentity::Match);
}

#[test]
fn a_stored_value_naming_another_known_board_is_a_mismatch() {
    // Act
    let identity = classify(KnownBoard::Ultra205, Some("601"));
    // Assert
    assert_eq!(identity, BoardIdentity::OtherBoard(KnownBoard::Gamma601));
}

#[test]
fn a_missing_stored_value_is_unknown() {
    // Act
    let identity = classify(KnownBoard::Gamma601, None);
    // Assert
    assert_eq!(identity, BoardIdentity::Unknown);
}

#[test]
fn an_unrecognized_stored_value_is_unknown() {
    // Act
    let identity = classify(KnownBoard::Ultra205, Some("000"));
    // Assert
    assert_eq!(identity, BoardIdentity::Unknown);
}

#[test]
fn unreadable_settings_are_unknown() {
    // Act
    let identity = BoardIdentity::classify(KnownBoard::Ultra205, StoredBoard::Unreadable);
    // Assert
    assert_eq!(identity, BoardIdentity::Unknown);
}

#[test]
fn only_another_known_board_blocks_hardware_control() {
    // Arrange
    let identities = [
        BoardIdentity::Match,
        BoardIdentity::Unknown,
        BoardIdentity::OtherBoard(KnownBoard::Ultra205),
    ];
    // Act
    let permits = identities.map(BoardIdentity::permits_hardware_control);
    // Assert
    assert_eq!(permits, [true, true, false]);
}

#[test]
fn stored_values_outside_the_known_boards_are_not_echoed() {
    // Act
    let labels = [Some("205"), Some("601"), Some("my-secret-board"), None]
        .map(|stored| StoredBoard::from_stored(stored).label());
    // Assert
    assert_eq!(labels, ["205", "601", "other", "missing"]);
}

#[test]
fn the_usb_marker_names_built_stored_and_identity() {
    // Act
    let marker = usb_marker(KnownBoard::Gamma601, StoredBoard::from_stored(Some("601")));
    // Assert
    assert_eq!(
        marker,
        "usb_board_identity schema=v1 built=601 stored=601 identity=match redacted=true"
    );
}
