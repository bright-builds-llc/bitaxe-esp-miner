//! Immutable, range-admitted ordinary updates that never overwrite NVS.
use crate::*;

pub(crate) const CANONICAL_PARTITIONS: &str =
    include_str!("../../../firmware/bitaxe/partitions-ultra205.csv");

pub(crate) fn segmented_display_command(port: &str, segments: &[(u32, Vec<u8>)]) -> CommandSpec {
    let mut args = write_prefix(port);
    let names: &[&str] = if segments.len() == 1 {
        &["factory-reset"]
    } else {
        &[
            "bootloader",
            "partition-table",
            "application",
            "www",
            "boot-selection",
        ]
    };
    for ((offset, _), name) in segments.iter().zip(names) {
        args.push(format!("0x{offset:x}"));
        args.push(format!("<admitted-{name}-snapshot>"));
    }
    CommandSpec::new("managed-esptool", args)
}

pub(crate) fn admit_update_segments(
    path: &Utf8Path,
    manifest: &PackageManifest,
    factory: &[u8],
    environment: &impl FlashEnvironment,
) -> Result<Vec<(u32, Vec<u8>)>> {
    if manifest.schema_version != 4 {
        return Ok(Vec::new());
    }
    bitaxe_api::update_segments::validate_update_segments(&manifest.update_segments)
        .context("identity_admission=blocked reason=update_geometry")?;
    let mut result = Vec::new();
    for segment in &manifest.update_segments {
        let artifact = require_artifact(manifest, &segment.artifact_kind)?;
        let artifact_path = resolve_manifest_sibling(path, Utf8Path::new(&artifact.path))?;
        let bytes = read_validated_artifact(artifact, &artifact_path, environment)?;
        if bytes.len() != segment.length as usize {
            bail!("identity_admission=blocked reason=update_length");
        }
        let begin = segment.offset as usize;
        let end = begin
            .checked_add(bytes.len())
            .context("identity_admission=blocked reason=update_geometry")?;
        if factory.get(begin..end) != Some(bytes.as_slice()) {
            bail!("identity_admission=blocked reason=update_factory_artifact_mismatch");
        }
        if segment.artifact_kind == "partition_table_binary" {
            let actual = esp_idf_part::PartitionTable::try_from_bytes(bytes.as_slice())
                .context("identity_admission=blocked reason=update_partition_table_invalid")?;
            let expected = esp_idf_part::PartitionTable::try_from_str(CANONICAL_PARTITIONS)
                .context("identity_admission=blocked reason=canonical_partition_table_invalid")?;
            if actual.partitions() != expected.partitions() {
                bail!("identity_admission=blocked reason=update_partition_layout_mismatch");
            }
        }
        result.push((segment.offset, bytes));
    }
    Ok(result)
}

pub(crate) fn prepare_segmented_write(
    program: Utf8PathBuf,
    port: &str,
    segments: &[(u32, Vec<u8>)],
    environment: &impl FlashEnvironment,
) -> Result<ManagedEsptoolWriteFlash> {
    if segments.len() != 5 {
        bail!("identity_admission=blocked reason=manifest_update_segments_required");
    }
    prepare_write(program, port, segments, environment)
}

/// Called only by the explicit factory-reset branch after merged-image admission.
pub(crate) fn prepare_factory_write(
    program: Utf8PathBuf,
    port: &str,
    factory: &[u8],
    environment: &impl FlashEnvironment,
) -> Result<ManagedEsptoolWriteFlash> {
    prepare_write(program, port, &[(0, factory.to_vec())], environment)
}

fn write_prefix(port: &str) -> Vec<String> {
    [
        "--chip",
        "esp32s3",
        "--port",
        port,
        "--before",
        "no_reset",
        "--after",
        "no_reset",
        "--no-stub",
        "write_flash",
        "--flash_size",
        "16MB",
    ]
    .map(str::to_owned)
    .to_vec()
}

fn prepare_write(
    program: Utf8PathBuf,
    port: &str,
    segments: &[(u32, Vec<u8>)],
    environment: &impl FlashEnvironment,
) -> Result<ManagedEsptoolWriteFlash> {
    let mut args = write_prefix(port);
    let mut snapshots = Vec::new();
    for (offset, bytes) in segments {
        let snapshot = environment.create_admitted_execution_snapshot(bytes)?;
        args.push(format!("0x{offset:x}"));
        args.push(snapshot.path().as_str().to_owned());
        snapshots.push(snapshot);
    }
    Ok(ManagedEsptoolWriteFlash::from_admitted_segments(
        program, args, snapshots,
    ))
}
