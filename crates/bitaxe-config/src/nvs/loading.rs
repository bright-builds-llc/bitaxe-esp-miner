use super::{LoadedValue, SettingDefault, SettingSchema, StoredType, StoredValue, StoredValueKind};

/// Loads a raw stored value through schema defaults without any NVS side effect.
#[must_use]
pub fn load_setting_value(
    schema: &SettingSchema,
    maybe_stored: Option<&StoredValue>,
) -> LoadedValue {
    match schema.stored_type {
        StoredType::Str => load_string(schema, maybe_stored),
        StoredType::U16 => load_u16(schema, maybe_stored),
        StoredType::I32 => load_i32(schema, maybe_stored),
        StoredType::U64 => load_u64(schema, maybe_stored),
        StoredType::FloatString => load_float_string(schema, maybe_stored),
        StoredType::BoolAsU16 => load_bool_as_u16(schema, maybe_stored),
    }
}
fn load_string(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::String(value),
        ..
    }) = maybe_stored
    {
        if !value.is_empty() {
            return LoadedValue::Str(value.clone());
        }
    }

    LoadedValue::Str(default_string(schema))
}

fn load_u16(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::U16(value),
        ..
    }) = maybe_stored
    {
        return LoadedValue::U16(*value);
    }

    LoadedValue::U16(default_u16(schema))
}

fn load_i32(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::I32(value),
        ..
    }) = maybe_stored
    {
        return LoadedValue::I32(*value);
    }

    LoadedValue::I32(default_i32(schema))
}

fn load_u64(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::U64(value),
        ..
    }) = maybe_stored
    {
        return LoadedValue::U64(*value);
    }

    LoadedValue::U64(default_u64(schema))
}

fn load_float_string(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::String(value),
        ..
    }) = maybe_stored
    {
        if let Ok(parsed) = value.parse::<f32>() {
            return LoadedValue::Float(parsed);
        }
    }

    LoadedValue::Float(default_float(schema))
}

fn load_bool_as_u16(schema: &SettingSchema, maybe_stored: Option<&StoredValue>) -> LoadedValue {
    if let Some(StoredValue {
        value: StoredValueKind::U16(value),
        ..
    }) = maybe_stored
    {
        return LoadedValue::Bool(*value != 0);
    }

    LoadedValue::Bool(default_bool(schema))
}

fn default_string(schema: &SettingSchema) -> String {
    match schema.default_value {
        Some(SettingDefault::Str(value)) => value.to_owned(),
        _ => String::new(),
    }
}

fn default_u16(schema: &SettingSchema) -> u16 {
    match schema.default_value {
        Some(SettingDefault::U16(value)) => value,
        _ => 0,
    }
}

fn default_i32(schema: &SettingSchema) -> i32 {
    match schema.default_value {
        Some(SettingDefault::I32(value)) => value,
        _ => 0,
    }
}

fn default_u64(schema: &SettingSchema) -> u64 {
    match schema.default_value {
        Some(SettingDefault::U64(value)) => value,
        _ => 0,
    }
}

fn default_float(schema: &SettingSchema) -> f32 {
    match schema.default_value {
        Some(SettingDefault::Float(value)) => value,
        _ => 0.0,
    }
}

fn default_bool(schema: &SettingSchema) -> bool {
    match schema.default_value {
        Some(SettingDefault::Bool(value)) => value,
        _ => false,
    }
}
