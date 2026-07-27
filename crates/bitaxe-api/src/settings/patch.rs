use std::collections::BTreeSet;

use bitaxe_config::{
    all_settings_schema, apply_settings_patch, ConfigValidationError, NvsWrite, RawSettingValue,
    SettingsPatch, SettingsUpdateDecision,
};
use serde_json::{Map, Value};
use thiserror::Error;

/// Public AxeOS-compatible settings PATCH error body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SettingsPatchPublicError {
    /// Malformed JSON or a non-object payload.
    #[error("Invalid JSON")]
    InvalidJson,
    /// Known setting validation failed.
    #[error("Wrong API input")]
    WrongApiInput,
}

impl SettingsPatchPublicError {
    /// Returns the exact upstream-compatible response body text.
    #[must_use]
    pub const fn body(self) -> &'static str {
        match self {
            Self::InvalidJson => "Invalid JSON",
            Self::WrongApiInput => "Wrong API input",
        }
    }
}

/// Internal typed settings PATCH failure with generic public mapping.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{public_error}")]
pub struct SettingsPatchFailure {
    public_error: SettingsPatchPublicError,
    reason: SettingsPatchFailureReason,
}

impl SettingsPatchFailure {
    /// Returns the public AxeOS-compatible error mapping.
    #[must_use]
    pub const fn public_error(&self) -> SettingsPatchPublicError {
        self.public_error
    }

    /// Returns the firmware/test-facing typed reason.
    #[must_use]
    pub const fn reason(&self) -> &SettingsPatchFailureReason {
        &self.reason
    }
}

/// Firmware/test-facing reason for settings PATCH rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPatchFailureReason {
    /// JSON parser rejected the body.
    MalformedJson { message: String },
    /// The parsed JSON value was not an object.
    NonObjectJson,
    /// One or more known settings failed conversion or schema validation.
    InvalidKnownFields(Vec<SettingsPatchFieldError>),
}

/// Internal known-field failure without exposing secret values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPatchFieldError {
    /// The pure config schema rejected the field.
    Validation(ConfigValidationError),
    /// The JSON value shape cannot be converted into a raw setting value.
    UnsupportedJsonType {
        /// REST field name.
        field: String,
        /// JSON kind name, not the raw value.
        kind: &'static str,
    },
    /// A numeric value was not finite or could not fit the accepted raw model.
    UnsupportedNumber {
        /// REST field name.
        field: String,
    },
}

/// Accepted settings PATCH planning result.
#[derive(Debug, Clone, PartialEq)]
pub struct AcceptedSettingsPatch {
    patch: SettingsPatch,
    writes: Vec<NvsWrite>,
    maybe_hostname: Option<String>,
}

impl AcceptedSettingsPatch {
    /// Returns the pure config patch used to produce this accepted plan.
    #[must_use]
    pub const fn patch(&self) -> &SettingsPatch {
        &self.patch
    }

    /// Returns inert NVS writes that a firmware adapter may persist.
    #[must_use]
    pub fn writes(&self) -> &[NvsWrite] {
        &self.writes
    }

    /// Returns a requested hostname when the accepted patch included one.
    #[must_use]
    pub fn maybe_hostname(&self) -> Option<&str> {
        self.maybe_hostname.as_deref()
    }
}

/// Parses a raw PATCH body string and plans accepted writes without side effects.
pub fn plan_settings_patch_body(body: &str) -> Result<AcceptedSettingsPatch, SettingsPatchFailure> {
    let value = parse_settings_patch_body(body)?;

    plan_settings_patch_value(&value)
}

pub(crate) fn parse_settings_patch_body(body: &str) -> Result<Value, SettingsPatchFailure> {
    serde_json::from_str::<Value>(body).map_err(|error| SettingsPatchFailure {
        public_error: SettingsPatchPublicError::InvalidJson,
        reason: SettingsPatchFailureReason::MalformedJson {
            message: error.to_string(),
        },
    })
}

