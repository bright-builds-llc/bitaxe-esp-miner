use super::*;

const TEST_ELF: &[u8] =
    b"synthetic elf firmware_variant=overheat-trip-test local_overheat_trip_c=55";

fn dry_run_command(
    manifest: Utf8PathBuf,
    maybe_allow_test_firmware: Option<FirmwareVariant>,
) -> FlashCommand {
    let mut common = common_args();
    common.dry_run = true;
    FlashCommand {
        factory_reset: false,
        common,
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
        maybe_web_ui: None,
        maybe_allow_test_firmware,
    }
}

fn manifest_document(manifest: &Utf8Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(manifest).expect("manifest")).expect("JSON")
}

fn write_manifest_document(manifest: &Utf8Path, document: &serde_json::Value) {
    std::fs::write(manifest, document.to_string()).expect("rewrite manifest");
}

fn record_firmware_variant(manifest: &Utf8Path, variant: &str) {
    let mut document = manifest_document(manifest);
    document["firmware_variant"] = variant.into();
    write_manifest_document(manifest, &document);
}

/// Replaces the package ELF and its artifact digest, leaving the rest of the package intact.
fn replace_firmware_elf(manifest: &Utf8Path, elf: &[u8]) {
    let mut document = manifest_document(manifest);
    let artifacts = document["artifacts"].as_array_mut().expect("artifacts");
    let artifact = artifacts
        .iter_mut()
        .find(|artifact| artifact["kind"] == "firmware_elf")
        .expect("firmware_elf artifact");
    artifact["sha256"] = sha256_bytes(elf).into();
    let path = artifact["path"].as_str().expect("path").to_owned();
    let parent = manifest.parent().expect("manifest parent");
    std::fs::write(parent.join(path), elf).expect("write ELF");
    write_manifest_document(manifest, &document);
}

#[test]
fn a_manifest_without_a_firmware_variant_flashes_as_production() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    let command = dry_run_command(manifest, None);
    let environment = FakeFlashEnvironment::default();

    // Act
    let outcome = run_flash(&command, &environment);

    // Assert
    assert!(outcome.is_ok(), "{outcome:?}");
}

#[test]
fn a_test_variant_package_is_refused_without_the_allowance() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    record_firmware_variant(&manifest, "overheat-trip-test");
    let command = dry_run_command(manifest, None);
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_flash(&command, &environment).expect_err("test firmware must be refused");

    // Assert
    assert!(error
        .to_string()
        .contains("reason=test_firmware_requires_allowance"));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn the_allowance_with_a_production_package_is_refused() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    record_firmware_variant(&manifest, "production");
    let command = dry_run_command(manifest, Some(FirmwareVariant::OverheatTripTest));
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_flash(&command, &environment).expect_err("allowance mismatch must block");

    // Assert
    assert!(error
        .to_string()
        .contains("reason=test_firmware_allowance_without_test_package"));
}

#[test]
fn test_firmware_bytes_in_a_production_package_are_refused() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    replace_firmware_elf(&manifest, TEST_ELF);
    let command = dry_run_command(manifest, None);
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_flash(&command, &environment).expect_err("disagreeing bytes must block");

    // Assert
    assert!(error
        .to_string()
        .contains("reason=test_firmware_bytes_in_production_package"));
}

#[test]
fn an_unknown_recorded_firmware_variant_is_refused() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    record_firmware_variant(&manifest, "debug");
    let command = dry_run_command(manifest, None);
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_flash(&command, &environment).expect_err("unknown variant must block");

    // Assert
    assert!(error
        .to_string()
        .contains("reason=manifest_firmware_variant_unknown"));
}

#[test]
fn the_matching_allowance_admits_a_test_variant_package() {
    // Arrange
    let recorded = FirmwareVariant::OverheatTripTest;

    // Act
    let admission = admit_firmware_variant(recorded, Some(FirmwareVariant::OverheatTripTest));
    let bytes = require_firmware_variant_bytes(recorded, TEST_ELF);

    // Assert
    assert!(admission.is_ok(), "{admission:?}");
    assert!(bytes.is_ok(), "{bytes:?}");
}

