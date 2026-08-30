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
pub(crate) const NOISE_AUTH_PREFLIGHT_ROOT: &str = "scratch/str005-noise-auth/preflight-001";
pub(crate) const NOISE_AUTH_DIAGNOSTIC_RESTORE_ROOT: &str =
    "scratch/str005-noise-auth/diagnostic-001/restoration";
pub(crate) const NOISE_AUTH_RECOVERY_ROOT: &str =
    "scratch/str005-noise-auth/recovery-001/restoration";
pub(crate) const NOISE_AUTH_PLAN_RELATIVE: &str =
    "docs/parity/work-plans/20260829T143226Z-STR-005-NOISE-AUTH/PLAN.md";
pub(crate) const NOISE_AUTH_PLAN_SHA256: &str =
    "9a3e5a630a52de6b8819dcb33aac64f5324df030fab50fd248fc33437b6587ea";
pub(crate) const TCP_PAYLOAD_RECOVERY_ROOT: &str =
    "scratch/str005-tcp-payload/recovery-003/restoration";
pub(crate) const TCP_PAYLOAD_PREFLIGHT_ROOT: &str = "scratch/str005-tcp-payload/preflight-009";
pub(crate) const TCP_PAYLOAD_DIAGNOSTIC_RESTORE_ROOT: &str =
    "scratch/str005-tcp-payload/diagnostic-009/restoration";
pub(crate) const TCP_PAYLOAD_PLAN_RELATIVE: &str =
    "docs/parity/work-plans/20260829T032813Z-STR-005-CONNECTION-IDENTITY/PLAN.md";
pub(crate) const TCP_PAYLOAD_PLAN_SHA256: &str =
    "544f57f8c940bc4e5cfeb69539928e153629b55dc12c5d04e404219ca48a5ba5";
pub(crate) const BWG_RESTORATION_PLAN_RELATIVE: &str =
    "docs/adr/0019-supervise-bwg-restoration-through-a-protected-browser-campaign.md";
pub(crate) const BWG_RESTORATION_PLAN_SHA256: &str =
    "eac4c2099b07f22f45c36e6c1daebad0723e759d86964b890a361525a4d1a2f2";
pub(crate) const NATIVE_USB_TRANSITION_PLAN_RELATIVE: &str =
    "docs/parity/work-plans/20260830T142327Z-NATIVE-USB-RECOVERY-TRANSITION/PLAN.md";
pub(crate) const NATIVE_USB_TRANSITION_PLAN_SHA256: &str =
    "cbc11639a51e67d24a04b33c05dd3dd2e570914be79f3a3d80b7326894e74eca";
pub(crate) const NATIVE_USB_TRANSITION_PREFLIGHT_ROOT: &str =
    "scratch/native-usb-transition/.preflight-002";
pub(crate) const NATIVE_USB_TRANSITION_PRIMARY_ROOT: &str =
    "scratch/native-usb-transition/recovery-002/restoration";
pub(crate) const NATIVE_USB_TRANSITION_CONTINGENCY_ROOT: &str =
    "scratch/native-usb-transition/recovery-003/restoration";

pub(crate) fn authorized_remediation_plan(
    action: &str,
    ordinal: u16,
) -> Result<(&'static str, &'static str)> {
    match (action, ordinal) {
        ("tcp_payload_diagnostic_restore", 9) => {
            Ok((TCP_PAYLOAD_PLAN_RELATIVE, TCP_PAYLOAD_PLAN_SHA256))
        }
        ("tcp_payload_restore_preflight", 9) => {
            Ok((TCP_PAYLOAD_PLAN_RELATIVE, TCP_PAYLOAD_PLAN_SHA256))
        }
        ("tcp_payload_recovery", 3) => Ok((TCP_PAYLOAD_PLAN_RELATIVE, TCP_PAYLOAD_PLAN_SHA256)),
        ("diagnostic_restore", 4) => {
            Ok((NOISE_DIAGNOSTIC_PLAN_RELATIVE, NOISE_DIAGNOSTIC_PLAN_SHA256))
        }
        (
            "noise_auth_restore_preflight"
            | "noise_auth_diagnostic_restore"
            | "noise_auth_recovery",
            1,
        ) => Ok((NOISE_AUTH_PLAN_RELATIVE, NOISE_AUTH_PLAN_SHA256)),
        ("preflight" | "start", 5) | ("campaign_restore", 7) => {
            Ok((REMEDIATION_PLAN_RELATIVE, REMEDIATION_PLAN_SHA256))
        }
        ("bwg_worker_restoration", 1..=999) => {
            Ok((BWG_RESTORATION_PLAN_RELATIVE, BWG_RESTORATION_PLAN_SHA256))
        }
        ("native_usb_recovery", 2 | 3) => Ok((
            NATIVE_USB_TRANSITION_PLAN_RELATIVE,
            NATIVE_USB_TRANSITION_PLAN_SHA256,
        )),
        _ => bail!("restore_installed=blocked reason=identity_contract"),
    }
}

