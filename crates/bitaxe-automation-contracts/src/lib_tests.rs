use super::*;

#[test]
fn contract_bundle_has_semantic_schema_names() {
    // Arrange
    let bundle = contract_bundle();
    // Act
    let encoded = serde_json::to_value(bundle).expect("bundle should serialize");
    // Assert
    assert_eq!(encoded["schema_version"], CONTRACT_BUNDLE_SCHEMA);
    assert_eq!(encoded["commands"][0], "doctor");
    assert!(encoded["evidence_schemas"]
        .as_array()
        .expect("evidence schemas should be an array")
        .iter()
        .all(|schema| !schema.as_str().unwrap_or_default().contains("phase")));
}

#[test]
fn version_evidence_accepts_legacy_base_projection() {
    // Arrange
    let evidence = valid_version_evidence(None);
    // Act
    let result = evidence.validate();
    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn version_evidence_rejects_a_failed_live_projection_comparison() {
    // Arrange
    let mut projection = valid_version_projection();
    projection.websocket_version_projection_matches_api = false;
    let evidence = valid_version_evidence(Some(projection));
    // Act
    let result = evidence.validate();
    // Assert
    assert_eq!(
        result,
        Err("version evidence projection comparison is invalid")
    );
}

#[test]
fn operator_snapshot_evidence_requires_two_complete_epochs_and_ready_restart() {
    // Arrange
    let valid = valid_operator_snapshot_evidence();
    let mut invalid = valid.clone();
    invalid.restart_session.request_attempt_count = 2;
    // Act
    let accepted = valid.validate();
    let rejected = invalid.validate();
    // Assert
    assert_eq!(accepted, Ok(()));
    assert_eq!(
        rejected,
        Err("operator snapshot restart transaction is incomplete")
    );
}

fn valid_version_evidence(
    version_projection: Option<VersionProjectionEvidence>,
) -> VersionEvidence {
    VersionEvidence {
        schema_version: VERSION_EVIDENCE_SCHEMA.to_owned(),
        board: 205,
        source_commit: "a".repeat(40),
        reference_commit: "b".repeat(40),
        package_manifest_sha256: "c".repeat(64),
        workflow: WorkflowIdentity {
            schema_version: "bitaxe-workflow-identity-v1".to_owned(),
            command: AutomationCommand::CaptureVersionEvidence,
            request_sha256: "d".repeat(64),
        },
        boot_observed: true,
        same_origin_api_observed: true,
        mining_state: "disabled".to_owned(),
        hardware_control_state: "disabled".to_owned(),
        redaction_status: "passed".to_owned(),
        version_projection,
    }
}

fn valid_version_projection() -> VersionProjectionEvidence {
    VersionProjectionEvidence {
        api_build_label_matches_manifest: true,
        api_static_asset_version_matches_manifest: true,
        api_extended_provenance_matches_manifest: true,
        api_esp_idf_version_matches_manifest: true,
        websocket_same_boot_revision_observed: true,
        websocket_version_projection_matches_api: true,
    }
}

fn valid_operator_snapshot_evidence() -> OperatorSnapshotEvidence {
    let epoch = |session: char, projection: char| OperatorSnapshotEpochEvidence {
        boot_session_sha256: session.to_string().repeat(64),
        http_snapshot_observed: true,
        websocket_snapshot_observed: true,
        same_boot_session: true,
        http_revision: 7,
        websocket_revision: 8,
        websocket_revision_not_earlier: true,
        retained_log_marker_matches_http: true,
        retained_log_marker_matches_websocket: true,
        substantive_fields_present: true,
        stable_fields_match: true,
        safe_operator_state_confirmed: true,
        substantive_projection_sha256: projection.to_string().repeat(64),
    };
    OperatorSnapshotEvidence {
        schema_version: OPERATOR_SNAPSHOT_EVIDENCE_SCHEMA.to_owned(),
        board: 205,
        source_commit: "a".repeat(40),
        reference_commit: "b".repeat(40),
        package_manifest_sha256: "c".repeat(64),
        workflow: WorkflowIdentity {
            schema_version: "bitaxe-workflow-identity-v1".to_owned(),
            command: AutomationCommand::CaptureOperatorSnapshotEvidence,
            request_sha256: "d".repeat(64),
        },
        baseline_epoch: epoch('1', 'e'),
        post_restart_epoch: epoch('2', 'f'),
        distinct_boot_sessions: true,
        restart_session: DeviceSessionEvidence {
            schema_version: "esp-device-session-v1".to_owned(),
            terminal_category: "ready".to_owned(),
            platform_category: "macos".to_owned(),
            board_category: "205".to_owned(),
            same_physical_device: true,
            stable_enumeration: true,
            reenumerated: false,
            reader_armed: true,
            pre_restart_serial_delivery: true,
            post_restart_serial_delivery: true,
            serial_delivery: "correlated".to_owned(),
            request_outcome: "response_received".to_owned(),
            request_attempt_count: 1,
            service_loss_observed: true,
            trusted_origin_preserved: true,
            application_recovered: true,
            build_identity_matches: true,
            boot_session_changed: true,
            boot_ordinal_advanced_by_one: true,
            software_reset_observed: true,
            postcondition_matches: true,
            cleanup_complete: true,
            usb_disappearance_count: 0,
            enumeration_change_count: 0,
            serial_byte_count: 128,
            http_observation_count: 3,
            duration_millis: 1_000,
        },
        mining_state: "disabled".to_owned(),
        hardware_control_state: "disabled".to_owned(),
        cleanup_complete: true,
        redaction_status: "passed".to_owned(),
    }
}