#[test]
fn a_test_variant_package_without_the_marker_is_refused() {
    // Arrange
    let production_elf = b"synthetic firmware elf";

    // Act
    let error = require_firmware_variant_bytes(FirmwareVariant::OverheatTripTest, production_elf)
        .expect_err("missing marker must block");

    // Assert
    assert!(error
        .to_string()
        .contains("reason=test_firmware_marker_missing"));
}

#[test]
fn flash_monitor_cli_parses_the_test_firmware_allowance() {
    // Arrange
    let argv = [
        "bitaxe-flash",
        "flash-monitor",
        "--board",
        "205",
        "--allow-test-firmware",
        "overheat-trip-test",
    ];

    // Act
    let cli = parse_cli(argv.iter().map(|value| value.to_string())).expect("parse flash-monitor");

    // Assert
    let CliCommand::FlashMonitor(command) = cli.command else {
        panic!("expected flash-monitor command");
    };
    assert_eq!(
        command.maybe_allow_test_firmware,
        Some(FirmwareVariant::OverheatTripTest)
    );
}

const GAMMA_CHECK_ELF: &[u8] =
    b"synthetic elf firmware_board=601 firmware_variant=gamma601-vout-check";

#[test]
fn the_core_voltage_check_installs_only_on_the_gamma_601() {
    // Act
    let gamma =
        require_firmware_variant_board(FirmwareVariant::Gamma601VoutCheck, BoardId::Gamma601);
    let ultra =
        require_firmware_variant_board(FirmwareVariant::Gamma601VoutCheck, BoardId::Ultra205)
            .expect_err("the 205 must refuse the 601 check");

    // Assert
    assert!(gamma.is_ok(), "{gamma:?}");
    assert!(ultra
        .to_string()
        .contains("reason=test_firmware_board_mismatch"));
}

#[test]
fn the_overheat_trip_test_installs_only_on_the_ultra_205() {
    // Act
    let error =
        require_firmware_variant_board(FirmwareVariant::OverheatTripTest, BoardId::Gamma601)
            .expect_err("the 601 must refuse the 205 test");

    // Assert
    assert!(error
        .to_string()
        .contains("reason=test_firmware_board_mismatch"));
}

#[test]
fn the_core_voltage_check_bytes_admit_only_their_own_variant() {
    // Act
    let own = require_firmware_variant_bytes(FirmwareVariant::Gamma601VoutCheck, GAMMA_CHECK_ELF);
    let production = require_firmware_variant_bytes(FirmwareVariant::Production, GAMMA_CHECK_ELF)
        .expect_err("production must refuse the check marker");
    let overheat =
        require_firmware_variant_bytes(FirmwareVariant::OverheatTripTest, GAMMA_CHECK_ELF)
            .expect_err("another variant must refuse the check marker");

    // Assert
    assert!(own.is_ok(), "{own:?}");
    assert!(production
        .to_string()
        .contains("reason=test_firmware_bytes_in_production_package"));
    assert!(overheat
        .to_string()
        .contains("reason=test_firmware_marker_missing"));
}

#[test]
fn bytes_carrying_two_test_markers_are_refused() {
    // Arrange
    let both = [GAMMA_CHECK_ELF, TEST_ELF].concat();

    // Act
    let error = require_firmware_variant_bytes(FirmwareVariant::Gamma601VoutCheck, &both)
        .expect_err("two markers must block");

    // Assert
    assert!(error
        .to_string()
        .contains("reason=test_firmware_marker_foreign"));
}

#[test]
fn flash_cli_parses_the_core_voltage_check_allowance() {
    // Arrange
    let argv = [
        "bitaxe-flash",
        "flash",
        "--board",
        "601",
        "--allow-test-firmware",
        "gamma601-vout-check",
    ];

    // Act
    let cli = parse_cli(argv.iter().map(|value| value.to_string())).expect("parse flash");

    // Assert
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    assert_eq!(
        command.maybe_allow_test_firmware,
        Some(FirmwareVariant::Gamma601VoutCheck)
    );
}

const GAMMA_MINING_ELF: &[u8] =
    b"synthetic elf firmware_board=601 firmware_variant=gamma601-mining";

