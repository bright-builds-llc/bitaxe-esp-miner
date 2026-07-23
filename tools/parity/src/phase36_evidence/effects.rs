//! Independent completeness admission for the bounded Phase 35 effect interval.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::contract::ComponentInsufficiency;
use crate::phase35_evidence::sha256_hex;

const EFFECT_SCHEMA: &str = "phase36-independent-effects-v1";
const EFFECT_DOCUMENT: &str = "independent-effects.json";
const MAX_INTERVAL_MILLIS: u64 = 3_600_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum IndependentEffectAdmission {
    Validated {
        interval: Box<ValidatedIndependentEffectInterval>,
    },
    Insufficient {
        category: ComponentInsufficiency,
    },
}

impl IndependentEffectAdmission {
    #[must_use]
    pub const fn insufficient() -> Self {
        Self::Insufficient {
            category: ComponentInsufficiency::IndependentEffectObservation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidatedIndependentEffectInterval {
    pub observation_source: IndependentEffectObservationSource,
    pub start_millis: u64,
    pub end_millis: u64,
    pub duration_millis: u64,
    pub effect_count: u8,
    pub ledger_digest: String,
    pub claim_fact_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndependentEffectObservationSource {
    IndependentLedger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyEffectCategory {
    PackageProbe,
    PackageFlash,
    PassiveObservation,
    HostnameRead,
    HostnamePatch,
    ApprovedReboot,
    HostnameRestoration,
    Cleanup,
    ActiveControl,
    SelfTest,
    Watchdog,
    Mining,
    CredentialMutation,
    Ota,
    OtherBoard,
}

impl LegacyEffectCategory {
    const REQUIRED: [Self; 8] = [
        Self::PackageProbe,
        Self::PackageFlash,
        Self::PassiveObservation,
        Self::HostnameRead,
        Self::HostnamePatch,
        Self::ApprovedReboot,
        Self::HostnameRestoration,
        Self::Cleanup,
    ];

    const fn prohibited(self) -> bool {
        matches!(
            self,
            Self::ActiveControl
                | Self::SelfTest
                | Self::Watchdog
                | Self::Mining
                | Self::CredentialMutation
                | Self::Ota
                | Self::OtherBoard
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum EffectRecordOwner {
    IndependentObserver,
    Supervisor,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IndependentEffectDocument {
    schema_version: String,
    interval_start_millis: u64,
    interval_end_millis: u64,
    interval_closed: bool,
    unledgered_effect_paths: u64,
    records: Vec<IndependentEffectRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IndependentEffectRecord {
    sequence: u8,
    monotonic_millis: u64,
    effect: LegacyEffectCategory,
    owner: EffectRecordOwner,
    authorization_digest: String,
    invocation_digest: String,
    result_digest: String,
    closure_digest: String,
    closed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum IndependentEffectEvidenceError {
    #[error("independent_effect_document_invalid")]
    DocumentInvalid,
    #[error("independent_effect_prohibited")]
    ProhibitedEffect,
    #[error("independent_effect_root_invalid")]
    ProtectedRootInvalid,
    #[error("independent_effect_input_symlink")]
    ProtectedInputSymlink,
    #[error("independent_effect_permissions_invalid")]
    WrongPermissions,
}

pub fn classify_independent_effect_document(
    maybe_independent_document: Option<&str>,
    _maybe_supervisor_attestation: Option<&str>,
) -> Result<IndependentEffectAdmission, IndependentEffectEvidenceError> {
    let Some(document) = maybe_independent_document else {
        return Ok(IndependentEffectAdmission::insufficient());
    };
    let ledger = serde_json::from_str::<IndependentEffectDocument>(document)
        .map_err(|_| IndependentEffectEvidenceError::DocumentInvalid)?;
    if ledger
        .records
        .iter()
        .any(|record| record.effect.prohibited())
    {
        return Err(IndependentEffectEvidenceError::ProhibitedEffect);
    }
    if !complete_ledger(&ledger) {
        return Ok(IndependentEffectAdmission::insufficient());
    }
    let ledger_digest = digest_serializable(&ledger)?;
    let duration_millis = ledger.interval_end_millis - ledger.interval_start_millis;
    let claim_fact_digest = digest_serializable(&(
        IndependentEffectObservationSource::IndependentLedger,
        ledger.interval_start_millis,
        ledger.interval_end_millis,
        &ledger_digest,
    ))?;
    Ok(IndependentEffectAdmission::Validated {
        interval: Box::new(ValidatedIndependentEffectInterval {
            observation_source: IndependentEffectObservationSource::IndependentLedger,
            start_millis: ledger.interval_start_millis,
            end_millis: ledger.interval_end_millis,
            duration_millis,
            effect_count: LegacyEffectCategory::REQUIRED.len() as u8,
            ledger_digest,
            claim_fact_digest,
        }),
    })
}

pub fn classify_independent_effect_root(
    protected_root: &Utf8Path,
) -> Result<IndependentEffectAdmission, IndependentEffectEvidenceError> {
    validate_root(protected_root)?;
    let input = protected_root.join(EFFECT_DOCUMENT);
    let metadata = match fs::symlink_metadata(input.as_std_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(IndependentEffectAdmission::insufficient());
        }
        Err(_) => return Err(IndependentEffectEvidenceError::ProtectedRootInvalid),
    };
    if metadata.file_type().is_symlink() {
        return Err(IndependentEffectEvidenceError::ProtectedInputSymlink);
    }
    if !metadata.is_file() {
        return Err(IndependentEffectEvidenceError::ProtectedRootInvalid);
    }
    if metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(IndependentEffectEvidenceError::WrongPermissions);
    }
    let document = fs::read_to_string(input.as_std_path())
        .map_err(|_| IndependentEffectEvidenceError::ProtectedRootInvalid)?;
    classify_independent_effect_document(Some(&document), None)
}

fn complete_ledger(ledger: &IndependentEffectDocument) -> bool {
    if ledger.schema_version != EFFECT_SCHEMA
        || ledger.interval_start_millis == 0
        || ledger.interval_end_millis <= ledger.interval_start_millis
        || ledger.interval_end_millis - ledger.interval_start_millis > MAX_INTERVAL_MILLIS
        || !ledger.interval_closed
        || ledger.unledgered_effect_paths != 0
        || ledger.records.len() != LegacyEffectCategory::REQUIRED.len()
    {
        return false;
    }
    let mut previous_millis = ledger.interval_start_millis;
    for (index, record) in ledger.records.iter().enumerate() {
        if record.sequence != index as u8 + 1
            || record.effect != LegacyEffectCategory::REQUIRED[index]
            || record.owner != EffectRecordOwner::IndependentObserver
            || record.monotonic_millis <= previous_millis
            || record.monotonic_millis >= ledger.interval_end_millis
            || !record.closed
            || !valid_digest(&record.authorization_digest)
            || !valid_digest(&record.invocation_digest)
            || !valid_digest(&record.result_digest)
            || !valid_digest(&record.closure_digest)
        {
            return false;
        }
        previous_millis = record.monotonic_millis;
    }
    true
}

fn validate_root(protected_root: &Utf8Path) -> Result<(), IndependentEffectEvidenceError> {
    let metadata = fs::symlink_metadata(protected_root.as_std_path())
        .map_err(|_| IndependentEffectEvidenceError::ProtectedRootInvalid)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(IndependentEffectEvidenceError::ProtectedRootInvalid);
    }
    if metadata.permissions().mode() & 0o777 != 0o700 {
        return Err(IndependentEffectEvidenceError::WrongPermissions);
    }
    Ok(())
}

fn digest_serializable(value: &impl Serialize) -> Result<String, IndependentEffectEvidenceError> {
    let bytes =
        serde_json::to_vec(value).map_err(|_| IndependentEffectEvidenceError::DocumentInvalid)?;
    Ok(sha256_hex(&bytes))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
