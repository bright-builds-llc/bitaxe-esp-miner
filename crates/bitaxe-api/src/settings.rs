//! Pure AxeOS settings PATCH request planning.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `reference/esp-miner/main/nvs_config.c`

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
    let value = serde_json::from_str::<Value>(body).map_err(|error| SettingsPatchFailure {
        public_error: SettingsPatchPublicError::InvalidJson,
        reason: SettingsPatchFailureReason::MalformedJson {
            message: error.to_string(),
        },
    })?;

    plan_settings_patch_value(&value)
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

fn wrong_input(errors: Vec<SettingsPatchFieldError>) -> SettingsPatchFailure {
    SettingsPatchFailure {
        public_error: SettingsPatchPublicError::WrongApiInput,
        reason: SettingsPatchFailureReason::InvalidKnownFields(errors),
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_config::{NvsSnapshot, NvsWrite, StoredValue};
    use serde::Deserialize;
    use serde_json::{json, Value};

    use super::{
        plan_settings_patch_body, plan_settings_patch_value, SettingsPatchFailureReason,
        SettingsPatchPublicError,
    };

    #[derive(Debug, Deserialize)]
    struct Fixture {
        valid: PatchCase,
        unknown_only: PatchCase,
        invalid_known: InvalidPatchCase,
        invalid_json_public_error: String,
        wrong_input_public_error: String,
    }

    #[derive(Debug, Deserialize)]
    struct PatchCase {
        body: Value,
        expected_writes: Vec<ExpectedWrite>,
    }

    #[derive(Debug, Deserialize)]
    struct InvalidPatchCase {
        body: Value,
        public_error: String,
    }

    #[derive(Debug, Deserialize)]
    struct ExpectedWrite {
        #[serde(rename = "type")]
        kind: String,
        key: String,
        value: Value,
    }

    fn fixture() -> Fixture {
        serde_json::from_str(include_str!("../fixtures/api/settings-patch-cases.json"))
            .expect("settings PATCH fixture must be valid JSON")
    }

    fn expected_writes(writes: Vec<ExpectedWrite>) -> Vec<NvsWrite> {
        writes
            .into_iter()
            .map(|write| match write.kind.as_str() {
                "string" => NvsWrite::string(
                    leaked_static_key(write.key),
                    write
                        .value
                        .as_str()
                        .expect("expected string write value must be a string"),
                ),
                "u16" => NvsWrite::u16(
                    leaked_static_key(write.key),
                    write
                        .value
                        .as_u64()
                        .expect("expected u16 write value must be numeric")
                        as u16,
                ),
                other => panic!("unsupported expected write kind: {other}"),
            })
            .collect()
    }

    fn leaked_static_key(value: String) -> &'static str {
        Box::leak(value.into_boxed_str())
    }

    #[test]
    fn settings_patch_valid_known_fields_emit_expected_writes_and_legacy_mirrors() {
        // Arrange
        let case = fixture().valid;
        let expected = expected_writes(case.expected_writes);

        // Act
        let plan = plan_settings_patch_value(&case.body).expect("valid PATCH should be accepted");

        // Assert
        assert_eq!(plan.writes(), expected);
    }

    #[test]
    fn settings_patch_ignores_unknown_fields_without_emitting_writes() {
        // Arrange
        let case = fixture().unknown_only;

        // Act
        let plan =
            plan_settings_patch_value(&case.body).expect("unknown-only PATCH should be accepted");

        // Assert
        assert_eq!(plan.writes(), expected_writes(case.expected_writes));
    }

    #[test]
    fn settings_patch_invalid_known_field_rejects_atomically_and_preserves_snapshot() {
        // Arrange
        let case = fixture().invalid_known;
        let snapshot = NvsSnapshot::from_values([StoredValue::u16("manualfanspeed", 42)]);
        let original_snapshot = snapshot.clone();

        // Act
        let error =
            plan_settings_patch_value(&case.body).expect_err("invalid known field must reject");

        // Assert
        assert_eq!(error.public_error().body(), case.public_error);
        assert!(matches!(
            error.reason(),
            SettingsPatchFailureReason::InvalidKnownFields(_)
        ));
        assert_eq!(snapshot, original_snapshot);
    }

    #[test]
    fn settings_patch_malformed_or_non_object_json_maps_to_invalid_json() {
        // Arrange
        let fixture = fixture();
        let malformed_body = "{bad json";
        let non_object_body = "[1, 2, 3]";

        // Act
        let malformed_error =
            plan_settings_patch_body(malformed_body).expect_err("malformed JSON must reject");
        let non_object_error =
            plan_settings_patch_body(non_object_body).expect_err("non-object JSON must reject");

        // Assert
        assert_eq!(
            malformed_error.public_error(),
            SettingsPatchPublicError::InvalidJson
        );
        assert_eq!(
            non_object_error.public_error(),
            SettingsPatchPublicError::InvalidJson
        );
        assert_eq!(
            malformed_error.public_error().body(),
            fixture.invalid_json_public_error
        );
        assert_eq!(
            non_object_error.public_error().body(),
            fixture.invalid_json_public_error
        );
    }

    #[test]
    fn settings_patch_internal_diagnostics_do_not_render_secret_values() {
        // Arrange
        let body = json!({
            "stratumPassword": "secret-password-that-must-not-appear".repeat(200),
            "stratumCert": "secret-cert-that-must-not-appear".repeat(200),
            "stratumUser": "secret-user-that-must-not-appear".repeat(200)
        })
        .to_string();

        // Act
        let error = plan_settings_patch_body(&body).expect_err("oversized secrets must reject");
        let diagnostics = format!("{error:?}");

        // Assert
        assert_eq!(
            error.public_error().body(),
            fixture().wrong_input_public_error
        );
        assert!(!diagnostics.contains("secret-password-that-must-not-appear"));
        assert!(!diagnostics.contains("secret-cert-that-must-not-appear"));
        assert!(!diagnostics.contains("secret-user-that-must-not-appear"));
    }
}
