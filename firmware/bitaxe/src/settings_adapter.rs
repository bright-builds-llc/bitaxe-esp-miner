//! ESP-IDF NVS adapter for storage-confirmed AxeOS hostname settings.

use std::ffi::CString;
use std::sync::{Mutex, MutexGuard, OnceLock};

use bitaxe_api::{
    Hostname, SettingsAdapterFailure, SettingsPersistenceAdapter, SettingsPersistenceTransaction,
};
use bitaxe_config::nvs::StoredValueKind;
use bitaxe_config::{
    all_settings_schema, confirm_hostname_snapshot, ConfirmedHostnameSnapshot,
    ConfirmedSnapshotCell, ConfirmedSnapshotReadHealth, NvsSnapshot, StoredValue, NVS_NAMESPACE,
};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDataType, NvsDefault};
use esp_idf_svc::sys;

static CURRENT_SETTINGS_SNAPSHOT: OnceLock<ConfirmedSnapshotCell> = OnceLock::new();
static SETTINGS_TRANSACTION_LOCK: Mutex<()> = Mutex::new(());

/// Firmware coordinator that opens writable NVS only after exact authority.
pub struct FirmwareSettingsAdapter {
    partition: EspDefaultNvsPartition,
}

impl FirmwareSettingsAdapter {
    /// Takes the default NVS partition without opening the settings namespace for writes.
    pub fn open() -> Result<Self, SettingsAdapterFailure> {
        let partition = EspDefaultNvsPartition::take().map_err(settings_failure)?;
        Ok(Self { partition })
    }
}

/// Exclusive hostname transaction held from writable open through publication.
pub struct FirmwareSettingsTransaction {
    _transaction_guard: MutexGuard<'static, ()>,
    partition: EspDefaultNvsPartition,
    nvs: EspNvs<NvsDefault>,
}

impl FirmwareSettingsTransaction {
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
}

impl SettingsPersistenceTransaction for FirmwareSettingsTransaction {
    fn write_hostname(&mut self, hostname: &Hostname) -> Result<(), SettingsAdapterFailure> {
        self.set_string("hostname", hostname.as_str())
    }

    fn commit(&mut self) -> Result<(), SettingsAdapterFailure> {
        let result = unsafe { sys::nvs_commit(self.nvs.handle()) };
        esp_result("nvs_commit", result)
    }

    fn reload(&mut self) -> Result<ConfirmedHostnameSnapshot, SettingsAdapterFailure> {
        let reloaded =
            EspNvs::new(self.partition.clone(), NVS_NAMESPACE, false).map_err(settings_failure)?;
        let candidate = read_current_settings_snapshot_strict(&reloaded)?;
        confirm_hostname_snapshot(candidate).map_err(settings_failure)
    }

    fn publish(
        &mut self,
        candidate: ConfirmedHostnameSnapshot,
    ) -> Result<(), SettingsAdapterFailure> {
        current_snapshot_cell()
            .publish(candidate.into_snapshot())
            .map_err(|_| SettingsAdapterFailure::failed("settings snapshot lock poisoned"))
    }
}

impl SettingsPersistenceAdapter for FirmwareSettingsAdapter {
    type Transaction<'adapter>
        = FirmwareSettingsTransaction
    where
        Self: 'adapter;

    fn validate_accepted(&mut self, _hostname: &Hostname) -> Result<(), SettingsAdapterFailure> {
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<Self::Transaction<'_>, SettingsAdapterFailure> {
        let transaction_guard = SETTINGS_TRANSACTION_LOCK
            .lock()
            .map_err(|_| SettingsAdapterFailure::failed("settings transaction lock poisoned"))?;
        let nvs =
            EspNvs::new(self.partition.clone(), NVS_NAMESPACE, true).map_err(settings_failure)?;

        Ok(FirmwareSettingsTransaction {
            _transaction_guard: transaction_guard,
            partition: self.partition.clone(),
            nvs,
        })
    }
}

/// Best-effort startup load for the API-visible settings snapshot.
pub fn initialize_current_settings_snapshot() -> Result<(), SettingsAdapterFailure> {
    let partition = EspDefaultNvsPartition::take().map_err(settings_failure)?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false).map_err(settings_failure)?;
    refresh_current_settings_snapshot_best_effort(&nvs);
    Ok(())
}

/// Returns the last atomically published settings snapshot.
#[must_use]
pub fn current_settings_snapshot() -> NvsSnapshot {
    let read = current_snapshot_cell().read();
    if read.health() == ConfirmedSnapshotReadHealth::PoisonRecovered {
        log::warn!("axeos_settings_snapshot=degraded reason=mutex_poisoned_inner_retained");
    }

    read.into_snapshot()
}

