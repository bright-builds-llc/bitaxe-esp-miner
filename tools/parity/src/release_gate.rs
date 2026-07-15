use camino::Utf8PathBuf;
use serde_json::Value;

use bitaxe_api::BuildProvenance;

pub(crate) const DEFAULT_LICENSE_INVENTORY_PATH: &str = "docs/release/license-inventory.md";
pub(crate) const DEFAULT_PROVENANCE_PATH: &str = "docs/release/provenance-manifest.md";
pub(crate) const DEFAULT_CARGO_ABOUT_PATH: &str = "docs/release/cargo-about.html";

#[derive(Debug)]
pub(crate) struct ReleaseGateDocuments {
    pub(crate) license_inventory_path: Utf8PathBuf,
    pub(crate) license_inventory_markdown: String,
    pub(crate) provenance_path: Utf8PathBuf,
    pub(crate) provenance_markdown: String,
    pub(crate) cargo_about_path: Utf8PathBuf,
    pub(crate) maybe_cargo_about_html: Option<String>,
    pub(crate) maybe_manifest_path: Option<Utf8PathBuf>,
    pub(crate) maybe_manifest_json: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ReleaseGateReport {
    pub(crate) errors: Vec<String>,
}

impl ReleaseGateReport {
    pub(crate) fn passed(&self) -> bool {
        self.errors.is_empty()
    }
}

pub(crate) fn validate_release_gate(documents: &ReleaseGateDocuments) -> ReleaseGateReport {
    let license_sections = parse_h2_sections(&documents.license_inventory_markdown);
    let provenance_sections = parse_h2_sections(&documents.provenance_markdown);
    let mut errors = Vec::new();

    validate_required_sections(
        &mut errors,
        "license inventory",
        &documents.license_inventory_path,
        &license_sections,
        &[
            "Cargo crates",
            "Bazel and rules",
            "ESP-IDF and esp-rs",
            "Flashing tools",
            "Static assets",
            "Release artifacts",
        ],
    );
    validate_required_sections(
        &mut errors,
        "provenance manifest",
        &documents.provenance_path,
        &provenance_sections,
        &[
            "Source commit",
            "Reference commit",
            "Static asset source",
            "Recovery page source",
            "GPL review status",
            "Release artifact review",
        ],
    );
    validate_cargo_about_report(
        &mut errors,
        &documents.cargo_about_path,
        &documents.maybe_cargo_about_html,
        &license_sections,
    );
    validate_unknown_follow_up(
        &mut errors,
        "license inventory",
        &documents.license_inventory_path,
        &license_sections,
    );
    validate_unknown_follow_up(
        &mut errors,
        "provenance manifest",
        &documents.provenance_path,
        &provenance_sections,
    );
    validate_manifest_if_provided(
        &mut errors,
        documents.maybe_manifest_path.as_ref(),
        documents.maybe_manifest_json.as_deref(),
    );
    validate_manifest_artifact_review_closure(
        &mut errors,
        documents.maybe_manifest_path.as_ref(),
        &documents.provenance_path,
        &documents.provenance_markdown,
    );

    ReleaseGateReport { errors }
}

pub(crate) fn render_release_gate_report(report: &ReleaseGateReport) -> String {
    if report.passed() {
        return "release_gate: passed\n".to_owned();
    }

    let mut output = String::from("release_gate: failed\nerrors:\n");
    for error in &report.errors {
        output.push_str("- ");
        output.push_str(error);
        output.push('\n');
    }
    output
}

#[derive(Debug)]
struct MarkdownSection {
    name: String,
    normalized_name: String,
    body: String,
    start_line: usize,
}

fn validate_required_sections(
    errors: &mut Vec<String>,
    label: &str,
    path: &Utf8PathBuf,
    sections: &[MarkdownSection],
    required_sections: &[&str],
) {
    for required_section in required_sections {
        if find_section(sections, required_section).is_some() {
            continue;
        }

        errors.push(format!(
            "{label} `{path}` missing required section `{required_section}`"
        ));
    }
}

fn validate_cargo_about_report(
    errors: &mut Vec<String>,
    cargo_about_path: &Utf8PathBuf,
    maybe_cargo_about_html: &Option<String>,
    license_sections: &[MarkdownSection],
) {
    match maybe_cargo_about_html {
        Some(contents) if !contents.trim().is_empty() => {}
        Some(_) => {
            errors.push(format!(
                "generated Cargo dependency license report `{cargo_about_path}` is empty"
            ));
        }
        None => {
            errors.push(format!(
                "generated Cargo dependency license report `{cargo_about_path}` is missing"
            ));
        }
    }

    let Some(cargo_section) = find_section(license_sections, "Cargo crates") else {
        return;
    };

    if cargo_section_references_report(cargo_section, cargo_about_path) {
        return;
    }

    errors.push(format!(
        "license inventory section `Cargo crates` does not reference `{cargo_about_path}`"
    ));
}

fn validate_unknown_follow_up(
    errors: &mut Vec<String>,
    label: &str,
    path: &Utf8PathBuf,
    sections: &[MarkdownSection],
) {
    for section in sections {
        for (line_index, line) in section.body.lines().enumerate() {
            let normalized = line.to_ascii_lowercase();
            if !normalized.contains("unknown") {
                continue;
            }

            let has_owner = normalized.contains("owner");
            let has_follow_up =
                normalized.contains("follow-up") || normalized.contains("follow up");
            if has_owner && has_follow_up {
                continue;
            }

            errors.push(format!(
                "{label} `{path}` section `{}` line {} says `unknown` without row-level owner and follow-up",
                section.name,
                section.start_line + line_index
            ));
        }
    }
}

fn validate_manifest_if_provided(
    errors: &mut Vec<String>,
    maybe_manifest_path: Option<&Utf8PathBuf>,
    maybe_manifest_json: Option<&str>,
) {
    let Some(manifest_path) = maybe_manifest_path else {
        return;
    };

    let Some(contents) = maybe_manifest_json else {
        errors.push(format!("package manifest `{manifest_path}` is missing"));
        return;
    };

    if contents.trim().is_empty() {
        errors.push(format!("package manifest `{manifest_path}` is empty"));
        return;
    }

    let manifest: Value = match serde_json::from_str(contents) {
        Ok(manifest) => manifest,
        Err(error) => {
            errors.push(format!(
                "package manifest `{manifest_path}` is not valid JSON: {error}"
            ));
            return;
        }
    };

    validate_manifest_schema_version(errors, manifest_path, &manifest);
    validate_manifest_required_strings(errors, manifest_path, &manifest);
    validate_manifest_build_identity(errors, manifest_path, &manifest);
    validate_manifest_exact_strings(errors, manifest_path, &manifest);
    validate_manifest_path(errors, manifest_path);
    validate_manifest_required_artifacts(errors, manifest_path, &manifest);
}

fn validate_manifest_schema_version(
    errors: &mut Vec<String>,
    manifest_path: &Utf8PathBuf,
    manifest: &Value,
) {
    if manifest.get("schema_version").and_then(Value::as_u64) == Some(3) {
        return;
    }

    errors.push(format!(
        "package manifest `{manifest_path}` schema_version must be 3"
    ));
}

fn validate_manifest_required_strings(
    errors: &mut Vec<String>,
    manifest_path: &Utf8PathBuf,
    manifest: &Value,
) {
    for (pointer, label) in [
        ("/semantic_version", "semantic_version"),
        ("/source_commit", "source_commit"),
        ("/reference_commit", "reference_commit"),
        ("/app_elf_sha256", "app_elf_sha256"),
        ("/build_identity/label", "build_identity.label"),
        ("/build_identity/channel", "build_identity.channel"),
        ("/otadata_source", "otadata_source"),
        ("/tool_versions/espflash", "tool_versions.espflash"),
    ] {
        let maybe_value = manifest.pointer(pointer).and_then(Value::as_str);
        if maybe_value.is_some_and(|value| !value.trim().is_empty()) {
            continue;
        }

        errors.push(format!(
            "package manifest `{manifest_path}` field `{label}` must be non-empty"
        ));
    }
}

fn validate_manifest_build_identity(
    errors: &mut Vec<String>,
    manifest_path: &Utf8PathBuf,
    manifest: &Value,
) {
    let Some(semantic_version) = manifest.get("semantic_version").and_then(Value::as_str) else {
        return;
    };
    let Some(source_commit) = manifest.get("source_commit").and_then(Value::as_str) else {
        return;
    };
    let Some(reference_commit) = manifest.get("reference_commit").and_then(Value::as_str) else {
        return;
    };
    let Some(source_dirty) = manifest
        .pointer("/build_identity/source_dirty")
        .and_then(Value::as_bool)
    else {
        errors.push(format!(
            "package manifest `{manifest_path}` field `build_identity.source_dirty` must be boolean"
        ));
        return;
    };
    let maybe_release_tag = match manifest.pointer("/build_identity/release_tag") {
        Some(Value::String(release_tag)) => Some(release_tag.as_str()),
        Some(Value::Null) => None,
        _ => {
            errors.push(format!(
                "package manifest `{manifest_path}` field `build_identity.release_tag` must be a string or null"
            ));
            return;
        }
    };
    let provenance = match BuildProvenance::new(
        semantic_version,
        source_commit,
        source_dirty,
        maybe_release_tag,
        reference_commit,
    ) {
        Ok(provenance) => provenance,
        Err(error) => {
            errors.push(format!(
                "package manifest `{manifest_path}` build identity is invalid: {error}"
            ));
            return;
        }
    };
    let identity = provenance.build_identity();
    let label_matches = manifest
        .pointer("/build_identity/label")
        .and_then(Value::as_str)
        == Some(identity.build_label());
    let channel_matches = manifest
        .pointer("/build_identity/channel")
        .and_then(Value::as_str)
        == Some(identity.build_channel().as_str());
    if !label_matches || !channel_matches {
        errors.push(format!(
            "package manifest `{manifest_path}` build identity fields are contradictory"
        ));
    }
    if source_dirty {
        errors.push(format!(
            "package manifest `{manifest_path}` is dirty and cannot qualify release evidence"
        ));
    }

    let app_elf_sha256 = manifest
        .get("app_elf_sha256")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let valid_app_hash = app_elf_sha256.len() == 64
        && app_elf_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && app_elf_sha256.bytes().any(|byte| byte != b'0');
    if !valid_app_hash {
        errors.push(format!(
            "package manifest `{manifest_path}` app_elf_sha256 must be a nonzero lowercase SHA-256"
        ));
    }
}

fn validate_manifest_exact_strings(
    errors: &mut Vec<String>,
    manifest_path: &Utf8PathBuf,
    manifest: &Value,
) {
    for (pointer, label, expected) in [
        ("/release_name", "release_name", "bitaxe-ultra205"),
        (
            "/default_flash_image",
            "default_flash_image",
            "bitaxe-ultra205.elf",
        ),
        ("/image_metadata/board", "image_metadata.board", "205"),
        (
            "/image_metadata/device_model",
            "image_metadata.device_model",
            "Ultra 205",
        ),
        ("/image_metadata/asic", "image_metadata.asic", "BM1366"),
        (
            "/image_metadata/esp_idf_version",
            "image_metadata.esp_idf_version",
            "v5.5.4",
        ),
        (
            "/image_metadata/rust_target",
            "image_metadata.rust_target",
            "xtensa-esp32s3-espidf",
        ),
        (
            "/install_notes/path",
            "install_notes.path",
            "docs/release/ultra-205.md",
        ),
        (
            "/install_notes/summary",
            "install_notes.summary",
            "Ultra 205 release operator guide",
        ),
        (
            "/license_inventory",
            "license_inventory",
            DEFAULT_LICENSE_INVENTORY_PATH,
        ),
        (
            "/provenance_manifest",
            "provenance_manifest",
            DEFAULT_PROVENANCE_PATH,
        ),
    ] {
        validate_manifest_exact_string(errors, manifest_path, manifest, pointer, label, expected);
    }
}

fn validate_manifest_exact_string(
    errors: &mut Vec<String>,
    manifest_path: &Utf8PathBuf,
    manifest: &Value,
    pointer: &str,
    label: &str,
    expected: &str,
) {
    if manifest.pointer(pointer).and_then(Value::as_str) == Some(expected) {
        return;
    }

    errors.push(format!(
        "package manifest `{manifest_path}` field `{label}` must be `{expected}`"
    ));
}

fn validate_manifest_path(errors: &mut Vec<String>, manifest_path: &Utf8PathBuf) {
    if manifest_path.file_name() == Some("bitaxe-ultra205-package.json") {
        return;
    }

    errors.push(format!(
        "package manifest path `{manifest_path}` must include bitaxe-ultra205-package.json"
    ));
}

fn validate_manifest_required_artifacts(
    errors: &mut Vec<String>,
    manifest_path: &Utf8PathBuf,
    manifest: &Value,
) {
    let Some(artifacts) = manifest.get("artifacts").and_then(Value::as_array) else {
        errors.push(format!(
            "package manifest `{manifest_path}` field `artifacts` must be an array"
        ));
        return;
    };

    for required_artifact in RequiredArtifact::all() {
        let maybe_artifact = artifacts.iter().find(|artifact| {
            artifact.get("kind").and_then(Value::as_str) == Some(required_artifact.kind)
                && artifact.get("path").and_then(Value::as_str) == Some(required_artifact.path)
                && artifact.get("offset").and_then(Value::as_str) == Some(required_artifact.offset)
        });

        let Some(artifact) = maybe_artifact else {
            errors.push(format!(
                "package manifest `{manifest_path}` missing artifact `{}` at path `{}` offset `{}`",
                required_artifact.kind, required_artifact.path, required_artifact.offset
            ));
            continue;
        };

        validate_manifest_artifact_sha256(errors, manifest_path, artifact, required_artifact.path);
    }
}

#[derive(Clone, Copy)]
struct RequiredArtifact {
    kind: &'static str,
    path: &'static str,
    offset: &'static str,
}

impl RequiredArtifact {
    const fn all() -> &'static [Self] {
        &[
            Self {
                kind: "firmware_elf",
                path: "bitaxe-ultra205.elf",
                offset: "Unavailable",
            },
            Self {
                kind: "firmware_ota_image",
                path: "esp-miner.bin",
                offset: "0x10000",
            },
            Self {
                kind: "www_spiffs_image",
                path: "www.bin",
                offset: "0x410000",
            },
            Self {
                kind: "factory_merged_image",
                path: "bitaxe-ultra205-factory.bin",
                offset: "0x0",
            },
            Self {
                kind: "partition_table",
                path: "firmware/bitaxe/partitions-ultra205.csv",
                offset: "Unavailable",
            },
            Self {
                kind: "otadata_initial",
                path: "otadata-initial.bin",
                offset: "0xf10000",
            },
        ]
    }
}

