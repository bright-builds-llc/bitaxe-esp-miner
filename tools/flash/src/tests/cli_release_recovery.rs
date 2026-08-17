use super::*;

#[test]
fn parses_closed_release_recovery_paths() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "rel003-large-erase",
        "--private-root",
        RELEASE_RECOVERY_PRIVATE_ROOT,
        "--package-manifest",
        RELEASE_RECOVERY_MANIFEST,
        "--wifi-credentials",
        RELEASE_RECOVERY_WIFI_CREDENTIALS,
        "--detector-output",
        RELEASE_RECOVERY_DETECTOR_OUTPUT,
        "--plan",
        RELEASE_RECOVERY_PLAN,
        "--projection",
        RELEASE_RECOVERY_PROJECTION,
        "--capture-timeout-seconds",
        "360",
    ];

    // Act
    let cli = parse_cli(args).expect("release recovery cli");

    // Assert
    let CliCommand::ReleaseRecovery(command) = cli.command else {
        panic!("expected rel003-large-erase command");
    };
    assert_eq!(command.private_root, RELEASE_RECOVERY_PRIVATE_ROOT);
    assert_eq!(command.package_manifest, RELEASE_RECOVERY_MANIFEST);
    assert_eq!(command.wifi_credentials, RELEASE_RECOVERY_WIFI_CREDENTIALS);
    assert_eq!(command.detector_output, RELEASE_RECOVERY_DETECTOR_OUTPUT);
    assert_eq!(command.plan, RELEASE_RECOVERY_PLAN);
    assert_eq!(command.projection, RELEASE_RECOVERY_PROJECTION);
    assert_eq!(command.capture_timeout_seconds, 360);
}
