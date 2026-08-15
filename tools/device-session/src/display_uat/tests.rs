use super::*;

use std::io::{Read, Write};
use std::net::TcpListener;

fn valid_intent(programmatic_evidence: &[u8]) -> DisplayUatIntent {
    DisplayUatIntent {
        schema_version: DISPLAY_UAT_INTENT_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        source_commit: "a".repeat(40),
        reference_commit: "b".repeat(40),
        app_elf_sha256: "c".repeat(64),
        programmatic_evidence_sha256: digest(programmatic_evidence),
    }
}

fn private_root() -> (tempfile::TempDir, camino::Utf8PathBuf) {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let root =
        camino::Utf8PathBuf::from_path_buf(temporary.path().to_path_buf()).expect("UTF-8 root");
    #[cfg(unix)]
    fs::set_permissions(root.as_std_path(), fs::Permissions::from_mode(0o700))
        .expect("private root mode");
    (temporary, root)
}

fn machine_result() -> DisplayUatMachineResult {
    DisplayUatMachineResult {
        schema_version: DISPLAY_UAT_MACHINE_SCHEMA.to_owned(),
        boot_session: "private".to_owned(),
        identify_generation: 1,
        identify_request_count: 1,
        machine_render_confirmed: true,
        machine_clear_confirmed: true,
        build_identity_matches: true,
        usb_admission_confirmed: true,
        programmatic_evidence_sha256: "a".repeat(64),
    }
}

#[test]
fn projection_contains_no_device_or_network_identity() {
    // Arrange
    let projection = DisplayUatProjection {
        schema_version: DISPLAY_UAT_PROJECTION_SCHEMA,
        board: 205,
        identify_request_count: 1,
        machine_render_confirmed: true,
        machine_clear_confirmed: true,
        operator_render_confirmed: true,
        operator_clear_confirmed: true,
        build_identity_matches: true,
        usb_admission_confirmed: true,
        programmatic_evidence_sha256: "a".repeat(64),
        redaction_status: "passed",
    };

    // Act
    let json = serde_json::to_string(&projection).expect("serialize projection");

    // Assert
    for forbidden in [
        "origin",
        "hostname",
        "port",
        "boot_session",
        "source_commit",
    ] {
        assert!(!json.contains(forbidden));
    }
}

#[test]
fn finalize_requires_both_private_operator_confirmations() {
    // Arrange
    let temporary = tempfile::tempdir().expect("temporary directory");
    let root =
        camino::Utf8PathBuf::from_path_buf(temporary.path().to_path_buf()).expect("UTF-8 root");
    let projection = root.join("projection.json");
    write_json_new(
        &root.join("display-uat-machine.private.json"),
        &machine_result(),
    )
    .expect("machine result");
    for checkpoint in ["rendered", "cleared"] {
        write_json_new(
            &root.join(format!("identify-{checkpoint}.response.json")),
            &CheckpointDocument {
                schema: CHECKPOINT_SCHEMA.to_owned(),
                checkpoint: checkpoint.to_owned(),
                status: "confirmed".to_owned(),
            },
        )
        .expect("checkpoint response");
    }

    // Act
    finalize_display_uat(&root, &projection).expect("finalize UAT");

    // Assert
    let public = fs::read_to_string(projection).expect("public evidence");
    assert!(public.contains(DISPLAY_UAT_PROJECTION_SCHEMA));
    assert!(!public.contains("private"));
}

#[test]
fn missing_second_confirmation_does_not_consume_the_first_or_publish() {
    // Arrange
    let temporary = tempfile::tempdir().expect("temporary directory");
    let root =
        camino::Utf8PathBuf::from_path_buf(temporary.path().to_path_buf()).expect("UTF-8 root");
    let projection = root.join("projection.json");
    write_json_new(
        &root.join("display-uat-machine.private.json"),
        &machine_result(),
    )
    .expect("machine result");
    write_json_new(
        &root.join("identify-rendered.response.json"),
        &CheckpointDocument {
            schema: CHECKPOINT_SCHEMA.to_owned(),
            checkpoint: "rendered".to_owned(),
            status: "confirmed".to_owned(),
        },
    )
    .expect("rendered response");

    // Act
    let result = finalize_display_uat(&root, &projection);

    // Assert
    assert!(result.is_err());
    assert!(root.join("identify-rendered.response.json").exists());
    assert!(!projection.exists());
}

#[test]
fn intent_rejects_unsealed_or_noncanonical_identity() {
    // Arrange
    let valid = DisplayUatIntent {
        schema_version: DISPLAY_UAT_INTENT_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        source_commit: "a".repeat(40),
        reference_commit: "b".repeat(40),
        app_elf_sha256: "c".repeat(64),
        programmatic_evidence_sha256: "d".repeat(64),
    };
    let uppercase = DisplayUatIntent {
        app_elf_sha256: "C".repeat(64),
        ..valid.clone()
    };

    // Act and assert
    assert!(valid.schema_is_valid());
    assert!(!uppercase.schema_is_valid());
}

#[test]
fn unavailable_fresh_origin_is_typed_before_identify() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral listener");
    let address = listener.local_addr().expect("listener address");
    drop(listener);
    let programmatic =
        br#"{"schema_version":"bitaxe-api-command-effects-evidence-v1","redaction_status":"passed"}"#;
    let runtime = format!(
        "runtime_origin session={} boot_ordinal=7 device_url=http://{address} redacted=true\n",
        "a".repeat(32)
    );
    let (_temporary, root) = private_root();

    // Act
    let category = run_display_uat_live(
        valid_intent(programmatic),
        "/dev/private-device".to_owned(),
        runtime.as_bytes(),
        programmatic,
        &root,
    )
    .expect("closed unavailable result");

    // Assert
    assert_eq!(category, TerminalCategory::ServiceRecoveryTimeout);
    assert!(!root.join("display-uat-machine.private.json").exists());
}

#[test]
fn malformed_command_status_is_typed_before_identify() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral listener");
    let address = listener.local_addr().expect("listener address");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept command-status request");
        let mut request = [0_u8; 1_024];
        let _ = stream
            .read(&mut request)
            .expect("read command-status request");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .expect("write malformed command status");
    });
    let programmatic =
        br#"{"schema_version":"bitaxe-api-command-effects-evidence-v1","redaction_status":"passed"}"#;
    let runtime = format!(
        "runtime_origin session={} boot_ordinal=7 device_url=http://{address} redacted=true\n",
        "a".repeat(32)
    );
    let (_temporary, root) = private_root();

    // Act
    let category = run_display_uat_live(
        valid_intent(programmatic),
        "/dev/private-device".to_owned(),
        runtime.as_bytes(),
        programmatic,
        &root,
    )
    .expect("closed malformed result");
    server.join().expect("server joins");

    // Assert
    assert_eq!(category, TerminalCategory::EvidenceInvalid);
    assert!(!root.join("display-uat-machine.private.json").exists());
}
