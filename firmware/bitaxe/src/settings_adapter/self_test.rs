//! Consume-before-use admission and terminal receipts for SELF-001.

use std::ffi::CString;

use bitaxe_config::NVS_NAMESPACE;
use bitaxe_safety::self_test::HardwareSelfTestCase;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::sys;

use super::SETTINGS_TRANSACTION_LOCK;

const KIND_KEY: &str = "selftestkind";
const LEASE_KEY: &str = "selftestlease";
const CASE_KEY: &str = "selftestcase";
const FLAG_KEY: &str = "selftest";
const RECEIPT_KEY: &str = "selftestrcpt";
const RECEIPT_LEASE_KEY: &str = "selftestrcid";
const KIND: &str = "ultra205_full_v1";
const ADMISSION_KEYS: [&str; 3] = [KIND_KEY, LEASE_KEY, CASE_KEY];
const MAX_STRING_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelfTestAdmission {
    lease: u64,
    case: HardwareSelfTestCase,
}

impl SelfTestAdmission {
    #[must_use]
    pub(crate) const fn lease(self) -> u64 {
        self.lease
    }

    #[must_use]
    pub(crate) const fn case(self) -> HardwareSelfTestCase {
        self.case
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelfTestReceipt {
    Cancelled,
    Passed,
}

impl SelfTestReceipt {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::Passed => "passed",
        }
    }

    fn parse(token: &str) -> Option<Self> {
        match token {
            "cancelled" => Some(Self::Cancelled),
            "passed" => Some(Self::Passed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelfTestAdmissionError {
    category: &'static str,
}

impl SelfTestAdmissionError {
    const fn new(category: &'static str) -> Self {
        Self { category }
    }

    #[must_use]
    pub(crate) const fn category(self) -> &'static str {
        self.category
    }
}

pub(crate) fn load_self_test_admission() -> Result<Option<SelfTestAdmission>, SelfTestAdmissionError>
{
    let _guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SelfTestAdmissionError::new("transaction_lock"))?;
    let partition =
        super::default_nvs_partition().map_err(|_| SelfTestAdmissionError::new("nvs_partition"))?;
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, false)
        .map_err(|_| SelfTestAdmissionError::new("nvs_open"))?;
    let mut any_present = false;
    for key in ADMISSION_KEYS {
        any_present |= nvs
            .find_key(key)
            .map_err(|_| SelfTestAdmissionError::new("tuple_presence"))?
            .is_some();
    }
    if !any_present {
        return Ok(None);
    }

    let maybe_kind = read_string(&nvs, KIND_KEY);
    let maybe_lease = nvs
        .get_u64(LEASE_KEY)
        .map_err(|_| SelfTestAdmissionError::new("lease_read"));
    let maybe_case = read_string(&nvs, CASE_KEY);
    let maybe_flag = nvs
        .get_u16(FLAG_KEY)
        .map_err(|_| SelfTestAdmissionError::new("flag_read"));
    let maybe_mine_on_boot = nvs
        .get_u16("mineonboot")
        .map_err(|_| SelfTestAdmissionError::new("mineonboot_read"));
    drop(nvs);

    erase_admission_tuple(&partition)?;

    let contract = (|| {
        let kind = maybe_kind?.ok_or_else(|| SelfTestAdmissionError::new("tuple_incomplete"))?;
        let lease = maybe_lease?.ok_or_else(|| SelfTestAdmissionError::new("tuple_incomplete"))?;
        let case = maybe_case?
            .and_then(|value| HardwareSelfTestCase::parse(&value))
            .ok_or_else(|| SelfTestAdmissionError::new("tuple_incomplete"))?;
        if kind != KIND
            || lease == 0
            || maybe_flag?.unwrap_or(0) != 1
            || maybe_mine_on_boot?.unwrap_or(1) != 0
        {
            return Err(SelfTestAdmissionError::new("tuple_contract"));
        }
        Ok(SelfTestAdmission { lease, case })
    })();
    if contract.is_err() {
        clear_flag(&partition)?;
    }
    contract.map(Some)
}

pub(crate) fn clear_self_test_flag_and_record_receipt(
    lease: u64,
    receipt: SelfTestReceipt,
) -> Result<(), SelfTestAdmissionError> {
    let _guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SelfTestAdmissionError::new("transaction_lock"))?;
    let partition =
        super::default_nvs_partition().map_err(|_| SelfTestAdmissionError::new("nvs_partition"))?;
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true)
        .map_err(|_| SelfTestAdmissionError::new("nvs_open_write"))?;
    nvs.set_u16(FLAG_KEY, 0)
        .map_err(|_| SelfTestAdmissionError::new("flag_clear"))?;
    set_string(&nvs, RECEIPT_KEY, receipt.token())?;
    nvs.set_u64(RECEIPT_LEASE_KEY, lease)
        .map_err(|_| SelfTestAdmissionError::new("receipt_lease_write"))?;
    commit(&nvs)?;
    drop(nvs);

    let confirmed = EspNvs::new(partition, NVS_NAMESPACE, false)
        .map_err(|_| SelfTestAdmissionError::new("confirm_open"))?;
    if confirmed
        .get_u16(FLAG_KEY)
        .map_err(|_| SelfTestAdmissionError::new("confirm_flag"))?
        != Some(0)
        || confirmed
            .get_u64(RECEIPT_LEASE_KEY)
            .map_err(|_| SelfTestAdmissionError::new("confirm_receipt_lease"))?
            != Some(lease)
        || read_string(&confirmed, RECEIPT_KEY)?.and_then(|value| SelfTestReceipt::parse(&value))
            != Some(receipt)
    {
        return Err(SelfTestAdmissionError::new("receipt_reconcile"));
    }
    Ok(())
}

