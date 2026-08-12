use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use bitaxe_config::{board_profile_defaults, BoardProfileDefaults, BoardProfileSeedKind};
use camino::Utf8Path;

use crate::ValidationError;

const CFG006_ROW_ID: &str = "CFG-006";
const REFERENCE_CONFIG_ROOT: &str = "reference/esp-miner";
const REFERENCE_CONFIG_PREFIX: &str = "config-";
const REFERENCE_CONFIG_SUFFIX: &str = ".cvs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeedKind {
    Numbered,
    CustomOverride,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SeedProjection {
    seed_id: String,
    source_path: String,
    seed_kind: SeedKind,
    board_version: String,
    device_model: String,
    asic_model: String,
    asic_frequency_mhz: u16,
    asic_voltage_mv: u16,
    rotation: u16,
    auto_fan_speed: bool,
    manual_fan_speed: u16,
    self_test: bool,
    overheat_mode: bool,
    primary_pool_port: u16,
}

#[derive(Debug, Clone, Copy)]
struct CsvField<'a> {
    kind: &'a str,
    encoding: &'a str,
    value: &'a str,
}

pub(crate) fn validate_pinned_reference(workspace_dir: &Utf8Path) -> Vec<ValidationError> {
    let expected = board_profile_defaults()
        .iter()
        .map(expected_projection)
        .collect::<Vec<_>>();
    let actual = match load_reference_projections(workspace_dir) {
        Ok(actual) => actual,
        Err(message) => return vec![validation_error(message)],
    };

    validate_projections(&expected, &actual)
        .into_iter()
        .map(validation_error)
        .collect()
}

fn load_reference_projections(workspace_dir: &Utf8Path) -> Result<Vec<SeedProjection>, String> {
    let reference_root = workspace_dir.join(REFERENCE_CONFIG_ROOT);
    let entries = fs::read_dir(reference_root.as_std_path())
        .map_err(|error| format!("failed to inventory {reference_root}: {error}"))?;
    let mut source_paths = Vec::new();

    for entry in entries {
        let entry = entry
            .map_err(|error| format!("failed to read an entry below {reference_root}: {error}"))?;
        let file_type = entry.file_type().map_err(|error| {
            format!("failed to classify an entry below {reference_root}: {error}")
        })?;
        if !file_type.is_file() {
            continue;
        }

        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("non-UTF-8 filename below {reference_root}"))?;
        if !file_name.starts_with(REFERENCE_CONFIG_PREFIX)
            || !file_name.ends_with(REFERENCE_CONFIG_SUFFIX)
        {
            continue;
        }
        source_paths.push(format!("{REFERENCE_CONFIG_ROOT}/{file_name}"));
    }
    source_paths.sort();

    source_paths
        .into_iter()
        .map(|source_path| {
            let path = workspace_dir.join(&source_path);
            let document = fs::read_to_string(path.as_std_path())
                .map_err(|error| format!("failed to read {source_path}: {error}"))?;
            parse_reference_seed(&source_path, &document)
        })
        .collect()
}

fn parse_reference_seed(source_path: &str, document: &str) -> Result<SeedProjection, String> {
    let (seed_id, seed_kind) = parse_seed_identity(source_path)?;
    let mut lines = document.lines();
    if lines.next() != Some("key,type,encoding,value") {
        return Err(format!("{source_path} has an invalid CSV header"));
    }

    let mut fields = BTreeMap::new();
    for (line_index, line) in lines.enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let cells = line.split(',').collect::<Vec<_>>();
        if cells.len() != 4 {
            return Err(format!(
                "{source_path} line {} does not contain exactly four CSV cells",
                line_index + 2
            ));
        }
        let key = cells[0];
        if fields
            .insert(
                key,
                CsvField {
                    kind: cells[1],
                    encoding: cells[2],
                    value: cells[3],
                },
            )
            .is_some()
        {
            return Err(format!("{source_path} contains duplicate key {key}"));
        }
    }

    Ok(SeedProjection {
        seed_id,
        source_path: source_path.to_owned(),
        seed_kind,
        board_version: require_string(&fields, source_path, "boardversion")?,
        device_model: require_string(&fields, source_path, "devicemodel")?,
        asic_model: require_string(&fields, source_path, "asicmodel")?,
        asic_frequency_mhz: require_u16(&fields, source_path, "asicfrequency")?,
        asic_voltage_mv: require_u16(&fields, source_path, "asicvoltage")?,
        rotation: require_u16(&fields, source_path, "rotation")?,
        auto_fan_speed: require_bool(&fields, source_path, "autofanspeed")?,
        manual_fan_speed: require_u16(&fields, source_path, "fanspeed")?,
        self_test: require_bool(&fields, source_path, "selftest")?,
        overheat_mode: require_bool(&fields, source_path, "overheat_mode")?,
        primary_pool_port: require_u16(&fields, source_path, "stratumport")?,
    })
}

