use super::*;

#[test]
fn parses_private_identify_confirmation() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "signal-identify",
        "--evidence-dir",
        "hardware-runs/api009/attempt-001/campaign",
        "--checkpoint",
        "ready",
    ];

    // Act
    let cli = parse_cli(args).expect("confirmation cli");

    // Assert
    let CliCommand::SignalIdentify(command) = cli.command else {
        panic!("expected signal-identify command");
    };
    assert_eq!(command.checkpoint, network::IdentifyCheckpointKind::Ready);
    assert_eq!(
        command.outcome,
        network::IdentifyCheckpointOutcome::Confirmed
    );
    assert_eq!(
        command.evidence_dir,
        Utf8PathBuf::from("hardware-runs/api009/attempt-001/campaign")
    );
}

#[test]
fn parses_declined_identify_observation() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "signal-identify",
        "--evidence-dir",
        "hardware-runs/api009/attempt-001/campaign",
        "--checkpoint",
        "rendered",
        "--outcome",
        "declined",
    ];

    // Act
    let cli = parse_cli(args).expect("declined observation cli");

    // Assert
    let CliCommand::SignalIdentify(command) = cli.command else {
        panic!("expected signal-identify command");
    };
    assert_eq!(
        command.outcome,
        network::IdentifyCheckpointOutcome::Declined
    );
}
