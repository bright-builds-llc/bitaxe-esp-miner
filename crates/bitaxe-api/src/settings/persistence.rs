use bitaxe_config::{ConfirmedHostnameSnapshot, NvsWrite};
use thiserror::Error;

use super::patch::SettingsPatchPublicError;
use crate::v12_settings::Hostname;

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
