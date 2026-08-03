use super::*;

const SOURCE: &str = "1111111111111111111111111111111111111111";
const REFERENCE: &str = "2222222222222222222222222222222222222222";
const ELF: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const PACKAGE: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const CAPABILITY: &str = "5555555555555555555555555555555555555555555555555555555555555555";

#[test]
fn exact_live_versions_project_without_private_transport_fields() {
    // Arrange
    let (manifest_bytes, private_bytes, seal_bytes, handle) = qualified_documents();

    // Act
    let evidence = classify_documents(&manifest_bytes, &private_bytes, &seal_bytes, &handle)
        .expect("qualified evidence should project");
    let serialized = serde_json::to_string(&evidence).expect("projection should serialize");

    // Assert
    assert_eq!(evidence.status, "verified");
    assert_eq!(evidence.build_label, "111111111111-dev");
    for private_field in ["hostname", "ssid", "ipv4", "macAddr", "device_url"] {
        assert!(!serialized.contains(private_field));
    }
}

#[test]
fn stale_static_asset_version_is_rejected() {
    // Arrange
    let (manifest_bytes, private_bytes, seal_bytes, handle) = qualified_documents();
    let mut private: serde_json::Value =
        serde_json::from_slice(&private_bytes).expect("private fixture");
    for field in ["system_info_document", "websocket_document"] {
        let document = private["substantive"][field]
            .as_str()
            .expect("live document")
            .replace("111111111111-dev", "aaaaaaaaaaaa-dev");
        private["substantive"][field] = serde_json::json!(document);
    }
    let changed_private = serde_json::to_vec(&private).expect("changed private fixture");
    let changed_seal = reseal(&seal_bytes, &changed_private);

    // Act
    let result = classify_documents(&manifest_bytes, &changed_private, &changed_seal, &handle);

    // Assert
    assert!(matches!(
        result,
        Err(Sys004VersionEvidenceError::LiveVersion)
    ));
}

#[test]
fn private_input_requires_owner_only_regular_files() {
    // Arrange
    let root = std::env::temp_dir().join(format!("sys004-private-input-{}", std::process::id()));
    let path = Utf8PathBuf::from_path_buf(root).expect("UTF-8 temp path");
    fs::write(&path, b"private").expect("private fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("set unsafe fixture mode");

    // Act
    let result = validate_private_file(&path);

    // Assert
    assert!(matches!(
        result,
        Err(Sys004VersionEvidenceError::PrivateBoundary)
    ));
    fs::remove_file(path).expect("remove private fixture");
}

#[test]
fn sealed_non_promotion_is_classified_before_eligible_capture_is_required() {
    // Arrange
    let (_, _, _, handle) = qualified_documents();
    let seal = AttemptSeal {
        schema_version: "phase36-attempt-seal-v2".to_owned(),
        status: "sealed_non_promotion".to_owned(),
        first_failure: Some(serde_json::json!("flash_failed")),
        secondary_failure: None,
        capability_digest: CAPABILITY.to_owned(),
        package_identity_digest: PACKAGE.to_owned(),
        candidate_digest: None,
        private_capture_digest: None,
    };

    // Act
    let result = require_eligible_attempt_seal(&seal, &handle);

    // Assert
    assert!(matches!(
        result,
        Err(Sys004VersionEvidenceError::AttemptNotEligible)
    ));
}

fn reseal(seal_bytes: &[u8], private_bytes: &[u8]) -> Vec<u8> {
    let mut seal: serde_json::Value = serde_json::from_slice(seal_bytes).expect("seal fixture");
    seal["private_capture_digest"] = serde_json::json!(sha256_hex(private_bytes));
    serde_json::to_vec(&seal).expect("resealed fixture")
}

fn qualified_documents() -> (Vec<u8>, Vec<u8>, Vec<u8>, AttemptHandle) {
    let manifest = serde_json::json!({
        "schema_version": 3,
        "semantic_version": "0.1.0",
        "source_commit": SOURCE,
        "reference_commit": REFERENCE,
        "app_elf_sha256": ELF,
        "build_identity": {
            "label": "111111111111-dev",
            "channel": "dev",
            "source_dirty": false,
            "release_tag": null
        },
        "image_metadata": {
            "board": "205",
            "asic": "BM1366",
            "esp_idf_version": "v5.5.4",
            "rust_target": "xtensa-esp32s3-espidf"
        }
    });
    let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest fixture");
    let manifest_digest = sha256_hex(&manifest_bytes);
    let live = serde_json::json!({
        "ASICModel": "BM1366",
        "boardVersion": "205",
        "version": "111111111111-dev",
        "semanticVersion": "0.1.0",
        "sourceCommit": SOURCE,
        "referenceCommit": REFERENCE,
        "appElfSha256": ELF,
        "buildTimestampUtc": "2026-08-03T00:00:00Z",
        "buildChannel": "dev",
        "sourceDirty": false,
        "releaseTag": null,
        "axeOSVersion": "111111111111-dev",
        "idfVersion": "v5.5.4",
        "platformIdentity": {
            "axeOsStaticAsset": {"state":"available", "value":"111111111111-dev"},
            "espIdfVersion": {"state":"available", "value":"v5.5.4"}
        },
        "hostname": "private-host",
        "ssid": "private-network",
        "ipv4": "192.0.2.1",
        "macAddr": "00:00:00:00:00:00"
    });
    let exact = serde_json::json!({
        "schema_version": "phase36-runtime-package-v1",
        "source_commit": SOURCE,
        "reference_commit": REFERENCE,
        "manifest_digest": manifest_digest,
        "firmware_elf_digest": ELF,
        "package_digest": PACKAGE
    });
    let private = serde_json::json!({
        "schema_version": "phase36-private-capture-v1",
        "board_category": "205",
        "substantive": {
            "system_info_document": format!("system_info_json: {live}\n"),
            "websocket_document": format!("live_websocket_json: {live}\n")
        },
        "runtime_identity": {"exact_package_document": exact.to_string()},
        "broker": {
            "capability_digest": CAPABILITY,
            "package_digest": PACKAGE,
            "same_physical_device_observed": true
        }
    });
    let private_bytes = serde_json::to_vec(&private).expect("private fixture");
    let private_digest = sha256_hex(&private_bytes);
    let seal = serde_json::json!({
        "schema_version": "phase36-attempt-seal-v2",
        "status": "sealed_eligible",
        "first_failure": null,
        "secondary_failure": null,
        "capability_digest": CAPABILITY,
        "package_identity_digest": PACKAGE,
        "candidate_digest": "6".repeat(64),
        "private_capture_digest": private_digest
    });
    let seal_bytes = serde_json::to_vec(&seal).expect("seal fixture");
    let handle = AttemptHandle {
        schema_version: "phase36-attempt-handle-v2".to_owned(),
        child_name: "attempt-0123456789abcdef".to_owned(),
        capability_digest: CAPABILITY.to_owned(),
        source_commit: SOURCE.to_owned(),
        reference_commit: REFERENCE.to_owned(),
        target: "xtensa-esp32s3-espidf".to_owned(),
        board: "205".to_owned(),
        asic: "BM1366".to_owned(),
        manifest_path: "unused-in-pure-test".to_owned(),
        manifest_digest,
        firmware_elf_digest: ELF.to_owned(),
        package_identity_digest: PACKAGE.to_owned(),
    };
    (manifest_bytes, private_bytes, seal_bytes, handle)
}
