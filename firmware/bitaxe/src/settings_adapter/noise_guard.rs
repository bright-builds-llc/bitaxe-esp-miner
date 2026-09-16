//! Admission excludes any current settings transaction without waiting on NVS.
use super::{start_mining_on_boot, SettingsAdapterFailure, SETTINGS_TRANSACTION_LOCK};

pub(crate) fn claim_noise_fence(
    claim: impl FnOnce() -> bool,
) -> Result<(), SettingsAdapterFailure> {
    let _guard = SETTINGS_TRANSACTION_LOCK
        .try_lock()
        .map_err(|_| SettingsAdapterFailure::failed("settings transaction active"))?;
    if start_mining_on_boot() || !claim() {
        return Err(SettingsAdapterFailure::failed("diagnostic fence"));
    }
    Ok(())
}
