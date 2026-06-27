//! ESP-IDF NVS adapter for accepted AxeOS settings PATCH plans.

use std::ffi::CString;

use bitaxe_api::{SettingsAdapterFailure, SettingsPersistenceAdapter};
use bitaxe_config::{NvsSnapshot, NvsWrite, NVS_NAMESPACE};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_svc::sys;

/// Firmware adapter that applies pure settings write plans to ESP-IDF NVS.
pub struct FirmwareSettingsAdapter {
    nvs: EspNvs<NvsDefault>,
}

impl FirmwareSettingsAdapter {
    /// Opens the upstream ESP-Miner NVS namespace for read/write settings persistence.
    pub fn open() -> Result<Self, SettingsAdapterFailure> {
        let partition = EspDefaultNvsPartition::take().map_err(settings_failure)?;
        let nvs = EspNvs::new(partition, NVS_NAMESPACE, true).map_err(settings_failure)?;

        Ok(Self { nvs })
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
        let partition = EspDefaultNvsPartition::take().map_err(settings_failure)?;
        EspNvs::new(partition, NVS_NAMESPACE, false).map_err(settings_failure)?;
        Ok(())
    }
}

/// Returns the current settings snapshot boundary used for pure effect planning.
#[must_use]
pub fn current_settings_snapshot() -> NvsSnapshot {
    NvsSnapshot::new()
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
