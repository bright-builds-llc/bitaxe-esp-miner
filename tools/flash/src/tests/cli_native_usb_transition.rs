use super::*;

#[test]
fn native_usb_transition_parser_requires_the_closed_no_write_surface() {
    // Arrange
    let args = [
        "flash",
        "verify-native-usb-transition",
        "--board",
        "205",
        "--port",
        "/dev/cu.usbmodem-test",
        "--manifest",
        "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
        "--plan",
        "docs/parity/work-plans/20260830T142327Z-NATIVE-USB-RECOVERY-TRANSITION/PLAN.md",
        "--private-root",
        "scratch/native-usb-transition/diagnostic-001",
        "--projection",
        "docs/parity/evidence/native-usb-transition/transition-projection-001.json",
        "--redact-evidence",
    ];

    // Act
    let cli = parse_cli(args).expect("native USB transition CLI");
    let CliCommand::VerifyNativeUsbTransition(command) = cli.command else {
        panic!("expected native USB transition command");
    };

    // Assert
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(command.port, "/dev/cu.usbmodem-test");
    assert!(command.redact_evidence);
    assert_eq!(
        command.private_root,
        Utf8PathBuf::from("scratch/native-usb-transition/diagnostic-001")
    );
}
