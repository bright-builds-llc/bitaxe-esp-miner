//! ESP-IDF NVS adapter for storage-confirmed AxeOS hostname settings.

use std::ffi::CString;
use std::sync::{Mutex, MutexGuard, OnceLock};

use bitaxe_api::{
    ReloadedSettings, SettingsAdapterFailure, SettingsPersistenceAdapter, SettingsPersistencePlan,
    SettingsPersistenceTransaction, ThemePostPlan,
};
use bitaxe_config::nvs::StoredValueKind;
use bitaxe_config::{
    all_settings_schema, project_settings_schema, ConfirmedSnapshotReadHealth, NvsSnapshot,
    NvsWrite, StoredValue, Ultra205DefaultsAttestation, NVS_NAMESPACE,
};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDataType, NvsDefault};
use esp_idf_svc::sys;

static CURRENT_SETTINGS_SNAPSHOT: OnceLock<crate::settings_snapshot_store::ConfirmedSnapshotStore> =
    OnceLock::new();
static SETTINGS_TRANSACTION_LOCK: Mutex<()> = Mutex::new(());
const NETWORK_RECONNECT_PROBE_KEY: &str = "netreconprobe";

mod noise_diagnostic;
mod nvs_owner;
mod production;
mod protocol_gate;
mod protocol_gate_adapter;
mod self_test;
mod stratum_v2;
mod tcp_payload_diagnostic;
mod thermal_fault_stimulus;

pub(crate) use noise_diagnostic::{load_noise_diagnostic_admission, NoiseDiagnosticAdmission};
pub(crate) use production::{
    load_production_campaign_admission, read_production_pool_set, MiningCampaignStage,
};
pub(crate) use protocol_gate::{
    ConfiguredProtocolPlan, ConfiguredStratumProtocol, ProductionProtocolGateDecision,
};
pub(crate) use self_test::{
    clear_self_test_flag_and_record_receipt, load_self_test_admission, maybe_self_test_receipt,
    SelfTestAdmission, SelfTestReceipt,
};
pub(crate) use stratum_v2::{read_stratum_v2_pool_set, V2PoolSettings};
pub(crate) use tcp_payload_diagnostic::{
    load_tcp_payload_diagnostic_admission, TcpPayloadDiagnosticAdmission,
};
pub(crate) use thermal_fault_stimulus::ThermalFaultStimulusAdmission;

pub(crate) fn initialize_default_nvs_partition() -> Result<(), SettingsAdapterFailure> {
    nvs_owner::initialize()
}

pub(crate) fn default_nvs_partition() -> Result<EspDefaultNvsPartition, SettingsAdapterFailure> {
    nvs_owner::shared()
}

/// Firmware coordinator that opens writable NVS only after exact authority.
pub struct FirmwareSettingsAdapter {
    partition: EspDefaultNvsPartition,
}

impl FirmwareSettingsAdapter {
    /// Takes the default NVS partition without opening the settings namespace for writes.
    pub fn open() -> Result<Self, SettingsAdapterFailure> {
        let partition = default_nvs_partition()?;
        Ok(Self { partition })
    }
}

/// Exclusive settings transaction held from writable open through publication.
pub struct FirmwareSettingsTransaction {
    _transaction_guard: MutexGuard<'static, ()>,
    partition: EspDefaultNvsPartition,
    nvs: EspNvs<NvsDefault>,
}

impl SettingsPersistenceTransaction for FirmwareSettingsTransaction {
    fn write(&mut self, write: &NvsWrite) -> Result<(), SettingsAdapterFailure> {
        write_nvs(&mut self.nvs, write)
    }

    fn commit(&mut self) -> Result<(), SettingsAdapterFailure> {
        let result = unsafe { sys::nvs_commit(self.nvs.handle()) };
        esp_result("nvs_commit", result)
    }

    fn reload(
        &mut self,
        expected: &[NvsWrite],
    ) -> Result<ReloadedSettings, SettingsAdapterFailure> {
        let reloaded =
            EspNvs::new(self.partition.clone(), NVS_NAMESPACE, false).map_err(settings_failure)?;
        let mut writes_match = true;
        for write in expected {
            writes_match &= nvs_write_matches(&reloaded, write)?;
        }
        let public_snapshot = read_current_settings_snapshot_strict(&reloaded)?;
        Ok(ReloadedSettings::new(public_snapshot, writes_match))
    }

    fn publish(&mut self, candidate: NvsSnapshot) -> Result<(), SettingsAdapterFailure> {
        current_snapshot_cell()
            .publish(candidate)
            .map_err(|_| SettingsAdapterFailure::failed("settings snapshot lock poisoned"))
    }
}

