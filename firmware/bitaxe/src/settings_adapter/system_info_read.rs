//! Existing response-only NVS read with exclusive transaction-wait/read timing.
use super::*;

pub(super) fn read(timing: &LiveStageProfiler) -> NvsSnapshot {
    let maybe_started = timing.maybe_start_stage();
    let lock_result = SETTINGS_TRANSACTION_LOCK.lock();
    timing.finish_stage(LiveStage::SettingsTransactionWait, maybe_started);
    let Ok(_transaction_guard) = lock_result else {
        log::warn!("system_info_settings=unavailable reason=transaction_lock_poisoned");
        return NvsSnapshot::new();
    };
    let maybe_started = timing.maybe_start_stage();
    let snapshot = read_in_transaction();
    timing.finish_stage(LiveStage::SettingsNvsRead, maybe_started);
    snapshot
}

pub(super) fn read_unprofiled() -> NvsSnapshot {
    let Ok(_transaction_guard) = SETTINGS_TRANSACTION_LOCK.lock() else {
        log::warn!("system_info_settings=unavailable reason=transaction_lock_poisoned");
        return NvsSnapshot::new();
    };
    read_in_transaction()
}

fn read_in_transaction() -> NvsSnapshot {
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
