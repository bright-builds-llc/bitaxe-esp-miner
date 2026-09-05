use super::*;

#[test]
fn explicit_factory_reset_exits_rom_once_after_the_final_nvs_write() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = FlashCommand {
        factory_reset: true,
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(write_manifest_v4(&dir, DEFAULT_ELF_NAME)),
        wifi_credentials: Some(write_wifi_credentials(&dir, "LabNet", "test-only")),
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    run_flash(&command, &environment).expect("flash workflow");

    // Assert
    assert_eq!(*environment.application_exit_write_counts.borrow(), vec![2]);
}

#[test]
fn canonical_flash_propagates_failed_final_rom_exit() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = FlashCommand {
        factory_reset: false,
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(write_manifest_v4(&dir, DEFAULT_ELF_NAME)),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment {
        application_exit_failure: true,
        ..FakeFlashEnvironment::default()
    };

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    assert!(result.is_err());
    assert_eq!(*environment.application_exit_write_counts.borrow(), vec![1]);
}

#[test]
fn start_installed_cli_accepts_only_the_no_write_identity_contract() {
    // Arrange
    let source = "1".repeat(40);
    let digest = "2".repeat(64);
    let args = [
        "bitaxe-flash",
        "native-usb-start-installed",
        "--board",
        "205",
        "--port",
        "/dev/test-only",
        "--expected-source-commit",
        &source,
        "--expected-app-elf-sha256",
        &digest,
        "--private-root",
        "scratch/new-attempt",
        "--redact-evidence",
    ];

    // Act / Assert
    assert!(parse_cli(args).is_ok());
    assert!(parse_cli(args.into_iter().chain(["--factory-reset"])).is_err());
}