impl SettingsPersistenceAdapter for FirmwareSettingsAdapter {
    type Transaction<'adapter>
        = FirmwareSettingsTransaction
    where
        Self: 'adapter;

    fn validate_accepted(
        &mut self,
        _plan: &SettingsPersistencePlan,
    ) -> Result<(), SettingsAdapterFailure> {
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
    let partition = default_nvs_partition()?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false).map_err(settings_failure)?;
    refresh_current_settings_snapshot_best_effort(&nvs);
    Ok(())
}

/// Atomically consumes the private one-shot network reconnect probe marker.
pub(crate) fn consume_network_reconnect_probe() -> Result<bool, SettingsAdapterFailure> {
    let _transaction_guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SettingsAdapterFailure::failed("settings transaction lock poisoned"))?;
    let partition = default_nvs_partition()?;
    let mut nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true).map_err(settings_failure)?;
    let marker = nvs
        .get_u16(NETWORK_RECONNECT_PROBE_KEY)
        .map_err(settings_failure)?;
    if marker != Some(1) {
        return Ok(false);
    }

    let c_key = c_string(NETWORK_RECONNECT_PROBE_KEY)?;
    let erase_result = unsafe { sys::nvs_erase_key(nvs.handle(), c_key.as_ptr()) };
    esp_result("nvs_erase_key", erase_result)?;
    let commit_result = unsafe { sys::nvs_commit(nvs.handle()) };
    esp_result("nvs_commit", commit_result)?;
    drop(nvs);

    let confirmed = EspNvs::new(partition, NVS_NAMESPACE, false).map_err(settings_failure)?;
    if confirmed
        .get_u16(NETWORK_RECONNECT_PROBE_KEY)
        .map_err(settings_failure)?
        .is_some()
    {
        return Err(SettingsAdapterFailure::failed(
            "network reconnect probe erasure was not confirmed",
        ));
    }
    Ok(true)
}

/// Loads and atomically consumes the private one-shot thermal fault stimulus.
pub(crate) fn load_thermal_fault_stimulus() -> Result<
    Option<ThermalFaultStimulusAdmission>,
    thermal_fault_stimulus::ThermalFaultStimulusReadError,
> {
    thermal_fault_stimulus::load()
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

/// Reads the response-only setting subset, including pool identity fields.
///
/// The returned snapshot is consumed by one system-info projection. Passwords,
/// Wi-Fi secrets, themes, and unrelated retained values are never admitted.
#[must_use]
pub fn current_system_info_settings_snapshot() -> NvsSnapshot {
    let Ok(_transaction_guard) = SETTINGS_TRANSACTION_LOCK.lock() else {
        log::warn!("system_info_settings=unavailable reason=transaction_lock_poisoned");
        return NvsSnapshot::new();
    };
    let partition = match default_nvs_partition() {
        Ok(partition) => partition,
        Err(error) => {
            log::warn!("system_info_settings=unavailable reason=nvs_partition error={error}");
            return NvsSnapshot::new();
        }
    };
    let nvs = match EspNvs::new(partition, NVS_NAMESPACE, false) {
        Ok(nvs) => nvs,
        Err(error) => {
            log::warn!("system_info_settings=unavailable reason=nvs_open error={error}");
            return NvsSnapshot::new();
        }
    };

    read_system_info_settings_snapshot_best_effort(&nvs)
}

/// Strictly reads and compares the loaded live settings with the Ultra 205 seed.
pub fn current_ultra205_defaults_attestation(
) -> Result<Ultra205DefaultsAttestation, SettingsAdapterFailure> {
    let _transaction_guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SettingsAdapterFailure::failed("settings transaction lock poisoned"))?;
    let partition = default_nvs_partition()?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false).map_err(settings_failure)?;
    let snapshot = read_all_settings_snapshot_strict(&nvs)?;
    Ok(Ultra205DefaultsAttestation::from_snapshot(&snapshot))
}

/// Returns the project-owned next-boot mining preference, defaulting to true.
#[must_use]
pub fn start_mining_on_boot() -> bool {
    let loaded = bitaxe_config::reload_snapshot(&current_settings_snapshot());
    match loaded.maybe_loaded_value("mineonboot") {
        Some(bitaxe_config::LoadedValue::Bool(value)) => *value,
        _ => true,
    }
}