fn validate_manifest_artifact_sha256(
    errors: &mut Vec<String>,
    manifest_path: &Utf8PathBuf,
    artifact: &Value,
    required_path: &str,
) {
    let maybe_sha256 = artifact.get("sha256").and_then(Value::as_str);
    if maybe_sha256.is_some_and(is_lowercase_sha256) {
        return;
    }

    errors.push(format!(
        "package manifest `{manifest_path}` artifact `{required_path}` sha256 must be a 64-character lowercase hex string"
    ));
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| character.is_ascii_digit() || matches!(character, 'a'..='f'))
}

fn validate_manifest_artifact_review_closure(
    errors: &mut Vec<String>,
    maybe_manifest_path: Option<&Utf8PathBuf>,
    provenance_path: &Utf8PathBuf,
    provenance_markdown: &str,
) {
    if maybe_manifest_path.is_none()
        || !provenance_markdown.contains("Awaiting package output evidence")
    {
        return;
    }

    errors.push(format!(
        "provenance manifest `{provenance_path}` still contains `Awaiting package output evidence` while a package manifest is supplied"
    ));
}

fn parse_h2_sections(markdown: &str) -> Vec<MarkdownSection> {
    let mut sections = Vec::new();
    let mut maybe_name: Option<String> = None;
    let mut maybe_start_line: Option<usize> = None;
    let mut body = String::new();

    for (line_index, line) in markdown.lines().enumerate() {
        if let Some(next_name) = h2_heading(line) {
            push_section(
                &mut sections,
                maybe_name.take(),
                maybe_start_line.take(),
                &mut body,
            );
            maybe_name = Some(next_name);
            maybe_start_line = Some(line_index + 1);
            continue;
        }

        if maybe_name.is_some() {
            body.push_str(line);
            body.push('\n');
        }
    }

    push_section(
        &mut sections,
        maybe_name.take(),
        maybe_start_line.take(),
        &mut body,
    );
    sections
}

