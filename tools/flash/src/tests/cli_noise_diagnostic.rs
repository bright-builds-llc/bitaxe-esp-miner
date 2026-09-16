use super::*;

#[test]
fn parses_private_noise_diagnostic_command_without_campaign_fields() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "noise-auth-diagnostic",
        "--board",
        "205",
        "--port",
        "/dev/cu.usbmodem101",
        "--manifest",
        "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
        "--wifi-credentials",
        "wifi-credentials.json",
        "--pool-credentials",
        "scratch/str005-noise-auth/diagnostic-001/fixture-pool.private.json",
        "--intent",
        "scratch/str005-noise-auth/diagnostic-001/intent.private.json",
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

#[test]
fn retired_noise_diagnostic_rejects_before_environment_reads_or_effects() {
    // Arrange
    let command = NoiseDiagnosticCommand {
        board: BoardId::Ultra205,
        port: "/dev/not-opened".to_owned(),
        manifest: "absent-manifest.json".into(),
        wifi_credentials: "absent-wifi.json".into(),
        pool_credentials: "absent-pool.json".into(),
        intent: "absent-intent.json".into(),
        capture_timeout_seconds: 120,
        redact_evidence: true,
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let admission = require_current_noise_effect(&CliCommand::NoiseDiagnostic(command.clone()));
    let execution = run_noise_diagnostic_command(&command, &environment);

    // Assert
    for result in [admission, execution] {
        assert!(result
            .expect_err("retired command")
            .to_string()
            .contains("legacy_effect_retired"));
    }
    assert!(environment.read_string_paths.borrow().is_empty());
    assert!(environment.executed_commands.borrow().is_empty());
    assert!(environment.captured_commands.borrow().is_empty());
    assert!(environment.written_files.borrow().is_empty());
    assert_eq!(environment.list_ports_calls.get(), 0);
}

#[test]
fn retired_noise_restoration_roots_reject_before_input_access() {
    // Arrange
    let roots = [
        NOISE_AUTH_PREFLIGHT_ROOT,
        NOISE_AUTH_DIAGNOSTIC_RESTORE_ROOT,
        NOISE_AUTH_RECOVERY_ROOT,
        NOISE_DIAGNOSTIC_RESTORE_ROOT,
    ];
    for root in roots {
        let environment = FakeFlashEnvironment::default();
        let command = RestoreInstalledCommand {
            board: BoardId::Ultra205,
            port: "/dev/not-opened".to_owned(),
            restore_bundle: "absent-bundle.json".into(),
            restore_authorization: "absent-authorization.json".into(),
            remediation_plan: "absent-plan.md".into(),
            private_root: root.into(),
            wifi_credentials: "absent-wifi.json".into(),
            redact_evidence: true,
            admission_only: root == NOISE_AUTH_PREFLIGHT_ROOT,
        };

        // Act
        let admission =
            require_current_noise_effect(&CliCommand::RestoreInstalled(command.clone()));
        let execution = run_restore_installed(&command, &environment);

        // Assert
        for result in [admission, execution] {
            assert!(result
                .expect_err("retired restoration")
                .to_string()
                .contains("legacy_effect_retired"));
        }
        assert!(environment.read_string_paths.borrow().is_empty());
        assert!(environment.executed_commands.borrow().is_empty());
        assert!(environment.captured_commands.borrow().is_empty());
        assert!(environment.written_files.borrow().is_empty());
        assert_eq!(environment.list_ports_calls.get(), 0);
    }
}
