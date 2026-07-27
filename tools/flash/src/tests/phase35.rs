use super::*;

#[test]
fn phase35_probe_parses_bounded_read_contract() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "phase35-probe",
        "board=205",
        "port=/dev/cu.usbmodem101",
        "stage-root=scratch/probe",
        "timeout-seconds=30",
    ];

    // Act
    let cli = parse_cli(args).expect("probe cli");

    // Assert
    let CliCommand::Phase35Probe(command) = cli.command else {
        panic!("expected Phase 35 probe command");
    };
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(command.port, "/dev/cu.usbmodem101");
    assert_eq!(command.stage_root, Utf8PathBuf::from("scratch/probe"));
    assert_eq!(command.timeout_seconds, 30);
}

#[test]
fn phase35_probe_checksum_accepts_espflash_variable_width_hex() {
    // Arrange
    let full_width = "Connecting...\n0x0123456789abcdef0123456789abcdef\n";
    let leading_zero_elided = "0x123456789abcdef0123456789abcdef\n";
    let shortest = "0x0\n";

    // Act and Assert
    assert!(phase35_probe_checksum_observed(full_width));
    assert!(phase35_probe_checksum_observed(leading_zero_elided));
    assert!(phase35_probe_checksum_observed(shortest));
}

#[test]
fn phase35_probe_checksum_rejects_ambiguous_or_malformed_lines() {
    // Arrange
    let overlong = "0x00123456789abcdef0123456789abcdef\n";
    let embedded = "checksum=0x0123456789abcdef0123456789abcdef\n";
    let uppercase = "0x0123456789ABCDEF0123456789ABCDEF\n";
    let multiple = "0x0123456789abcdef0123456789abcdef\n0x1\n";

    // Act and Assert
    assert!(!phase35_probe_checksum_observed(overlong));
    assert!(!phase35_probe_checksum_observed(embedded));
    assert!(!phase35_probe_checksum_observed(uppercase));
    assert!(!phase35_probe_checksum_observed(multiple));
}

#[test]
fn phase35_probe_command_is_bounded_read_only_and_reset_explicit() {
    // Arrange
    let executable = Utf8Path::new("/opt/espflash");

    // Act
    let command = phase35_probe_command(executable, "/dev/private-device");

    // Assert
    assert_eq!(command.program, "/opt/espflash");
    assert_eq!(
        command.args,
        vec![
            "checksum-md5",
            "--chip",
            "esp32s3",
            "--port",
            "/dev/private-device",
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "hard-reset",
            "--skip-update-check",
            "0x0",
            "4096",
        ]
    );
    assert!(!command
        .args
        .iter()
        .any(|argument| { matches!(argument.as_str(), "write-bin" | "flash" | "erase-flash") }));
}

#[test]
fn phase35_readiness_output_rejects_missing_duplicate_or_raw_fields() {
    // Arrange
    let digest = "a".repeat(64);
    let valid = format!(
        "category=ready\ncombined_identity={digest}\nphysical_identity={digest}\nenumeration_identity={digest}\n"
    );
    let missing =
        format!("category=ready\ncombined_identity={digest}\nphysical_identity={digest}\n");
    let raw = format!(
        "category=ready\ncombined_identity={digest}\nphysical_identity={digest}\nenumeration_identity={digest}\nport=/dev/private\n"
    );

    // Act and Assert
    assert!(validate_phase35_readiness_output(&valid).is_ok());
    assert!(validate_phase35_readiness_output(&missing).is_err());
    assert!(validate_phase35_readiness_output(&raw).is_err());
}

#[cfg(unix)]
#[test]
fn phase35_probe_real_process_uses_private_sanitized_no_clobber_artifacts() {
    use std::os::unix::fs::PermissionsExt;

    // Arrange
    let dir = tempdir().expect("tempdir");
    let workspace =
        Utf8PathBuf::from_path_buf(fs::canonicalize(dir.path()).expect("canonical tempdir"))
            .expect("UTF-8 tempdir");
    let git_status = Command::new("git")
        .current_dir(workspace.as_std_path())
        .args(["init", "--quiet"])
        .status()
        .expect("git init");
    assert!(git_status.success());
    fs::write(workspace.join(".gitignore").as_std_path(), "scratch/\n").expect("gitignore");
    let bin_dir = workspace.join("bin");
    fs::create_dir_all(bin_dir.as_std_path()).expect("bin dir");
    let espflash = bin_dir.join("espflash");
    fs::write(
        espflash.as_std_path(),
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >\"$(dirname \"$0\")/args.log\"\nprintf 'password=probe-secret\\n' >&2\nprintf 'Connecting...\\n0x123456789abcdef0123456789abcdef\\n'\n",
    )
    .expect("fake espflash");
    fs::set_permissions(espflash.as_std_path(), fs::Permissions::from_mode(0o700))
        .expect("fake espflash mode");
    let environment = LocalFlashEnvironment {
        workspace_dir: workspace.clone(),
        espflash_bin: espflash.clone(),
        espflash_version: "espflash 4.5.0".to_owned(),
        espflash_sha256: sha256_bytes(b"fake espflash"),
        usb_session: RefCell::new(None),
    };
    let command = Phase35ProbeCommand {
        board: BoardId::Ultra205,
        port: "/dev/private-device".to_owned(),
        stage_root: Utf8PathBuf::from("scratch/probe"),
        timeout_seconds: 180,
    };

    // Act
    run_phase35_probe(&command, &environment).expect("Phase 35 probe");

    // Assert
    let stage_root = workspace.join("scratch/probe");
    let private_log = stage_root.join("probe.private.log");
    let metrics = stage_root.join("probe.metrics.json");
    let captured = fs::read_to_string(private_log.as_std_path()).expect("private log");
    assert!(captured.contains("password=[redacted]"));
    assert!(!captured.contains("probe-secret"));
    assert!(captured.contains("0x123456789abcdef0123456789abcdef"));
    assert_eq!(
        fs::metadata(private_log.as_std_path())
            .expect("private metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(metrics.as_std_path())
            .expect("metrics metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let args = fs::read_to_string(bin_dir.join("args.log").as_std_path()).expect("args");
    assert_eq!(
        args,
        "checksum-md5\n--chip\nesp32s3\n--port\n/dev/private-device\n--non-interactive\n--before\nusb-reset\n--after\nhard-reset\n--skip-update-check\n0x0\n4096\n"
    );
    let second = run_phase35_probe(&command, &environment).expect_err("no clobber");
    assert!(format!("{second:#}").contains("destination_exists"));
}
