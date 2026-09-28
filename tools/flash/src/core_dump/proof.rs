//! Closed private proof admission, before any USB ownership or ROM effect.
use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecoveryProof {
    schema: String,
    source_commit: String,
    gate_commit: String,
    firmware_commit: String,
    app_elf_sha256: String,
    physical_identity_sha256: String,
    observed_at_unix_ms: u64,
    ledger: Ledger,
    original_budget: Budget,
    safe_baseline: bool,
    restoration_confirmed: bool,
    device_lease_inactive: bool,
    serial_ownership_released: bool,
    preservation_matches: bool,
    mine_on_boot: bool,
    #[serde(rename = "current_v2_idle")]
    maybe_current_v2_idle: Option<bool>,
    #[serde(rename = "current_v2_released")]
    maybe_current_v2_released: Option<bool>,
    #[serde(rename = "retained_attempt_id")]
    maybe_retained_attempt_id: Option<String>,
    #[serde(rename = "retained_status_sha256")]
    maybe_retained_status_sha256: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    schema: String,
    next_ordinal: u64,
    last_completed_ordinal: u64,
    total_charged_ms: u64,
    pending: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Budget {
    schema: String,
    campaign_match: bool,
    reserved_mask: u8,
    completed_mask: u8,
    charged_ms: u64,
    pending: bool,
}

pub(super) fn validate(
    bytes: &[u8],
    command: &CoreDumpReadCommand,
    source: &str,
    now: u64,
) -> Result<()> {
    let proof: RecoveryProof =
        serde_json::from_slice(bytes).map_err(|_| anyhow::anyhow!("recovery_proof_shape"))?;
    let shape: serde_json::Value = serde_json::from_slice(bytes)?;
    let ledger = &proof.ledger;
    let budget = &proof.original_budget;
    const MAX_SAFE: u64 = 9_007_199_254_740_991;
    let released = match proof.schema.as_str() {
        "str005-current-recovery-proof-v1" => {
            proof.maybe_current_v2_idle == Some(true)
                && shape.get("current_v2_released").is_none()
                && shape.get("retained_attempt_id").is_none()
                && shape.get("retained_status_sha256").is_none()
        }
        "str005-current-recovery-proof-v2" => {
            shape.get("current_v2_idle").is_none()
                && proof.maybe_current_v2_released == Some(true)
                && proof
                    .maybe_retained_attempt_id
                    .as_deref()
                    .is_some_and(|id| {
                        id.len() == 22
                            && id.bytes().all(|byte| {
                                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
                            })
                    })
                && proof
                    .maybe_retained_status_sha256
                    .as_deref()
                    .is_some_and(|digest| contract::hex(digest, 64))
        }
        _ => false,
    };
    if !released
        || proof.source_commit != source
        || !contract::hex(&proof.gate_commit, 40)
        || proof.firmware_commit != command.expected_installed_source
        || proof.app_elf_sha256 != command.expected_installed_elf
        || proof.physical_identity_sha256 != command.expected_physical_sha256
        || proof.observed_at_unix_ms > now
        || now - proof.observed_at_unix_ms > 120_000
        || ledger.schema != "worker-qualification-ledger-v1"
        || ledger.pending
        || ledger.next_ordinal == 0
        || ledger.next_ordinal > 0x1_0000_0000
        || ledger.last_completed_ordinal > u32::MAX as u64
        || ledger.last_completed_ordinal.checked_add(1) != Some(ledger.next_ordinal)
        || ledger.total_charged_ms > MAX_SAFE
        || budget.schema != "worker-budget-review-v1"
        || !budget.campaign_match
        || budget.reserved_mask != 7
        || budget.completed_mask != 7
        || budget.charged_ms != 240_000
        || budget.pending
        || !proof.safe_baseline
        || !proof.restoration_confirmed
        || !proof.device_lease_inactive
        || !proof.serial_ownership_released
        || !proof.preservation_matches
        || proof.mine_on_boot
    {
        bail!("recovery_proof_invalid");
    }
    Ok(())
}

pub(super) fn admit(
    command: &CoreDumpReadCommand,
    workspace: &Utf8Path,
    source: &str,
) -> Result<()> {
    let bytes = private_file(&command.recovery_proof, workspace, 16_384)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    validate(&bytes, command, source, u64::try_from(now)?)
}

pub(super) fn private_file(path: &Utf8Path, workspace: &Utf8Path, bound: u64) -> Result<Vec<u8>> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, camino::Utf8Component::ParentDir))
    {
        bail!("private_input_path");
    }
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() == 0 || meta.len() > bound {
        bail!("private_input_shape");
    }
    let parent = path.parent().context("private_input_parent")?;
    #[cfg(unix)]
    if meta.permissions().mode() & 0o777 != 0o600
        || fs::symlink_metadata(parent)?.permissions().mode() & 0o777 != 0o700
    {
        bail!("private_input_mode");
    }
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor)?.file_type().is_symlink() {
            bail!("private_input_symlink");
        }
    }
    let ignored = Command::new("git")
        .current_dir(workspace)
        .args(["check-ignore", "--quiet", "--", path.as_str()])
        .output()?;
    if !ignored.status.success() {
        bail!("private_input_not_ignored");
    }
    let bytes = fs::read(path)?;
    if bytes.len() as u64 != meta.len() {
        bail!("private_input_changed");
    }
    Ok(bytes)
}

