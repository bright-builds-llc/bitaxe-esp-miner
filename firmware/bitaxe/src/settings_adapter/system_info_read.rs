//! Existing response-only NVS read with exclusive transaction-wait/read timing.
use super::*;

pub(super) fn read(timing: &LiveStageProfiler) -> NvsSnapshot {
    let Ok(_transaction_guard) = timing.measure(LiveStage::SettingsTransactionWait, || {
        SETTINGS_TRANSACTION_LOCK.lock()
    }) else {
        log::warn!("system_info_settings=unavailable reason=transaction_lock_poisoned");
        return NvsSnapshot::new();
    };
    timing.measure(LiveStage::SettingsNvsRead, || {
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
    })
}
