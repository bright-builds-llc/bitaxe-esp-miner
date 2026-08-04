//! Pure Bitaxe Accessory Port protocol contracts.
//!
//! Behavioral breadcrumbs:
//! - `reference/esp-miner/main/bap/bap_protocol.c`
//! - `reference/esp-miner/main/bap/bap_handlers.c`
//! - `reference/esp-miner/main/bap/bap_subscription.c`

mod semantics;
mod wire;

pub use semantics::{
    plan_command, BapConnectionMode, BapEffect, BapErrorCode, BapPlan, BapPlanError,
    BapRequestSnapshot, BapRestartPolicy, BapSettingIntent, BAP_DEFAULT_SUBSCRIPTION_INTERVAL_MS,
    BAP_SUBSCRIPTION_TIMEOUT_MS,
};
pub use wire::{
    bap_checksum, BapAdmission, BapChecksumDisposition, BapCommand, BapFrame, BapFrameError,
    BapIngress, BapParameter, BAP_DUPLICATE_WINDOW_MS, BAP_MAX_MESSAGE_LEN,
};

#[cfg(test)]
mod tests;
