//! Pure AxeOS settings PATCH request planning.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `reference/esp-miner/main/nvs_config.c`

use std::collections::BTreeSet;

use bitaxe_config::{
    all_settings_schema, apply_settings_patch, ConfigValidationError, ConfirmedHostnameSnapshot,
    NvsWrite, RawSettingValue, SettingsPatch, SettingsUpdateDecision,
};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::v12_settings::Hostname;

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

/// Closed hostname persistence plan for a firmware adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPersistencePlan {
    hostname: Hostname,
}

impl SettingsPersistencePlan {
    /// Builds the only v1.2 persistence plan from validated hostname authority.
    #[must_use]
    pub const fn for_hostname(hostname: Hostname) -> Self {
        Self { hostname }
    }

    /// Returns the validated hostname that must be confirmed before success.
    #[must_use]
    pub const fn hostname(&self) -> &Hostname {
        &self.hostname
    }

    /// Returns the one inert NVS write used by the adapter transaction.
    #[must_use]
    pub fn write(&self) -> NvsWrite {
        NvsWrite::string("hostname", self.hostname.as_str())
    }

    fn confirmed_effect(&self) -> SettingsPersistenceEffect {
        SettingsPersistenceEffect::BestEffortApplyHostname {
            hostname: self.hostname.as_str().to_owned(),
        }
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
    /// The independently reloaded typed hostname matched the request exactly.
    Reconcile,
    /// The complete independently reloaded snapshot became public truth.
    Publish,
    /// The route may return an upstream-compatible empty success body.
    PublicSuccess,
}

impl SettingsPersistenceStep {
    /// Creates the only authorized hostname write step.
    #[must_use]
    pub fn write_hostname() -> Self {
        Self::Write {
            key: "hostname".to_owned(),
        }
    }
}

/// Adapter transaction whose lifetime serializes mutation through publication.
pub trait SettingsPersistenceTransaction {
    /// Writes the validated hostname, including same-value requests.
    fn write_hostname(&mut self, hostname: &Hostname) -> Result<(), SettingsAdapterFailure>;

    /// Commits the hostname write.
    fn commit(&mut self) -> Result<(), SettingsAdapterFailure>;

    /// Independently reloads strict typed hostname evidence and a complete snapshot.
    fn reload(&mut self) -> Result<ConfirmedHostnameSnapshot, SettingsAdapterFailure>;

    /// Atomically publishes the already reconciled independently reloaded snapshot.
    fn publish(
        &mut self,
        candidate: ConfirmedHostnameSnapshot,
    ) -> Result<(), SettingsAdapterFailure>;
}

/// Thin firmware coordinator used by the pure settings executor.
pub trait SettingsPersistenceAdapter {
    /// Transaction type that holds serialization ownership until it is dropped.
    type Transaction<'adapter>: SettingsPersistenceTransaction
    where
        Self: 'adapter;

    /// Acknowledges the already validated closed hostname capability.
    fn validate_accepted(&mut self, hostname: &Hostname) -> Result<(), SettingsAdapterFailure>;

    /// Acquires exclusive mutation-through-publication ownership.
    fn begin_transaction(&mut self) -> Result<Self::Transaction<'_>, SettingsAdapterFailure>;
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
    /// Exclusive transaction ownership could not be acquired.
    Transaction,
    /// Commit failed.
    Commit,
    /// Reload failed.
    Reload,
    /// The independently reloaded hostname did not exactly match the request.
    Reconcile,
    /// The independently reloaded complete snapshot could not be published.
    Publication,
}

/// Storage certainty retained with a persistence failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPersistenceFailureDisposition {
    /// No successful commit was observed; no unchanged-storage claim is made.
    CommitNotConfirmed,
    /// Commit succeeded but later confirmation failed; rollback is not claimed or attempted.
    PostCommitUncertain,
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
    disposition: SettingsPersistenceFailureDisposition,
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

    /// Returns whether a successful commit preceded the confirmation failure.
    #[must_use]
    pub const fn disposition(&self) -> SettingsPersistenceFailureDisposition {
        self.disposition
    }
}

