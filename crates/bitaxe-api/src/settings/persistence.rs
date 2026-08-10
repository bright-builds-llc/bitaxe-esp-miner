use std::fmt;

use bitaxe_config::{NvsSnapshot, NvsWrite};
use thiserror::Error;

use super::patch::{AcceptedSettingsPatch, SettingsPatchPublicError};

/// Public settings route response shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPublicResponse {
    /// Upstream-compatible empty response body on success.
    EmptySuccess,
    /// Upstream-compatible generic error body.
    Error(SettingsPatchPublicError),
}

/// Closed validated settings persistence plan for a firmware adapter.
#[derive(Clone, PartialEq, Eq)]
pub struct SettingsPersistencePlan {
    writes: Vec<NvsWrite>,
    maybe_hostname: Option<String>,
}

impl SettingsPersistencePlan {
    /// Builds a persistence plan from the effect-free validated PATCH result.
    #[must_use]
    pub fn from_accepted(accepted: &AcceptedSettingsPatch) -> Self {
        Self {
            writes: accepted.writes().to_vec(),
            maybe_hostname: accepted.maybe_hostname().map(str::to_owned),
        }
    }

    /// Returns the complete validated write set.
    #[must_use]
    pub fn writes(&self) -> &[NvsWrite] {
        &self.writes
    }

    /// Returns whether this plan contains no known settings writes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }

    fn confirmed_effects(&self) -> Vec<SettingsPersistenceEffect> {
        self.maybe_hostname
            .as_ref()
            .map(
                |hostname| SettingsPersistenceEffect::BestEffortApplyHostname {
                    hostname: hostname.clone(),
                },
            )
            .into_iter()
            .collect()
    }
}

impl fmt::Debug for SettingsPersistencePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys: Vec<&str> = self
            .writes
            .iter()
            .map(|write| match write {
                NvsWrite::String { key, .. }
                | NvsWrite::U16 { key, .. }
                | NvsWrite::I32 { key, .. }
                | NvsWrite::U64 { key, .. } => key.as_str(),
            })
            .collect();
        formatter
            .debug_struct("SettingsPersistencePlan")
            .field("keys", &keys)
            .field("hostname_present", &self.maybe_hostname.is_some())
            .finish()
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
    /// Every independently reloaded typed value matched the write set exactly.
    Reconcile,
    /// The independently reloaded non-secret snapshot became public truth.
    Publish,
    /// The route may return an upstream-compatible empty success body.
    PublicSuccess,
}

impl SettingsPersistenceStep {
    /// Creates an authorized typed write step without retaining its value.
    #[must_use]
    pub fn write(key: impl Into<String>) -> Self {
        Self::Write { key: key.into() }
    }
}

/// Private reconciliation result carrying only the public non-secret snapshot.
pub struct ReloadedSettings {
    public_snapshot: NvsSnapshot,
    writes_match: bool,
}

impl ReloadedSettings {
    /// Creates a reload result after the adapter privately compares expected values.
    #[must_use]
    pub const fn new(public_snapshot: NvsSnapshot, writes_match: bool) -> Self {
        Self {
            public_snapshot,
            writes_match,
        }
    }

    const fn writes_match(&self) -> bool {
        self.writes_match
    }

    fn into_public_snapshot(self) -> NvsSnapshot {
        self.public_snapshot
    }
}

/// Adapter transaction whose lifetime serializes mutation through publication.
pub trait SettingsPersistenceTransaction {
    /// Writes one validated typed value, including same-value requests.
    fn write(&mut self, write: &NvsWrite) -> Result<(), SettingsAdapterFailure>;

    /// Commits the complete write set once.
    fn commit(&mut self) -> Result<(), SettingsAdapterFailure>;

    /// Privately reloads and compares all writes, plus the non-secret public snapshot.
    fn reload(&mut self, expected: &[NvsWrite])
        -> Result<ReloadedSettings, SettingsAdapterFailure>;

    /// Atomically publishes the already reconciled independently reloaded snapshot.
    fn publish(&mut self, candidate: NvsSnapshot) -> Result<(), SettingsAdapterFailure>;
}

/// Thin firmware coordinator used by the pure settings executor.
pub trait SettingsPersistenceAdapter {
    /// Transaction type that holds serialization ownership until it is dropped.
    type Transaction<'adapter>: SettingsPersistenceTransaction
    where
        Self: 'adapter;

    /// Acknowledges the already validated closed write capability.
    fn validate_accepted(
        &mut self,
        plan: &SettingsPersistencePlan,
    ) -> Result<(), SettingsAdapterFailure>;

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

    /// Best-effort transfers the confirmed live effects without changing success authority.
    #[must_use]
    pub fn maybe_acquire_best_effort_effect_lease<EffectLease, AcquisitionError>(
        &self,
        acquire: impl FnOnce(Vec<SettingsPersistenceEffect>) -> Result<EffectLease, AcquisitionError>,
    ) -> Option<EffectLease> {
        acquire(self.effects.clone()).ok()
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

/// Executes one serialized settings transaction and returns success only after publication.
pub fn execute_settings_persistence_plan(
    plan: &SettingsPersistencePlan,
    adapter: &mut impl SettingsPersistenceAdapter,
) -> Result<SettingsPersistenceSuccess, SettingsPersistenceFailureReport> {
    let mut steps = Vec::new();

    steps.push(SettingsPersistenceStep::Validate);
    adapter.validate_accepted(plan).map_err(|_| {
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

    for write in plan.writes() {
        let key = write_key(write).to_owned();
        steps.push(SettingsPersistenceStep::write(key.clone()));
        transaction.write(write).map_err(|_| {
            persistence_failure(
                SettingsPersistenceFailure::Write { key },
                &steps,
                SettingsPersistenceFailureDisposition::CommitNotConfirmed,
            )
        })?;
    }

    steps.push(SettingsPersistenceStep::Commit);
    transaction.commit().map_err(|_| {
        persistence_failure(
            SettingsPersistenceFailure::Commit,
            &steps,
            SettingsPersistenceFailureDisposition::CommitNotConfirmed,
        )
    })?;

    steps.push(SettingsPersistenceStep::Reload);
    let candidate = transaction.reload(plan.writes()).map_err(|_| {
        persistence_failure(
            SettingsPersistenceFailure::Reload,
            &steps,
            SettingsPersistenceFailureDisposition::PostCommitUncertain,
        )
    })?;

    steps.push(SettingsPersistenceStep::Reconcile);
    if !candidate.writes_match() {
        return Err(persistence_failure(
            SettingsPersistenceFailure::Reconcile,
            &steps,
            SettingsPersistenceFailureDisposition::PostCommitUncertain,
        ));
    }

    steps.push(SettingsPersistenceStep::Publish);
    transaction
        .publish(candidate.into_public_snapshot())
        .map_err(|_| {
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
        effects: plan.confirmed_effects(),
        public_response: SettingsPublicResponse::EmptySuccess,
    })
}

fn write_key(write: &NvsWrite) -> &str {
    match write {
        NvsWrite::String { key, .. }
        | NvsWrite::U16 { key, .. }
        | NvsWrite::I32 { key, .. }
        | NvsWrite::U64 { key, .. } => key.as_str(),
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
