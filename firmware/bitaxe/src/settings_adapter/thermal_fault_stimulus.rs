//! Consume-before-use admission for the private thermal fault stimulus.

use std::ffi::CString;

use bitaxe_config::NVS_NAMESPACE;
use bitaxe_safety::thermal_fault_stimulus::{
    THERMAL_FAULT_STIMULUS_KIND, THERMAL_FAULT_STIMULUS_SAMPLE_COUNT,
};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::sys;

use super::SETTINGS_TRANSACTION_LOCK;

const KIND_KEY: &str = "thermfault";
const LEASE_KEY: &str = "thermlease";
const SAMPLE_COUNT_KEY: &str = "thermcount";
const STIMULUS_KEYS: [&str; 3] = [KIND_KEY, LEASE_KEY, SAMPLE_COUNT_KEY];
const MAX_KIND_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ThermalFaultStimulusAdmission {
    lease: u64,
    sample_count: u16,
}

impl ThermalFaultStimulusAdmission {
    #[must_use]
    pub(crate) const fn sample_count(self) -> u16 {
        self.sample_count
    }

    #[must_use]
    pub(crate) const fn has_nonzero_lease(self) -> bool {
        self.lease != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ThermalFaultStimulusReadError {
    category: &'static str,
}

impl ThermalFaultStimulusReadError {
    const fn new(category: &'static str) -> Self {
        Self { category }
    }

    #[must_use]
    pub(crate) const fn category(self) -> &'static str {
        self.category
    }
}

/// Reads and clears the complete private tuple before returning authority.
///
/// Any present tuple is erased and committed even when malformed, preventing
/// a reboot from replaying either valid or partial stimulus state.
pub(crate) fn load() -> Result<Option<ThermalFaultStimulusAdmission>, ThermalFaultStimulusReadError>
{
    let _transaction_guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| ThermalFaultStimulusReadError::new("transaction_lock"))?;
    let partition = super::default_nvs_partition()
        .map_err(|_| ThermalFaultStimulusReadError::new("nvs_partition"))?;
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, false)
        .map_err(|_| ThermalFaultStimulusReadError::new("nvs_open"))?;
    let mut any_present = false;
    for key in STIMULUS_KEYS {
        any_present |= nvs
            .find_key(key)
            .map_err(|_| ThermalFaultStimulusReadError::new("tuple_presence"))?
            .is_some();
    }
    if !any_present {
        return Ok(None);
    }

    let kind = read_optional_string_bounded(&nvs, KIND_KEY);
    let lease = nvs
        .get_u64(LEASE_KEY)
        .map_err(|_| ThermalFaultStimulusReadError::new("lease_read"));
    let sample_count = nvs
        .get_u16(SAMPLE_COUNT_KEY)
        .map_err(|_| ThermalFaultStimulusReadError::new("sample_count_read"));
    let mine_on_boot = nvs
        .get_u16("mineonboot")
        .map_err(|_| ThermalFaultStimulusReadError::new("mineonboot_read"));
    drop(nvs);

    let writable = EspNvs::new(partition.clone(), NVS_NAMESPACE, true)
        .map_err(|_| ThermalFaultStimulusReadError::new("nvs_open_write"))?;
    erase_tuple(&writable)?;
    drop(writable);
    confirm_erased(partition)?;

    let kind = kind?.ok_or_else(|| ThermalFaultStimulusReadError::new("tuple_incomplete"))?;
    let lease = lease?.ok_or_else(|| ThermalFaultStimulusReadError::new("tuple_incomplete"))?;
    let sample_count =
        sample_count?.ok_or_else(|| ThermalFaultStimulusReadError::new("tuple_incomplete"))?;
    if kind != THERMAL_FAULT_STIMULUS_KIND
        || lease == 0
        || sample_count != THERMAL_FAULT_STIMULUS_SAMPLE_COUNT
        || mine_on_boot?.unwrap_or(1) != 0
    {
        return Err(ThermalFaultStimulusReadError::new("tuple_contract"));
    }

    Ok(Some(ThermalFaultStimulusAdmission {
        lease,
        sample_count,
    }))
}

fn read_optional_string_bounded(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<Option<String>, ThermalFaultStimulusReadError> {
    let Some(len) = nvs
        .str_len(key)
        .map_err(|_| ThermalFaultStimulusReadError::new("kind_length"))?
    else {
        return Ok(None);
    };
    if len == 0 || len > MAX_KIND_BYTES {
        return Err(ThermalFaultStimulusReadError::new("kind_size"));
    }
    let mut buffer = vec![0; len];
    nvs.get_str(key, &mut buffer)
        .map_err(|_| ThermalFaultStimulusReadError::new("kind_read"))
        .map(|maybe| maybe.map(str::to_owned))
}

fn erase_tuple(nvs: &EspNvs<NvsDefault>) -> Result<(), ThermalFaultStimulusReadError> {
    for key in STIMULUS_KEYS {
        let key = CString::new(key).map_err(|_| ThermalFaultStimulusReadError::new("tuple_key"))?;
        let result = unsafe { sys::nvs_erase_key(nvs.handle(), key.as_ptr()) };
        if result != sys::ESP_OK && result != sys::ESP_ERR_NVS_NOT_FOUND {
            return Err(ThermalFaultStimulusReadError::new("tuple_clear"));
        }
    }
    let result = unsafe { sys::nvs_commit(nvs.handle()) };
    if result != sys::ESP_OK {
        return Err(ThermalFaultStimulusReadError::new("tuple_commit"));
    }
    Ok(())
}

fn confirm_erased(
    partition: esp_idf_svc::nvs::EspDefaultNvsPartition,
) -> Result<(), ThermalFaultStimulusReadError> {
    let confirmed = EspNvs::new(partition, NVS_NAMESPACE, false)
        .map_err(|_| ThermalFaultStimulusReadError::new("confirm_open"))?;
    for key in STIMULUS_KEYS {
        if confirmed
            .find_key(key)
            .map_err(|_| ThermalFaultStimulusReadError::new("confirm_read"))?
            .is_some()
        {
            return Err(ThermalFaultStimulusReadError::new("tuple_replay"));
        }
    }
    Ok(())
}
