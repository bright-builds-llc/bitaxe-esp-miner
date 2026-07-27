use super::*;

pub(super) fn rows() -> Vec<SettingSchema> {
    vec![
        SettingSchema {
            key: key("wifissid"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("ssid")),
            min: Some(1),
            max: Some(32),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("wifipass"),
            stored_type: StoredType::Str,
            default_value: None,
            rest_name: Some(rest("wifiPass")),
            min: Some(0),
            max: Some(63),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
        SettingSchema {
            key: key("hostname"),
            stored_type: StoredType::Str,
            default_value: Some(SettingDefault::Str("bitaxe")),
            rest_name: Some(rest("hostname")),
            min: Some(1),
            max: Some(32),
            array_size: None,
            provenance: SETTINGS_PROVENANCE,
        },
    ]
}
