use anyhow::{bail, Result};

use super::{ArtifactKind, PackageManifestV3, ReleaseArtifact};

pub(super) fn require_non_empty(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} must not be empty");
    }

    Ok(())
}

pub(super) fn require_artifact_kind(
    manifest: &PackageManifestV3,
    kind: ArtifactKind,
) -> Result<&ReleaseArtifact> {
    let mut matches = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind);
    let Some(artifact) = matches.next() else {
        bail!(required_artifact_message(kind));
    };
    if matches.next().is_some() {
        bail!(duplicate_artifact_message(kind));
    }

    Ok(artifact)
}

pub(super) fn validate_sha256(kind: &ArtifactKind, sha256: &str) -> Result<()> {
    if sha256.len() == 64
        && sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }

    bail!("{kind} sha256 must be a 64 character hex string");
}

pub(super) fn validate_app_elf_sha256(sha256: &str) -> Result<()> {
    if sha256.len() == 64
        && sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && sha256.bytes().any(|byte| byte != b'0')
    {
        return Ok(());
    }

    bail!("app_elf_sha256 must be a nonzero 64-character lowercase hexadecimal string")
}

pub(super) fn require_artifact_offset(
    manifest: &PackageManifestV3,
    kind: ArtifactKind,
    expected_offset: &str,
) -> Result<()> {
    let artifact = require_artifact_kind(manifest, kind)?;
    if artifact.offset == expected_offset {
        return Ok(());
    }

    bail!(
        "{kind} artifact must use offset {expected_offset}, found {}",
        artifact.offset
    );
}

fn required_artifact_message(kind: ArtifactKind) -> String {
    match kind {
        ArtifactKind::Bootloader => "required artifact kind bootloader missing".to_owned(),
        ArtifactKind::PartitionTableBinary => {
            "required artifact kind partition_table_binary missing".to_owned()
        }
        ArtifactKind::FirmwareElf => "required artifact kind firmware_elf missing".to_owned(),
        ArtifactKind::FirmwareOtaImage => {
            "required artifact kind firmware_ota_image missing".to_owned()
        }
        ArtifactKind::WwwSpiffsImage => {
            "required artifact kind www_spiffs_image missing".to_owned()
        }
        ArtifactKind::FactoryMergedImage => {
            "required artifact kind factory_merged_image missing".to_owned()
        }
        ArtifactKind::PartitionTable => "required artifact kind partition_table missing".to_owned(),
        ArtifactKind::OtadataInitial => "required artifact kind otadata_initial missing".to_owned(),
        ArtifactKind::UpdateOnlyImage => {
            "required artifact kind update_only_image missing".to_owned()
        }
    }
}

fn duplicate_artifact_message(kind: ArtifactKind) -> String {
    format!("required artifact kind {kind} duplicate")
}
