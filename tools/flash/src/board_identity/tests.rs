use super::*;

const BOARD_INFO_A: &[u8] = b"Chip type: ESP32-S3\nMAC address: 02:00:00:00:a1:b1\n";
const BOARD_INFO_B: &[u8] = b"Chip type: ESP32-S3\nMAC address: 02:00:00:00:c2:d2\n";

fn workspace() -> (tempfile::TempDir, Utf8PathBuf) {
    let dir = tempfile::tempdir().expect("temporary directory");
    let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf-8 path");
    (dir, path)
}

fn private_file(dir: &Utf8Path, name: &str, contents: &[u8]) -> Utf8PathBuf {
    let path = dir.join(name);
    fs::write(path.as_std_path(), contents).expect("write");
    fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o600)).expect("mode");
    path
}

fn digest_of(board_info: &[u8]) -> String {
    base_mac_digest(base_mac(board_info).expect("mac"))
}

#[test]
fn the_base_mac_comes_from_the_single_mac_line() {
    // Arrange / Act
    let mac = base_mac(BOARD_INFO_A).expect("mac");
    // Assert
    assert_eq!(mac, [0x02, 0x00, 0x00, 0x00, 0xa1, 0xb1]);
}

#[test]
fn board_info_without_exactly_one_mac_is_refused() {
    // Arrange
    let none = b"Chip type: ESP32-S3\n".as_slice();
    let two = b"MAC address: 02:00:00:00:a1:b1\nMAC address: 02:00:00:00:a1:b2\n".as_slice();
    let short = b"MAC address: 02:00:00:a1:b1\n".as_slice();
    // Act / Assert
    assert!(base_mac(none).is_err());
    assert!(base_mac(two).is_err());
    assert!(base_mac(short).is_err());
}

#[test]
fn the_digest_is_not_a_plain_hash_of_the_mac() {
    // Arrange
    let mac = [0x02, 0x00, 0x00, 0x00, 0xa1, 0xb1];
    // Act
    let digest = base_mac_digest(mac);
    // Assert
    assert_ne!(digest, sha256_bytes(b"02:00:00:00:a1:b1"));
}

#[test]
fn different_macs_have_different_digests() {
    // Arrange
    let one = [0x02, 0x00, 0x00, 0x00, 0xa1, 0xb1];
    let other = [0x02, 0x00, 0x00, 0x00, 0xa1, 0xb2];
    // Act / Assert
    assert_ne!(base_mac_digest(one), base_mac_digest(other));
}

#[test]
fn rom_admission_checks_the_registry_before_its_reset_and_the_mac_before_any_effect() {
    // Arrange
    let source = include_str!("../environment/usb_ownership.rs");
    let body = &source[source.find("fn enter_rom").expect("admission")..];
    let at = |text: &str| body.find(text).unwrap_or_else(|| panic!("missing {text}"));
    // Act
    let (pre_check, reset, admitted, gate, done) = (
        at("crate::board_identity::require_registered("),
        at("\"usb-reset\""),
        at("admit_rom_downloader("),
        at("crate::board_identity::require_board("),
        at("Ok(board_info)"),
    );
    // Assert
    assert!(pre_check < reset && reset < admitted && admitted < gate && gate < done);
}

#[test]
fn every_flash_write_runs_after_rom_admission() {
    // Arrange
    let source = include_str!("../environment.rs");
    let segment = |from: &str| &source[source.find(from).expect(from)..];
    let esptool = segment("fn execute_esptool_write_flash");
    let espflash = segment("fn execute_with_output");
    // Act
    let esptool_order = esptool.find("self.ensure_bootloader()?").expect("admitted")
        < esptool.find("run_esptool_write_flash(").expect("write");
    let espflash_order = espflash
        .find("self.ensure_bootloader()?")
        .expect("admitted")
        < espflash.find(".run_espflash(").expect("write");
    // Assert: the esptool writer, and espflash's write-bin, erase and flash commands.
    assert!(esptool_order && espflash_order);
}

#[test]
fn the_core_dump_erase_checks_the_board_before_erasing() {
    // Arrange
    let erase = include_str!("../environment/core_dump.rs");
    // Act
    let gate = erase
        .find("crate::board_identity::require_board(")
        .expect("gate");
    let effect = erase.find("erase_command(").expect("erase effect");
    // Assert
    assert!(gate < effect);
}

#[test]
fn a_missing_registry_admits_no_device() {
    // Arrange
    let (_dir, workspace) = workspace();
    // Act
    let error = require_board(&workspace, RegisteredBoard::Ultra205, BOARD_INFO_A)
        .expect_err("unregistered");
    // Assert
    assert!(error.to_string().contains("reason=unregistered"), "{error}");
}

