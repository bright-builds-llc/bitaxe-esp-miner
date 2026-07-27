use super::{NvsKeyName, RestFieldName, SettingDefault, SettingSchema, StoredType};

mod controls;
mod fallback_stratum;
mod legacy;
mod misc;
mod network;
mod platform;
mod primary_stratum;
mod selftest;
mod sensors;

const SETTINGS_PROVENANCE: &str = "reference/esp-miner/main/nvs_config.c settings table";
const MIGRATION_PROVENANCE: &str = "reference/esp-miner/main/nvs_config.c fallback migration";
const PROJECT_SETTINGS_PROVENANCE: &str =
    "docs/adr/0016-production-mining-session.md project-owned boot preference";

pub(super) fn key(value: &'static str) -> NvsKeyName {
    NvsKeyName::parse(value).expect("static upstream NVS key names must fit ESP-IDF limits")
}

pub(super) fn rest(value: &'static str) -> RestFieldName {
    RestFieldName::parse(value).expect("static upstream REST field names must be non-empty")
}

/// Returns typed settings schema rows derived from the pinned upstream table.
#[must_use]
pub fn all_settings_schema() -> Vec<SettingSchema> {
    [
        network::rows(),
        primary_stratum::rows(),
        fallback_stratum::rows(),
        controls::rows(),
        misc::rows(),
        platform::rows(),
        sensors::rows(),
        selftest::rows(),
        legacy::rows(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Returns project-owned settings kept separate from the upstream-exact schema.
#[must_use]
pub fn project_settings_schema() -> Vec<SettingSchema> {
    vec![SettingSchema {
        key: key("mineonboot"),
        stored_type: StoredType::BoolAsU16,
        default_value: Some(SettingDefault::Bool(true)),
        rest_name: Some(rest("startMiningOnBoot")),
        min: Some(0),
        max: Some(1),
        array_size: None,
        provenance: PROJECT_SETTINGS_PROVENANCE,
    }]
}
