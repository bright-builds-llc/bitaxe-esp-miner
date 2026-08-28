use crate::*;

pub(crate) const RESTORE_BUNDLE_RELATIVE: &str =
    "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
pub(crate) const RESTORE_PLAN_RELATIVE: &str =
    "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";
pub(crate) const REMEDIATION_PLAN_RELATIVE: &str =
    "docs/parity/work-plans/20260826T135721Z-STR-005-INACTIVE-RESTORATION/PLAN.md";
pub(crate) const REMEDIATION_PLAN_SHA256: &str =
    "14c7676fb26b6291a24d08d229bc38717691835978d61ae24fd8cff91736470a";
pub(crate) const PREFLIGHT_ROOT: &str = "scratch/str005-exact-restoration/preflight-005";
pub(crate) const EFFECT_ROOT: &str = "scratch/str005-exact-restoration/remediation-005";
pub(crate) const CAMPAIGN_RESTORE_ROOT: &str = "scratch/str005-stratum-v2/attempt-007/restoration";
pub(crate) const NOISE_DIAGNOSTIC_RESTORE_ROOT: &str =
    "scratch/str005-noise-diagnostic/diagnostic-004/restoration";
pub(crate) const NOISE_DIAGNOSTIC_PLAN_RELATIVE: &str =
    "docs/parity/work-plans/20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY/PLAN.md";
pub(crate) const NOISE_DIAGNOSTIC_PLAN_SHA256: &str =
    "3bbdf04402a0a51c4d380ef4efa65b4ee3d434bf865970c161a7faf0760b6658";
pub(crate) const TCP_PAYLOAD_RECOVERY_ROOT: &str =
    "scratch/str005-tcp-payload/recovery-001/restoration";
pub(crate) const TCP_PAYLOAD_PLAN_RELATIVE: &str =
    "docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md";
pub(crate) const TCP_PAYLOAD_PLAN_SHA256: &str =
    "14bd8aef5d78f38881a3da1a99a6808f7f6e8c93bb1d1a02d7972fcaaeb1d843";

pub(crate) fn authorized_remediation_plan(
    action: &str,
    ordinal: u16,
) -> Result<(&'static str, &'static str)> {
    match (action, ordinal) {
        ("tcp_payload_recovery", 1) => Ok((TCP_PAYLOAD_PLAN_RELATIVE, TCP_PAYLOAD_PLAN_SHA256)),
        ("diagnostic_restore", 4) => {
            Ok((NOISE_DIAGNOSTIC_PLAN_RELATIVE, NOISE_DIAGNOSTIC_PLAN_SHA256))
        }
        ("preflight" | "start", 5) | ("campaign_restore", 7) => {
            Ok((REMEDIATION_PLAN_RELATIVE, REMEDIATION_PLAN_SHA256))
        }
        _ => bail!("restore_installed=blocked reason=identity_contract"),
    }
}

pub(crate) fn restore_invocation_contract(
    private_root: &Utf8Path,
    admission_only: bool,
) -> (&'static Utf8Path, &'static str) {
    if admission_only {
        (Utf8Path::new(PREFLIGHT_ROOT), REMEDIATION_PLAN_RELATIVE)
    } else if private_root == Utf8Path::new(TCP_PAYLOAD_RECOVERY_ROOT) {
        (
            Utf8Path::new(TCP_PAYLOAD_RECOVERY_ROOT),
            TCP_PAYLOAD_PLAN_RELATIVE,
        )
    } else if private_root == Utf8Path::new(NOISE_DIAGNOSTIC_RESTORE_ROOT) {
        (
            Utf8Path::new(NOISE_DIAGNOSTIC_RESTORE_ROOT),
            NOISE_DIAGNOSTIC_PLAN_RELATIVE,
        )
    } else if private_root == Utf8Path::new(CAMPAIGN_RESTORE_ROOT) {
        (
            Utf8Path::new(CAMPAIGN_RESTORE_ROOT),
            REMEDIATION_PLAN_RELATIVE,
        )
    } else {
        (Utf8Path::new(EFFECT_ROOT), REMEDIATION_PLAN_RELATIVE)
    }
}