#[test]
fn a_205_package_cannot_be_written_to_a_registered_601() {
    // Arrange
    let (_dir, workspace) = workspace();
    let info = private_file(&workspace, "board-info.txt", BOARD_INFO_A);
    register(&workspace, RegisteredBoard::Gamma601, &info).expect("register 601");
    // Act
    let error = require_board(&workspace, RegisteredBoard::Ultra205, BOARD_INFO_A)
        .expect_err("other board");
    // Assert
    assert!(
        error
            .to_string()
            .contains("reason=registered_as_other_board requested=205 registered=601"),
        "{error}"
    );
}

#[test]
fn a_registered_board_is_admitted() {
    // Arrange
    let (_dir, workspace) = workspace();
    let info = private_file(&workspace, "board-info.txt", BOARD_INFO_B);
    register(&workspace, RegisteredBoard::Ultra205, &info).expect("register 205");
    // Act
    let result = require_board(&workspace, RegisteredBoard::Ultra205, BOARD_INFO_B);
    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn registration_is_private_holds_no_mac_and_is_idempotent() {
    // Arrange
    let (_dir, workspace) = workspace();
    let info = private_file(&workspace, "board-info.txt", BOARD_INFO_A);
    // Act
    let first = register(&workspace, RegisteredBoard::Ultra205, &info).expect("register");
    let second = register(&workspace, RegisteredBoard::Ultra205, &info).expect("register again");
    // Assert
    let path = workspace.join(REGISTRY_FILE);
    let text = fs::read_to_string(path.as_std_path()).expect("registry");
    assert_eq!((first, second), ("added", "already_registered"));
    assert_eq!(
        fs::metadata(path.as_std_path())
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(!text.to_ascii_lowercase().contains("a1:b1"));
    assert_eq!(load_registry(&path).expect("load").len(), 1);
}

#[test]
fn registering_one_device_as_another_board_is_refused() {
    // Arrange
    let (_dir, workspace) = workspace();
    let info = private_file(&workspace, "board-info.txt", BOARD_INFO_A);
    register(&workspace, RegisteredBoard::Ultra205, &info).expect("register 205");
    // Act
    let error = register(&workspace, RegisteredBoard::Gamma601, &info).expect_err("conflict");
    // Assert
    assert!(error.to_string().contains("registered=205"), "{error}");
}

#[test]
fn a_registry_that_is_not_private_is_refused() {
    // Arrange
    let (_dir, workspace) = workspace();
    let path = private_file(
        &workspace,
        REGISTRY_FILE,
        br#"{"schema":"bitaxe-board-registry-v1","boards":[]}"#,
    );
    fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o644)).expect("mode");
    // Act
    let error = load_registry(&path).expect_err("not private");
    // Assert
    assert!(
        error.to_string().contains("registry_not_private"),
        "{error}"
    );
}

#[test]
fn a_registry_with_a_duplicate_digest_is_refused() {
    // Arrange
    let (_dir, workspace) = workspace();
    let digest = digest_of(BOARD_INFO_A);
    let document = format!(
        r#"{{"schema":"bitaxe-board-registry-v1","boards":[{{"board":"205","base_mac_sha256":"{digest}"}},{{"board":"601","base_mac_sha256":"{digest}"}}]}}"#
    );
    let path = private_file(&workspace, REGISTRY_FILE, document.as_bytes());
    // Act
    let error = load_registry(&path).expect_err("duplicate");
    // Assert
    assert!(error.to_string().contains("registry_malformed"), "{error}");
}

#[test]
fn a_board_info_file_that_is_not_private_is_refused() {
    // Arrange
    let (_dir, workspace) = workspace();
    let info = private_file(&workspace, "board-info.txt", BOARD_INFO_A);
    fs::set_permissions(info.as_std_path(), fs::Permissions::from_mode(0o644)).expect("mode");
    // Act
    let error = register(&workspace, RegisteredBoard::Ultra205, &info).expect_err("not private");
    // Assert
    assert!(
        error.to_string().contains("board_info_not_private"),
        "{error}"
    );
}

#[test]
fn a_symlinked_registry_is_refused() {
    // Arrange
    let (_dir, workspace) = workspace();
    let real = private_file(
        &workspace,
        "real.json",
        br#"{"schema":"bitaxe-board-registry-v1","boards":[]}"#,
    );
    let link = workspace.join(REGISTRY_FILE);
    std::os::unix::fs::symlink(real.as_std_path(), link.as_std_path()).expect("symlink");
    // Act
    let error = load_registry(&link).expect_err("symlink");
    // Assert
    assert!(
        error.to_string().contains("registry_not_private"),
        "{error}"
    );
}

