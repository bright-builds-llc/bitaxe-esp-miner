//! Device-owned settings (ADR-0038): what the device saves on its own behalf, such
//! as an overheat or the crash-loop streak. Writes go through the verified settings
//! transaction the API uses (write, commit, reload, reconcile, publish), restricted
//! to the device-owned allowlist; device-internal keys are read raw.
use bitaxe_api::{execute_settings_persistence_plan, SettingsPersistencePlan};
use bitaxe_config::{NvsWrite, NVS_NAMESPACE};
use esp_idf_svc::nvs::EspNvs;

/// Commits one device-owned write set; the error names what failed, for the log.
pub(crate) fn persist(writes: Vec<NvsWrite>) -> Result<(), String> {
    let plan = SettingsPersistencePlan::device_owned(writes).map_err(|error| error.to_string())?;
    let mut adapter = crate::settings_adapter::FirmwareSettingsAdapter::open()
        .map_err(|error| error.to_string())?;
    execute_settings_persistence_plan(&plan, &mut adapter)
        .map(|_success| ())
        .map_err(|report| format!("{:?}", report.reason()))
}

/// Reads one device-internal `u16`; `None` when it was never saved.
pub(crate) fn read_u16(key: &str) -> Result<Option<u16>, String> {
    let partition =
        crate::settings_adapter::default_nvs_partition().map_err(|error| error.to_string())?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false).map_err(|error| error.to_string())?;
    nvs.get_u16(key).map_err(|error| error.to_string())
}
