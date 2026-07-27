//! Pure retained log download and raw `/api/ws` stream contracts.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/log_buffer.c`
//! - `reference/esp-miner/main/log_buffer.h`
//! - `reference/esp-miner/main/http_server/websocket_log.c`

mod heartbeat;
mod retention;
mod stream;

pub use heartbeat::{
    AcceptedStateReplayCadence, RuntimeHeartbeatModel, RuntimeHeartbeatSample,
    ACCEPTED_STATE_MONITOR_ATTACHMENT_MS, ACCEPTED_STATE_REPLAY_INTERVAL_MS,
    ACCEPTED_STATE_REPLAY_WINDOW_MS, ACCEPTED_STATE_RESTORE_WATCH_MS,
    RUNTIME_HEARTBEAT_EARLY_CADENCE_MS, RUNTIME_HEARTBEAT_EARLY_WINDOW_MS,
    RUNTIME_HEARTBEAT_STEADY_CADENCE_MS,
};
pub use retention::{
    log_download_headers, LogDownloadHeaders, RetainedLogBuffer, RetainedPair, RetainedPairError,
    DOWNLOAD_CONTENT_DISPOSITION, DOWNLOAD_CONTENT_TYPE, LOG_CHUNK_BYTES, LOG_RETENTION_BYTES,
};
pub use stream::RawLogStreamPlanner;

#[cfg(test)]
use retention::checked_retained_pair_bytes;
#[cfg(test)]
mod tests;