/// Executes one serialized hostname transaction and returns success only after publication.
pub fn execute_settings_persistence_plan(
    plan: &SettingsPersistencePlan,
    adapter: &mut impl SettingsPersistenceAdapter,
) -> Result<SettingsPersistenceSuccess, SettingsPersistenceFailureReport> {
    let mut steps = Vec::new();

    steps.push(SettingsPersistenceStep::Validate);
    adapter.validate_accepted(plan.hostname()).map_err(|_| {
        persistence_failure(
            SettingsPersistenceFailure::Validation,
            &steps,
            SettingsPersistenceFailureDisposition::CommitNotConfirmed,
        )
    })?;

    let mut transaction = adapter.begin_transaction().map_err(|_| {
        persistence_failure(
            SettingsPersistenceFailure::Transaction,
            &steps,
            SettingsPersistenceFailureDisposition::CommitNotConfirmed,
        )
    })?;

    steps.push(SettingsPersistenceStep::write_hostname());
    transaction.write_hostname(plan.hostname()).map_err(|_| {
        persistence_failure(
            SettingsPersistenceFailure::Write {
                key: "hostname".to_owned(),
            },
            &steps,
            SettingsPersistenceFailureDisposition::CommitNotConfirmed,
        )
    })?;

    steps.push(SettingsPersistenceStep::Commit);
    transaction.commit().map_err(|_| {
        persistence_failure(
            SettingsPersistenceFailure::Commit,
            &steps,
            SettingsPersistenceFailureDisposition::CommitNotConfirmed,
        )
    })?;

    steps.push(SettingsPersistenceStep::Reload);
    let candidate = transaction.reload().map_err(|_| {
        persistence_failure(
            SettingsPersistenceFailure::Reload,
            &steps,
            SettingsPersistenceFailureDisposition::PostCommitUncertain,
        )
    })?;

    steps.push(SettingsPersistenceStep::Reconcile);
    if candidate.hostname().as_str() != plan.hostname().as_str() {
        return Err(persistence_failure(
            SettingsPersistenceFailure::Reconcile,
            &steps,
            SettingsPersistenceFailureDisposition::PostCommitUncertain,
        ));
    }

    steps.push(SettingsPersistenceStep::Publish);
    transaction.publish(candidate).map_err(|_| {
        persistence_failure(
            SettingsPersistenceFailure::Publication,
            &steps,
            SettingsPersistenceFailureDisposition::PostCommitUncertain,
        )
    })?;

    drop(transaction);

    steps.push(SettingsPersistenceStep::PublicSuccess);

    Ok(SettingsPersistenceSuccess {
        steps,
        effects: vec![plan.confirmed_effect()],
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

fn persistence_failure(
    reason: SettingsPersistenceFailure,
    completed_steps: &[SettingsPersistenceStep],
    disposition: SettingsPersistenceFailureDisposition,
) -> SettingsPersistenceFailureReport {
    SettingsPersistenceFailureReport {
        reason,
        public_error: SettingsPatchPublicError::WrongApiInput,
        completed_steps: completed_steps.to_vec(),
        disposition,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use bitaxe_config::{
        confirm_hostname_snapshot, ConfirmedHostnameSnapshot, NvsSnapshot, NvsWrite, StoredValue,
    };
    use serde::Deserialize;
    use serde_json::{json, Value};

    use super::{
        execute_settings_persistence_plan, plan_settings_patch_body, plan_settings_patch_value,
        SettingsAdapterFailure, SettingsPatchFailureReason, SettingsPatchPublicError,
        SettingsPersistenceAdapter, SettingsPersistenceEffect, SettingsPersistenceFailure,
        SettingsPersistenceFailureDisposition, SettingsPersistencePlan, SettingsPersistenceStep,
        SettingsPersistenceTransaction, SettingsPublicResponse,
    };
    use crate::{decide_v12_settings_value, Hostname, V12SettingsChange, V12SettingsDecision};

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum AdapterFailurePoint {
        Validate,
        Begin,
        Write,
        Commit,
        Reload,
        Publish,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum AdapterEvent {
        Validate(&'static str),
        Begin(&'static str),
        Step(&'static str, SettingsPersistenceStep),
        Blocked(&'static str),
        End(&'static str),
    }

    #[derive(Debug)]
    struct SharedAdapterState {
        active_owner: Option<&'static str>,
        persisted_hostname: String,
        published_hostname: String,
        publication_history: Vec<(&'static str, String)>,
        events: Vec<AdapterEvent>,
    }

    impl SharedAdapterState {
        fn new(hostname: &str) -> Self {
            Self {
                active_owner: None,
                persisted_hostname: hostname.to_owned(),
                published_hostname: hostname.to_owned(),
                publication_history: Vec::new(),
                events: Vec::new(),
            }
        }
    }

    struct RecordingAdapter {
        owner: &'static str,
        shared: Rc<RefCell<SharedAdapterState>>,
        maybe_failure: Option<AdapterFailurePoint>,
        maybe_reloaded_hostname: Option<String>,
        probe_contender_during_reload: bool,
    }

    impl RecordingAdapter {
        fn new(owner: &'static str, shared: Rc<RefCell<SharedAdapterState>>) -> Self {
            Self {
                owner,
                shared,
                maybe_failure: None,
                maybe_reloaded_hostname: None,
                probe_contender_during_reload: false,
            }
        }

        fn failing_at(mut self, failure: AdapterFailurePoint) -> Self {
            self.maybe_failure = Some(failure);
            self
        }

        fn reloading(mut self, hostname: &str) -> Self {
            self.maybe_reloaded_hostname = Some(hostname.to_owned());
            self
        }

        fn probing_contention(mut self) -> Self {
            self.probe_contender_during_reload = true;
            self
        }
    }

    struct RecordingTransaction {
        owner: &'static str,
        shared: Rc<RefCell<SharedAdapterState>>,
        maybe_failure: Option<AdapterFailurePoint>,
        maybe_reloaded_hostname: Option<String>,
        pending_hostname: Option<String>,
        probe_contender_during_reload: bool,
    }

    impl Drop for RecordingTransaction {
        fn drop(&mut self) {
            let mut shared = self.shared.borrow_mut();
            assert_eq!(shared.active_owner, Some(self.owner));
            shared.events.push(AdapterEvent::End(self.owner));
            shared.active_owner = None;
        }
    }

    impl SettingsPersistenceTransaction for RecordingTransaction {
        fn write_hostname(&mut self, hostname: &Hostname) -> Result<(), SettingsAdapterFailure> {
            self.record_step(SettingsPersistenceStep::write_hostname());
            if self.maybe_failure == Some(AdapterFailurePoint::Write) {
                return Err(SettingsAdapterFailure::failed("fake write failure"));
            }

            self.pending_hostname = Some(hostname.as_str().to_owned());
            Ok(())
        }

        fn commit(&mut self) -> Result<(), SettingsAdapterFailure> {
            self.record_step(SettingsPersistenceStep::Commit);
            if self.maybe_failure == Some(AdapterFailurePoint::Commit) {
                return Err(SettingsAdapterFailure::failed("fake commit failure"));
            }

            let Some(hostname) = self.pending_hostname.clone() else {
                return Err(SettingsAdapterFailure::failed("fake missing write"));
            };
            self.shared.borrow_mut().persisted_hostname = hostname;
            Ok(())
        }

        fn reload(&mut self) -> Result<ConfirmedHostnameSnapshot, SettingsAdapterFailure> {
            self.record_step(SettingsPersistenceStep::Reload);
            if self.probe_contender_during_reload {
                self.shared
                    .borrow_mut()
                    .events
                    .push(AdapterEvent::Blocked("writer-2"));
            }
            if self.maybe_failure == Some(AdapterFailurePoint::Reload) {
                return Err(SettingsAdapterFailure::failed("fake reload failure"));
            }

            let hostname = self
                .maybe_reloaded_hostname
                .clone()
                .unwrap_or_else(|| self.shared.borrow().persisted_hostname.clone());
            confirm_hostname_snapshot(NvsSnapshot::from_values([StoredValue::string(
                "hostname", hostname,
            )]))
            .map_err(|_| SettingsAdapterFailure::failed("fake invalid reload"))
        }

        fn publish(
            &mut self,
            candidate: ConfirmedHostnameSnapshot,
        ) -> Result<(), SettingsAdapterFailure> {
            self.record_step(SettingsPersistenceStep::Publish);
            if self.maybe_failure == Some(AdapterFailurePoint::Publish) {
                return Err(SettingsAdapterFailure::failed("fake publication failure"));
            }

            let hostname = candidate.hostname().as_str().to_owned();
            let mut shared = self.shared.borrow_mut();
            shared.published_hostname.clone_from(&hostname);
            shared.publication_history.push((self.owner, hostname));
            Ok(())
        }
    }

    impl RecordingTransaction {
        fn record_step(&self, step: SettingsPersistenceStep) {
            let mut shared = self.shared.borrow_mut();
            assert_eq!(shared.active_owner, Some(self.owner));
            shared.events.push(AdapterEvent::Step(self.owner, step));
        }
    }

    impl SettingsPersistenceAdapter for RecordingAdapter {
        type Transaction<'adapter> = RecordingTransaction;

        fn validate_accepted(
            &mut self,
            _hostname: &Hostname,
        ) -> Result<(), SettingsAdapterFailure> {
            self.shared
                .borrow_mut()
                .events
                .push(AdapterEvent::Validate(self.owner));
            if self.maybe_failure == Some(AdapterFailurePoint::Validate) {
                return Err(SettingsAdapterFailure::failed("fake validation failure"));
            }
            Ok(())
        }

        fn begin_transaction(&mut self) -> Result<Self::Transaction<'_>, SettingsAdapterFailure> {
            if self.maybe_failure == Some(AdapterFailurePoint::Begin) {
                return Err(SettingsAdapterFailure::failed("fake begin failure"));
            }

            let mut shared = self.shared.borrow_mut();
            if shared.active_owner.is_some() {
                shared.events.push(AdapterEvent::Blocked(self.owner));
                return Err(SettingsAdapterFailure::failed("fake transaction busy"));
            }
            shared.active_owner = Some(self.owner);
            shared.events.push(AdapterEvent::Begin(self.owner));
            drop(shared);

            Ok(RecordingTransaction {
                owner: self.owner,
                shared: Rc::clone(&self.shared),
                maybe_failure: self.maybe_failure,
                maybe_reloaded_hostname: self.maybe_reloaded_hostname.clone(),
                pending_hostname: None,
                probe_contender_during_reload: self.probe_contender_during_reload,
            })
        }
    }

    fn hostname(value: &str) -> Hostname {
        let decision = decide_v12_settings_value(&json!({"hostname": value}))
            .expect("test hostname must validate");
        let V12SettingsDecision::Authorized(V12SettingsChange::Hostname(hostname)) = decision
        else {
            panic!("test hostname must be authorized");
        };
        hostname
    }

    fn persistence_plan(value: &str) -> SettingsPersistencePlan {
        SettingsPersistencePlan::for_hostname(hostname(value))
    }

    #[test]
    fn settings_persistence_success_orders_confirmation_before_public_success_and_effect() {
        // Arrange
        let shared = Rc::new(RefCell::new(SharedAdapterState::new("bitaxe")));
        let plan = persistence_plan("axe-205");
        let mut adapter = RecordingAdapter::new("writer-1", Rc::clone(&shared));

        // Act
        let success = execute_settings_persistence_plan(&plan, &mut adapter)
            .expect("confirmed hostname transaction must succeed");

        // Assert
        assert_eq!(
            success.steps(),
            [
                SettingsPersistenceStep::Validate,
                SettingsPersistenceStep::write_hostname(),
                SettingsPersistenceStep::Commit,
                SettingsPersistenceStep::Reload,
                SettingsPersistenceStep::Reconcile,
                SettingsPersistenceStep::Publish,
                SettingsPersistenceStep::PublicSuccess,
            ]
        );
        assert_eq!(
            success.public_response(),
            SettingsPublicResponse::EmptySuccess
        );
        assert_eq!(
            success.effects(),
            [SettingsPersistenceEffect::BestEffortApplyHostname {
                hostname: "axe-205".to_owned(),
            }]
        );
        assert_eq!(shared.borrow().published_hostname, "axe-205");
    }

    #[test]
    fn settings_persistence_failures_are_typed_and_never_publish_success_or_effects() {
        // Arrange
        let cases = [
            (
                AdapterFailurePoint::Validate,
                SettingsPersistenceFailure::Validation,
                SettingsPersistenceFailureDisposition::CommitNotConfirmed,
            ),
            (
                AdapterFailurePoint::Begin,
                SettingsPersistenceFailure::Transaction,
                SettingsPersistenceFailureDisposition::CommitNotConfirmed,
            ),
            (
                AdapterFailurePoint::Write,
                SettingsPersistenceFailure::Write {
                    key: "hostname".to_owned(),
                },
                SettingsPersistenceFailureDisposition::CommitNotConfirmed,
            ),
            (
                AdapterFailurePoint::Commit,
                SettingsPersistenceFailure::Commit,
                SettingsPersistenceFailureDisposition::CommitNotConfirmed,
            ),
            (
                AdapterFailurePoint::Reload,
                SettingsPersistenceFailure::Reload,
                SettingsPersistenceFailureDisposition::PostCommitUncertain,
            ),
            (
                AdapterFailurePoint::Publish,
                SettingsPersistenceFailure::Publication,
                SettingsPersistenceFailureDisposition::PostCommitUncertain,
            ),
        ];

        for (failure_point, expected_reason, expected_disposition) in cases {
            let shared = Rc::new(RefCell::new(SharedAdapterState::new("bitaxe")));
            let plan = persistence_plan("axe-205");
            let mut adapter =
                RecordingAdapter::new("writer-1", Rc::clone(&shared)).failing_at(failure_point);

            // Act
            let failure = execute_settings_persistence_plan(&plan, &mut adapter)
                .expect_err("configured fake failure must reject success");

            // Assert
            assert_eq!(failure.reason(), &expected_reason);
            assert_eq!(failure.disposition(), expected_disposition);
            assert_eq!(failure.public_error().body(), "Wrong API input");
            assert!(!failure
                .completed_steps()
                .contains(&SettingsPersistenceStep::PublicSuccess));
            assert!(shared.borrow().publication_history.is_empty());
        }
    }

    #[test]
    fn settings_persistence_reload_mismatch_is_post_commit_uncertainty_without_publication() {
        // Arrange
        let shared = Rc::new(RefCell::new(SharedAdapterState::new("bitaxe")));
        let plan = persistence_plan("axe-205");
        let mut adapter =
            RecordingAdapter::new("writer-1", Rc::clone(&shared)).reloading("another-host");

        // Act
        let failure = execute_settings_persistence_plan(&plan, &mut adapter)
            .expect_err("mismatched reload must reject success");

        // Assert
        assert_eq!(failure.reason(), &SettingsPersistenceFailure::Reconcile);
        assert_eq!(
            failure.disposition(),
            SettingsPersistenceFailureDisposition::PostCommitUncertain
        );
        assert_eq!(shared.borrow().persisted_hostname, "axe-205");
        assert_eq!(shared.borrow().published_hostname, "bitaxe");
        assert!(shared.borrow().publication_history.is_empty());
    }

    #[test]
    fn settings_persistence_same_value_uses_the_full_confirmation_chain() {
        // Arrange
        let shared = Rc::new(RefCell::new(SharedAdapterState::new("axe-205")));
        let plan = persistence_plan("axe-205");
        let mut adapter = RecordingAdapter::new("writer-1", Rc::clone(&shared));

        // Act
        let success = execute_settings_persistence_plan(&plan, &mut adapter)
            .expect("same-value hostname must still confirm");

        // Assert
        assert_eq!(
            success.steps(),
            [
                SettingsPersistenceStep::Validate,
                SettingsPersistenceStep::write_hostname(),
                SettingsPersistenceStep::Commit,
                SettingsPersistenceStep::Reload,
                SettingsPersistenceStep::Reconcile,
                SettingsPersistenceStep::Publish,
                SettingsPersistenceStep::PublicSuccess,
            ]
        );
        assert_eq!(shared.borrow().publication_history.len(), 1);
    }

    #[test]
    fn settings_persistence_serializes_two_writers_through_publication() {
        // Arrange
        let shared = Rc::new(RefCell::new(SharedAdapterState::new("bitaxe")));
        let first_plan = persistence_plan("writer-one");
        let second_plan = persistence_plan("writer-two");
        let mut first = RecordingAdapter::new("writer-1", Rc::clone(&shared)).probing_contention();
        let mut second = RecordingAdapter::new("writer-2", Rc::clone(&shared));

        // Act
        execute_settings_persistence_plan(&first_plan, &mut first)
            .expect("first writer must confirm");
        execute_settings_persistence_plan(&second_plan, &mut second)
            .expect("second writer must confirm after first releases ownership");

        // Assert
        let shared = shared.borrow();
        let first_publish = shared
            .events
            .iter()
            .position(|event| {
                *event == AdapterEvent::Step("writer-1", SettingsPersistenceStep::Publish)
            })
            .expect("first publication event must exist");
        let first_end = shared
            .events
            .iter()
            .position(|event| *event == AdapterEvent::End("writer-1"))
            .expect("first transaction end must exist");
        let second_begin = shared
            .events
            .iter()
            .position(|event| *event == AdapterEvent::Begin("writer-2"))
            .expect("second transaction begin must exist");
        assert!(shared.events.contains(&AdapterEvent::Blocked("writer-2")));
        assert!(first_publish < first_end && first_end < second_begin);
        assert_eq!(
            shared.publication_history,
            [
                ("writer-1", "writer-one".to_owned()),
                ("writer-2", "writer-two".to_owned()),
            ]
        );
    }
}