/// Returns the confirmed upstream statistics retention frequency in seconds.
#[must_use]
pub fn statistics_frequency_seconds() -> u16 {
    let loaded = bitaxe_config::reload_snapshot(&current_settings_snapshot());
    match loaded.maybe_loaded_value("statsFrequency") {
        Some(bitaxe_config::LoadedValue::U16(value)) => *value,
        _ => 0,
    }
}

/// Persists and confirms the project-owned next-boot mining preference.
pub fn persist_start_mining_on_boot(value: bool) -> Result<(), SettingsAdapterFailure> {
    let _transaction_guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SettingsAdapterFailure::failed("settings transaction lock poisoned"))?;
    let partition = default_nvs_partition()?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, true).map_err(settings_failure)?;
    nvs.set_u16("mineonboot", u16::from(value))
        .map_err(settings_failure)?;
    let result = unsafe { sys::nvs_commit(nvs.handle()) };
    esp_result("nvs_commit", result)?;
    refresh_current_settings_snapshot_best_effort(&nvs);
    if start_mining_on_boot() != value {
        return Err(SettingsAdapterFailure::failed(
            "boot mining preference confirmation mismatch",
        ));
    }
    Ok(())
}

/// Persists, independently reloads, reconciles, and publishes a theme update.
pub fn persist_theme_update(plan: &ThemePostPlan) -> Result<(), SettingsAdapterFailure> {
    let _transaction_guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SettingsAdapterFailure::failed("settings transaction lock poisoned"))?;
    let partition = default_nvs_partition()?;
    let mut nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true).map_err(settings_failure)?;
    for write in plan.writes() {
        let NvsWrite::String { key, value } = write else {
            return Err(SettingsAdapterFailure::failed(
                "theme plan contained a non-string write",
            ));
        };
        set_nvs_string(&mut nvs, key.as_str(), &value)?;
    }
    let result = unsafe { sys::nvs_commit(nvs.handle()) };
    esp_result("nvs_commit", result)?;

    let reloaded = EspNvs::new(partition, NVS_NAMESPACE, false).map_err(settings_failure)?;
    let candidate = read_current_settings_snapshot_strict(&reloaded)?;
    if !plan.reconciles(&candidate) {
        return Err(SettingsAdapterFailure::failed(
            "theme settings confirmation mismatch",
        ));
    }
    current_snapshot_cell()
        .publish(candidate)
        .map_err(|_| SettingsAdapterFailure::failed("settings snapshot lock poisoned"))
}

/// Reads only the non-secret protocol selectors needed by the protocol gate.
///
/// Missing selectors use the project default of Stratum V1. Pool endpoints,
/// users, passwords, certificates, and ports are not opened by this gate.
#[must_use]
pub fn configured_protocol_gate() -> ProductionProtocolGateDecision {
    protocol_gate_adapter::read()
}

pub(crate) fn configured_protocol_plan(
) -> Result<ConfiguredProtocolPlan, ProductionProtocolGateDecision> {
    protocol_gate_adapter::read_plan()
}

fn current_snapshot_cell() -> &'static crate::settings_snapshot_store::ConfirmedSnapshotStore {
    CURRENT_SETTINGS_SNAPSHOT.get_or_init(|| {
        crate::settings_snapshot_store::ConfirmedSnapshotStore::new(NvsSnapshot::new())
    })
}

fn refresh_current_settings_snapshot_best_effort(nvs: &EspNvs<NvsDefault>) {
    let snapshot = read_current_settings_snapshot_best_effort(nvs);
    if current_snapshot_cell().publish(snapshot).is_err() {
        log::warn!("axeos_settings_snapshot=refresh_failed reason=mutex_poisoned");
    }
}

fn read_current_settings_snapshot_best_effort(nvs: &EspNvs<NvsDefault>) -> NvsSnapshot {
    let mut values = Vec::new();
    for schema in general_settings_schema()
        .into_iter()
        .filter(|schema| !is_pool_configuration_key(schema.key.as_str()))
    {
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
        let Some(value) = maybe_read_stored_value_best_effort(nvs, key, stored_type) else {
            continue;
        };

        values.push(StoredValue {
            key: schema.key,
            value,
        });
    }

    NvsSnapshot::from_values(values)
}

fn read_system_info_settings_snapshot_best_effort(nvs: &EspNvs<NvsDefault>) -> NvsSnapshot {
    let values = general_settings_schema()
        .into_iter()
        .filter(|schema| is_system_info_setting_key(schema.key.as_str()))
        .filter_map(|schema| {
            let key = schema.key.as_str();
            let maybe_stored_type = match nvs.find_key(key) {
                Ok(maybe_stored_type) => maybe_stored_type,
                Err(error) => {
                    log::warn!(
                        "system_info_settings=skip_key key={key} reason=find_key_failed error={error}"
                    );
                    return None;
                }
            };
            let stored_type = maybe_stored_type?;
            let value = maybe_read_stored_value_best_effort(nvs, key, stored_type)?;
            Some(StoredValue {
                key: schema.key,
                value,
            })
        })
        .collect::<Vec<_>>();

    NvsSnapshot::from_values(values)
}

