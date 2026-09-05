//! Read-only proof of public settings under the existing transaction owner.
use super::*;
mod model;

pub(crate) fn read() -> Result<bitaxe_worker_control::SettingsPreservation, SettingsAdapterFailure>
{
    let _transaction = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SettingsAdapterFailure::failed("preservation transaction unavailable"))?;
    let nvs =
        EspNvs::new(default_nvs_partition()?, NVS_NAMESPACE, false).map_err(settings_failure)?;
    model::fingerprint(|key| {
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
                "public setting string exceeds storage bound",
            ));
        }
        read_stored_value_strict(&nvs, key, kind).map(Some)
    })
}
