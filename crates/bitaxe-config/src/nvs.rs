//! Pure NVS schema model.
//!
//! Breadcrumbs:
//! - `reference/esp-miner/main/nvs_config.h` defines upstream stored types and
//!   the settings table shape.
//! - `reference/esp-miner/main/nvs_config.c` defines namespace, key names,
//!   REST names, defaults, ranges, indexed behavior, and legacy migrations.

mod loading;
mod migration;
mod schema;
mod types;

#[cfg(test)]
mod tests;

pub use loading::load_setting_value;
pub use migration::{compatibility_writes_for_active, migration_decisions, migration_rules};
pub use schema::{
    all_settings_schema, project_settings_schema, DEFAULT_THEME_ACCENT_COLORS_JSON,
    DEFAULT_THEME_COLOR_SCHEME,
};
pub use types::{
    LoadedValue, MigrationDecision, MigrationRule, NvsErase, NvsKeyName, NvsSchemaError, NvsWrite,
    RestFieldName, SettingDefault, SettingSchema, StoredType, StoredValue, StoredValueKind,
    NVS_KEY_NAME_MAX_BYTES, NVS_NAMESPACE,
};
