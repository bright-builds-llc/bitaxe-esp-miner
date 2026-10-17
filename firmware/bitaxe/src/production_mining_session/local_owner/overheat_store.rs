//! Saves and reads what local mining keeps across an overheat (ADR-0038), through the
//! verified settings transaction the API uses: write, commit, reload, reconcile and
//! publish. The write sets are pure (`bitaxe_runtime::local_mining::persistence`).
use bitaxe_api::{execute_settings_persistence_plan, SettingsPersistencePlan};
use bitaxe_config::{NvsWrite, NVS_NAMESPACE};
use bitaxe_runtime::local_mining::persistence::{OverheatBootState, OVERHEAT_STOP_MARKER_KEY};
use esp_idf_svc::nvs::EspNvs;

/// Commits one device-owned write set; the error names what failed, for the log.
pub(super) fn persist(writes: Vec<NvsWrite>) -> Result<(), String> {
    let plan = SettingsPersistencePlan::device_owned(writes).map_err(|error| error.to_string())?;
    let mut adapter = crate::settings_adapter::FirmwareSettingsAdapter::open()
        .map_err(|error| error.to_string())?;
    execute_settings_persistence_plan(&plan, &mut adapter)
        .map(|_success| ())
        .map_err(|report| format!("{:?}", report.reason()))
}

/// `overheat_mode` and the stop marker as saved at boot. An unreadable marker reads
/// as set, so a failed read keeps an overheat stopped until the operator resumes.
pub(super) fn boot_state() -> OverheatBootState {
    let maybe_marker = match read_stop_marker() {
        Ok(maybe_marker) => maybe_marker,
        Err(category) => {
            log::warn!("local_mining=overheat_marker_unreadable category={category}");
            Some(1)
        }
    };
    OverheatBootState::read(
        &crate::settings_adapter::current_settings_snapshot(),
        maybe_marker,
    )
}

fn read_stop_marker() -> Result<Option<u16>, String> {
    let partition =
        crate::settings_adapter::default_nvs_partition().map_err(|error| error.to_string())?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false).map_err(|error| error.to_string())?;
    nvs.get_u16(OVERHEAT_STOP_MARKER_KEY)
        .map_err(|error| error.to_string())
}
