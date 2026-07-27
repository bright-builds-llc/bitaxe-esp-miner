use super::*;

pub(super) fn validate_manifest_if_provided(
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

pub(super) fn validate_manifest_artifact_review_closure(
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
