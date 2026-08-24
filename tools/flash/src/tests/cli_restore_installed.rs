use super::*;

#[test]
fn parses_only_the_explicit_installed_restore_surface() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "restore-installed",
        "--board",
        "205",
        "--port",
        "/dev/cu.usbmodem101",
        "--restore-bundle",
        RESTORE_BUNDLE_RELATIVE,
        "--wifi-credentials",
        "wifi-credentials.json",
        "--redact-evidence",
    ];

    // Act
    let cli = parse_cli(args).expect("restore-installed cli");

    // Assert
    let CliCommand::RestoreInstalled(command) = cli.command else {
        panic!("expected restore-installed command");
    };
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(command.port, "/dev/cu.usbmodem101");
    assert_eq!(
        command.restore_bundle,
        Utf8Path::new(RESTORE_BUNDLE_RELATIVE)
    );
    assert!(command.redact_evidence);
}
