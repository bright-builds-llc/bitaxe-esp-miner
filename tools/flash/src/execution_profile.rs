//! Virtual packages must fail before device discovery or capture preparation.
use crate::*;

const MARKER: &[u8] = b"BITAXE_EXECUTION_PROFILE=virtual-ultra205";

pub(crate) fn reject_virtual_manifest(
    maybe_manifest: Option<&Utf8Path>,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let Some(path) = maybe_manifest else {
        return Ok(());
    };
    let bytes = environment.read_to_string(&environment.workspace_path(path))?;
    let value: serde_json::Value = serde_json::from_str(&bytes)?;
    if value
        .get("execution_profile")
        .is_some_and(|profile| profile != "physical-ultra205")
        || value
            .pointer("/build_identity/channel")
            .and_then(|v| v.as_str())
            == Some("virtual-ultra205")
        || value.get("hardware_eligible") == Some(&serde_json::Value::Bool(false))
    {
        bail!("identity_admission=blocked reason=virtual_image_not_physical");
    }
    Ok(())
}

pub(crate) fn reject_virtual_bytes(bytes: &[u8]) -> Result<()> {
    if bytes.windows(MARKER.len()).any(|window| window == MARKER) {
        bail!("identity_admission=blocked reason=virtual_image_not_physical");
    }
    Ok(())
}
