use super::*;

#[test]
fn flash_cli_parses_optional_physical_binding_and_rejects_malformed_digest() {
    // Arrange
    let digest = "6".repeat(64);
    // Act
    let cli = Cli::try_parse_from([
        "bitaxe-flash",
        "flash",
        "--expected-physical-sha256",
        &digest,
    ])
    .expect("physical binding");
    // Assert
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    assert_eq!(
        command.common.maybe_expected_physical_sha256.as_deref(),
        Some(digest.as_str())
    );
    assert!(Cli::try_parse_from([
        "bitaxe-flash",
        "flash-monitor",
        "--expected-physical-sha256",
        "invalid"
    ])
    .is_err());
}
