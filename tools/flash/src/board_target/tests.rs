use super::*;

#[test]
fn commands_other_than_flash_monitor_and_detect_refuse_the_601() {
    // Act
    let error = parse_board("601").expect_err("205 only");
    // Assert
    assert!(error.contains("flash, flash-monitor, monitor and detect"));
}

#[test]
fn flash_commands_accept_both_boards() {
    // Act
    let boards = ["205", "601"].map(|value| parse_flash_board(value).expect("board"));
    // Assert
    assert_eq!(boards, [BoardId::Ultra205, BoardId::Gamma601]);
}

#[test]
fn an_unknown_board_is_refused() {
    // Act
    let error = parse_flash_board("401").expect_err("unknown");
    // Assert
    assert!(error.contains("unsupported board 401"));
}

#[test]
fn the_601_builds_its_own_package() {
    // Act
    let names = [
        BoardId::Gamma601.package_build_target(),
        BoardId::Gamma601.package_manifest_relative_path(),
        BoardId::Gamma601.default_elf_name(),
        BoardId::Gamma601.factory_image_name(),
    ];
    // Assert
    assert!(names.iter().all(|name| name.contains("gamma601")));
}

#[test]
fn the_205_keeps_its_package_names() {
    // Act
    let names = [
        BoardId::Ultra205.package_build_target(),
        BoardId::Ultra205.package_manifest_relative_path(),
        BoardId::Ultra205.default_elf_name(),
    ];
    // Assert
    assert_eq!(
        names,
        [
            "//firmware/bitaxe:firmware_image",
            "firmware/bitaxe/bitaxe-ultra205-package.json",
            "bitaxe-ultra205.elf",
        ]
    );
}

#[test]
fn a_manifest_naming_another_board_is_refused() {
    // Act
    let error = require_manifest_board(BoardId::Gamma601, Some("205")).expect_err("mismatch");
    // Assert
    assert!(error.to_string().contains("manifest_board_mismatch"));
}

#[test]
fn a_manifest_without_a_board_is_refused() {
    // Act
    let error = require_manifest_board(BoardId::Ultra205, None).expect_err("missing");
    // Assert
    assert!(error.to_string().contains("manifest_board_missing"));
}

#[test]
fn application_bytes_must_carry_the_requested_board() {
    // Arrange
    let gamma = b"..firmware_board=601..".as_slice();
    // Act
    let as_gamma = require_board_bytes(BoardId::Gamma601, gamma);
    let as_ultra = require_board_bytes(BoardId::Ultra205, gamma);
    // Assert
    assert!(as_gamma.is_ok());
    assert!(as_ultra
        .expect_err("mismatch")
        .to_string()
        .contains("firmware_board_mismatch"));
}

#[test]
fn application_bytes_without_a_marker_are_refused() {
    // Act
    let error = require_board_bytes(BoardId::Ultra205, b"no marker").expect_err("missing");
    // Assert
    assert!(error.to_string().contains("firmware_board_marker_missing"));
}

#[test]
fn application_bytes_carrying_both_boards_are_refused() {
    // Arrange
    let both = b"firmware_board=205 firmware_board=601".as_slice();
    // Act
    let error = require_board_bytes(BoardId::Ultra205, both).expect_err("both");
    // Assert
    assert!(error.to_string().contains("firmware_board_mismatch"));
}

#[test]
fn every_board_has_an_upstream_seed() {
    // Act
    let seeded = [BoardId::Ultra205, BoardId::Gamma601].map(|board| {
        bitaxe_config::board_default_seed_values(board.registered().label()).is_some()
    });
    // Assert
    assert_eq!(seeded, [true, true]);
}
