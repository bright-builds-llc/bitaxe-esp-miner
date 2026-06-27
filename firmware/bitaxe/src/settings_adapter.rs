//! ESP-IDF NVS adapter for accepted AxeOS settings PATCH plans.

use std::ffi::CString;
use std::sync::{Mutex, OnceLock};

use bitaxe_api::{SettingsAdapterFailure, SettingsPersistenceAdapter};
use bitaxe_config::nvs::StoredValueKind;
use bitaxe_config::{
    all_settings_schema, NvsSnapshot, NvsWrite, StoredType, StoredValue, NVS_NAMESPACE,
};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDataType, NvsDefault};
use esp_idf_svc::sys;

static CURRENT_SETTINGS_SNAPSHOT: OnceLock<Mutex<NvsSnapshot>> = OnceLock::new();

/// Firmware adapter that applies pure settings write plans to ESP-IDF NVS.
pub struct FirmwareSettingsAdapter {
    partition: EspDefaultNvsPartition,
    nvs: EspNvs<NvsDefault>,
}

impl FirmwareSettingsAdapter {
    /// Opens the upstream ESP-Miner NVS namespace for read/write settings persistence.
    pub fn open() -> Result<Self, SettingsAdapterFailure> {
        let partition = EspDefaultNvsPartition::take().map_err(settings_failure)?;
        let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true).map_err(settings_failure)?;
        refresh_current_settings_snapshot(&nvs);

        Ok(Self { partition, nvs })
    }

    fn set_string(&mut self, key: &str, value: &str) -> Result<(), SettingsAdapterFailure> {
        let c_key = c_string(key)?;
        let c_value = c_string(value)?;
        let erase_result = unsafe { sys::nvs_erase_key(self.nvs.handle(), c_key.as_ptr()) };
        if erase_result != sys::ESP_OK && erase_result != sys::ESP_ERR_NVS_NOT_FOUND {
            return Err(settings_failure_code("nvs_erase_key", erase_result));
        }

        let result =
            unsafe { sys::nvs_set_str(self.nvs.handle(), c_key.as_ptr(), c_value.as_ptr()) };
        esp_result("nvs_set_str", result)
    }

    fn set_u16(&mut self, key: &str, value: u16) -> Result<(), SettingsAdapterFailure> {
        let c_key = c_string(key)?;
        let result = unsafe { sys::nvs_set_u16(self.nvs.handle(), c_key.as_ptr(), value) };
        esp_result("nvs_set_u16", result)
    }

    fn set_i32(&mut self, key: &str, value: i32) -> Result<(), SettingsAdapterFailure> {
        let c_key = c_string(key)?;
        let result = unsafe { sys::nvs_set_i32(self.nvs.handle(), c_key.as_ptr(), value) };
        esp_result("nvs_set_i32", result)
    }

    fn set_u64(&mut self, key: &str, value: u64) -> Result<(), SettingsAdapterFailure> {
        let c_key = c_string(key)?;
        let result = unsafe { sys::nvs_set_u64(self.nvs.handle(), c_key.as_ptr(), value) };
        esp_result("nvs_set_u64", result)
    }
}

impl SettingsPersistenceAdapter for FirmwareSettingsAdapter {
    fn validate_accepted(&mut self) -> Result<(), SettingsAdapterFailure> {
        Ok(())
    }

    fn write(&mut self, write: &NvsWrite) -> Result<(), SettingsAdapterFailure> {
        match write {
            NvsWrite::String { key, value } => self.set_string(key.as_str(), value),
            NvsWrite::U16 { key, value } => self.set_u16(key.as_str(), *value),
            NvsWrite::I32 { key, value } => self.set_i32(key.as_str(), *value),
            NvsWrite::U64 { key, value } => self.set_u64(key.as_str(), *value),
        }
    }

    fn commit(&mut self) -> Result<(), SettingsAdapterFailure> {
        let result = unsafe { sys::nvs_commit(self.nvs.handle()) };
        esp_result("nvs_commit", result)
    }

    fn reload(&mut self) -> Result<(), SettingsAdapterFailure> {
        let reloaded =
            EspNvs::new(self.partition.clone(), NVS_NAMESPACE, false).map_err(settings_failure)?;
        refresh_current_settings_snapshot(&reloaded);
        Ok(())
    }
}

/// Best-effort startup load for the API-visible settings snapshot.
pub fn initialize_current_settings_snapshot() -> Result<(), SettingsAdapterFailure> {
    let partition = EspDefaultNvsPartition::take().map_err(settings_failure)?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false).map_err(settings_failure)?;
    refresh_current_settings_snapshot(&nvs);
    Ok(())
}

/// Returns the current settings snapshot boundary used for pure effect planning.
#[must_use]
pub fn current_settings_snapshot() -> NvsSnapshot {
    let snapshot = current_snapshot_cell();
    let Ok(snapshot) = snapshot.lock() else {
        log::warn!("axeos_settings_snapshot=unavailable reason=mutex_poisoned");
        return NvsSnapshot::new();
    };

    snapshot.clone()
}

