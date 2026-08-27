use super::*;

#[test]
fn parses_private_noise_diagnostic_command_without_campaign_fields() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "noise-diagnostic",
        "--board",
        "205",
        "--port",
        "/dev/cu.usbmodem101",
        "--manifest",
        "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
        "--wifi-credentials",
        "wifi-credentials.json",
        "--pool-credentials",
        "scratch/str005-noise-diagnostic/diagnostic-001/fixture-pool.private.json",
        "--intent",
        "scratch/str005-noise-diagnostic/diagnostic-001/intent.private.json",
        "--capture-timeout-seconds",
        "120",
        "--redact-evidence",
    ];

    // Act
    let cli = parse_cli(args).expect("noise diagnostic CLI");

    // Assert
    let CliCommand::NoiseDiagnostic(command) = cli.command else {
        panic!("expected noise diagnostic command");
    };
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(command.port, "/dev/cu.usbmodem101");
    assert_eq!(command.capture_timeout_seconds, 120);
    assert!(command.redact_evidence);
}
