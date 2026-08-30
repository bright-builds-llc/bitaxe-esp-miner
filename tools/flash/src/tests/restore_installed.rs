use super::*;

use std::fs;

fn write_protected(path: &Utf8Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
    fs::write(path, bytes).expect("fixture write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("fixture mode");
    }
}

#[test]
fn historical_snapshot_restore_is_admitted_from_clean_current_host() {
    // Arrange
    let directory = tempdir().expect("restore fixture");
    let workspace = Utf8PathBuf::from_path_buf(directory.path().to_owned()).expect("utf8 fixture");
    let plan_path = workspace.join(RESTORE_PLAN_RELATIVE);
    let plan = b"immutable recovery plan\n";
    write_protected(&plan_path, plan);
    let cargo_repository = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let current_repository =
        Utf8PathBuf::from_path_buf(std::env::current_dir().expect("test current directory"))
            .expect("utf8 test directory");
    let remediation_plan = [current_repository, cargo_repository]
        .into_iter()
        .map(|root| root.join(REMEDIATION_PLAN_RELATIVE))
        .find_map(|candidate| fs::read(candidate).ok())
        .expect("checked-in remediation plan");
    write_protected(
        &workspace.join(REMEDIATION_PLAN_RELATIVE),
        &remediation_plan,
    );
    let bundle_path = workspace.join(RESTORE_BUNDLE_RELATIVE);
    let root = bundle_path.parent().expect("bundle root");
    fs::create_dir_all(root).expect("bundle root create");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root, fs::Permissions::from_mode(0o700)).expect("bundle root mode");
    }
    let mut ranges = Vec::new();
    for (name, address, size) in RESTORE_RANGES {
        let relative = format!("snapshot/{name}.bin");
        let bytes = vec![0x5a; size as usize];
        write_protected(&root.join(&relative), &bytes);
        ranges.push(serde_json::json!({
            "name": name,
            "address": address,
            "size": size,
            "path": relative,
            "sha256": sha256_bytes(&bytes),
        }));
    }
    let historical_source = "7d5d9504433d54ae28fe853c5827d6dd05693eef";
    let bundle = serde_json::json!({
        "schema_version": "bitaxe-stratum-v2-restore-bundle-v1",
        "kind": "flash_snapshot_v1",
        "board": 205,
        "installed_identity": {
            "source_commit": "1111111111111111111111111111111111111111",
            "reference_commit": REFERENCE_COMMIT,
            "app_elf_sha256": "2222222222222222222222222222222222222222222222222222222222222222",
            "build_timestamp_utc": "2026-08-24T00:00:00Z",
            "semantic_version": "0.1.0",
            "build_label": "111111111111-dev",
            "build_channel": "dev",
            "source_dirty": false,
            "release_tag": null,
            "idf_version": "v5.5.4",
            "running_partition": "factory"
        },
        "ranges": ranges,
        "capture_source_commit": historical_source,
        "plan_sha256": sha256_bytes(plan),
    });
    let bundle_document = format!(
        "{}\n",
        serde_json::to_string_pretty(&bundle).expect("bundle JSON")
    );
    write_protected(&bundle_path, bundle_document.as_bytes());
    let wifi = workspace.join("wifi-credentials.json");
    write_protected(&wifi, br#"{"ssid":"fixture","wifiPass":"password"}"#);
    let esptool = workspace.join(".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py");
    write_protected(&esptool, b"fixture");
    let private_root = workspace.join(EFFECT_ROOT);
    fs::create_dir_all(&private_root).expect("private root");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&private_root, fs::Permissions::from_mode(0o700))
            .expect("private root mode");
    }
    let authorization = serde_json::json!({
        "schema_version": "bitaxe-stratum-v2-restore-authorization-v1",
        "board": 205,
        "ordinal": 5,
        "action": "start",
        "current_source_commit": SOURCE_COMMIT,
        "reference_commit": REFERENCE_COMMIT,
        "bundle_sha256": sha256_bytes(bundle_document.as_bytes()),
        "bundle_capture_source_commit": historical_source,
        "recovery_plan_sha256": sha256_bytes(plan),
        "remediation_plan_sha256": REMEDIATION_PLAN_SHA256,
    });
    let authorization_path = private_root.join("restore-authorization.private.json");
    write_protected(
        &authorization_path,
        serde_json::to_string_pretty(&authorization)
            .expect("authorization JSON")
            .as_bytes(),
    );
    let preflight_root = workspace.join(PREFLIGHT_ROOT);
    fs::create_dir_all(&preflight_root).expect("preflight root");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&preflight_root, fs::Permissions::from_mode(0o700))
            .expect("preflight root mode");
    }
    let mut preflight_authorization = authorization.clone();
    preflight_authorization["action"] = serde_json::json!("preflight");
    write_protected(
        &preflight_root.join("restore-authorization.private.json"),
        serde_json::to_string_pretty(&preflight_authorization)
            .expect("preflight authorization JSON")
            .as_bytes(),
    );
    let campaign_root = workspace.join(CAMPAIGN_RESTORE_ROOT);
    fs::create_dir_all(&campaign_root).expect("campaign restore root");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&campaign_root, fs::Permissions::from_mode(0o700))
            .expect("campaign restore root mode");
    }
    let mut campaign_authorization = authorization.clone();
    campaign_authorization["action"] = serde_json::json!("campaign_restore");
    campaign_authorization["ordinal"] = serde_json::json!(7);
    write_protected(
        &campaign_root.join("restore-authorization.private.json"),
        serde_json::to_string_pretty(&campaign_authorization)
            .expect("campaign authorization JSON")
            .as_bytes(),
    );
    let admission_command = RestoreInstalledCommand {
        board: BoardId::Ultra205,
        port: "/dev/cu.usbmodem101".to_owned(),
        restore_bundle: Utf8PathBuf::from(RESTORE_BUNDLE_RELATIVE),
        restore_authorization: Utf8PathBuf::from(format!(
            "{PREFLIGHT_ROOT}/restore-authorization.private.json"
        )),
        remediation_plan: Utf8PathBuf::from(REMEDIATION_PLAN_RELATIVE),
        private_root: Utf8PathBuf::from(PREFLIGHT_ROOT),
        wifi_credentials: Utf8PathBuf::from("wifi-credentials.json"),
        redact_evidence: true,
        admission_only: true,
    };
    let command = RestoreInstalledCommand {
        board: BoardId::Ultra205,
        port: "/dev/cu.usbmodem101".to_owned(),
        restore_bundle: Utf8PathBuf::from(RESTORE_BUNDLE_RELATIVE),
        restore_authorization: Utf8PathBuf::from(format!(
            "{EFFECT_ROOT}/restore-authorization.private.json"
        )),
        remediation_plan: Utf8PathBuf::from(REMEDIATION_PLAN_RELATIVE),
        private_root: Utf8PathBuf::from(EFFECT_ROOT),
        wifi_credentials: Utf8PathBuf::from("wifi-credentials.json"),
        redact_evidence: true,
        admission_only: false,
    };
    let campaign_command = RestoreInstalledCommand {
        board: BoardId::Ultra205,
        port: "/dev/cu.usbmodem101".to_owned(),
        restore_bundle: Utf8PathBuf::from(RESTORE_BUNDLE_RELATIVE),
        restore_authorization: Utf8PathBuf::from(format!(
            "{CAMPAIGN_RESTORE_ROOT}/restore-authorization.private.json"
        )),
        remediation_plan: Utf8PathBuf::from(REMEDIATION_PLAN_RELATIVE),
        private_root: Utf8PathBuf::from(CAMPAIGN_RESTORE_ROOT),
        wifi_credentials: Utf8PathBuf::from("wifi-credentials.json"),
        redact_evidence: true,
        admission_only: false,
    };
    let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace.clone());

    // Act
    let admission_environment =
        FakeFlashEnvironment::default().with_workspace_dir(workspace.clone());
    let admission = run_restore_installed(&admission_command, &admission_environment);
    let result = run_restore_installed(&command, &environment);
    let campaign_environment =
        FakeFlashEnvironment::default().with_workspace_dir(workspace.clone());
    let campaign_result = run_restore_installed(&campaign_command, &campaign_environment);

    // Assert
    assert!(admission.is_ok(), "{admission:#?}");
    assert!(admission_environment.created_snapshot_paths().is_empty());
    assert!(admission_environment.executed_commands().is_empty());
    assert!(result.is_ok(), "{result:#?}");
    assert_eq!(environment.executed_commands().len(), 2);
    assert!(campaign_result.is_ok(), "{campaign_result:#?}");
    assert_eq!(campaign_environment.executed_commands().len(), 2);

    let mut tampered = authorization;
    tampered["current_source_commit"] = serde_json::json!(historical_source);
    write_protected(
        &authorization_path,
        serde_json::to_string_pretty(&tampered)
            .expect("tampered authorization JSON")
            .as_bytes(),
    );
    let rejected_environment = FakeFlashEnvironment::default().with_workspace_dir(workspace);
    assert!(run_restore_installed(&command, &rejected_environment).is_err());
    assert!(rejected_environment.executed_commands().is_empty());
}