/// Records successfully persisted writes in the runtime settings snapshot.
pub fn apply_persisted_settings_writes(writes: &[NvsWrite]) {
    if writes.is_empty() {
        return;
    }

    let snapshot = current_snapshot_cell();
    let Ok(mut snapshot) = snapshot.lock() else {
        log::warn!("axeos_settings_snapshot=update_failed reason=mutex_poisoned");
        return;
    };

    snapshot.apply_writes(writes);
}

fn current_snapshot_cell() -> &'static Mutex<NvsSnapshot> {
    CURRENT_SETTINGS_SNAPSHOT.get_or_init(|| Mutex::new(NvsSnapshot::new()))
}

fn refresh_current_settings_snapshot(nvs: &EspNvs<NvsDefault>) {
    let snapshot = read_current_settings_snapshot(nvs);
    let cell = current_snapshot_cell();
    let Ok(mut current) = cell.lock() else {
        log::warn!("axeos_settings_snapshot=refresh_failed reason=mutex_poisoned");
        return;
    };

    *current = snapshot;
}

fn read_current_settings_snapshot(nvs: &EspNvs<NvsDefault>) -> NvsSnapshot {
    let mut values = Vec::new();
    for schema in all_settings_schema() {
        let key = schema.key.as_str();
        let maybe_stored_type = match nvs.find_key(key) {
            Ok(maybe_stored_type) => maybe_stored_type,
            Err(error) => {
                log::warn!("axeos_settings_snapshot=skip_key key={key} reason=find_key_failed error={error}");
                continue;
            }
        };
        let Some(stored_type) = maybe_stored_type else {
            continue;
        };
        let Some(value) = read_stored_value(nvs, key, schema.stored_type, stored_type) else {
            continue;
        };

        values.push(StoredValue {
            key: schema.key,
            value,
        });
    }

    NvsSnapshot::from_values(values)
}

fn read_stored_value(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
    expected_type: StoredType,
    stored_type: NvsDataType,
) -> Option<StoredValueKind> {
    match stored_type {
        NvsDataType::Str => read_string_value(nvs, key).map(StoredValueKind::String),
        NvsDataType::U16 => read_u16_value(nvs, key).map(StoredValueKind::U16),
        NvsDataType::I32 => read_i32_value(nvs, key).map(StoredValueKind::I32),
        NvsDataType::U64 => read_u64_value(nvs, key).map(StoredValueKind::U64),
        unsupported => {
            log::warn!(
                "axeos_settings_snapshot=skip_key key={key} expected_type={expected_type:?} stored_type={unsupported:?}"
            );
            None
        }
    }
}

fn read_string_value(nvs: &EspNvs<NvsDefault>, key: &str) -> Option<String> {
    let maybe_len = match nvs.str_len(key) {
        Ok(maybe_len) => maybe_len,
        Err(error) => {
            log::warn!(
                "axeos_settings_snapshot=skip_key key={key} reason=str_len_failed error={error}"
            );
            return None;
        }
    };
    let Some(len) = maybe_len else {
        return None;
    };

    let mut buffer = vec![0; len];
    match nvs.get_str(key, &mut buffer) {
        Ok(Some(value)) => Some(value.to_owned()),
        Ok(None) => None,
        Err(error) => {
            log::warn!(
                "axeos_settings_snapshot=skip_key key={key} reason=get_str_failed error={error}"
            );
            None
        }
    }
}

fn read_u16_value(nvs: &EspNvs<NvsDefault>, key: &str) -> Option<u16> {
    match nvs.get_u16(key) {
        Ok(value) => value,
        Err(error) => {
            log::warn!(
                "axeos_settings_snapshot=skip_key key={key} reason=get_u16_failed error={error}"
            );
            None
        }
    }
}

fn read_i32_value(nvs: &EspNvs<NvsDefault>, key: &str) -> Option<i32> {
    match nvs.get_i32(key) {
        Ok(value) => value,
        Err(error) => {
            log::warn!(
                "axeos_settings_snapshot=skip_key key={key} reason=get_i32_failed error={error}"
            );
            None
        }
    }
}

fn read_u64_value(nvs: &EspNvs<NvsDefault>, key: &str) -> Option<u64> {
    match nvs.get_u64(key) {
        Ok(value) => value,
        Err(error) => {
            log::warn!(
                "axeos_settings_snapshot=skip_key key={key} reason=get_u64_failed error={error}"
            );
            None
        }
    }
}

fn c_string(value: &str) -> Result<CString, SettingsAdapterFailure> {
    CString::new(value).map_err(settings_failure)
}

fn esp_result(operation: &str, result: sys::esp_err_t) -> Result<(), SettingsAdapterFailure> {
    if result == sys::ESP_OK {
        return Ok(());
    }

    Err(settings_failure_code(operation, result))
}

fn settings_failure(error: impl core::fmt::Display) -> SettingsAdapterFailure {
    SettingsAdapterFailure::failed(error.to_string())
}

fn settings_failure_code(operation: &str, result: sys::esp_err_t) -> SettingsAdapterFailure {
    SettingsAdapterFailure::failed(format!("{operation} failed with esp_err={result}"))
}
