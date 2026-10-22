use super::*;

fn gamma_flash(manifest: Option<Utf8PathBuf>, dry_run: bool) -> FlashCommand {
    let mut common = common_args();
    common.board = BoardId::Gamma601;
    common.dry_run = dry_run;
    FlashCommand {
        factory_reset: false,
        common,
        image: None,
        manifest,
        wifi_credentials: None,
        maybe_web_ui: None,
        maybe_allow_test_firmware: None,
    }
}

#[test]
fn flash_monitor_and_detect_accept_the_601() {
    // Arrange
    let commands = ["flash", "flash-monitor", "monitor", "detect"];
    // Act
    let parsed = commands.map(|command| {
        parse_cli(["bitaxe-flash", command, "--board", "601"].map(str::to_owned)).is_ok()
    });
    // Assert
    assert_eq!(parsed, [true; 4]);
}

#[test]
fn other_commands_refuse_the_601() {
    // Arrange
    let args = ["bitaxe-flash", "nvs-readback", "--board", "601"].map(str::to_owned);
    // Act
    let error = parse_cli(args).expect_err("205 only");
    // Assert
    assert!(error.to_string().contains("accepted only by flash"));
}

#[test]
fn a_205_package_is_refused_for_the_601() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_flash(&gamma_flash(Some(manifest), true), &environment).expect_err("board");
    // Assert
    assert!(
        error.to_string().contains("manifest_board_mismatch"),
        "{error}"
    );
}

#[test]
fn a_601_factory_install_without_a_seed_is_refused() {
    // Arrange
    let mut command = gamma_flash(None, true);
    command.factory_reset = true;
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_flash(&command, &environment).expect_err("factory");
    // Assert
    assert!(error
        .to_string()
        .contains("factory_install_601_requires_seed"));
    assert!(environment.built_boards.borrow().is_empty());
}

#[test]
fn a_601_factory_install_seeds_the_ordinary_settings_only() {
    // Arrange
    let mut command = gamma_flash(None, true);
    command.factory_reset = true;
    command.wifi_credentials = Some(Utf8PathBuf::from("wifi-credentials.json"));
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_flash_with_wifi_mode(
        &command,
        WifiNvsSeedMode::NetworkReconnectProbe,
        &environment,
    )
    .expect_err("probe seed");
    // Assert
    assert!(error
        .to_string()
        .contains("factory_install_601_seeds_ordinary_only"));
}

#[test]
fn the_601_seed_names_the_601_and_its_asic() {
    // Arrange
    let credentials = WifiCredentials {
        ssid: "Lab".to_owned(),
        wifi_pass: "secret".to_owned(),
    };
    // Act
    let csv = wifi_nvs_csv_for_board(&credentials, WifiNvsSeedMode::Ordinary, BoardId::Gamma601);
    // Assert
    for row in [
        "boardversion,data,string,601",
        "asicmodel,data,string,BM1370",
        "devicemodel,data,string,gamma",
        "asicfrequency,data,u16,525",
        "asicvoltage,data,u16,1150",
        "wifissid,data,string,Lab",
    ] {
        assert!(csv.contains(row), "{row}");
    }
    assert!(!csv.contains("BM1366"));
}

#[test]
fn flashing_the_601_builds_the_601_package() {
    // Arrange
    let environment = FakeFlashEnvironment::default();
    // Act
    let _ = run_flash(&gamma_flash(None, false), &environment);
    // Assert
    assert_eq!(*environment.built_boards.borrow(), [BoardId::Gamma601]);
}

#[test]
fn monitoring_the_601_binds_its_session_to_the_601() {
    // Arrange
    let mut common = common_args();
    common.board = BoardId::Gamma601;
    common.dry_run = false;
    let command = MonitorCommand {
        common,
        capture_timeout_seconds: 1,
    };
    let environment = FakeFlashEnvironment::default();
    // Act
    let _ = run_monitor(&command, &environment);
    // Assert
    assert_eq!(*environment.session_boards.borrow(), [BoardId::Gamma601]);
}
