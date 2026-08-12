use super::*;

#[test]
fn dry_run_flash_with_explicit_image_renders_vector_command() {
    // Arrange
    let command = FlashCommand {
        common: common_args(),
        image: Some(Utf8PathBuf::from("/tmp/bitaxe-ultra205.elf")),
        manifest: None,
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let outcome = run_flash(&command, &environment).expect("flash");

    // Assert
    assert_eq!(
        outcome.command,
        CommandSpec::new(
            "espflash",
            [
                "flash",
                "--chip",
                "esp32s3",
                "--port",
                "/dev/cu.usbmodem101",
                "/tmp/bitaxe-ultra205.elf",
            ],
        )
    );
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn flash_with_wifi_credentials_generates_and_executes_nvs_seed_after_flash() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let credentials_path = write_wifi_credentials(&dir, "LabNet", "super-secret");
    let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
    let command = FlashCommand {
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: Some(credentials_path),
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let outcome = run_flash(&command, &environment).expect("flash");

    // Assert
    let nvs_seed = outcome.nvs_seed.as_ref().expect("nvs seed");
    let observed = environment.observed_flashes();
    let executed_flash_path = observed[0].path.as_str();
    assert_eq!(
        environment.generated_nvs_partitions(),
        vec![(
            nvs_seed
                .image
                .parent()
                .expect("nvs seed parent")
                .join("wifi-nvs.csv"),
            nvs_seed.image.clone(),
            NVS_PARTITION_SIZE.to_owned(),
        )]
    );
    assert_eq!(
        environment.executed_commands(),
        vec![
            CommandSpec::new(
                "espflash",
                [
                    "write-bin",
                    "--chip",
                    "esp32s3",
                    "--port",
                    "/dev/cu.usbmodem101",
                    "--non-interactive",
                    "--before",
                    "usb-reset",
                    "--after",
                    "hard-reset",
                    "--skip-update-check",
                    "0x0",
                    executed_flash_path,
                ],
            ),
            CommandSpec::new(
                "espflash",
                [
                    "write-bin",
                    "--chip",
                    "esp32s3",
                    "--port",
                    "/dev/cu.usbmodem101",
                    "--non-interactive",
                    "--before",
                    "usb-reset",
                    "--after",
                    "hard-reset",
                    "--skip-update-check",
                    NVS_PARTITION_OFFSET,
                    nvs_seed.image.as_str(),
                ],
            ),
        ]
    );
    assert_eq!(
        environment.phase35_stage_gates(),
        vec![
            ("after-factory".to_owned(), "/dev/cu.usbmodem101".to_owned()),
            ("after-nvs".to_owned(), "/dev/cu.usbmodem101".to_owned()),
        ]
    );
}

#[test]
fn wifi_credentials_nvs_csv_uses_main_namespace_and_upstream_keys() {
    // Arrange
    let credentials = WifiCredentials {
        ssid: "Lab,Net".to_owned(),
        wifi_pass: "quoted\"secret".to_owned(),
    };

    // Act
    let csv = wifi_nvs_csv(&credentials);

    // Assert
    assert!(csv.contains("main,namespace,,"));
    assert!(csv.contains("wifissid,data,string,\"Lab,Net\""));
    assert!(csv.contains("wifipass,data,string,\"quoted\"\"secret\""));
    assert!(csv.contains("hostname,data,string,bitaxe"));
    assert!(csv.contains("asicfrequency,data,u16,485"));
    assert!(csv.contains("asicvoltage,data,u16,1200"));
    assert!(csv.contains("boardversion,data,string,205"));
    assert!(csv.contains("mineonboot,data,u16,0"));
}

#[test]
fn network_reconnect_probe_marker_is_opt_in() {
    // Arrange
    let credentials = WifiCredentials {
        ssid: "test-network".to_owned(),
        wifi_pass: "test-password".to_owned(),
    };

    // Act
    let ordinary = wifi_nvs_csv_for_mode(&credentials, WifiNvsSeedMode::Ordinary);
    let probe = wifi_nvs_csv_for_mode(&credentials, WifiNvsSeedMode::NetworkReconnectProbe);

    // Assert
    assert!(!ordinary.contains("netreconprobe"));
    assert!(probe.contains("netreconprobe,data,u16,1"));
}

#[test]
fn wifi_credentials_reject_invalid_lengths_without_secret_value() {
    // Arrange
    let file = WifiCredentialsFile {
        ssid: String::new(),
        wifi_pass: "p".repeat(64),
    };

    // Act
    let result = validate_wifi_credentials(file);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("ssid length 0 is outside 1..=32"));
    assert!(error.contains("wifiPass length 64 is outside 0..=63"));
    assert!(!error.contains(&"p".repeat(64)));
}
