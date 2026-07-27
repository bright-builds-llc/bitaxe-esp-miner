use super::{
    MigrationDecision, MigrationRule, NvsErase, NvsKeyName, NvsWrite, StoredValue, StoredValueKind,
};

fn key(value: &'static str) -> NvsKeyName {
    NvsKeyName::parse(value).expect("static upstream NVS key names must fit ESP-IDF limits")
}

/// Returns the upstream legacy migration rules captured by this model.
#[must_use]
pub fn migration_rules() -> Vec<MigrationRule> {
    vec![
        MigrationRule {
            source_key: key("asicfrequency"),
            target_key: key("asicfrequency_f"),
            description: "legacy u16 ASIC frequency to active float string key",
        },
        MigrationRule {
            source_key: key("fanspeed"),
            target_key: key("manualfanspeed"),
            description: "legacy manual fan speed key to active u16 key",
        },
        MigrationRule {
            source_key: key("stratumprot"),
            target_key: key("stratumprot"),
            description: "stratum protocol u16 storage to string storage",
        },
        MigrationRule {
            source_key: key("fbstratumprot"),
            target_key: key("fbstratumprot"),
            description: "fallback stratum protocol u16 storage to string storage",
        },
        MigrationRule {
            source_key: key("sv2chantype"),
            target_key: key("sv2chantype"),
            description: "SV2 channel type u16 storage to string storage",
        },
        MigrationRule {
            source_key: key("fbsv2chantype"),
            target_key: key("fbsv2chantype"),
            description: "fallback SV2 channel type u16 storage to string storage",
        },
        MigrationRule {
            source_key: key("fbSv2ChanType"),
            target_key: key("sv2chantype"),
            description:
                "legacy mixed-case fallback SV2 channel type key to first missing SV2 channel key",
        },
    ]
}

/// Returns ordered migration decisions for raw values already read from NVS.
#[must_use]
pub fn migration_decisions(stored_values: &[StoredValue]) -> Vec<MigrationDecision> {
    let mut decisions = Vec::new();

    if !has_key(stored_values, "asicfrequency_f") {
        if let Some(value) = maybe_u16(stored_values, "asicfrequency") {
            decisions.push(MigrationDecision::Write(NvsWrite::string(
                "asicfrequency_f",
                value.to_string(),
            )));
        }
    }

    if !has_key(stored_values, "manualfanspeed") {
        if let Some(value) = maybe_u16(stored_values, "fanspeed") {
            decisions.push(MigrationDecision::Write(NvsWrite::u16(
                "manualfanspeed",
                value,
            )));
        }
    }

    for protocol_key in ["stratumprot", "fbstratumprot"] {
        if let Some(value) = maybe_u16(stored_values, protocol_key) {
            decisions.push(MigrationDecision::Erase(NvsErase::key(protocol_key)));
            decisions.push(MigrationDecision::Write(NvsWrite::string(
                protocol_key,
                stratum_protocol_name(value),
            )));
        }
    }

    let mut mixed_case_sv2_channel_type_consumed = false;
    for channel_type_key in ["sv2chantype", "fbsv2chantype"] {
        if let Some(value) = maybe_u16(stored_values, channel_type_key) {
            decisions.push(MigrationDecision::Erase(NvsErase::key(channel_type_key)));
            decisions.push(MigrationDecision::Write(NvsWrite::string(
                channel_type_key,
                sv2_channel_type_name(value),
            )));
            continue;
        }

        if has_key(stored_values, channel_type_key) {
            continue;
        }

        if !mixed_case_sv2_channel_type_consumed {
            if let Some(value) = maybe_u16(stored_values, "fbSv2ChanType") {
                mixed_case_sv2_channel_type_consumed = true;
                decisions.push(MigrationDecision::Erase(NvsErase::key("fbSv2ChanType")));
                decisions.push(MigrationDecision::Write(NvsWrite::string(
                    channel_type_key,
                    sv2_channel_type_name(value),
                )));
            }
        }
    }

    decisions
}

/// Returns legacy compatibility writes for an active write decision.
#[must_use]
pub fn compatibility_writes_for_active(write: &NvsWrite) -> Vec<NvsWrite> {
    match write {
        NvsWrite::String { key, value } if key.as_str() == "asicfrequency_f" => value
            .parse::<f32>()
            .ok()
            .map(|frequency| NvsWrite::u16("asicfrequency", frequency as u16))
            .into_iter()
            .collect(),
        NvsWrite::U16 { key, value } if key.as_str() == "manualfanspeed" => {
            vec![NvsWrite::u16("fanspeed", *value)]
        }
        _ => Vec::new(),
    }
}
fn maybe_stored_value<'a>(stored_values: &'a [StoredValue], key: &str) -> Option<&'a StoredValue> {
    stored_values
        .iter()
        .find(|stored| stored.key.as_str() == key)
}

fn has_key(stored_values: &[StoredValue], key: &str) -> bool {
    maybe_stored_value(stored_values, key).is_some()
}

fn maybe_u16(stored_values: &[StoredValue], key: &str) -> Option<u16> {
    let stored = maybe_stored_value(stored_values, key)?;

    match stored.value {
        StoredValueKind::U16(value) => Some(value),
        _ => None,
    }
}

fn stratum_protocol_name(value: u16) -> &'static str {
    if value == 1 {
        return "SV2";
    }

    "SV1"
}

fn sv2_channel_type_name(value: u16) -> &'static str {
    if value == 1 {
        return "standard";
    }

    "extended"
}