/// Plans accepted settings writes from a parsed JSON value without side effects.
pub fn plan_settings_patch_value(
    value: &Value,
) -> Result<AcceptedSettingsPatch, SettingsPatchFailure> {
    let Some(object) = value.as_object() else {
        return Err(SettingsPatchFailure {
            public_error: SettingsPatchPublicError::InvalidJson,
            reason: SettingsPatchFailureReason::NonObjectJson,
        });
    };

    accepted_patch_from_object(object)
}

fn accepted_patch_from_object(
    object: &Map<String, Value>,
) -> Result<AcceptedSettingsPatch, SettingsPatchFailure> {
    let known_fields = known_rest_field_names();
    let mut patch = SettingsPatch::new();
    let mut errors = Vec::new();
    let mut maybe_hostname = None;

    for (field, value) in object {
        if !known_fields.contains(field) {
            continue;
        }

        match raw_setting_value(field, value) {
            Ok(raw_value) => {
                if field == "hostname" {
                    if let RawSettingValue::String(hostname) = &raw_value {
                        maybe_hostname = Some(hostname.clone());
                    }
                }
                patch.insert(field.clone(), raw_value);
            }
            Err(error) => errors.push(error),
        }
    }

    if !errors.is_empty() {
        return Err(wrong_input(errors));
    }

    match apply_settings_patch(&patch) {
        SettingsUpdateDecision::Accepted { writes } => Ok(AcceptedSettingsPatch {
            patch,
            writes,
            maybe_hostname,
        }),
        SettingsUpdateDecision::Rejected { errors } => Err(wrong_input(
            errors
                .into_iter()
                .map(SettingsPatchFieldError::Validation)
                .collect(),
        )),
    }
}

fn known_rest_field_names() -> BTreeSet<String> {
    all_settings_schema()
        .into_iter()
        .filter_map(|setting| {
            setting
                .rest_name
                .map(|rest_name| rest_name.as_str().to_owned())
        })
        .collect()
}

fn raw_setting_value(
    field: &str,
    value: &Value,
) -> Result<RawSettingValue, SettingsPatchFieldError> {
    match value {
        Value::String(value) => Ok(RawSettingValue::String(value.clone())),
        Value::Number(value) => raw_number_value(field, value),
        Value::Bool(value) => Ok(RawSettingValue::Bool(*value)),
        Value::Null => Err(unsupported_json_type(field, "null")),
        Value::Array(_) => Err(unsupported_json_type(field, "array")),
        Value::Object(_) => Err(unsupported_json_type(field, "object")),
    }
}

fn raw_number_value(
    field: &str,
    value: &serde_json::Number,
) -> Result<RawSettingValue, SettingsPatchFieldError> {
    if let Some(value) = value.as_i64() {
        return Ok(RawSettingValue::Number(value));
    }

    if let Some(value) = value.as_u64() {
        if value <= i64::MAX as u64 {
            return Ok(RawSettingValue::Number(value as i64));
        }
    }

    let Some(value) = value.as_f64() else {
        return Err(SettingsPatchFieldError::UnsupportedNumber {
            field: field.to_owned(),
        });
    };

    if !value.is_finite() {
        return Err(SettingsPatchFieldError::UnsupportedNumber {
            field: field.to_owned(),
        });
    }

    Ok(RawSettingValue::Float(value))
}

fn unsupported_json_type(field: &str, kind: &'static str) -> SettingsPatchFieldError {
    SettingsPatchFieldError::UnsupportedJsonType {
        field: field.to_owned(),
        kind,
    }
}

pub(crate) fn wrong_input(errors: Vec<SettingsPatchFieldError>) -> SettingsPatchFailure {
    SettingsPatchFailure {
        public_error: SettingsPatchPublicError::WrongApiInput,
        reason: SettingsPatchFailureReason::InvalidKnownFields(errors),
    }
}
