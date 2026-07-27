use super::*;

const LICENSE_INVENTORY: &str = r#"
# Phase 7 Release License Inventory

## Cargo crates

- Report: `docs/release/cargo-about.html`
- Owner: release tooling
- Follow-up: regenerate before release

## Bazel and rules

- Owner: release tooling
- Follow-up: review Bzlmod inputs before release

## ESP-IDF and esp-rs

- Owner: firmware release
- Follow-up: review linked components before release

## Flashing tools

- Owner: firmware release
- Follow-up: review espflash and image generation tools before release

## Static assets

- Owner: firmware release
- Follow-up: review static asset source paths before release

## Release artifacts

- Owner: release tooling
- Follow-up: review checksums before publication
"#;

const PROVENANCE_MANIFEST: &str = r#"
# Phase 7 Release Provenance Manifest

## Source commit

- Owner: release tooling
- Follow-up: record the source commit before release

## Reference commit

- Owner: release tooling
- Follow-up: record the pinned reference commit before release

## Static asset source

- Owner: firmware release
- Follow-up: record static source paths before release

## Recovery page source

- Owner: firmware release
- Follow-up: record recovery source path before release

## GPL review status

- Owner: release reviewer
- Follow-up: complete GPL review before release

## Release artifact review

- Owner: release reviewer
- Follow-up: review artifact checksums before release
"#;

#[test]
fn release_gate_passes_complete_inventory_and_provenance() {
    // Arrange
    let documents = complete_documents();

    // Act
    let report = validate_release_gate(&documents);
    let output = render_release_gate_report(&report);

    // Assert
    assert!(report.passed(), "{output}");
    assert_eq!(output, "release_gate: passed\n");
}

#[test]
fn release_gate_fails_when_license_section_is_missing() {
    // Arrange
    let mut documents = complete_documents();
    documents.license_inventory_markdown =
        LICENSE_INVENTORY.replace("## Bazel and rules", "## Bazel inputs");

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("Bazel and rules")));
}

#[test]
fn release_gate_fails_when_provenance_section_is_missing() {
    // Arrange
    let mut documents = complete_documents();
    documents.provenance_markdown =
        PROVENANCE_MANIFEST.replace("## GPL review status", "## GPL notes");

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("GPL review status")));
}

#[test]
fn release_gate_fails_when_cargo_about_report_is_missing() {
    // Arrange
    let mut documents = complete_documents();
    documents.maybe_cargo_about_html = None;

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("cargo-about.html")));
}

#[test]
fn release_gate_fails_when_cargo_about_report_is_empty() {
    // Arrange
    let mut documents = complete_documents();
    documents.maybe_cargo_about_html = Some("   \n".to_owned());

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report.errors.iter().any(|error| error.contains("empty")));
}

#[test]
fn release_gate_fails_when_cargo_about_report_is_not_referenced() {
    // Arrange
    let mut documents = complete_documents();
    documents.license_inventory_markdown = LICENSE_INVENTORY.replace(
        "docs/release/cargo-about.html",
        "docs/release/other-report.html",
    );

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("Cargo crates")));
}

#[test]
fn release_gate_fails_when_unknown_lacks_owner_and_follow_up() {
    // Arrange
    let mut documents = complete_documents();
    documents.license_inventory_markdown =
        LICENSE_INVENTORY.replacen("- Owner: release tooling", "- Status: unknown", 1);

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report.errors.iter().any(|error| {
        error.contains("unknown") && error.contains("owner") && error.contains("follow-up")
    }));
}

#[test]
fn release_gate_checks_unknown_follow_up_per_row() {
    // Arrange
    let mut documents = complete_documents();
    documents.license_inventory_markdown = LICENSE_INVENTORY.replace(
        "## Release artifacts\n\n- Owner: release tooling",
        "## Release artifacts\n\n- Status: unknown\n- Owner: release tooling",
    );

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report.errors.iter().any(|error| {
        error.contains("unknown")
            && error.contains("row-level owner")
            && error.contains("follow-up")
    }));
}

#[test]
fn release_gate_manifest_requires_schema_three() {
    // Arrange
    let mut manifest = valid_manifest_value();
    manifest["schema_version"] = serde_json::json!(1);
    let documents = documents_with_manifest(manifest);

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("schema_version") && error.contains('3')));
}

#[test]
fn release_gate_manifest_rejects_wrong_board_metadata() {
    // Arrange
    let mut manifest = valid_manifest_value();
    manifest["release_name"] = serde_json::json!("bitaxe-gamma601");
    manifest["default_flash_image"] = serde_json::json!("bitaxe-gamma601.elf");
    manifest["image_metadata"]["board"] = serde_json::json!("601");
    manifest["image_metadata"]["device_model"] = serde_json::json!("Gamma 601");
    manifest["image_metadata"]["asic"] = serde_json::json!("BM1370");
    manifest["artifacts"][0]["path"] = serde_json::json!("bitaxe-gamma601.elf");
    manifest["artifacts"][3]["path"] = serde_json::json!("bitaxe-gamma601-factory.bin");
    let documents = documents_with_manifest(manifest);

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("image_metadata.board") && error.contains("205")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("image_metadata.device_model") && error.contains("Ultra 205")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("image_metadata.asic") && error.contains("BM1366")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("bitaxe-ultra205-factory.bin")));
}