fn read_current_settings_snapshot_strict(
    nvs: &EspNvs<NvsDefault>,
) -> Result<NvsSnapshot, SettingsAdapterFailure> {
    let mut values = Vec::new();
    for schema in general_settings_schema()
        .into_iter()
        .filter(|schema| !is_pool_configuration_key(schema.key.as_str()))
    {
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

fn read_all_settings_snapshot_strict(
    nvs: &EspNvs<NvsDefault>,
) -> Result<NvsSnapshot, SettingsAdapterFailure> {
    let mut values = Vec::new();
    for schema in general_settings_schema() {
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

fn is_pool_configuration_key(key: &str) -> bool {
    key.starts_with("stratum")
        || key.starts_with("fbstratum")
        || key.starts_with("sv2")
        || key.starts_with("fbsv2")
        || key == "usefbstartum"
}

fn is_system_info_setting_key(key: &str) -> bool {
    matches!(
        key,
        "display"
            | "rotation"
            | "invertscreen"
            | "displayTimeout"
            | "manualfanspeed"
            | "minfanspeed"
            | "temptarget"
            | "statsFrequency"
            | "oc_enabled"
            | "overheat_mode"
            | "stratumurl"
            | "stratumport"
            | "stratumuser"
            | "stratumdiff"
            | "stratumxnsub"
            | "stratumtls"
            | "stratumcert"
            | "stratumdecode"
            | "stratumprot"
            | "sv2authpubkey"
            | "sv2chantype"
            | "fbstratumurl"
            | "fbstratumport"
            | "fbstratumuser"
            | "fbstratumdiff"
            | "stratumfbxnsub"
            | "fbstratumtls"
            | "fbstratumcert"
            | "fbstratumdecode"
            | "fbstratumprot"
            | "fbsv2authpubk"
            | "fbsv2chantype"
    )
}

fn general_settings_schema() -> Vec<bitaxe_config::SettingSchema> {
    all_settings_schema()
        .into_iter()
        .chain(project_settings_schema())
        .collect()
}

fn maybe_read_stored_value_best_effort(
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

fn set_nvs_string(
    nvs: &mut EspNvs<NvsDefault>,
    key: &str,
    value: &str,
) -> Result<(), SettingsAdapterFailure> {
    let c_key = c_string(key)?;
    let c_value = c_string(value)?;
    let erase_result = unsafe { sys::nvs_erase_key(nvs.handle(), c_key.as_ptr()) };
    if erase_result != sys::ESP_OK && erase_result != sys::ESP_ERR_NVS_NOT_FOUND {
        return Err(settings_failure_code("nvs_erase_key", erase_result));
    }
    let result = unsafe { sys::nvs_set_str(nvs.handle(), c_key.as_ptr(), c_value.as_ptr()) };
    esp_result("nvs_set_str", result)
}

fn write_nvs(nvs: &mut EspNvs<NvsDefault>, write: &NvsWrite) -> Result<(), SettingsAdapterFailure> {
    match write {
        NvsWrite::String { key, value } => set_nvs_string(nvs, key.as_str(), value),
        NvsWrite::U16 { key, value } => nvs.set_u16(key.as_str(), *value).map_err(settings_failure),
        NvsWrite::I32 { key, value } => nvs.set_i32(key.as_str(), *value).map_err(settings_failure),
        NvsWrite::U64 { key, value } => nvs.set_u64(key.as_str(), *value).map_err(settings_failure),
    }
}

fn nvs_write_matches(
    nvs: &EspNvs<NvsDefault>,
    write: &NvsWrite,
) -> Result<bool, SettingsAdapterFailure> {
    match write {
        NvsWrite::String { key, value } => {
            read_string_value_strict(nvs, key.as_str()).map(|stored| stored == *value)
        }
        NvsWrite::U16 { key, value } => {
            read_u16_value_strict(nvs, key.as_str()).map(|stored| stored == *value)
        }
        NvsWrite::I32 { key, value } => {
            read_i32_value_strict(nvs, key.as_str()).map(|stored| stored == *value)
        }
        NvsWrite::U64 { key, value } => {
            read_u64_value_strict(nvs, key.as_str()).map(|stored| stored == *value)
        }
    }
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
