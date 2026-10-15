//! NVS owner for the all-time best nonce difficulty, upstream key `bestdiff` (u64).
//!
//! It takes neither the settings transaction lock nor the noise mutation guard, like the
//! scoreboard owner, so a record on the mining owner thread never waits on HTTP.

use std::fmt;

use bitaxe_config::NVS_NAMESPACE;
use esp_idf_svc::nvs::EspNvs;

/// The reference `NVS_CONFIG_BEST_DIFF` key name.
const BEST_DIFF_KEY: &str = "bestdiff";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BestDifficultyAdapterError {
    category: &'static str,
}

impl BestDifficultyAdapterError {
    const fn new(category: &'static str) -> Self {
        Self { category }
    }

    #[must_use]
    pub const fn category(self) -> &'static str {
        self.category
    }
}

impl fmt::Display for BestDifficultyAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.category)
    }
}

impl std::error::Error for BestDifficultyAdapterError {}

/// Restores this boot's all-time best from NVS; a missing key is 0, as in the reference.
pub fn initialize() -> Result<(), BestDifficultyAdapterError> {
    let all_time = load()?;
    crate::runtime_snapshot::restore_best_difficulty(all_time);
    Ok(())
}

/// Writes a new all-time best, commits, and confirms it by reading it back.
pub fn persist(all_time: u64) -> Result<(), BestDifficultyAdapterError> {
    let partition = crate::settings_adapter::default_nvs_partition()
        .map_err(|_| BestDifficultyAdapterError::new("partition_unavailable"))?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, true)
        .map_err(|_| BestDifficultyAdapterError::new("namespace_open_failed"))?;
    nvs.set_u64(BEST_DIFF_KEY, all_time)
        .map_err(|_| BestDifficultyAdapterError::new("write_failed"))?;
    // esp-idf-svc commits inside `set_u64`; the reload confirms the stored value.
    drop(nvs);
    if load()? != all_time {
        return Err(BestDifficultyAdapterError::new("reload_mismatch"));
    }
    Ok(())
}

fn load() -> Result<u64, BestDifficultyAdapterError> {
    let partition = crate::settings_adapter::default_nvs_partition()
        .map_err(|_| BestDifficultyAdapterError::new("partition_unavailable"))?;
    let nvs = EspNvs::new(partition, NVS_NAMESPACE, false)
        .map_err(|_| BestDifficultyAdapterError::new("namespace_open_failed"))?;
    let maybe_value = nvs
        .get_u64(BEST_DIFF_KEY)
        .map_err(|_| BestDifficultyAdapterError::new("read_failed"))?;
    Ok(maybe_value.unwrap_or(0))
}