#[test]
fn the_mining_variant_installs_only_on_the_gamma_601() {
    // Act
    let gamma = require_firmware_variant_board(FirmwareVariant::Gamma601Mining, BoardId::Gamma601);
    let ultra = require_firmware_variant_board(FirmwareVariant::Gamma601Mining, BoardId::Ultra205)
        .expect_err("the 205 must refuse the 601 mining variant");

    // Assert
    assert!(gamma.is_ok(), "{gamma:?}");
    assert!(ultra
        .to_string()
        .contains("reason=test_firmware_board_mismatch"));
}

#[test]
fn the_mining_variant_bytes_admit_only_their_own_variant() {
    // Act
    let own = require_firmware_variant_bytes(FirmwareVariant::Gamma601Mining, GAMMA_MINING_ELF);
    let check =
        require_firmware_variant_bytes(FirmwareVariant::Gamma601VoutCheck, GAMMA_MINING_ELF)
            .expect_err("the core-voltage check must refuse the mining marker");
    let production = require_firmware_variant_bytes(FirmwareVariant::Production, GAMMA_MINING_ELF)
        .expect_err("production must refuse the mining marker");

    // Assert
    assert!(own.is_ok(), "{own:?}");
    assert!(check
        .to_string()
        .contains("reason=test_firmware_marker_missing"));
    assert!(production
        .to_string()
        .contains("reason=test_firmware_bytes_in_production_package"));
}

#[test]
fn installing_the_mining_variant_requires_naming_it() {
    // Act
    let without = admit_firmware_variant(FirmwareVariant::Gamma601Mining, None)
        .expect_err("the mining variant needs its allowance");
    let named = admit_firmware_variant(
        FirmwareVariant::Gamma601Mining,
        Some(FirmwareVariant::Gamma601Mining),
    );

    // Assert
    assert!(without
        .to_string()
        .contains("reason=test_firmware_requires_allowance"));
    assert!(named.is_ok(), "{named:?}");
}

const GAMMA_CHIP_CHECK_ELF: &[u8] =
    b"synthetic elf firmware_board=601 firmware_variant=gamma601-chip-check";

#[test]
fn the_chip_check_installs_only_on_the_gamma_601_and_only_when_named() {
    // Act
    let gamma =
        require_firmware_variant_board(FirmwareVariant::Gamma601ChipCheck, BoardId::Gamma601);
    let ultra =
        require_firmware_variant_board(FirmwareVariant::Gamma601ChipCheck, BoardId::Ultra205)
            .expect_err("the 205 must refuse the 601 chip check");
    let unnamed = admit_firmware_variant(FirmwareVariant::Gamma601ChipCheck, None)
        .expect_err("the chip check needs its allowance");

    // Assert
    assert!(gamma.is_ok(), "{gamma:?}");
    assert!(ultra
        .to_string()
        .contains("reason=test_firmware_board_mismatch"));
    assert!(unnamed
        .to_string()
        .contains("reason=test_firmware_requires_allowance"));
}

#[test]
fn the_chip_check_bytes_admit_only_their_own_variant() {
    // Act
    let own =
        require_firmware_variant_bytes(FirmwareVariant::Gamma601ChipCheck, GAMMA_CHIP_CHECK_ELF);
    let mining =
        require_firmware_variant_bytes(FirmwareVariant::Gamma601Mining, GAMMA_CHIP_CHECK_ELF)
            .expect_err("the mining variant must refuse the chip check marker");

    // Assert
    assert!(own.is_ok(), "{own:?}");
    assert!(mining
        .to_string()
        .contains("reason=test_firmware_marker_missing"));
}

const GAMMA_HASH_CHECK_ELF: &[u8] =
    b"synthetic elf firmware_board=601 firmware_variant=gamma601-hash-check";

#[test]
fn the_hash_check_installs_only_on_the_gamma_601_and_only_when_named() {
    // Act
    let gamma =
        require_firmware_variant_board(FirmwareVariant::Gamma601HashCheck, BoardId::Gamma601);
    let ultra =
        require_firmware_variant_board(FirmwareVariant::Gamma601HashCheck, BoardId::Ultra205)
            .expect_err("the 205 must refuse the 601 hash check");
    let unnamed = admit_firmware_variant(FirmwareVariant::Gamma601HashCheck, None)
        .expect_err("the hash check needs its allowance");

    // Assert
    assert!(gamma.is_ok(), "{gamma:?}");
    assert!(ultra
        .to_string()
        .contains("reason=test_firmware_board_mismatch"));
    assert!(unnamed
        .to_string()
        .contains("reason=test_firmware_requires_allowance"));
}