#[test]
fn dry_run_flash_with_explicit_image_renders_vector_command() {
    // Arrange
    let command = FlashCommand {
        factory_reset: false,
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
fn explicit_factory_reset_provisions_wifi_after_the_factory_write() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let credentials_path = write_wifi_credentials(&dir, "LabNet", "super-secret");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let command = FlashCommand {
        factory_reset: true,
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
    let commands = environment.executed_commands();
    assert_eq!(commands.len(), 2);
    assert!(commands[0].args.iter().any(|arg| arg == "write_flash"));
    assert!(commands[0]
        .args
        .windows(2)
        .any(|pair| pair == ["0x0", executed_flash_path]));
    assert_eq!(commands[1], nvs_seed.command);
    assert!(commands[1]
        .args
        .windows(2)
        .any(|pair| pair == ["--after", "no-reset"]));

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
fn noise_diagnostic_seed_contains_only_transport_marker_and_v2_pool_keys() {
    // Arrange
    let credentials = WifiCredentials {
        ssid: "wifi-canary".to_owned(),
        wifi_pass: "wifi-secret-canary".to_owned(),
    };
    let pool = crate::campaign::admission::validate_pool_credentials(
        crate::campaign::admission::PoolCredentialsFile {
            pool_url: "fixture-host-canary".to_owned(),
            pool_port: 1234,
            pool_user: "fixture-user-canary".to_owned(),
            pool_password: String::new(),
            stratum_protocol: Some("SV2".to_owned()),
            stratum_v2_channel_type: Some("standard".to_owned()),
            stratum_v2_authority_pubkey: Some(
                bitaxe_stratum::v2::authority::encode_authority_public_key([0x22; 32]),
            ),
        },
    )
    .expect("V2 pool");
    let seed = NoiseDiagnosticNvsSeed { lease: 7, pool };

    // Act
    let csv = wifi_nvs_csv_for_mode(&credentials, WifiNvsSeedMode::NoiseDiagnostic(seed));

    // Assert
    assert!(csv.contains("sv2diagkind,data,string,stratum_v2_noise_v1"));
    assert!(csv.contains("sv2diaglease,data,u64,7"));
    assert!(csv.contains("sv2diagcase,data,string,noise_auth_v1"));
    assert!(csv.contains("stratumprot,data,string,SV2"));
    assert!(csv.contains("sv2chantype,data,string,standard"));
    assert!(csv.contains("mineonboot,data,u16,0"));
    for prohibited in [
        "campstage",
        "campprofile",
        "camplease",
        "campdurms",
        "selftestkind",
    ] {
        assert!(!csv.contains(prohibited));
    }
}

#[test]
fn tcp_payload_seed_contains_only_fixed_payload_marker_and_fixture_keys() {
    // Arrange
    let credentials = WifiCredentials {
        ssid: "wifi-canary".to_owned(),
        wifi_pass: "wifi-secret-canary".to_owned(),
    };
    let pool = crate::campaign::admission::validate_pool_credentials(
        crate::campaign::admission::PoolCredentialsFile {
            pool_url: "fixture-host-canary".to_owned(),
            pool_port: 1234,
            pool_user: "fixture-user-canary".to_owned(),
            pool_password: String::new(),
            stratum_protocol: Some("SV2".to_owned()),
            stratum_v2_channel_type: Some("standard".to_owned()),
            stratum_v2_authority_pubkey: Some(
                bitaxe_stratum::v2::authority::encode_authority_public_key([0x22; 32]),
            ),
        },
    )
    .expect("V2 fixture");
    let seed = TcpPayloadDiagnosticNvsSeed { lease: 9, pool };

    // Act
    let csv = wifi_nvs_csv_for_mode(&credentials, WifiNvsSeedMode::TcpPayloadDiagnostic(seed));

    // Assert
    assert!(csv.contains("tcpdiagkind,data,string,str005_tcp_v1"));
    assert!(csv.contains("tcpdiaglease,data,u64,9"));
    assert!(csv.contains("tcpdiagcase,data,string,fixed_64_v1"));
    assert!(csv.contains("mineonboot,data,u16,0"));
    for prohibited in ["sv2diagkind", "campstage", "selftestkind"] {
        assert!(!csv.contains(prohibited));
    }
}

#[test]
fn thermal_fault_nvs_tuple_is_exact_and_ordinary_mode_has_no_stimulus() {
    // Arrange
    let credentials = WifiCredentials {
        ssid: "test-network".to_owned(),
        wifi_pass: "test-password".to_owned(),
    };
    let seed = ThermalFaultNvsSeed {
        lease: 42,
        sample_count: 5,
    };

    // Act
    let ordinary = wifi_nvs_csv_for_mode(&credentials, WifiNvsSeedMode::Ordinary);
    let stimulus = wifi_nvs_csv_for_mode(&credentials, WifiNvsSeedMode::ThermalFaultStimulus(seed));

    // Assert
    for key in ["thermfault", "thermlease", "thermcount"] {
        assert!(!ordinary.contains(key));
    }
    assert!(stimulus.contains("thermfault,data,string,emc2101_invalid_sample"));
    assert!(stimulus.contains("thermlease,data,u64,42"));
    assert!(stimulus.contains("thermcount,data,u16,5"));
    assert!(stimulus.contains("mineonboot,data,u16,0"));
}

#[cfg(unix)]
#[test]
fn thermal_fault_intent_binds_private_mode_plan_and_exact_package() {
    use std::os::unix::fs::PermissionsExt;

    // Arrange
    let dir = tempdir().expect("tempdir");
    let root = dir_path(&dir);
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let attempt_root = root.join("scratch/thr001-emc2101-fault/attempt-007");
    std::fs::create_dir_all(attempt_root.as_std_path()).expect("attempt root");
    std::fs::set_permissions(
        attempt_root.as_std_path(),
        std::fs::Permissions::from_mode(0o700),
    )
    .expect("attempt root mode");
    let plan = root.join(THERMAL_FAULT_PLAN_RELATIVE_PATH);
    std::fs::create_dir_all(plan.parent().expect("plan parent").as_std_path())
        .expect("plan parent");
    let plan_document = "fixture thermal fault plan\n";
    std::fs::write(plan.as_std_path(), plan_document).expect("plan");
    let plan_sha256 = sha256_bytes(plan_document.as_bytes());
    let intent = attempt_root.join("thermal-fault-intent.private.json");
    std::fs::write(
        intent.as_std_path(),
        serde_json::json!({
            "schema_version": "esp-thermal-fault-stimulus-intent-v1",
            "board": 205,
            "attempt_ordinal": 7,
            "source_commit": SOURCE_COMMIT,
            "reference_commit": REFERENCE_COMMIT,
            "app_elf_sha256": APP_ELF_SHA256,
            "plan_path": THERMAL_FAULT_PLAN_RELATIVE_PATH,
            "plan_sha256": plan_sha256.clone(),
            "stimulus_kind": "emc2101_invalid_sample",
            "sample_count": 5,
            "lease_hex": "000000000000002a"
        })
        .to_string(),
    )
    .expect("intent");
    std::fs::set_permissions(intent.as_std_path(), std::fs::Permissions::from_mode(0o600))
        .expect("intent mode");
    let environment = FakeFlashEnvironment::default().with_workspace_dir(root);

    // Act
    let admitted = admit_thermal_fault_stimulus_intent_with_plan_sha256(
        Utf8Path::new(THERMAL_FAULT_INTENT_RELATIVE_PATH),
        Some(&manifest),
        BoardId::Ultra205,
        &environment,
        &plan_sha256,
    )
    .expect("strict intent admission");

    // Assert
    assert_eq!(admitted.lease, 42);
    assert_eq!(admitted.sample_count, 5);
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
