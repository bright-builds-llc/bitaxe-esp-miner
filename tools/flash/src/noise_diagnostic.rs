use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::campaign::admission::read_pool_credentials;
use crate::*;

const INTENT_RELATIVE_PATH: &str =
    "scratch/str005-noise-diagnostic/diagnostic-004/intent.private.json";
const POOL_RELATIVE_PATH: &str =
    "scratch/str005-noise-diagnostic/diagnostic-004/fixture-pool.private.json";
const PLAN_RELATIVE_PATH: &str =
    "docs/parity/work-plans/20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY/PLAN.md";
const PLAN_SHA256: &str = "3bbdf04402a0a51c4d380ef4efa65b4ee3d434bf865970c161a7faf0760b6658";
const INTENT_SCHEMA: &str = "bitaxe-stratum-v2-noise-diagnostic-intent-v1";
const DIAGNOSTIC_ORDINAL: u16 = 4;
const CAPTURE_TIMEOUT_SECONDS: u64 = 120;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NoiseDiagnosticIntent {
    schema_version: String,
    board: u16,
    diagnostic_ordinal: u16,
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    plan_path: String,
    plan_sha256: String,
    lease_hex: String,
}

pub(crate) fn run_noise_diagnostic_command(
    command: &NoiseDiagnosticCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if !command.redact_evidence || command.capture_timeout_seconds != CAPTURE_TIMEOUT_SECONDS {
        bail!("noise_diagnostic=blocked reason=command_contract");
    }
    let seed = admit_noise_diagnostic(command, environment)?;
    let common = CommonArgs {
        board: command.board,
        port: Some(command.port.clone()),
        dry_run: false,
        redact_evidence: true,
        evidence_mode: None,
        evidence_dir: None,
    };
    let flash = FlashCommand {
        common: common.clone(),
        image: None,
        manifest: Some(command.manifest.clone()),
        wifi_credentials: Some(command.wifi_credentials.clone()),
    };
    run_flash_with_wifi_mode(&flash, WifiNvsSeedMode::NoiseDiagnostic(seed), environment)?;
    run_monitor(
        &MonitorCommand {
            common,
            capture_timeout_seconds: command.capture_timeout_seconds,
        },
        environment,
    )
}

fn admit_noise_diagnostic(
    command: &NoiseDiagnosticCommand,
    environment: &impl FlashEnvironment,
) -> Result<NoiseDiagnosticNvsSeed> {
    if command.intent != Utf8Path::new(INTENT_RELATIVE_PATH)
        || command.pool_credentials != Utf8Path::new(POOL_RELATIVE_PATH)
    {
        bail!("noise_diagnostic=blocked reason=path_contract");
    }
    let intent_path = environment.workspace_path(&command.intent);
    let pool_path = environment.workspace_path(&command.pool_credentials);
    require_private_file(&intent_path)?;
    require_private_file(&pool_path)?;
    let intent_document = environment
        .read_to_string(&intent_path)
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=intent_unreadable"))?;
    let intent: NoiseDiagnosticIntent = serde_json::from_str(&intent_document)
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=intent_malformed"))?;
    let manifest_path = environment.workspace_path(&command.manifest);
    let manifest_document = environment
        .read_to_string(&manifest_path)
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=manifest_unreadable"))?;
    let manifest: PackageManifest = serde_json::from_str(&manifest_document)
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=manifest_malformed"))?;
    let plan_path = environment.workspace_path(Utf8Path::new(PLAN_RELATIVE_PATH));
    let plan_document = environment
        .read_to_string(&plan_path)
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=plan_unreadable"))?;
    if intent.schema_version != INTENT_SCHEMA
        || intent.board != 205
        || intent.diagnostic_ordinal != DIAGNOSTIC_ORDINAL
        || intent.source_commit != manifest.source_commit
        || intent.reference_commit != manifest.reference_commit
        || intent.app_elf_sha256 != manifest.app_elf_sha256
        || intent.plan_path != PLAN_RELATIVE_PATH
        || intent.plan_sha256 != PLAN_SHA256
        || sha256_bytes(plan_document.as_bytes()) != PLAN_SHA256
    {
        bail!("noise_diagnostic=blocked reason=identity_contract");
    }
    let lease = parse_lease(&intent.lease_hex)?;
    let pool = read_pool_credentials(&pool_path, environment)
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=pool_contract"))?;
    if pool.stratum_protocol != "SV2"
        || pool.stratum_v2_channel_type.as_deref() != Some("standard")
        || pool.stratum_v2_authority_pubkey.is_none()
    {
        bail!("noise_diagnostic=blocked reason=pool_contract");
    }
    Ok(NoiseDiagnosticNvsSeed { lease, pool })
}

fn parse_lease(value: &str) -> Result<u64> {
    if value.len() != 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("noise_diagnostic=blocked reason=lease_contract");
    }
    let lease = u64::from_str_radix(value, 16)
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=lease_contract"))?;
    if lease == 0 {
        bail!("noise_diagnostic=blocked reason=lease_contract");
    }
    Ok(lease)
}

fn require_private_file(path: &Utf8Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=file_metadata"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("noise_diagnostic=blocked reason=file_type");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        bail!("noise_diagnostic=blocked reason=file_mode");
    }
    let parent = path
        .parent()
        .context("noise_diagnostic=blocked reason=root_missing")?;
    let root = fs::symlink_metadata(parent.as_std_path())
        .map_err(|_| anyhow::anyhow!("noise_diagnostic=blocked reason=root_metadata"))?;
    if root.file_type().is_symlink() || !root.is_dir() {
        bail!("noise_diagnostic=blocked reason=root_type");
    }
    #[cfg(unix)]
    if root.permissions().mode() & 0o777 != 0o700 {
        bail!("noise_diagnostic=blocked reason=root_mode");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_digest_and_first_ordinal_are_immutable() {
        // Arrange
        let plan = include_str!(
            "../../../docs/parity/work-plans/20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY/PLAN.md"
        );

        // Act
        let digest = sha256_bytes(plan.as_bytes());

        // Assert
        assert_eq!(digest, PLAN_SHA256);
        assert_eq!(DIAGNOSTIC_ORDINAL, 4);
        assert!(INTENT_RELATIVE_PATH.contains("diagnostic-004"));
    }

    #[test]
    fn lease_parser_rejects_zero_and_noncanonical_values() {
        // Arrange / Act / Assert
        assert_eq!(parse_lease("0000000000000001").expect("lease"), 1);
        for invalid in [
            "0000000000000000",
            "1",
            "AAAAAAAAAAAAAAAA",
            "gggggggggggggggg",
        ] {
            assert!(parse_lease(invalid).is_err());
        }
    }
}
