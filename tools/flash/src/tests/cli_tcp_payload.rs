use super::*;

#[test]
fn parses_private_tcp_payload_diagnostic_without_campaign_fields() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "tcp-payload-diagnostic",
        "--board",
        "205",
        "--port",
        "/dev/cu.usbmodem101",
        "--manifest",
        "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
        "--wifi-credentials",
        "wifi-credentials.json",
        "--pool-credentials",
        "scratch/str005-tcp-payload/diagnostic-006/fixture-pool.private.json",
        "--intent",
        "scratch/str005-tcp-payload/diagnostic-006/intent.private.json",
        "--capture-timeout-seconds",
        "360",
        "--redact-evidence",
    ];

    // Act
    let cli = parse_cli(args).expect("TCP payload diagnostic CLI");

    // Assert
    let CliCommand::TcpPayloadDiagnostic(command) = cli.command else {
        panic!("expected TCP payload diagnostic command");
    };
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(command.capture_timeout_seconds, 360);
    assert!(command.redact_evidence);
}
