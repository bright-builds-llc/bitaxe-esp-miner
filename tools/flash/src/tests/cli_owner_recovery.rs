use super::*;

#[test]
fn owner_recovery_cli_requires_the_closed_action_and_paths() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "owner-recovery",
        "--board",
        "205",
        "--port",
        "/dev/cu.fixture",
        "--action",
        "observe",
        "--package-manifest",
        OWNER_RECOVERY_MANIFEST,
        "--restore-bundle",
        OWNER_RECOVERY_BUNDLE,
        "--private-root",
        OWNER_RECOVERY_ROOT,
        "--plan",
        OWNER_RECOVERY_PLAN,
        "--redact-evidence",
    ];

    // Act
    let parsed = parse_cli(args).expect("owner recovery command");

    // Assert
    let CliCommand::OwnerRecovery(command) = parsed.command else {
        panic!("expected owner recovery command");
    };
    assert_eq!(command.action, OwnerRecoveryAction::Observe);
    assert!(command.manual_checkpoint.is_none());
}

#[test]
fn owner_recovery_rejects_an_unknown_action() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "owner-recovery",
        "--board",
        "205",
        "--port",
        "/dev/cu.fixture",
        "--action",
        "repeat",
        "--package-manifest",
        OWNER_RECOVERY_MANIFEST,
        "--restore-bundle",
        OWNER_RECOVERY_BUNDLE,
        "--private-root",
        OWNER_RECOVERY_ROOT,
        "--plan",
        OWNER_RECOVERY_PLAN,
        "--redact-evidence",
    ];

    // Act / Assert
    assert!(parse_cli(args).is_err());
}
