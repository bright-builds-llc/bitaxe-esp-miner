//! Pure Bitaxe Accessory Port protocol contracts.
//!
//! Behavioral breadcrumbs:
//! - `reference/esp-miner/main/bap/bap_protocol.c`
//! - `reference/esp-miner/main/bap/bap_handlers.c`
//! - `reference/esp-miner/main/bap/bap_subscription.c`

mod reference_numbers;
mod semantics;
mod settings;
mod wire;

pub use reference_numbers::{reference_atof, reference_atoi, reference_atoi_u16};
pub use semantics::{
    plan_command, BapConnectionMode, BapEffect, BapErrorCode, BapPlan, BapPlanError,
    BapRequestSnapshot, BapRestartPolicy, BapSettingIntent, BAP_DEFAULT_SUBSCRIPTION_INTERVAL_MS,
    BAP_SUBSCRIPTION_TIMEOUT_MS,
};
pub use settings::{complete_setting, BapPendingSetting, BapSettingCompletion, BapSettingResult};
pub use wire::{
    bap_checksum, BapAdmission, BapChecksumDisposition, BapCommand, BapFrame, BapFrameError,
    BapIngress, BapParameter, BAP_DUPLICATE_WINDOW_MS, BAP_MAX_MESSAGE_LEN,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod reference_tests;

#[cfg(test)]
mod settings_tests;