fn push_section(
    sections: &mut Vec<MarkdownSection>,
    maybe_name: Option<String>,
    maybe_start_line: Option<usize>,
    body: &mut String,
) {
    let Some(name) = maybe_name else {
        return;
    };

    sections.push(MarkdownSection {
        normalized_name: normalize_section_name(&name),
        name,
        body: std::mem::take(body),
        start_line: maybe_start_line.unwrap_or(0),
    });
}

fn h2_heading(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let heading = trimmed.strip_prefix("## ")?;
    if heading.starts_with('#') {
        return None;
    }

    let name = heading.trim();
    if name.is_empty() {
        return None;
    }

    Some(name.to_owned())
}

fn find_section<'a>(
    sections: &'a [MarkdownSection],
    required_section: &str,
) -> Option<&'a MarkdownSection> {
    let normalized_required = normalize_section_name(required_section);
    sections
        .iter()
        .find(|section| section.normalized_name == normalized_required)
}

fn normalize_section_name(section: &str) -> String {
    let mut normalized = String::new();

    for character in section.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        } else {
            normalized.push(' ');
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cargo_section_references_report(
    section: &MarkdownSection,
    cargo_about_path: &Utf8PathBuf,
) -> bool {
    let normalized_body = section.body.replace('\\', "/");
    let normalized_path = cargo_about_path.as_str().replace('\\', "/");
    if normalized_body.contains(&normalized_path) {
        return true;
    }

    let Some(file_name) = cargo_about_path.file_name() else {
        return false;
    };

    normalized_body.contains(file_name)
}

#[cfg(test)]
mod tests {
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
            .any(|error| error.contains("image_metadata.device_model")
                && error.contains("Ultra 205")));
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
        assert!(report.errors.iter().any(|error| {
            error.contains("sha256") && error.contains("64-character lowercase hex")
        }));
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
            error.contains("Awaiting package output evidence")
                && error.contains("provenance manifest")
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
        documents.maybe_manifest_json =
            Some(serde_json::to_string(&manifest).expect("manifest json"));
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
}
