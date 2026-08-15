use bitaxe_device_session::{
    DisplayUatIntent, DISPLAY_UAT_ADMISSION_SCHEMA, DISPLAY_UAT_INTENT_SCHEMA,
};

use super::*;

#[test]
fn built_cli_creates_private_root_before_typing_missing_fresh_origin() {
    // Arrange
    let temporary = tempfile::tempdir().expect("temporary directory must be available");
    let temporary = Utf8Path::from_path(temporary.path()).expect("temporary path must be UTF-8");
    let intent_path = temporary.join("display-intent.json");
    let observation_path = temporary.join("runtime-observation.log");
    let evidence_path = temporary.join("programmatic-evidence.json");
    let private_root = temporary.join("private-root");
    let programmatic_evidence =
        br#"{"schema_version":"bitaxe-api-command-effects-evidence-v1","redaction_status":"passed"}"#;
    write_private_json(
        &intent_path,
        &DisplayUatIntent {
            schema_version: DISPLAY_UAT_INTENT_SCHEMA.to_owned(),
            board_category: "205".to_owned(),
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            app_elf_sha256: "c".repeat(64),
            programmatic_evidence_sha256: sha256(programmatic_evidence),
        },
    );
    fs::write(evidence_path.as_std_path(), programmatic_evidence)
        .expect("programmatic evidence must be writable");
    fs::write(
        observation_path.as_std_path(),
        b"receive-only capture without a runtime origin\n",
    )
    .expect("runtime observation must be writable");
    #[cfg(unix)]
    fs::set_permissions(
        observation_path.as_std_path(),
        fs::Permissions::from_mode(0o600),
    )
    .expect("runtime observation mode must be set");
    assert!(!private_root.exists());

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_device-session"))
        .args([
            "display-uat-live",
            "--port",
            "/dev/private-device",
            "--private-root",
            private_root.as_str(),
            "--intent-input",
            intent_path.as_str(),
            "--runtime-observation-input",
            observation_path.as_str(),
            "--programmatic-evidence",
            evidence_path.as_str(),
        ])
        .output()
        .expect("device-session CLI must launch");

    // Assert
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr must be UTF-8"),
        "device_session_status=failed category=evidence_invalid\n"
    );
    let admission_path = private_root.join("display-uat-admission.private.json");
    let admission: serde_json::Value = serde_json::from_slice(
        &fs::read(admission_path.as_std_path()).expect("admission result must exist"),
    )
    .expect("admission result must be JSON");
    assert_eq!(admission["schema_version"], DISPLAY_UAT_ADMISSION_SCHEMA);
    assert_eq!(admission["terminal_category"], "evidence_invalid");
    assert_eq!(admission["identify_request_count"], 0);
    assert!(private_root.is_dir());
    assert_eq!(
        fs::read_dir(private_root.as_std_path())
            .expect("private root must remain readable")
            .count(),
        1
    );
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(private_root.as_std_path())
            .expect("private root metadata")
            .mode()
            & 0o777,
        0o700
    );
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(admission_path.as_std_path())
            .expect("admission metadata")
            .mode()
            & 0o777,
        0o600
    );
}