#[test]
fn release_gate_manifest_requires_named_ultra205_artifacts() {
    // Arrange
    let mut manifest = valid_manifest_value();
    manifest["artifacts"] = serde_json::json!([
        {
            "kind": "firmware_elf",
            "path": "bitaxe-ultra205.elf",
            "offset": "Unavailable",
            "sha256": "0".repeat(64)
        }
    ]);
    let mut documents = documents_with_manifest(manifest);
    documents.maybe_manifest_path = Some(Utf8PathBuf::from("bazel-bin/package.json"));

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("esp-miner.bin")));
    assert!(report.errors.iter().any(|error| error.contains("www.bin")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("bitaxe-ultra205-factory.bin")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("otadata-initial.bin")));
    assert!(report
        .errors
        .iter()
        .any(|error| error.contains("bitaxe-ultra205-package.json")));
}

#[test]
fn release_gate_manifest_rejects_bad_sha256() {
    // Arrange
    let mut manifest = valid_manifest_value();
    manifest["artifacts"][0]["sha256"] = serde_json::json!("A".repeat(64));
    let documents = documents_with_manifest(manifest);

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report
        .errors
        .iter()
        .any(|error| { error.contains("sha256") && error.contains("64-character lowercase hex") }));
}

#[test]
fn release_gate_manifest_requires_artifact_review_closure() {
    // Arrange
    let mut documents = documents_with_manifest(valid_manifest_value());
    documents.provenance_markdown = PROVENANCE_MANIFEST.replace(
        "- Follow-up: review artifact checksums before release",
        "- Current review status: Awaiting package output evidence.",
    );

    // Act
    let report = validate_release_gate(&documents);

    // Assert
    assert!(!report.passed());
    assert!(report.errors.iter().any(|error| {
        error.contains("Awaiting package output evidence") && error.contains("provenance manifest")
    }));
}

fn complete_documents() -> ReleaseGateDocuments {
    ReleaseGateDocuments {
        license_inventory_path: Utf8PathBuf::from(DEFAULT_LICENSE_INVENTORY_PATH),
        license_inventory_markdown: LICENSE_INVENTORY.to_owned(),
        provenance_path: Utf8PathBuf::from(DEFAULT_PROVENANCE_PATH),
        provenance_markdown: PROVENANCE_MANIFEST.to_owned(),
        cargo_about_path: Utf8PathBuf::from(DEFAULT_CARGO_ABOUT_PATH),
        maybe_cargo_about_html: Some("<html>licenses</html>".to_owned()),
        maybe_manifest_path: None,
        maybe_manifest_json: None,
    }
}

fn documents_with_manifest(manifest: serde_json::Value) -> ReleaseGateDocuments {
    let mut documents = complete_documents();
    documents.maybe_manifest_path = Some(Utf8PathBuf::from(
        "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
    ));
    documents.maybe_manifest_json = Some(serde_json::to_string(&manifest).expect("manifest json"));
    documents
}

fn valid_manifest_value() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 3,
        "release_name": "bitaxe-ultra205",
        "semantic_version": "0.1.0",
        "source_commit": "0123456789abcdef0123456789abcdef01234567",
        "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
        "app_elf_sha256": "6".repeat(64),
        "build_identity": {
            "label": "0123456789ab-dev",
            "channel": "dev",
            "source_dirty": false,
            "release_tag": null
        },
        "default_flash_image": "bitaxe-ultra205.elf",
        "image_metadata": {
            "board": "205",
            "device_model": "Ultra 205",
            "asic": "BM1366",
            "esp_idf_version": "v5.5.4",
            "rust_target": "xtensa-esp32s3-espidf"
        },
        "tool_versions": {
            "cargo": "cargo 1.88.0",
            "rustc": "rustc 1.88.0",
            "bazel": "bazel 9.1.1",
            "espflash": "espflash 4.0.1"
        },
        "install_notes": {
            "path": "docs/release/ultra-205.md",
            "summary": "Ultra 205 release operator guide"
        },
        "license_inventory": "docs/release/license-inventory.md",
        "provenance_manifest": "docs/release/provenance-manifest.md",
        "otadata_source": "generated-erased-flash",
        "artifacts": [
            {
                "kind": "firmware_elf",
                "path": "bitaxe-ultra205.elf",
                "offset": "Unavailable",
                "sha256": "0".repeat(64)
            },
            {
                "kind": "firmware_ota_image",
                "path": "esp-miner.bin",
                "offset": "0x10000",
                "sha256": "1".repeat(64)
            },
            {
                "kind": "www_spiffs_image",
                "path": "www.bin",
                "offset": "0x410000",
                "sha256": "2".repeat(64)
            },
            {
                "kind": "factory_merged_image",
                "path": "bitaxe-ultra205-factory.bin",
                "offset": "0x0",
                "sha256": "3".repeat(64)
            },
            {
                "kind": "partition_table",
                "path": "firmware/bitaxe/partitions-ultra205.csv",
                "offset": "Unavailable",
                "sha256": "4".repeat(64)
            },
            {
                "kind": "otadata_initial",
                "path": "otadata-initial.bin",
                "offset": "0xf10000",
                "sha256": "5".repeat(64)
            }
        ]
    })
}
