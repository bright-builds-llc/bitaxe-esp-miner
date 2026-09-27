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
    current_v2_idle: bool,
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
    let ledger = &proof.ledger;
    let budget = &proof.original_budget;
    const MAX_SAFE: u64 = 9_007_199_254_740_991;
    if proof.schema != "str005-current-recovery-proof-v1"
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
        || !proof.current_v2_idle
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
