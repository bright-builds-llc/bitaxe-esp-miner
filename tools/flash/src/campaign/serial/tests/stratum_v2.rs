use super::*;

#[test]
fn terminal_requires_every_ordered_runtime_boundary_and_safe_stop() {
    // Arrange
    let admission = v2_admission();
    let mut bytes = Vec::new();
    for (sequence, stage) in [
        (1, "hardware_prepared"),
        (2, "channel_ready"),
        (3, "work_dispatched"),
        (4, "share_accepted"),
    ] {
        bytes.extend_from_slice(
            format!(
                "{STRATUM_V2_RUNTIME_PREFIX}{{\"schema\":\"{STRATUM_V2_RUNTIME_SCHEMA}\",\"stage\":\"{stage}\",\"sequence\":{sequence}}}\n"
            )
            .as_bytes(),
        );
    }
    bytes.extend_from_slice(accepted_terminal(true).as_bytes());

    // Act
    let should_stop = campaign_serial_should_stop(&bytes, admission);
    let capture = analyze_campaign_serial_bytes(&bytes, admission);

    // Assert
    assert!(should_stop);
    assert_eq!(capture.maybe_failure, None);
    assert_eq!(
        capture.stratum_v2.assess().expect("accepted V2 terminal"),
        CampaignTerminalCategory::StratumV2Accepted
    );
}

#[test]
fn terminal_without_safe_stop_fails_closed() {
    // Arrange
    let admission = v2_admission();
    let bytes = accepted_terminal(false);

    // Act
    let capture = analyze_campaign_serial_bytes(bytes.as_bytes(), admission);

    // Assert
    let failure = capture
        .stratum_v2
        .assess()
        .expect_err("safe stop must be required");
    assert_eq!(
        failure.category,
        CampaignTerminalCategory::SafeStopUnconfirmed
    );
}

fn v2_admission() -> CampaignAdmission {
    CampaignAdmission {
        stage: MiningCampaignStage::StratumV2,
        maybe_profile: Some(MiningCampaignProfile::Conservative),
        duration_seconds: 180,
        maybe_lease_id: Some(7),
    }
}

fn accepted_terminal(safe_stop_complete: bool) -> String {
    format!(
        "{STRATUM_V2_TERMINAL_PREFIX}{{\"schema\":\"{STRATUM_V2_TERMINAL_SCHEMA}\",\"category\":\"accepted\",\"accepted\":true,\"safe_stop_complete\":{safe_stop_complete}}}\n"
    )
}
