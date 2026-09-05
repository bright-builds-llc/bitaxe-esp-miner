use super::*;

#[test]
fn typed_usb_flash_failures_are_not_collapsed_to_generic_flash_failed() {
    // Arrange
    let cases = [
        (
            UsbTerminalCategory::BootloaderConnectFailed,
            CampaignTerminalCategory::BootloaderConnectFailed,
        ),
        (
            UsbTerminalCategory::HandoffCommitTimeout,
            CampaignTerminalCategory::BootloaderConnectFailed,
        ),
        (
            UsbTerminalCategory::FlashFailedBeforeTransfer,
            CampaignTerminalCategory::FlashFailedBeforeTransfer,
        ),
        (
            UsbTerminalCategory::FlashFailedAfterTransfer,
            CampaignTerminalCategory::FlashFailedAfterTransfer,
        ),
        (
            UsbTerminalCategory::RecoveryNotObserved,
            CampaignTerminalCategory::RecoveryNotObserved,
        ),
    ];

    for (usb_category, expected) in cases {
        let diagnostic = UsbCommandDiagnostic {
            schema_version: "esp-usb-command-diagnostic-v1".to_owned(),
            terminal_category: usb_category,
            device_effect_state: UsbDeviceEffectState::None,
            termination: UsbCommandTermination::ExitedFailure,
            attempt_count: 1,
            connection_signature: UsbConnectionSignature::DiagnosticUnavailable,
            stdout_bytes: 0,
            stderr_bytes: 0,
            stdout_sha256: sha256_bytes(&[]),
            stderr_sha256: sha256_bytes(&[]),
            transfer_started: false,
            transfer_completed: false,
            raw_output_included: false,
        };

        // Act
        let failure =
            campaign_flash_failure(Some(&diagnostic), CampaignTerminalCategory::FlashFailed);

        // Assert
        assert_eq!(failure.category, expected);
    }
}

#[test]
fn hardware_preparation_failure_precedes_missing_pool_configuration() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = campaign_command(
        &dir,
        MiningCampaignStage::LiveShare,
        Some(MiningCampaignProfile::Conservative),
    );
    let terminal = campaign_marker_with_failure(
        CampaignMarkerFixture {
            stage: "live-share",
            lease_id: serde_json::Value::Null,
            state: "consumed",
            profile: "conservative",
            active_ms: 0,
            submit_outcome: "none",
            terminal_reason: "production_asic_unavailable",
            safety: "fresh",
            pool_config: "not_read",
            actuation: "safe_stopped",
            safe_stop: "confirmed",
        },
        serde_json::json!({
            "phase": "hardware_preparation",
            "step": "reset_and_detect_exactly_one_chip",
            "detail": "asic_actuation_failed",
            "rollback_step": "wait_for_fresh_temperature_at_or_below_45_c",
            "rollback_detail": "cooling_proof_timed_out",
        }),
    );
    let environment = FakeFlashEnvironment::default().with_log_contents(&campaign_log(&[terminal]));

    // Act
    let error = run_campaign_observation_fixture(&command, &environment)
        .expect_err("typed preparation failure must remain terminal");

    // Assert
    assert!(format!("{error:#}").contains("category=hardware_preparation_failed"));
    let result = read_campaign_result(&command);
    assert_eq!(result["terminal_category"], "hardware_preparation_failed");
    assert_eq!(
        result["campaign_failure"],
        serde_json::json!({
            "phase": "hardware_preparation",
            "step": "reset_and_detect_exactly_one_chip",
            "detail": "asic_actuation_failed",
            "rollback_step": "wait_for_fresh_temperature_at_or_below_45_c",
            "rollback_detail": "cooling_proof_timed_out",
        })
    );
}
