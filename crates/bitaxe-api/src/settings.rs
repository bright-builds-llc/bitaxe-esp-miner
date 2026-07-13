//! Pure AxeOS settings PATCH request planning.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `reference/esp-miner/main/nvs_config.c`

use std::collections::BTreeSet;

use bitaxe_config::{
    all_settings_schema, apply_settings_patch, reload_snapshot, ConfigValidationError, LoadedValue,
    NvsSnapshot, NvsWrite, RawSettingValue, SettingsPatch, SettingsUpdateDecision,
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

/// Public settings route response shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPublicResponse {
    /// Upstream-compatible empty response body on success.
    EmptySuccess,
    /// Upstream-compatible generic error body.
    Error(SettingsPatchPublicError),
}

/// Internal persistence plan for a firmware adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsPersistencePlan {
    writes: Vec<NvsWrite>,
    effects: Vec<SettingsPersistenceEffect>,
}

impl SettingsPersistencePlan {
    /// Builds a persistence plan from an accepted settings patch and current snapshot.
    #[must_use]
    pub fn from_accepted_patch(snapshot: &NvsSnapshot, accepted: AcceptedSettingsPatch) -> Self {
        let effects = hostname_effect(snapshot, accepted.maybe_hostname.as_deref());

        Self {
            writes: accepted.writes,
            effects,
        }
    }

    /// Returns the inert NVS writes that must be persisted before success.
    #[must_use]
    pub fn writes(&self) -> &[NvsWrite] {
        &self.writes
    }

    /// Returns best-effort effects that firmware may attempt after persistence success.
    #[must_use]
    pub fn effects(&self) -> &[SettingsPersistenceEffect] {
        &self.effects
    }
}

/// Best-effort firmware effects emitted only after persistence success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceEffect {
    /// Attempt to apply the new hostname live after NVS commit/reload succeeds.
    BestEffortApplyHostname { hostname: String },
}

/// Ordered settings persistence execution steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceStep {
    /// Accepted validation was acknowledged before storage mutation.
    Validate,
    /// A single inert NVS write was passed to the adapter.
    Write { key: String },
    /// All writes were committed.
    Commit,
    /// Settings were reloaded from storage after commit.
    Reload,
    /// The route may return an upstream-compatible empty success body.
    PublicSuccess,
}

impl SettingsPersistenceStep {
    /// Creates a write step from an NVS write decision.
    #[must_use]
    pub fn write(write: &NvsWrite) -> Self {
        Self::Write {
            key: nvs_write_key(write).to_owned(),
        }
    }
}

/// Thin firmware adapter used by the pure settings executor.
pub trait SettingsPersistenceAdapter {
    /// Acknowledge that pure validation has accepted the plan.
    fn validate_accepted(&mut self) -> Result<(), SettingsAdapterFailure>;

    /// Persist one write decision.
    fn write(&mut self, write: &NvsWrite) -> Result<(), SettingsAdapterFailure>;

    /// Commit all writes.
    fn commit(&mut self) -> Result<(), SettingsAdapterFailure>;

    /// Reload settings after commit.
    fn reload(&mut self) -> Result<(), SettingsAdapterFailure>;
}

/// Adapter-local failure detail for firmware logs.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct SettingsAdapterFailure {
    message: String,
}

impl SettingsAdapterFailure {
    /// Creates a typed adapter failure without exposing it publicly.
    #[must_use]
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Firmware-visible persistence failure reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPersistenceFailure {
    /// Accepted validation failed in the adapter shell.
    Validation,
    /// A write failed for the given NVS key.
    Write { key: String },
    /// Commit failed.
    Commit,
    /// Reload failed.
    Reload,
}

/// Successful settings persistence execution.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsPersistenceSuccess {
    steps: Vec<SettingsPersistenceStep>,
    effects: Vec<SettingsPersistenceEffect>,
    public_response: SettingsPublicResponse,
}

impl SettingsPersistenceSuccess {
    /// Returns the complete ordered sequence including public success.
    #[must_use]
    pub fn steps(&self) -> &[SettingsPersistenceStep] {
        &self.steps
    }

    /// Returns the adapter-facing steps before public response.
    #[must_use]
    pub fn steps_without_public_response(&self) -> Vec<SettingsPersistenceStep> {
        self.steps
            .iter()
            .filter(|step| **step != SettingsPersistenceStep::PublicSuccess)
            .cloned()
            .collect()
    }

    /// Returns the public response shape.
    #[must_use]
    pub const fn public_response(&self) -> SettingsPublicResponse {
        self.public_response
    }

    /// Returns effects available only after persistence success.
    #[must_use]
    pub fn effects(&self) -> &[SettingsPersistenceEffect] {
        &self.effects
    }
}

/// Failed settings persistence execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPersistenceFailureReport {
    reason: SettingsPersistenceFailure,
    public_error: SettingsPatchPublicError,
    completed_steps: Vec<SettingsPersistenceStep>,
}

impl SettingsPersistenceFailureReport {
    /// Returns the firmware-visible typed failure reason.
    #[must_use]
    pub const fn reason(&self) -> &SettingsPersistenceFailure {
        &self.reason
    }

