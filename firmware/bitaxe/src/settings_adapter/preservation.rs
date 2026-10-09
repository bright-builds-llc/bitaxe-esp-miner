//! Read-only proof of public settings under the existing transaction owner.
use super::*;
mod model;
mod pool_configuration;

use pool_configuration::PoolConfigurationDigest;
use std::sync::OnceLock;

/// RAM-only boot capture of the stored pool configuration; never logged, serialized or persisted.
static BOOT_POOL_CONFIGURATION: OnceLock<PoolConfigurationDigest> = OnceLock::new();

/// Captures the stored pool configuration once per boot. Startup calls this before Wi-Fi, HTTP
/// and the Worker owner exist, so no lease or settings write can precede it. On failure the
/// capture stays empty and every later status reports the boolean as false.
#[inline(never)]
pub(crate) fn capture_boot_pool_configuration() {
    let Ok(digest) = with_settings(|nvs| pool_configuration::digest(|key| read_bounded(nvs, key)))
    else {
        log::warn!("pool_configuration_boot_capture=unavailable");
        return;
    };
    if BOOT_POOL_CONFIGURATION.set(digest).is_err() {
        log::warn!("pool_configuration_boot_capture=already_captured");
    }
}

pub(crate) fn read() -> Result<bitaxe_worker_control::SettingsPreservation, SettingsAdapterFailure>
{
    with_settings(|nvs| {
        let pool_configuration_unchanged = pool_configuration::unchanged_since_boot(
            BOOT_POOL_CONFIGURATION.get(),
            pool_configuration::digest(|key| read_bounded(nvs, key)),
        );
        model::fingerprint(|key| read_bounded(nvs, key), pool_configuration_unchanged)
    })
}

fn with_settings<T>(
    inspect: impl FnOnce(&EspNvs<NvsDefault>) -> Result<T, SettingsAdapterFailure>,
) -> Result<T, SettingsAdapterFailure> {
    let _transaction = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SettingsAdapterFailure::failed("preservation transaction unavailable"))?;
    let nvs =
        EspNvs::new(default_nvs_partition()?, NVS_NAMESPACE, false).map_err(settings_failure)?;
    inspect(&nvs)
}

fn read_bounded(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<Option<bitaxe_config::nvs::StoredValueKind>, SettingsAdapterFailure> {
    let Some(kind) = nvs.find_key(key).map_err(settings_failure)? else {
        return Ok(None);
    };
    if kind == NvsDataType::Str
        && nvs
            .str_len(key)
            .map_err(settings_failure)?
            .is_none_or(|length| length > 4096)
    {
        return Err(SettingsAdapterFailure::failed(
            "setting string exceeds storage bound",
        ));
    }
    read_stored_value_strict(nvs, key, kind).map(Some)
}