pub(crate) fn restore_invocation_contract(
    private_root: &Utf8Path,
    admission_only: bool,
) -> (&Utf8Path, &'static str) {
    if admission_only && private_root == Utf8Path::new(NATIVE_USB_TRANSITION_PREFLIGHT_ROOT) {
        (
            Utf8Path::new(NATIVE_USB_TRANSITION_PREFLIGHT_ROOT),
            NATIVE_USB_TRANSITION_PLAN_RELATIVE,
        )
    } else if admission_only && private_root == Utf8Path::new(TCP_PAYLOAD_PREFLIGHT_ROOT) {
        (
            Utf8Path::new(TCP_PAYLOAD_PREFLIGHT_ROOT),
            TCP_PAYLOAD_PLAN_RELATIVE,
        )
    } else if admission_only && private_root == Utf8Path::new(NOISE_AUTH_PREFLIGHT_ROOT) {
        (
            Utf8Path::new(NOISE_AUTH_PREFLIGHT_ROOT),
            NOISE_AUTH_PLAN_RELATIVE,
        )
    } else if admission_only {
        (Utf8Path::new(PREFLIGHT_ROOT), REMEDIATION_PLAN_RELATIVE)
    } else if private_root == Utf8Path::new(TCP_PAYLOAD_DIAGNOSTIC_RESTORE_ROOT) {
        (
            Utf8Path::new(TCP_PAYLOAD_DIAGNOSTIC_RESTORE_ROOT),
            TCP_PAYLOAD_PLAN_RELATIVE,
        )
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
    } else if private_root == Utf8Path::new(NOISE_AUTH_DIAGNOSTIC_RESTORE_ROOT) {
        (
            Utf8Path::new(NOISE_AUTH_DIAGNOSTIC_RESTORE_ROOT),
            NOISE_AUTH_PLAN_RELATIVE,
        )
    } else if private_root == Utf8Path::new(NOISE_AUTH_RECOVERY_ROOT) {
        (
            Utf8Path::new(NOISE_AUTH_RECOVERY_ROOT),
            NOISE_AUTH_PLAN_RELATIVE,
        )
    } else if private_root == Utf8Path::new(NATIVE_USB_TRANSITION_PRIMARY_ROOT) {
        (
            Utf8Path::new(NATIVE_USB_TRANSITION_PRIMARY_ROOT),
            NATIVE_USB_TRANSITION_PLAN_RELATIVE,
        )
    } else if private_root == Utf8Path::new(NATIVE_USB_TRANSITION_CONTINGENCY_ROOT) {
        (
            Utf8Path::new(NATIVE_USB_TRANSITION_CONTINGENCY_ROOT),
            NATIVE_USB_TRANSITION_PLAN_RELATIVE,
        )
    } else if private_root == Utf8Path::new(CAMPAIGN_RESTORE_ROOT) {
        (
            Utf8Path::new(CAMPAIGN_RESTORE_ROOT),
            REMEDIATION_PLAN_RELATIVE,
        )
    } else if is_bwg_restoration_root(private_root) {
        (private_root, BWG_RESTORATION_PLAN_RELATIVE)
    } else {
        (Utf8Path::new(EFFECT_ROOT), REMEDIATION_PLAN_RELATIVE)
    }
}

pub(crate) fn is_bwg_restoration_root(private_root: &Utf8Path) -> bool {
    let value = private_root.as_str();
    let Some(attempt) = value
        .strip_prefix("scratch/bwg-worker-restoration/bwg007-attempt-")
        .and_then(|suffix| suffix.strip_suffix("/recovery"))
    else {
        return false;
    };
    attempt.len() == 3 && attempt.bytes().all(|byte| byte.is_ascii_digit())
}
