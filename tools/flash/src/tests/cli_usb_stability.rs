use super::*;

#[test]
fn usb_stability_read_cli_exposes_only_bounded_read_inputs() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "usb-stability-read",
        "--board",
        "205",
        "--port",
        "/dev/cu.fixture",
        "--restore-bundle",
        "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
        "--private-root",
        "scratch/usb-stability-read/calibration-001",
        "--chunk-bytes",
        "65536",
        "--repetitions",
        "4",
        "--pattern",
        "sequential",
        "--redact-evidence",
    ];

    // Act
    let parsed = parse_cli(args).expect("USB stability read command");

    // Assert
    let CliCommand::UsbStabilityRead(command) = parsed.command else {
        panic!("expected USB stability read command");
    };
    assert_eq!(command.chunk_bytes, 65_536);
    assert_eq!(command.repetitions, 4);
    assert_eq!(command.pattern, UsbStabilityPattern::Sequential);
}