fn parse_seed_identity(source_path: &str) -> Result<(String, SeedKind), String> {
    let maybe_name = source_path
        .strip_prefix(&format!(
            "{REFERENCE_CONFIG_ROOT}/{REFERENCE_CONFIG_PREFIX}"
        ))
        .and_then(|name| name.strip_suffix(REFERENCE_CONFIG_SUFFIX));
    let Some(name) = maybe_name else {
        return Err(format!("invalid reference seed path {source_path}"));
    };

    if name == "custom" {
        return Ok((name.to_owned(), SeedKind::CustomOverride));
    }
    if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("invalid numbered reference seed {source_path}"));
    }

    Ok((name.to_owned(), SeedKind::Numbered))
}

fn require_string(
    fields: &BTreeMap<&str, CsvField<'_>>,
    source_path: &str,
    key: &str,
) -> Result<String, String> {
    let field = require_field(fields, source_path, key, "data", "string")?;
    Ok(field.value.to_owned())
}

fn require_u16(
    fields: &BTreeMap<&str, CsvField<'_>>,
    source_path: &str,
    key: &str,
) -> Result<u16, String> {
    let field = require_field(fields, source_path, key, "data", "u16")?;
    let value = field
        .value
        .parse::<u16>()
        .map_err(|_| format!("{source_path} key {key} is not a canonical u16"))?;
    if value.to_string() != field.value {
        return Err(format!("{source_path} key {key} is not a canonical u16"));
    }
    Ok(value)
}

fn require_bool(
    fields: &BTreeMap<&str, CsvField<'_>>,
    source_path: &str,
    key: &str,
) -> Result<bool, String> {
    let field = require_field(fields, source_path, key, "data", "u16")?;
    match field.value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!("{source_path} key {key} is not boolean 0 or 1")),
    }
}

fn require_field<'a>(
    fields: &'a BTreeMap<&str, CsvField<'a>>,
    source_path: &str,
    key: &str,
    expected_kind: &str,
    expected_encoding: &str,
) -> Result<CsvField<'a>, String> {
    let Some(field) = fields.get(key).copied() else {
        return Err(format!("{source_path} is missing key {key}"));
    };
    if field.kind != expected_kind || field.encoding != expected_encoding {
        return Err(format!(
            "{source_path} key {key} expected {expected_kind}/{expected_encoding}, found {}/{}",
            field.kind, field.encoding
        ));
    }
    Ok(field)
}

fn expected_projection(defaults: &BoardProfileDefaults) -> SeedProjection {
    SeedProjection {
        seed_id: defaults.seed_id().to_owned(),
        source_path: defaults.source_path().to_owned(),
        seed_kind: match defaults.seed_kind() {
            BoardProfileSeedKind::Numbered => SeedKind::Numbered,
            BoardProfileSeedKind::CustomOverride => SeedKind::CustomOverride,
        },
        board_version: defaults.board_version().to_owned(),
        device_model: defaults.device_model().to_owned(),
        asic_model: defaults.asic_model().to_owned(),
        asic_frequency_mhz: defaults.asic_frequency_mhz(),
        asic_voltage_mv: defaults.asic_voltage_mv(),
        rotation: defaults.rotation(),
        auto_fan_speed: defaults.auto_fan_speed(),
        manual_fan_speed: defaults.manual_fan_speed(),
        self_test: defaults.self_test(),
        overheat_mode: defaults.overheat_mode(),
        primary_pool_port: defaults.primary_pool_port(),
    }
}

fn validate_projections(expected: &[SeedProjection], actual: &[SeedProjection]) -> Vec<String> {
    let mut errors = Vec::new();
    validate_unique_identity("Rust matrix", expected, &mut errors);
    validate_unique_identity("reference inventory", actual, &mut errors);

    let expected_by_path = expected
        .iter()
        .map(|seed| (seed.source_path.as_str(), seed))
        .collect::<BTreeMap<_, _>>();
    let actual_by_path = actual
        .iter()
        .map(|seed| (seed.source_path.as_str(), seed))
        .collect::<BTreeMap<_, _>>();
    let expected_paths = expected_by_path.keys().copied().collect::<BTreeSet<_>>();
    let actual_paths = actual_by_path.keys().copied().collect::<BTreeSet<_>>();

    for missing in expected_paths.difference(&actual_paths) {
        errors.push(format!("reference inventory is missing {missing}"));
    }
    for extra in actual_paths.difference(&expected_paths) {
        errors.push(format!("reference inventory has unmodeled seed {extra}"));
    }
    for shared in expected_paths.intersection(&actual_paths) {
        let expected_seed = expected_by_path[shared];
        let actual_seed = actual_by_path[shared];
        if expected_seed != actual_seed {
            errors.push(format!(
                "Rust defaults matrix does not match pinned reference seed {shared}"
            ));
        }
    }
    errors
}

fn validate_unique_identity(label: &str, seeds: &[SeedProjection], errors: &mut Vec<String>) {
    let mut seed_ids = BTreeSet::new();
    let mut source_paths = BTreeSet::new();
    for seed in seeds {
        if !seed_ids.insert(seed.seed_id.as_str()) {
            errors.push(format!("{label} has duplicate seed id {}", seed.seed_id));
        }
        if !source_paths.insert(seed.source_path.as_str()) {
            errors.push(format!(
                "{label} has duplicate source path {}",
                seed.source_path
            ));
        }
    }
}

fn validation_error(message: impl Into<String>) -> ValidationError {
    ValidationError {
        id: CFG006_ROW_ID.to_owned(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests;
