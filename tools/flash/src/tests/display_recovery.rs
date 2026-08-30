use super::*;

#[test]
fn display_recovery_accepts_only_origin_only_rfc1918_ipv4() {
    // Arrange
    let accepted = ["10.0.0.1", "172.16.0.1", "172.31.255.254", "192.168.1.2"];
    let rejected = [
        "0.0.0.0",
        "127.0.0.1",
        "169.254.1.1",
        "172.32.0.1",
        "192.168.1.2:80",
        "http://192.168.1.2",
        "192.168.1.2/path",
        "example.local",
        "::1",
    ];

    // Act / Assert
    for candidate in accepted {
        assert_eq!(
            DisplayIpv4Origin::parse(candidate)
                .expect("RFC1918 address")
                .origin(),
            format!("http://{candidate}")
        );
    }
    for candidate in rejected {
        assert!(DisplayIpv4Origin::parse(candidate).is_err(), "{candidate}");
    }
}

#[test]
fn display_recovery_normalizes_usb_and_api_mac_before_hashing() {
    // Arrange
    let expected = "5fa0cd6c75427a39f3195f822113b763d923e70c268e2bc75b5120608b57a66a";

    // Act / Assert
    assert_eq!(
        display_mac_sha256("02:00:00:00:A1:B1").expect("USB MAC"),
        expected
    );
    assert_eq!(
        display_mac_sha256("02:00:00:00:a1:b1").expect("API MAC"),
        expected
    );
    for invalid in ["02-00-00-00-a1-b1", "02:00:00:00:a1", "not-a-mac"] {
        assert!(display_mac_sha256(invalid).is_err());
    }
}

#[test]
fn display_recovery_builds_one_fail_closed_settings_and_theme_transaction() {
    // Arrange
    let backup = serde_json::json!({
        "settings": {"hostname": "fixture", "frequency": 485, "useFallbackStratum": true},
        "theme": {"colorScheme": "dark", "accentColors": {"primary": "blue"}}
    });
    let wifi = serde_json::json!({"ssid": "fixture-wifi", "wifiPass": "fixture-pass"});
    let pool = serde_json::json!({
        "poolURL": "fixture.pool",
        "poolPort": 3333,
        "poolUser": "fixture-user",
        "poolPassword": "fixture-password"
    });

    // Act
    let transaction = plan_display_restoration(&backup, &wifi, &pool).expect("transaction");

    // Assert
    assert_eq!(transaction.settings["startMiningOnBoot"], false);
    assert_eq!(transaction.settings["useFallbackStratum"], false);
    assert_eq!(transaction.settings["fallbackStratumURL"], "");
    assert_eq!(transaction.settings["hostname"], "fixture");
    assert_eq!(transaction.settings["ssid"], "fixture-wifi");
    assert_eq!(transaction.settings["stratumURL"], "fixture.pool");
    assert_eq!(transaction.theme, backup["theme"]);
}

#[test]
fn display_recovery_cli_requires_every_task_bound_input() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "display-recovery-start",
        "--board",
        "205",
        "--port",
        "/dev/cu.fixture",
        "--package-manifest",
        DISPLAY_RECOVERY_MANIFEST,
        "--restore-bundle",
        DISPLAY_RECOVERY_BUNDLE,
        "--settings-backup",
        DISPLAY_RECOVERY_BACKUP,
        "--wifi-credentials",
        "wifi-credentials.json",
        "--pool-credentials",
        "pool-credentials.json",
        "--capture-input",
        "scratch/native-usb-display-recovery/attempt-001/display-origin-capture-001.private.json",
        "--private-root",
        DISPLAY_RECOVERY_ROOT,
        "--plan",
        DISPLAY_RECOVERY_PLAN,
        "--redact-evidence",
    ];

    // Act
    let cli = parse_cli(args).expect("display recovery CLI");

    // Assert
    let CliCommand::DisplayRecoveryStart(command) = cli.command else {
        panic!("expected display recovery command");
    };
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(command.private_root, Utf8Path::new(DISPLAY_RECOVERY_ROOT));
    assert!(command.redact_evidence);
}

#[test]
fn display_recovery_identity_and_final_state_require_every_bound_field() {
    // Arrange
    let expected = serde_json::json!({
        "source_commit": "1".repeat(40), "reference_commit": "2".repeat(40),
        "app_elf_sha256": "3".repeat(64), "build_timestamp_utc": "2026-08-01T00:00:00Z",
        "build_label": "111111111111-dev", "running_partition": "factory"
    });
    let system = serde_json::json!({
        "sourceCommit": "1".repeat(40), "referenceCommit": "2".repeat(40),
        "appElfSha256": "3".repeat(64), "buildTimestampUtc": "2026-08-01T00:00:00Z",
        "version": "111111111111-dev", "runningPartition": "factory"
    });

    // Act / Assert
    assert!(display_recovery_identity_matches(
        &system,
        expected.as_object().expect("expected identity")
    ));
    let mut drifted = system;
    drifted["runningPartition"] = serde_json::json!("ota_0");
    assert!(!display_recovery_identity_matches(
        &drifted,
        expected.as_object().expect("expected identity")
    ));
}
