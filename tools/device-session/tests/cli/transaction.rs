use bitaxe_device_session::{
    DeviceTransactionIntent, RebootIntent, TransactionGoal, REBOOT_INTENT_SCHEMA,
    TRANSACTION_INTENT_SCHEMA,
};

use super::*;

fn transaction_intent() -> DeviceTransactionIntent {
    let request = request();
    DeviceTransactionIntent {
        schema_version: TRANSACTION_INTENT_SCHEMA.to_owned(),
        goal: TransactionGoal::SettingsDurability {
            reboot: RebootIntent {
                schema_version: REBOOT_INTENT_SCHEMA.to_owned(),
                board_category: request.board_category,
                trusted_origin: request.trusted_origin,
                baseline: request.baseline,
                expected_postcondition: request.expected_postcondition,
            },
        },
    }
}

#[test]
fn built_cli_runs_transaction_intent_through_a_real_child_process() {
    // Arrange
    let temporary = tempfile::tempdir().expect("temporary directory must be available");
    let temporary = Utf8Path::from_path(temporary.path()).expect("temporary path must be UTF-8");
    let intent_path = temporary.join("intent.json");
    let fixture_path = temporary.join("fixture.json");
    let private_root = temporary.join("private-root");
    let projection_path = temporary.join("projection.json");
    write_private_json(&intent_path, &transaction_intent());
    write_private_json(&fixture_path, &fixture());
    fs::create_dir(private_root.as_std_path()).expect("private root must be created");
    #[cfg(unix)]
    fs::set_permissions(
        private_root.as_std_path(),
        fs::Permissions::from_mode(0o700),
    )
    .expect("private root mode must be set");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_device-session"))
        .args([
            "transact",
            "--private-root",
            private_root.as_str(),
            "--intent-input",
            intent_path.as_str(),
            "--projection-output",
            projection_path.as_str(),
            "--fixture-input",
            fixture_path.as_str(),
        ])
        .output()
        .expect("device-session CLI must launch");

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let projection: serde_json::Value = serde_json::from_slice(
        &fs::read(projection_path.as_std_path()).expect("projection must exist"),
    )
    .expect("projection must be valid JSON");
    assert_eq!(projection["terminal_category"], "ready");
    let public = serde_json::to_string(&projection).expect("serialize projection");
    for forbidden in ["private-device", "private-session", "fixture-only"] {
        assert!(!public.contains(forbidden));
    }
}
