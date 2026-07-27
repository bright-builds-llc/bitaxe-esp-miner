use super::*;

pub(super) fn rows() -> Vec<SettingSchema> {
    vec![
        SettingSchema {
            key: key("asicfrequency"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(485)),
            rest_name: None,
            min: Some(1),
            max: Some(65535),
            array_size: None,
            provenance: MIGRATION_PROVENANCE,
        },
        SettingSchema {
            key: key("fanspeed"),
            stored_type: StoredType::U16,
            default_value: Some(SettingDefault::U16(100)),
            rest_name: None,
            min: Some(0),
            max: Some(100),
            array_size: None,
            provenance: MIGRATION_PROVENANCE,
        },
    ]
}
