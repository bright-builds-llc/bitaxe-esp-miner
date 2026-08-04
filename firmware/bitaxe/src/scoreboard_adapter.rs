//! Transactional indexed-NVS owner for the production nonce scoreboard.

use std::ffi::CString;
use std::fmt;
use std::sync::{Mutex, OnceLock};

use bitaxe_api::{
    Scoreboard, ScoreboardEntry, ScoreboardOwner, ScoreboardOwnerError, MAX_SCOREBOARD_ENTRIES,
};
use bitaxe_config::NVS_NAMESPACE;
use bitaxe_stratum::v1::production_work::ScoreboardCandidate;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_svc::sys;

static SCOREBOARD: OnceLock<Mutex<ScoreboardOwner>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreboardAdapterError {
    category: &'static str,
}

impl ScoreboardAdapterError {
    const fn new(category: &'static str) -> Self {
        Self { category }
    }

    #[must_use]
    pub const fn category(self) -> &'static str {
        self.category
    }
}

impl fmt::Display for ScoreboardAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.category)
    }
}

impl std::error::Error for ScoreboardAdapterError {}

/// Loads the exact indexed scoreboard once NVS is available.
pub fn initialize() -> Result<(), ScoreboardAdapterError> {
    let confirmed = load_scoreboard()?;
    let mut owner = scoreboard_owner()
        .lock()
        .map_err(|_| ScoreboardAdapterError::new("owner_lock_poisoned"))?;
    *owner = ScoreboardOwner::new(confirmed);
    Ok(())
}

/// Persists and publishes one valid current-generation nonce candidate.
pub fn record_candidate(candidate: ScoreboardCandidate) -> Result<(), ScoreboardAdapterError> {
    let submission = candidate.submission();
    let entry = ScoreboardEntry::new(
        candidate.difficulty(),
        submission.job_id.clone(),
        submission.extranonce2.clone(),
        submission.ntime,
        submission.nonce,
        submission.version_bits,
    );
    let mut owner = scoreboard_owner()
        .lock()
        .map_err(|_| ScoreboardAdapterError::new("owner_lock_poisoned"))?;
    owner
        .record_with(entry, persist_and_confirm)
        .map(|_mutation| ())
        .map_err(map_owner_error)
}

/// Returns an immutable API projection clone.
#[must_use]
pub fn entries() -> Vec<ScoreboardEntry> {
    let owner = scoreboard_owner();
    match owner.lock() {
        Ok(owner) => owner.entries().to_vec(),
        Err(poisoned) => {
            log::warn!("scoreboard=degraded category=owner_lock_poisoned inner_retained=true");
            poisoned.into_inner().entries().to_vec()
        }
    }
}

fn scoreboard_owner() -> &'static Mutex<ScoreboardOwner> {
    SCOREBOARD.get_or_init(|| Mutex::new(ScoreboardOwner::default()))
}

fn map_owner_error(error: ScoreboardOwnerError<ScoreboardAdapterError>) -> ScoreboardAdapterError {
    match error {
        ScoreboardOwnerError::InvalidEntry(_) => ScoreboardAdapterError::new("candidate_invalid"),
        ScoreboardOwnerError::Persistence(error) => error,
    }
}

fn persist_and_confirm(
    candidate: &Scoreboard,
    changed_from: usize,
) -> Result<(), ScoreboardAdapterError> {
    let partition = EspDefaultNvsPartition::take()
        .map_err(|_| ScoreboardAdapterError::new("partition_unavailable"))?;
    let mut nvs = EspNvs::new(partition, NVS_NAMESPACE, true)
        .map_err(|_| ScoreboardAdapterError::new("namespace_open_failed"))?;

    for (index, entry) in candidate.entries().iter().enumerate().skip(changed_from) {
        let value = entry
            .to_persisted()
            .map_err(|_| ScoreboardAdapterError::new("entry_encode_failed"))?;
        set_string(&mut nvs, &scoreboard_key(index), &value)?;
    }
    for index in candidate.len()..MAX_SCOREBOARD_ENTRIES {
        erase_key(&mut nvs, &scoreboard_key(index))?;
    }
    commit(&nvs)?;
    drop(nvs);

    let confirmed = load_scoreboard()?;
    let expected = candidate
        .persisted_projection()
        .map_err(|_| ScoreboardAdapterError::new("entry_encode_failed"))?;
    if confirmed != expected {
        return Err(ScoreboardAdapterError::new("reload_mismatch"));
    }
    Ok(())
}

fn load_scoreboard() -> Result<Scoreboard, ScoreboardAdapterError> {
    let partition = EspDefaultNvsPartition::take()
        .map_err(|_| ScoreboardAdapterError::new("partition_unavailable"))?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false)
        .map_err(|_| ScoreboardAdapterError::new("namespace_open_failed"))?;
    let mut entries = Vec::with_capacity(MAX_SCOREBOARD_ENTRIES);
    for index in 0..MAX_SCOREBOARD_ENTRIES {
        let Some(value) = read_string(&nvs, &scoreboard_key(index))? else {
            break;
        };
        if value.is_empty() {
            break;
        }
        match ScoreboardEntry::from_persisted(&value) {
            Ok(entry) => entries.push(entry),
            Err(_) => log::warn!(
                "scoreboard=load_skipped category=entry_malformed slot={}",
                index + 1
            ),
        }
    }
    Scoreboard::from_entries(entries).map_err(|_| ScoreboardAdapterError::new("load_invalid"))
}

fn scoreboard_key(index: usize) -> String {
    format!("scoreboard_{:02}", index + 1)
}

fn read_string(
    nvs: &EspNvs<NvsDefault>,
    key: &str,
) -> Result<Option<String>, ScoreboardAdapterError> {
    let Some(len) = nvs
        .str_len(key)
        .map_err(|_| ScoreboardAdapterError::new("entry_length_read_failed"))?
    else {
        return Ok(None);
    };
    let mut buffer = vec![0; len];
    nvs.get_str(key, &mut buffer)
        .map_err(|_| ScoreboardAdapterError::new("entry_read_failed"))
        .map(|maybe_value| maybe_value.map(str::to_owned))
}

fn set_string(
    nvs: &mut EspNvs<NvsDefault>,
    key: &str,
    value: &str,
) -> Result<(), ScoreboardAdapterError> {
    let key = CString::new(key).map_err(|_| ScoreboardAdapterError::new("key_invalid"))?;
    let value = CString::new(value).map_err(|_| ScoreboardAdapterError::new("value_invalid"))?;
    let result = unsafe { sys::nvs_set_str(nvs.handle(), key.as_ptr(), value.as_ptr()) };
    esp_result(result, "entry_write_failed")
}

fn erase_key(nvs: &mut EspNvs<NvsDefault>, key: &str) -> Result<(), ScoreboardAdapterError> {
    let key = CString::new(key).map_err(|_| ScoreboardAdapterError::new("key_invalid"))?;
    let result = unsafe { sys::nvs_erase_key(nvs.handle(), key.as_ptr()) };
    if result == sys::ESP_ERR_NVS_NOT_FOUND {
        return Ok(());
    }
    esp_result(result, "entry_erase_failed")
}

fn commit(nvs: &EspNvs<NvsDefault>) -> Result<(), ScoreboardAdapterError> {
    let result = unsafe { sys::nvs_commit(nvs.handle()) };
    esp_result(result, "commit_failed")
}

fn esp_result(
    result: sys::esp_err_t,
    category: &'static str,
) -> Result<(), ScoreboardAdapterError> {
    if result == sys::ESP_OK {
        return Ok(());
    }
    Err(ScoreboardAdapterError::new(category))
}
