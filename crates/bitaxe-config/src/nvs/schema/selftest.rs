use super::*;

pub(super) fn rows() -> Vec<SettingSchema> {
    vec![
        SettingSchema {
            key: key("selftest_temp"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(65)),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("selftest_warm"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(55)),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("selftest_max"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(70)),
            rest_name: None,
            min: None,
            max: None,
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
    ]
}
