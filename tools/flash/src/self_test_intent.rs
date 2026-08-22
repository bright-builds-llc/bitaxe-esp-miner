use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::*;

pub(crate) const SELF_TEST_FAILURE_INTENT_RELATIVE_PATH: &str =
    "scratch/self001-full-lifecycle/attempt-005/failure-intent.private.json";
pub(crate) const SELF_TEST_PASS_INTENT_RELATIVE_PATH: &str =
    "scratch/self001-full-lifecycle/attempt-005/pass-intent.private.json";
pub(crate) const SELF_TEST_PLAN_RELATIVE_PATH: &str =
    "docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md";
pub(crate) const SELF_TEST_PLAN_SHA256: &str =
    "0c9a03ec490967fc95989d88b91848c7d4ed740a76825822a8107d94e8fd7f84";
const SELF_TEST_INTENT_SCHEMA: &str = "bitaxe-self-test-intent-v1";
const SELF_TEST_ATTEMPT_ORDINAL: u16 = 5;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelfTestIntent {
    schema_version: String,
    board: u16,
    attempt_ordinal: u16,
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    plan_path: String,
    plan_sha256: String,
    case: String,
    lease_hex: String,
}

pub(crate) fn admit_self_test_intent(
    intent_input: &Utf8Path,
    maybe_manifest: Option<&Utf8PathBuf>,
    board: BoardId,
    environment: &impl FlashEnvironment,
) -> Result<SelfTestNvsSeed> {
    ensure_ultra_205(board)?;
    let expected_case = match intent_input.as_str() {
        SELF_TEST_FAILURE_INTENT_RELATIVE_PATH => "planned_failure",
        SELF_TEST_PASS_INTENT_RELATIVE_PATH => "pass",
        _ => bail!("self_test_intent=blocked reason=path_contract"),
    };
    let manifest_input =
        maybe_manifest.context("self_test_intent=blocked reason=explicit_manifest_required")?;
    let intent_path = environment.workspace_path(intent_input);
    require_private_file(&intent_path)?;
    let intent_document = environment
        .read_to_string(&intent_path)
        .map_err(|_| anyhow::anyhow!("self_test_intent=blocked reason=unreadable"))?;
    let intent: SelfTestIntent = serde_json::from_str(&intent_document)
        .map_err(|_| anyhow::anyhow!("self_test_intent=blocked reason=malformed"))?;
    let manifest_path = environment.workspace_path(manifest_input);
    let manifest_document = environment
        .read_to_string(&manifest_path)
        .map_err(|_| anyhow::anyhow!("self_test_intent=blocked reason=manifest_unreadable"))?;
    let manifest: PackageManifest = serde_json::from_str(&manifest_document)
        .map_err(|_| anyhow::anyhow!("self_test_intent=blocked reason=manifest_malformed"))?;
    let plan_path = environment.workspace_path(Utf8Path::new(SELF_TEST_PLAN_RELATIVE_PATH));
    let plan_document = environment
        .read_to_string(&plan_path)
        .map_err(|_| anyhow::anyhow!("self_test_intent=blocked reason=plan_unreadable"))?;

    validate_commit_hex("source_commit", &intent.source_commit)?;
    validate_commit_hex("reference_commit", &intent.reference_commit)?;
    validate_lower_hex("app_elf_sha256", &intent.app_elf_sha256, true)?;
    validate_lower_hex("plan_sha256", &intent.plan_sha256, true)?;
    if intent.schema_version != SELF_TEST_INTENT_SCHEMA
        || intent.board != 205
        || intent.attempt_ordinal != SELF_TEST_ATTEMPT_ORDINAL
        || intent.source_commit != manifest.source_commit
        || intent.reference_commit != manifest.reference_commit
        || intent.app_elf_sha256 != manifest.app_elf_sha256
        || intent.plan_path != SELF_TEST_PLAN_RELATIVE_PATH
        || intent.plan_sha256 != SELF_TEST_PLAN_SHA256
        || sha256_bytes(plan_document.as_bytes()) != SELF_TEST_PLAN_SHA256
        || intent.case != expected_case
    {
        bail!("self_test_intent=blocked reason=contract_mismatch");
    }
    if intent.lease_hex.len() != 16
        || !intent
            .lease_hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("self_test_intent=blocked reason=lease_invalid");
    }
    let lease = u64::from_str_radix(&intent.lease_hex, 16)
        .map_err(|_| anyhow::anyhow!("self_test_intent=blocked reason=lease_invalid"))?;
    if lease == 0 {
        bail!("self_test_intent=blocked reason=lease_invalid");
    }
    Ok(SelfTestNvsSeed {
        lease,
        case: expected_case,
    })
}

fn validate_commit_hex(label: &str, value: &str) -> Result<()> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    bail!("identity_admission=blocked reason=invalid_{label}")
}

fn require_private_file(path: &Utf8Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())
        .map_err(|_| anyhow::anyhow!("self_test_intent=blocked reason=file_metadata"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("self_test_intent=blocked reason=file_type");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        bail!("self_test_intent=blocked reason=file_mode");
    }
    let parent = path
        .parent()
        .context("self_test_intent=blocked reason=root_missing")?;
    let root = fs::symlink_metadata(parent.as_std_path())
        .map_err(|_| anyhow::anyhow!("self_test_intent=blocked reason=root_metadata"))?;
    if root.file_type().is_symlink() || !root.is_dir() {
        bail!("self_test_intent=blocked reason=root_type");
    }
    #[cfg(unix)]
    if root.permissions().mode() & 0o777 != 0o700 {
        bail!("self_test_intent=blocked reason=root_mode");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attempt_five_contract_binds_exact_plan_and_private_paths() {
        // Arrange
        let plan = include_str!(
            "../../../docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md"
        );

        // Act
        let plan_sha256 = sha256_bytes(plan.as_bytes());

        // Assert
        assert_eq!(SELF_TEST_ATTEMPT_ORDINAL, 5);
        assert_eq!(plan_sha256, SELF_TEST_PLAN_SHA256);
        assert!(SELF_TEST_FAILURE_INTENT_RELATIVE_PATH.contains("attempt-005"));
        assert!(SELF_TEST_PASS_INTENT_RELATIVE_PATH.contains("attempt-005"));
    }

    #[test]
    fn commit_validator_accepts_only_exact_lowercase_sha1_shape() {
        // Arrange
        let valid = "a".repeat(40);

        // Act / Assert
        validate_commit_hex("source_commit", &valid).expect("valid commit");
        for invalid in [
            "a".repeat(39),
            "a".repeat(41),
            "A".repeat(40),
            "g".repeat(40),
        ] {
            assert!(validate_commit_hex("source_commit", &invalid).is_err());
        }
    }
}