#[test]
fn native_usb_recovery_contract_binds_both_ordinals_to_the_immutable_plan() {
    // Arrange / Act
    let primary =
        authorized_remediation_plan("native_usb_recovery", 2).expect("primary recovery contract");
    let contingency = authorized_remediation_plan("native_usb_recovery", 3)
        .expect("contingency recovery contract");
    let (primary_root, primary_plan) =
        restore_invocation_contract(Utf8Path::new(NATIVE_USB_TRANSITION_PRIMARY_ROOT), false);
    let (contingency_root, contingency_plan) =
        restore_invocation_contract(Utf8Path::new(NATIVE_USB_TRANSITION_CONTINGENCY_ROOT), false);

    // Assert
    assert_eq!(
        primary,
        (
            NATIVE_USB_TRANSITION_PLAN_RELATIVE,
            NATIVE_USB_TRANSITION_PLAN_SHA256
        )
    );
    assert_eq!(contingency, primary);
    assert_eq!(
        primary_root,
        Utf8Path::new(NATIVE_USB_TRANSITION_PRIMARY_ROOT)
    );
    assert_eq!(
        contingency_root,
        Utf8Path::new(NATIVE_USB_TRANSITION_CONTINGENCY_ROOT)
    );
    assert_eq!(primary_plan, NATIVE_USB_TRANSITION_PLAN_RELATIVE);
    assert_eq!(contingency_plan, NATIVE_USB_TRANSITION_PLAN_RELATIVE);
    assert!(authorized_remediation_plan("native_usb_recovery", 4).is_err());
}