    /// Returns the generic upstream-compatible public error mapping.
    #[must_use]
    pub const fn public_error(&self) -> SettingsPatchPublicError {
        self.public_error
    }

    /// Returns steps completed or attempted before failure.
    #[must_use]
    pub fn completed_steps(&self) -> &[SettingsPersistenceStep] {
        &self.completed_steps
    }
}

/// Executes an accepted persistence plan and only returns success after reload.
pub fn execute_settings_persistence_plan(
    plan: &SettingsPersistencePlan,
    adapter: &mut impl SettingsPersistenceAdapter,
) -> Result<SettingsPersistenceSuccess, SettingsPersistenceFailureReport> {
    let mut steps = Vec::new();

    steps.push(SettingsPersistenceStep::Validate);
    adapter
        .validate_accepted()
        .map_err(|_| persistence_failure(SettingsPersistenceFailure::Validation, &steps))?;

    for write in plan.writes() {
        let step = SettingsPersistenceStep::write(write);
        steps.push(step);
        adapter.write(write).map_err(|_| {
            persistence_failure(
                SettingsPersistenceFailure::Write {
                    key: nvs_write_key(write).to_owned(),
                },
                &steps,
            )
        })?;
    }

    steps.push(SettingsPersistenceStep::Commit);
    adapter
        .commit()
        .map_err(|_| persistence_failure(SettingsPersistenceFailure::Commit, &steps))?;

    steps.push(SettingsPersistenceStep::Reload);
    adapter
        .reload()
        .map_err(|_| persistence_failure(SettingsPersistenceFailure::Reload, &steps))?;

    steps.push(SettingsPersistenceStep::PublicSuccess);

    Ok(SettingsPersistenceSuccess {
        steps,
        effects: plan.effects.clone(),
        public_response: SettingsPublicResponse::EmptySuccess,
    })
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

fn hostname_effect(
    snapshot: &NvsSnapshot,
    maybe_hostname: Option<&str>,
) -> Vec<SettingsPersistenceEffect> {
    let Some(hostname) = maybe_hostname else {
        return Vec::new();
    };

    let reloaded = reload_snapshot(snapshot);
    if matches!(
        reloaded.loaded_value("hostname"),
        Some(LoadedValue::Str(current)) if current == hostname
    ) {
        return Vec::new();
    }

    vec![SettingsPersistenceEffect::BestEffortApplyHostname {
        hostname: hostname.to_owned(),
    }]
}

fn persistence_failure(
    reason: SettingsPersistenceFailure,
    completed_steps: &[SettingsPersistenceStep],
) -> SettingsPersistenceFailureReport {
    SettingsPersistenceFailureReport {
        reason,
        public_error: SettingsPatchPublicError::WrongApiInput,
        completed_steps: completed_steps.to_vec(),
    }
}

fn nvs_write_key(write: &NvsWrite) -> &str {
    match write {
        NvsWrite::String { key, .. }
        | NvsWrite::U16 { key, .. }
        | NvsWrite::I32 { key, .. }
        | NvsWrite::U64 { key, .. } => key.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_config::{NvsSnapshot, NvsWrite, StoredValue};
    use serde::Deserialize;
    use serde_json::{json, Value};

    use super::{
        execute_settings_persistence_plan, plan_settings_patch_body, plan_settings_patch_value,
        SettingsAdapterFailure, SettingsPatchFailureReason, SettingsPatchPublicError,
        SettingsPersistenceAdapter, SettingsPersistenceEffect, SettingsPersistenceFailure,
        SettingsPersistencePlan, SettingsPersistenceStep, SettingsPublicResponse,
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

    #[derive(Debug, Default)]
    struct RecordingAdapter {
        steps: Vec<SettingsPersistenceStep>,
        maybe_failure: Option<AdapterFailurePoint>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum AdapterFailurePoint {
        FirstWrite,
        Commit,
        Reload,
    }

    impl RecordingAdapter {
        fn failing_at(failure: AdapterFailurePoint) -> Self {
            Self {
                steps: Vec::new(),
                maybe_failure: Some(failure),
            }
        }
    }

    impl SettingsPersistenceAdapter for RecordingAdapter {
        fn validate_accepted(&mut self) -> Result<(), SettingsAdapterFailure> {
            self.steps.push(SettingsPersistenceStep::Validate);
            Ok(())
        }

        fn write(&mut self, write: &NvsWrite) -> Result<(), SettingsAdapterFailure> {
            let step = SettingsPersistenceStep::write(write);
            self.steps.push(step);

            if self.maybe_failure == Some(AdapterFailurePoint::FirstWrite) {
                return Err(SettingsAdapterFailure::failed("fake write failure"));
            }

            Ok(())
        }

        fn commit(&mut self) -> Result<(), SettingsAdapterFailure> {
            self.steps.push(SettingsPersistenceStep::Commit);

            if self.maybe_failure == Some(AdapterFailurePoint::Commit) {
                return Err(SettingsAdapterFailure::failed("fake commit failure"));
            }

            Ok(())
        }

        fn reload(&mut self) -> Result<(), SettingsAdapterFailure> {
            self.steps.push(SettingsPersistenceStep::Reload);

            if self.maybe_failure == Some(AdapterFailurePoint::Reload) {
                return Err(SettingsAdapterFailure::failed("fake reload failure"));
            }

            Ok(())
        }
    }

    fn accepted_persistence_plan(body: &str) -> SettingsPersistencePlan {
        let accepted = plan_settings_patch_body(body).expect("test PATCH should parse");
        SettingsPersistencePlan::from_accepted_patch(&NvsSnapshot::new(), accepted)
    }

    #[test]
    fn settings_persistence_plan_requires_write_commit_reload_before_public_success() {
        // Arrange
        let plan = accepted_persistence_plan(r#"{"frequency":485,"manualFanSpeed":42}"#);
        let mut adapter = RecordingAdapter::default();

        // Act
        let success = execute_settings_persistence_plan(&plan, &mut adapter)
            .expect("accepted persistence plan should complete");

        // Assert
        assert_eq!(
            success.steps(),
            [
                SettingsPersistenceStep::Validate,
                SettingsPersistenceStep::write(&NvsWrite::string("asicfrequency_f", "485.000000")),
                SettingsPersistenceStep::write(&NvsWrite::u16("asicfrequency", 485)),
                SettingsPersistenceStep::write(&NvsWrite::u16("manualfanspeed", 42)),
                SettingsPersistenceStep::write(&NvsWrite::u16("fanspeed", 42)),
                SettingsPersistenceStep::Commit,
                SettingsPersistenceStep::Reload,
                SettingsPersistenceStep::PublicSuccess,
            ]
        );
        assert_eq!(adapter.steps, success.steps_without_public_response());
        assert_eq!(
            success.public_response(),
            SettingsPublicResponse::EmptySuccess
        );
    }

    #[test]
    fn settings_persistence_failures_are_typed_but_public_response_stays_generic() {
        // Arrange
        let cases = [
            (
                AdapterFailurePoint::FirstWrite,
                SettingsPersistenceFailure::Write {
                    key: "asicfrequency_f".to_owned(),
                },
            ),
            (
                AdapterFailurePoint::Commit,
                SettingsPersistenceFailure::Commit,
            ),
            (
                AdapterFailurePoint::Reload,
                SettingsPersistenceFailure::Reload,
            ),
        ];

        for (failure_point, expected_reason) in cases {
            let plan = accepted_persistence_plan(r#"{"frequency":485}"#);
            let mut adapter = RecordingAdapter::failing_at(failure_point);

            // Act
            let failure = execute_settings_persistence_plan(&plan, &mut adapter)
                .expect_err("configured fake failure must reject route success");

            // Assert
            assert_eq!(failure.reason(), &expected_reason);
            assert_eq!(
                failure.public_error(),
                SettingsPatchPublicError::WrongApiInput
            );
            assert_eq!(failure.public_error().body(), "Wrong API input");
            assert!(!failure
                .completed_steps()
                .contains(&SettingsPersistenceStep::PublicSuccess));
        }
    }

    #[test]
    fn settings_persistence_hostname_live_apply_is_best_effort_after_persistence() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([StoredValue::string("hostname", "bitaxe")]);
        let accepted =
            plan_settings_patch_body(r#"{"hostname":"axe-205"}"#).expect("hostname patch parses");
        let plan = SettingsPersistencePlan::from_accepted_patch(&snapshot, accepted);
        let mut adapter = RecordingAdapter::default();

        // Act
        let success = execute_settings_persistence_plan(&plan, &mut adapter)
            .expect("hostname persistence must succeed before live apply");

        // Assert
        assert_eq!(
            plan.effects(),
            [SettingsPersistenceEffect::BestEffortApplyHostname {
                hostname: "axe-205".to_owned(),
            }]
        );
        assert_eq!(success.effects(), plan.effects());
        assert!(
            success
                .steps()
                .iter()
                .position(|step| *step == SettingsPersistenceStep::Reload)
                < success
                    .steps()
                    .iter()
                    .position(|step| *step == SettingsPersistenceStep::PublicSuccess)
        );
    }

    #[test]
    fn settings_persistence_hostname_effect_cannot_override_reload_failure() {
        // Arrange
        let snapshot = NvsSnapshot::from_values([StoredValue::string("hostname", "bitaxe")]);
        let accepted =
            plan_settings_patch_body(r#"{"hostname":"axe-205"}"#).expect("hostname patch parses");
        let plan = SettingsPersistencePlan::from_accepted_patch(&snapshot, accepted);
        let mut adapter = RecordingAdapter::failing_at(AdapterFailurePoint::Reload);

        // Act
        let failure = execute_settings_persistence_plan(&plan, &mut adapter)
            .expect_err("reload failure must prevent public success");

        // Assert
        assert_eq!(failure.reason(), &SettingsPersistenceFailure::Reload);
        assert!(!failure
            .completed_steps()
            .contains(&SettingsPersistenceStep::PublicSuccess));
    }
}
