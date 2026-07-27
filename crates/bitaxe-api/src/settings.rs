//! Pure AxeOS settings PATCH request planning.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `reference/esp-miner/main/nvs_config.c`

mod patch;
mod persistence;

pub(crate) use patch::{parse_settings_patch_body, wrong_input};
pub use patch::{
    plan_settings_patch_body, plan_settings_patch_value, AcceptedSettingsPatch,
    SettingsPatchFailure, SettingsPatchFailureReason, SettingsPatchFieldError,
    SettingsPatchPublicError,
};
pub use persistence::{
    execute_settings_persistence_plan, SettingsAdapterFailure, SettingsPersistenceAdapter,
    SettingsPersistenceEffect, SettingsPersistenceFailure, SettingsPersistenceFailureDisposition,
    SettingsPersistenceFailureReport, SettingsPersistencePlan, SettingsPersistenceStep,
    SettingsPersistenceSuccess, SettingsPersistenceTransaction, SettingsPublicResponse,
};

#[cfg(test)]
mod tests;
