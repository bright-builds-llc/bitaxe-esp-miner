//! Production retained-correlation adapter for completed operator snapshots.

use crate::log_buffer::{retain_operator_snapshot_pair, RetainedPairStorageError};

/// Retains and then logs one completed operator-snapshot correlation pair.
pub fn retain_completed_operator_snapshot(
    marker: &str,
    runtime_health: &str,
) -> Result<(), RetainedPairStorageError> {
    retain_operator_snapshot_pair(marker, runtime_health)?;
    log::info!("{marker}");
    log::info!("{runtime_health}");
    Ok(())
}