#[test]
fn a_registry_with_another_schema_is_refused() {
    // Arrange
    let (_dir, workspace) = workspace();
    let path = private_file(
        &workspace,
        REGISTRY_FILE,
        br#"{"schema":"other-v1","boards":[]}"#,
    );
    // Act
    let error = load_registry(&path).expect_err("schema");
    // Assert
    assert!(error.to_string().contains("registry_schema"), "{error}");
}

#[test]
fn a_registry_entry_with_an_unknown_field_is_refused() {
    // Arrange
    let (_dir, workspace) = workspace();
    let digest = digest_of(BOARD_INFO_A);
    let document = format!(
        r#"{{"schema":"bitaxe-board-registry-v1","boards":[{{"board":"205","base_mac_sha256":"{digest}","mac":"x"}}]}}"#
    );
    let path = private_file(&workspace, REGISTRY_FILE, document.as_bytes());
    // Act
    let error = load_registry(&path).expect_err("unknown field");
    // Assert
    assert!(error.to_string().contains("registry_malformed"), "{error}");
}

#[test]
fn a_registry_digest_in_uppercase_is_refused() {
    // Arrange
    let (_dir, workspace) = workspace();
    let digest = digest_of(BOARD_INFO_A).to_ascii_uppercase();
    let document = format!(
        r#"{{"schema":"bitaxe-board-registry-v1","boards":[{{"board":"205","base_mac_sha256":"{digest}"}}]}}"#
    );
    let path = private_file(&workspace, REGISTRY_FILE, document.as_bytes());
    // Act
    let error = load_registry(&path).expect_err("uppercase");
    // Assert
    assert!(error.to_string().contains("registry_malformed"), "{error}");
}

#[test]
fn espflash_board_info_output_yields_its_mac() {
    // Arrange: the shape espflash 4 prints, with padding and other fields.
    let output = b"[2026-10-21T00:00:00Z INFO ] Serial port: 'port'\nChip type:         esp32s3 (revision v0.2)\nCrystal frequency: 40 MHz\nFlash size:        16MB\nFeatures:          WiFi, BLE\nMAC address:       02:00:00:00:a1:b1\n";
    // Act
    let mac = base_mac(output).expect("mac");
    // Assert
    assert_eq!(mac, [0x02, 0x00, 0x00, 0x00, 0xa1, 0xb1]);
}

#[test]
fn rom_entry_refuses_first_when_no_device_is_registered_for_the_board() {
    // Arrange
    let (_dir, workspace) = workspace();
    let info = private_file(&workspace, "board-info.txt", BOARD_INFO_A);
    register(&workspace, RegisteredBoard::Gamma601, &info).expect("register 601");
    // Act
    let error =
        require_registered(&workspace, RegisteredBoard::Ultra205).expect_err("none for 205");
    // Assert
    assert!(
        error
            .to_string()
            .contains("reason=none_registered requested=205"),
        "{error}"
    );
}

#[test]
fn ordinary_rom_admission_requests_the_session_board() {
    // Arrange
    let source = include_str!("../environment/usb_ownership.rs");
    let body = &source[source.find("fn ensure_bootloader").expect("admission")..];
    // Act
    let call = &body[..body.find("fn open_usb_session").expect("open")];
    // Assert
    assert!(call.contains("self.enter_rom(Some(self.rom_board.get()))"));
}

#[test]
fn a_new_session_without_a_board_requires_the_205() {
    // Arrange
    let source = include_str!("../environment.rs");
    let start = source.find("fn begin_usb_session(&self").expect("begin");
    // Act
    let body = &source[start
        ..source[start..]
            .find("fn begin_board_usb_session")
            .expect("next")
            + start];
    // Assert
    assert!(body.contains(
        "} else {\n            crate::board_identity::RegisteredBoard::Ultra205\n        };"
    ));
}

#[test]
fn a_session_cannot_change_its_board() {
    // Arrange
    let source = include_str!("../environment/usb_ownership.rs");
    let body = &source[source.find("fn open_usb_session").expect("open")..];
    // Act
    let guard = body.find("reason=session_board_changed").expect("guard");
    let set = body.find("self.rom_board.set(board)").expect("set");
    // Assert
    assert!(guard < set);
}

#[test]
fn only_identification_enters_rom_without_a_board() {
    // Arrange
    let environment = include_str!("../environment.rs");
    let identify = include_str!("identify.rs");
    // Act
    let ungated = environment.matches("self.enter_rom(None)").count();
    // Assert
    assert_eq!(ungated, 1);
    assert!(environment.contains(
        "fn admit_rom_for_identification(&self) -> Result<Vec<u8>> {\n        self.enter_rom(None)"
    ));
    assert!(identify.contains("environment.admit_rom_for_identification()"));
}
