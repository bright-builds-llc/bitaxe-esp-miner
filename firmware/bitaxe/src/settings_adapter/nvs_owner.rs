use std::sync::OnceLock;

use esp_idf_svc::nvs::EspDefaultNvsPartition;

use super::{settings_failure, SettingsAdapterFailure};

static DEFAULT_NVS_PARTITION: OnceLock<EspDefaultNvsPartition> = OnceLock::new();

pub(super) fn initialize() -> Result<(), SettingsAdapterFailure> {
    if DEFAULT_NVS_PARTITION.get().is_some() {
        return Ok(());
    }
    let partition = EspDefaultNvsPartition::take().map_err(settings_failure)?;
    DEFAULT_NVS_PARTITION
        .set(partition)
        .map_err(|_| SettingsAdapterFailure::failed("default NVS partition owner already set"))
}

pub(super) fn shared() -> Result<EspDefaultNvsPartition, SettingsAdapterFailure> {
    DEFAULT_NVS_PARTITION
        .get()
        .cloned()
        .ok_or_else(|| SettingsAdapterFailure::failed("default NVS partition owner unavailable"))
}