#[test]
fn the_hash_check_bytes_admit_only_their_own_variant() {
    // Act
    let own =
        require_firmware_variant_bytes(FirmwareVariant::Gamma601HashCheck, GAMMA_HASH_CHECK_ELF);
    let chip_check =
        require_firmware_variant_bytes(FirmwareVariant::Gamma601ChipCheck, GAMMA_HASH_CHECK_ELF)
            .expect_err("the chip check variant must refuse the hash check marker");

    // Assert
    assert!(own.is_ok(), "{own:?}");
    assert!(chip_check
        .to_string()
        .contains("reason=test_firmware_marker_missing"));
}

const GAMMA_OVERHEAT_TRIP_TEST_ELF: &[u8] =
    b"synthetic elf firmware_board=601 firmware_variant=gamma601-overheat-trip-test local_overheat_trip_c=55";

#[test]
fn the_601_overheat_variant_installs_only_on_the_gamma_601_and_only_when_named() {
    // Act
    let gamma = require_firmware_variant_board(
        FirmwareVariant::Gamma601OverheatTripTest,
        BoardId::Gamma601,
    );
    let ultra = require_firmware_variant_board(
        FirmwareVariant::Gamma601OverheatTripTest,
        BoardId::Ultra205,
    )
    .expect_err("the 205 must refuse the 601 overheat variant");
    let unnamed = admit_firmware_variant(FirmwareVariant::Gamma601OverheatTripTest, None)
        .expect_err("the 601 overheat variant needs its allowance");
    let mining_allowance = admit_firmware_variant(
        FirmwareVariant::Gamma601OverheatTripTest,
        Some(FirmwareVariant::Gamma601Mining),
    )
    .expect_err("the mining allowance must not admit the 601 overheat variant");

    // Assert
    assert!(gamma.is_ok(), "{gamma:?}");
    assert!(ultra
        .to_string()
        .contains("reason=test_firmware_board_mismatch"));
    assert!(unnamed
        .to_string()
        .contains("reason=test_firmware_requires_allowance"));
    assert!(mining_allowance
        .to_string()
        .contains("reason=test_firmware_variant_mismatch"));
}

#[test]
fn the_601_overheat_bytes_admit_only_their_own_variant() {
    // Act
    let own = require_firmware_variant_bytes(
        FirmwareVariant::Gamma601OverheatTripTest,
        GAMMA_OVERHEAT_TRIP_TEST_ELF,
    );
    let mining = require_firmware_variant_bytes(
        FirmwareVariant::Gamma601Mining,
        GAMMA_OVERHEAT_TRIP_TEST_ELF,
    )
    .expect_err("the mining variant must refuse the 601 overheat marker");
    let overheat_205 = require_firmware_variant_bytes(
        FirmwareVariant::OverheatTripTest,
        GAMMA_OVERHEAT_TRIP_TEST_ELF,
    )
    .expect_err("the 205 overheat variant must refuse the 601 overheat marker");

    // Assert
    assert!(own.is_ok(), "{own:?}");
    assert!(mining
        .to_string()
        .contains("reason=test_firmware_marker_missing"));
    assert!(overheat_205
        .to_string()
        .contains("reason=test_firmware_marker_missing"));
}

#[test]
fn a_601_overheat_application_that_also_carries_the_mining_marker_is_refused() {
    // Arrange
    let both = b"synthetic elf firmware_board=601 firmware_variant=gamma601-overheat-trip-test firmware_variant=gamma601-mining";

    // Act
    let error = require_firmware_variant_bytes(FirmwareVariant::Gamma601OverheatTripTest, both)
        .expect_err("two markers must be refused");

    // Assert
    assert!(error
        .to_string()
        .contains("reason=test_firmware_marker_foreign"));
}