pub(crate) fn maybe_self_test_receipt(
) -> Result<Option<(u64, SelfTestReceipt)>, SelfTestAdmissionError> {
    let _guard = SETTINGS_TRANSACTION_LOCK
        .lock()
        .map_err(|_| SelfTestAdmissionError::new("transaction_lock"))?;
    let partition =
        super::default_nvs_partition().map_err(|_| SelfTestAdmissionError::new("nvs_partition"))?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false)
        .map_err(|_| SelfTestAdmissionError::new("nvs_open"))?;
    let maybe_lease = nvs
        .get_u64(RECEIPT_LEASE_KEY)
        .map_err(|_| SelfTestAdmissionError::new("receipt_lease_read"))?;
    let maybe_receipt =
        read_string(&nvs, RECEIPT_KEY)?.and_then(|value| SelfTestReceipt::parse(&value));
    match (maybe_lease, maybe_receipt) {
        (None, None) => Ok(None),
        (Some(lease), Some(receipt)) if lease != 0 => Ok(Some((lease, receipt))),
        _ => Err(SelfTestAdmissionError::new("receipt_contract")),
    }
}

fn erase_admission_tuple(
    partition: &esp_idf_svc::nvs::EspDefaultNvsPartition,
) -> Result<(), SelfTestAdmissionError> {
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true)
        .map_err(|_| SelfTestAdmissionError::new("nvs_open_write"))?;
    for key in ADMISSION_KEYS {
        erase_key(&nvs, key)?;
    }
    commit(&nvs)?;
    drop(nvs);
    let confirmed = EspNvs::new(partition.clone(), NVS_NAMESPACE, false)
        .map_err(|_| SelfTestAdmissionError::new("confirm_open"))?;
    for key in ADMISSION_KEYS {
        if confirmed
            .find_key(key)
            .map_err(|_| SelfTestAdmissionError::new("confirm_read"))?
            .is_some()
        {
            return Err(SelfTestAdmissionError::new("tuple_replay"));
        }
    }
    Ok(())
}

fn clear_flag(
    partition: &esp_idf_svc::nvs::EspDefaultNvsPartition,
) -> Result<(), SelfTestAdmissionError> {
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true)
        .map_err(|_| SelfTestAdmissionError::new("nvs_open_write"))?;
    nvs.set_u16(FLAG_KEY, 0)
        .map_err(|_| SelfTestAdmissionError::new("flag_clear"))?;
    commit(&nvs)
}

fn read_string(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<Option<String>, SelfTestAdmissionError> {
    let Some(length) = nvs
        .str_len(key)
        .map_err(|_| SelfTestAdmissionError::new("string_length"))?
    else {
        return Ok(None);
    };
    if length == 0 || length > MAX_STRING_BYTES {
        return Err(SelfTestAdmissionError::new("string_size"));
    }
    let mut buffer = vec![0; length];
    nvs.get_str(key, &mut buffer)
        .map_err(|_| SelfTestAdmissionError::new("string_read"))
        .map(|maybe| maybe.map(str::to_owned))
}

fn set_string(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
    value: &str,
) -> Result<(), SelfTestAdmissionError> {
    nvs.set_str(key, value)
        .map_err(|_| SelfTestAdmissionError::new("string_write"))
}

fn erase_key(nvs: &EspNvs<NvsDefault>, key: &str) -> Result<(), SelfTestAdmissionError> {
    let key = CString::new(key).map_err(|_| SelfTestAdmissionError::new("tuple_key"))?;
    let result = unsafe { sys::nvs_erase_key(nvs.handle(), key.as_ptr()) };
    if result != sys::ESP_OK && result != sys::ESP_ERR_NVS_NOT_FOUND {
        return Err(SelfTestAdmissionError::new("tuple_clear"));
    }
    Ok(())
}

fn commit(nvs: &EspNvs<NvsDefault>) -> Result<(), SelfTestAdmissionError> {
    let result = unsafe { sys::nvs_commit(nvs.handle()) };
    if result != sys::ESP_OK {
        return Err(SelfTestAdmissionError::new("nvs_commit"));
    }
    Ok(())
}