/// Consume one semantic proof before USB effects. Claims survive every failed attempt.
pub(super) fn claim(
    command: &CoreDumpReadCommand,
    workspace: &Utf8Path,
    source: &str,
    clear: bool,
    output_root: &Utf8Path,
) -> Result<()> {
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
    let bytes = private_file(&command.recovery_proof, workspace, 16_384)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    validate(&bytes, command, source, u64::try_from(now)?)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    let key = sha256_bytes(&serde_json::to_vec(&canonical(value))?);
    let canonical_workspace = fs::canonicalize(workspace)?;
    let canonical_workspace = Utf8PathBuf::from_path_buf(canonical_workspace)
        .map_err(|_| anyhow::anyhow!("core_proof_claim_workspace"))?;
    for ancestor in workspace.ancestors() {
        if fs::symlink_metadata(ancestor)?.file_type().is_symlink() {
            bail!("core_proof_claim_symlink");
        }
    }
    let parent = canonical_workspace.join("scratch");
    let root = parent.join("core-dump-proof-claims");
    let ignored = Command::new("git")
        .current_dir(workspace)
        .args(["check-ignore", "--quiet", "--", root.as_str()])
        .output()?;
    if !ignored.status.success() {
        bail!("core_proof_claim_not_ignored");
    }
    for directory in [&parent, &root] {
        match fs::DirBuilder::new().mode(0o700).create(directory) {
            Ok(()) => (),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
            Err(error) => return Err(error.into()),
        }
        let metadata = fs::symlink_metadata(directory)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            bail!("core_proof_claim_directory");
        }
    }
    if fs::symlink_metadata(&root)?.permissions().mode() & 0o777 != 0o700 {
        bail!("core_proof_claim_mode");
    }
    let path = root.join(format!("{key}.json"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .map_err(|error| match error.kind() {
            io::ErrorKind::AlreadyExists => anyhow::anyhow!("core_proof_already_consumed"),
            _ => anyhow::anyhow!("core_proof_claim_failed"),
        })?;
    let record = serde_json::json!({
        "schema": "bitaxe-core-dump-proof-claim-v1",
        "proof_semantic_sha256": key,
        "source_commit": source,
        "physical_identity_sha256": command.expected_physical_sha256,
        "operation": if clear { "clear" } else { "read" },
        "output_root": output_root.as_str(),
    });
    file.write_all(&serde_json::to_vec(&record)?)?;
    file.sync_all()?;
    fs::File::open(&root)?.sync_all()?;
    fs::File::open(&parent)?.sync_all()?;
    fs::File::open(&canonical_workspace)?.sync_all()?;
    Ok(())
}

fn canonical(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(values) => {
            let sorted: std::collections::BTreeMap<_, _> = values.into_iter().collect();
            serde_json::Value::Object(
                sorted
                    .into_iter()
                    .map(|(key, value)| (key, canonical(value)))
                    .collect(),
            )
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonical).collect())
        }
        value => value,
    }
}
