use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::*;

pub(crate) const THERMAL_FAULT_INTENT_RELATIVE_PATH: &str =
    "scratch/thr001-emc2101-fault/attempt-005/thermal-fault-intent.private.json";
pub(crate) const THERMAL_FAULT_PLAN_RELATIVE_PATH: &str =
    "docs/parity/work-plans/20260815T182438Z-THR-001/PLAN.md";
pub(crate) const THERMAL_FAULT_PLAN_SHA256: &str =
    "8e8049fd6fbb19575f6abe593afcdd9ac2303eee0204b5f188d4b65aa7607d58";
const THERMAL_FAULT_INTENT_SCHEMA: &str = "esp-thermal-fault-stimulus-intent-v1";
const THERMAL_FAULT_STIMULUS_KIND: &str = "emc2101_invalid_sample";
const THERMAL_FAULT_SAMPLE_COUNT: u16 = 5;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThermalFaultStimulusIntent {
    schema_version: String,
    board: u16,
    attempt_ordinal: u16,
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    plan_path: String,
    plan_sha256: String,
    stimulus_kind: String,
    sample_count: u16,
    lease_hex: String,
}

pub(crate) fn admit_thermal_fault_stimulus_intent(
    intent_input: &Utf8Path,
    maybe_manifest: Option<&Utf8PathBuf>,
    board: BoardId,
    environment: &impl FlashEnvironment,
) -> Result<ThermalFaultNvsSeed> {
    admit_thermal_fault_stimulus_intent_with_plan_sha256(
        intent_input,
        maybe_manifest,
        board,
        environment,
        THERMAL_FAULT_PLAN_SHA256,
    )
}

pub(crate) fn admit_thermal_fault_stimulus_intent_with_plan_sha256(
    intent_input: &Utf8Path,
    maybe_manifest: Option<&Utf8PathBuf>,
    board: BoardId,
    environment: &impl FlashEnvironment,
    expected_plan_sha256: &str,
) -> Result<ThermalFaultNvsSeed> {
    ensure_ultra_205(board)?;
    if intent_input != Utf8Path::new(THERMAL_FAULT_INTENT_RELATIVE_PATH) {
        bail!("thermal_fault_intent=blocked reason=path_contract");
    }
    let Some(manifest_input) = maybe_manifest else {
        bail!("thermal_fault_intent=blocked reason=explicit_manifest_required");
    };
    let intent_path = environment.workspace_path(intent_input);
    require_private_file(&intent_path, 0o600)?;
    let root = intent_path
        .parent()
        .context("thermal_fault_intent=blocked reason=root_missing")?;
    require_private_directory(root, 0o700)?;

    let intent_document = environment
        .read_to_string(&intent_path)
        .map_err(|_| anyhow::anyhow!("thermal_fault_intent=blocked reason=unreadable"))?;
    let intent: ThermalFaultStimulusIntent = serde_json::from_str(&intent_document)
        .map_err(|_| anyhow::anyhow!("thermal_fault_intent=blocked reason=malformed"))?;
    let manifest_path = environment.workspace_path(manifest_input);
    let manifest_document = environment
        .read_to_string(&manifest_path)
        .map_err(|_| anyhow::anyhow!("thermal_fault_intent=blocked reason=manifest_unreadable"))?;
    let manifest: PackageManifest = serde_json::from_str(&manifest_document)
        .map_err(|_| anyhow::anyhow!("thermal_fault_intent=blocked reason=manifest_malformed"))?;
    let plan_path = environment.workspace_path(Utf8Path::new(THERMAL_FAULT_PLAN_RELATIVE_PATH));
    let plan_document = environment
        .read_to_string(&plan_path)
        .map_err(|_| anyhow::anyhow!("thermal_fault_intent=blocked reason=plan_unreadable"))?;

    validate_commit("source_commit", &intent.source_commit)?;
    validate_commit("reference_commit", &intent.reference_commit)?;
    validate_lower_hex("app_elf_sha256", &intent.app_elf_sha256, true)?;
    validate_lower_hex("plan_sha256", &intent.plan_sha256, true)?;
    if intent.schema_version != THERMAL_FAULT_INTENT_SCHEMA
        || intent.board != 205
        || intent.attempt_ordinal != 5
        || intent.source_commit != manifest.source_commit
        || intent.reference_commit != manifest.reference_commit
        || intent.app_elf_sha256 != manifest.app_elf_sha256
        || intent.plan_path != THERMAL_FAULT_PLAN_RELATIVE_PATH
        || intent.plan_sha256 != expected_plan_sha256
        || sha256_bytes(plan_document.as_bytes()) != expected_plan_sha256
        || intent.stimulus_kind != THERMAL_FAULT_STIMULUS_KIND
        || intent.sample_count != THERMAL_FAULT_SAMPLE_COUNT
    {
        bail!("thermal_fault_intent=blocked reason=contract_mismatch");
    }
    if intent.lease_hex.len() != 16
        || !intent
            .lease_hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("thermal_fault_intent=blocked reason=lease_invalid");
    }
    let lease = u64::from_str_radix(&intent.lease_hex, 16)
        .map_err(|_| anyhow::anyhow!("thermal_fault_intent=blocked reason=lease_invalid"))?;
    if lease == 0 {
        bail!("thermal_fault_intent=blocked reason=lease_invalid");
    }

    Ok(ThermalFaultNvsSeed {
        lease,
        sample_count: intent.sample_count,
    })
}

fn validate_commit(label: &str, value: &str) -> Result<()> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    bail!("thermal_fault_intent=blocked reason=invalid_{label}")
}

fn require_private_file(path: &Utf8Path, expected_mode: u32) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())
        .map_err(|_| anyhow::anyhow!("thermal_fault_intent=blocked reason=file_metadata"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("thermal_fault_intent=blocked reason=file_type");
    }
    require_mode(&metadata, expected_mode, "file")
}

fn require_private_directory(path: &Utf8Path, expected_mode: u32) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())
        .map_err(|_| anyhow::anyhow!("thermal_fault_intent=blocked reason=root_metadata"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("thermal_fault_intent=blocked reason=root_type");
    }
    require_mode(&metadata, expected_mode, "root")
}

#[cfg(unix)]
fn require_mode(metadata: &fs::Metadata, expected: u32, label: &str) -> Result<()> {
    if metadata.permissions().mode() & 0o777 != expected {
        bail!("thermal_fault_intent=blocked reason={label}_mode");
    }
    Ok(())
}

#[cfg(not(unix))]
fn require_mode(_metadata: &fs::Metadata, _expected: u32, _label: &str) -> Result<()> {
    bail!("thermal_fault_intent=blocked reason=unsupported_platform")
}