fn current_snapshot_cell() -> &'static ConfirmedSnapshotCell {
    CURRENT_SETTINGS_SNAPSHOT.get_or_init(|| ConfirmedSnapshotCell::new(NvsSnapshot::new()))
}

fn refresh_current_settings_snapshot_best_effort(nvs: &EspNvs<NvsDefault>) {
    let snapshot = read_current_settings_snapshot_best_effort(nvs);
    if current_snapshot_cell().publish(snapshot).is_err() {
        log::warn!("axeos_settings_snapshot=refresh_failed reason=mutex_poisoned");
    }
}

fn read_current_settings_snapshot_best_effort(nvs: &EspNvs<NvsDefault>) -> NvsSnapshot {
    let mut values = Vec::new();
    for schema in all_settings_schema() {
        let key = schema.key.as_str();
        let maybe_stored_type = match nvs.find_key(key) {
            Ok(maybe_stored_type) => maybe_stored_type,
            Err(error) => {
                log::warn!(
                    "axeos_settings_snapshot=skip_key key={key} reason=find_key_failed error={error}"
                );
                continue;
            }
        };
        let Some(stored_type) = maybe_stored_type else {
            continue;
        };
        let Some(value) = read_stored_value_best_effort(nvs, key, stored_type) else {
            continue;
        };

        values.push(StoredValue {
            key: schema.key,
            value,
        });
    }

    NvsSnapshot::from_values(values)
}

fn read_current_settings_snapshot_strict(
    nvs: &EspNvs<NvsDefault>,
) -> Result<NvsSnapshot, SettingsAdapterFailure> {
    let mut values = Vec::new();
    for schema in all_settings_schema() {
        let key = schema.key.as_str();
        let maybe_stored_type = nvs.find_key(key).map_err(settings_failure)?;
        let Some(stored_type) = maybe_stored_type else {
            continue;
        };
        let value = read_stored_value_strict(nvs, key, stored_type)?;

        values.push(StoredValue {
            key: schema.key,
            value,
        });
    }

    Ok(NvsSnapshot::from_values(values))
}

fn read_stored_value_best_effort(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
    stored_type: NvsDataType,
) -> Option<StoredValueKind> {
    match read_stored_value_strict(nvs, key, stored_type) {
        Ok(value) => Some(value),
        Err(error) => {
            log::warn!(
                "axeos_settings_snapshot=skip_key key={key} reason=read_failed error={error}"
            );
            None
        }
    }
}

fn read_stored_value_strict(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
    stored_type: NvsDataType,
) -> Result<StoredValueKind, SettingsAdapterFailure> {
    match stored_type {
        NvsDataType::Str => read_string_value_strict(nvs, key).map(StoredValueKind::String),
        NvsDataType::U16 => read_u16_value_strict(nvs, key).map(StoredValueKind::U16),
        NvsDataType::I32 => read_i32_value_strict(nvs, key).map(StoredValueKind::I32),
        NvsDataType::U64 => read_u64_value_strict(nvs, key).map(StoredValueKind::U64),
        _ => Err(SettingsAdapterFailure::failed(
            "settings key has unsupported storage type",
        )),
    }
}

fn read_string_value_strict(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<String, SettingsAdapterFailure> {
    let len = nvs
        .str_len(key)
        .map_err(settings_failure)?
        .ok_or_else(|| SettingsAdapterFailure::failed("settings string length missing"))?;
    let mut buffer = vec![0; len];
    nvs.get_str(key, &mut buffer)
        .map_err(settings_failure)?
        .map(str::to_owned)
        .ok_or_else(|| SettingsAdapterFailure::failed("settings string value missing"))
}

fn read_u16_value_strict(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<u16, SettingsAdapterFailure> {
    nvs.get_u16(key)
        .map_err(settings_failure)?
        .ok_or_else(|| SettingsAdapterFailure::failed("settings u16 value missing"))
}

fn read_i32_value_strict(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<i32, SettingsAdapterFailure> {
    nvs.get_i32(key)
        .map_err(settings_failure)?
        .ok_or_else(|| SettingsAdapterFailure::failed("settings i32 value missing"))
}

fn read_u64_value_strict(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<u64, SettingsAdapterFailure> {
    nvs.get_u64(key)
        .map_err(settings_failure)?
        .ok_or_else(|| SettingsAdapterFailure::failed("settings u64 value missing"))
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
