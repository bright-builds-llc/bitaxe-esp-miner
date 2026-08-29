//! Consume-before-use admission for the private Stratum V2 Noise diagnostic.

use std::ffi::CString;

use bitaxe_config::NVS_NAMESPACE;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::sys;

use super::SETTINGS_TRANSACTION_LOCK;

const KIND_KEY: &str = "sv2diagkind";
const LEASE_KEY: &str = "sv2diaglease";
const CASE_KEY: &str = "sv2diagcase";
const KIND: &str = "stratum_v2_noise_v1";
const CASE: &str = "noise_auth_v1";
const ADMISSION_KEYS: [&str; 3] = [KIND_KEY, LEASE_KEY, CASE_KEY];
const MAX_STRING_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NoiseDiagnosticAdmission {
    lease: u64,
}

impl NoiseDiagnosticAdmission {
    #[must_use]
    pub(crate) const fn lease(self) -> u64 {
        self.lease
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NoiseDiagnosticAdmissionError {
    category: &'static str,
}

impl NoiseDiagnosticAdmissionError {
    const fn new(category: &'static str) -> Self {
        Self { category }
    }

    #[must_use]
    pub(crate) const fn category(self) -> &'static str {
        self.category
    }
}

pub(crate) fn load_noise_diagnostic_admission(
) -> Result<Option<NoiseDiagnosticAdmission>, NoiseDiagnosticAdmissionError> {
    let _guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| NoiseDiagnosticAdmissionError::new("transaction_lock"))?;
    let partition = super::default_nvs_partition()
        .map_err(|_| NoiseDiagnosticAdmissionError::new("nvs_partition"))?;
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, false)
        .map_err(|_| NoiseDiagnosticAdmissionError::new("nvs_open"))?;
    let mut any_present = false;
    for key in ADMISSION_KEYS {
        any_present |= nvs
            .find_key(key)
            .map_err(|_| NoiseDiagnosticAdmissionError::new("tuple_presence"))?
            .is_some();
    }
    if !any_present {
        return Ok(None);
    }

    let maybe_kind = read_string(&nvs, KIND_KEY);
    let maybe_lease = nvs
        .get_u64(LEASE_KEY)
        .map_err(|_| NoiseDiagnosticAdmissionError::new("lease_read"));
    let maybe_case = read_string(&nvs, CASE_KEY);
    let maybe_mine_on_boot = nvs
        .get_u16("mineonboot")
        .map_err(|_| NoiseDiagnosticAdmissionError::new("mineonboot_read"));
    drop(nvs);

    erase_admission_tuple(&partition)?;
    let kind = maybe_kind?.ok_or_else(|| NoiseDiagnosticAdmissionError::new("tuple_incomplete"))?;
    let lease =
        maybe_lease?.ok_or_else(|| NoiseDiagnosticAdmissionError::new("tuple_incomplete"))?;
    let case = maybe_case?.ok_or_else(|| NoiseDiagnosticAdmissionError::new("tuple_incomplete"))?;
    if kind != KIND || case != CASE || lease == 0 || maybe_mine_on_boot?.unwrap_or(1) != 0 {
        return Err(NoiseDiagnosticAdmissionError::new("tuple_contract"));
    }
    Ok(Some(NoiseDiagnosticAdmission { lease }))
}

fn erase_admission_tuple(
    partition: &esp_idf_svc::nvs::EspDefaultNvsPartition,
) -> Result<(), NoiseDiagnosticAdmissionError> {
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true)
        .map_err(|_| NoiseDiagnosticAdmissionError::new("nvs_open_write"))?;
    for key in ADMISSION_KEYS {
        erase_key(&nvs, key)?;
    }
    commit(&nvs)?;
    drop(nvs);
    let confirmed = EspNvs::new(partition.clone(), NVS_NAMESPACE, false)
        .map_err(|_| NoiseDiagnosticAdmissionError::new("confirm_open"))?;
    for key in ADMISSION_KEYS {
        if confirmed
            .find_key(key)
            .map_err(|_| NoiseDiagnosticAdmissionError::new("confirm_read"))?
            .is_some()
        {
            return Err(NoiseDiagnosticAdmissionError::new("tuple_replay"));
        }
    }
    Ok(())
}

fn read_string(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<Option<String>, NoiseDiagnosticAdmissionError> {
    let Some(length) = nvs
        .str_len(key)
        .map_err(|_| NoiseDiagnosticAdmissionError::new("string_length"))?
    else {
        return Ok(None);
    };
    if length == 0 || length > MAX_STRING_BYTES {
        return Err(NoiseDiagnosticAdmissionError::new("string_size"));
    }
    let mut buffer = vec![0; length];
    nvs.get_str(key, &mut buffer)
        .map_err(|_| NoiseDiagnosticAdmissionError::new("string_read"))
        .map(|maybe| maybe.map(str::to_owned))
}

fn erase_key(nvs: &EspNvs<NvsDefault>, key: &str) -> Result<(), NoiseDiagnosticAdmissionError> {
    let key = CString::new(key).map_err(|_| NoiseDiagnosticAdmissionError::new("tuple_key"))?;
    let result = unsafe { sys::nvs_erase_key(nvs.handle(), key.as_ptr()) };
    if result != sys::ESP_OK && result != sys::ESP_ERR_NVS_NOT_FOUND {
        return Err(NoiseDiagnosticAdmissionError::new("tuple_clear"));
    }
    Ok(())
}

fn commit(nvs: &EspNvs<NvsDefault>) -> Result<(), NoiseDiagnosticAdmissionError> {
    let result = unsafe { sys::nvs_commit(nvs.handle()) };
    if result != sys::ESP_OK {
        return Err(NoiseDiagnosticAdmissionError::new("nvs_commit"));
    }
    Ok(())
}
