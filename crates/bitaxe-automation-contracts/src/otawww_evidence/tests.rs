use serde_json::{json, Value};

use super::OtawwwEvidence;

fn ready_session() -> Value {
    json!({
        "schema_version": "esp-device-session-v1",
        "terminal_category": "ready",
        "platform_category": "macos",
        "board_category": "205",
        "same_physical_device": true,
        "stable_enumeration": true,
        "reenumerated": true,
        "reader_armed": true,
        "pre_restart_serial_delivery": true,
        "post_restart_serial_delivery": true,
        "serial_delivery": "observed",
        "request_outcome": "response_received",
        "request_attempt_count": 1,
        "service_loss_observed": true,
        "trusted_origin_preserved": true,
        "application_recovered": true,
        "build_identity_matches": true,
        "boot_session_changed": true,
        "boot_ordinal_advanced_by_one": true,
        "software_reset_observed": true,
        "postcondition_matches": true,
        "cleanup_complete": true,
        "usb_disappearance_count": 1,
        "enumeration_change_count": 1,
        "serial_byte_count": 4096,
        "http_observation_count": 3,
        "duration_millis": 9000
    })
}

fn complete_evidence() -> Value {
    json!({
        "schema_version": "bitaxe-otawww-evidence-v1",
        "board": 205,
        "source_commit": "a".repeat(40),
        "reference_commit": "b".repeat(40),
        "package_manifest_sha256": "1".repeat(64),
        "www_probe_metadata_sha256": "2".repeat(64),
        "package_www_sha256": "3".repeat(64),
        "probe_www_sha256": "4".repeat(64),
        "workflow": {
            "schema_version": "bitaxe-workflow-identity-v1",
            "command": "capture-otawww-evidence",
            "request_sha256": "5".repeat(64)
        },
        "detector_admitted": true,
        "otawww": {
            "update_request_count": 3,
            "probe_update_response_complete": true,
            "probe_finished_status_retained": true,
            "probe_version_reported": true,
            "probe_version_txt_digest_matches": true,
            "probe_index_html_digest_matches": true,
            "interrupted_upload_attempt_count": 1,
            "interrupted_upload_prefix_bytes": 4096,
            "interruption_protocol_error_retained": true,
            "interruption_boot_session_unchanged": true,
            "interrupted_assets_unavailable_after_restart": true,
            "recovery_page_served_after_interruption": true,
            "recovery_update_response_complete": true,
            "recovery_version_reported": true,
            "recovery_version_txt_digest_matches": true,
            "recovery_index_html_digest_matches": true,
            "build_identity_unchanged": true,
            "hostname_unchanged": true,
            "settings_digest_unchanged": true
        },
        "probe_restart_session": ready_session(),
        "interrupted_restart_session": ready_session(),
        "recovery_restart_session": ready_session(),
        "mining_state": "disabled",
        "hardware_control_state": "disabled",
        "cleanup_complete": true,
        "recovery_flash_used": false,
        "private_modes_valid": true,
        "redaction_status": "passed"
    })
}

fn parse(value: Value) -> OtawwwEvidence {
    serde_json::from_value(value).expect("fixture must match the contract shape")
}

#[test]
fn complete_evidence_validates() {
    // Arrange
    let evidence = parse(complete_evidence());

    // Act
    let result = evidence.validate();

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn missing_interruption_observation_is_rejected() {
    // Arrange
    let mut value = complete_evidence();
    value["otawww"]["interruption_protocol_error_retained"] = json!(false);

    // Act
    let result = parse(value).validate();

    // Assert
    assert_eq!(
        result,
        Err("OTAWWW update, interruption, or recovery observation is incomplete")
    );
}

#[test]
fn changed_settings_are_rejected() {
    // Arrange
    let mut value = complete_evidence();
    value["otawww"]["settings_digest_unchanged"] = json!(false);

    // Act
    let result = parse(value).validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn probe_identical_to_package_is_rejected() {
    // Arrange
    let mut value = complete_evidence();
    value["probe_www_sha256"] = value["package_www_sha256"].clone();

    // Act
    let result = parse(value).validate();

    // Assert
    assert_eq!(
        result,
        Err("OTAWWW probe image is not distinct from the package image")
    );
}

#[test]
fn incomplete_restart_session_is_rejected() {
    // Arrange
    let mut value = complete_evidence();
    value["interrupted_restart_session"]["software_reset_observed"] = json!(false);

    // Act
    let result = parse(value).validate();

    // Assert
    assert_eq!(result, Err("OTAWWW restart device session is incomplete"));
}

#[test]
fn recovery_flash_fallback_is_never_success() {
    // Arrange
    let mut value = complete_evidence();
    value["recovery_flash_used"] = json!(true);

    // Act
    let result = parse(value).validate();

    // Assert
    assert_eq!(
        result,
        Err("OTAWWW safety, restoration, or redaction evidence is invalid")
    );
}

#[test]
fn other_workflow_command_is_rejected() {
    // Arrange
    let mut value = complete_evidence();
    value["workflow"]["command"] = json!("capture-sdkconfig-rollback-evidence");

    // Act
    let result = parse(value).validate();

    // Assert
    assert_eq!(result, Err("OTAWWW workflow identity is invalid"));
}
