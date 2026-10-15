//! Best nonce difficulty on the mining owner thread, as the reference updates it for
//! every nonce on a valid job (`asic_result_task.c` -> `SYSTEM_notify_found_nonce`).

/// Records one nonce difficulty and persists a new all-time best. Like the reference,
/// memory updates first and a failed write is only logged.
pub(super) fn record(difficulty: f64) {
    let Some(all_time) = crate::runtime_snapshot::record_nonce_difficulty(difficulty) else {
        return;
    };
    if let Err(error) = crate::best_difficulty_adapter::persist(all_time) {
        log::warn!(
            "best_difficulty=persist_failed category={}",
            error.category()
        );
    }
}
