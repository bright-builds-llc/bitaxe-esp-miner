use std::fmt;
use std::fs;
use std::io;
use std::sync::atomic::AtomicU64;

use camino::{Utf8Path, Utf8PathBuf};
use clap::ValueEnum;

mod filesystem;
mod rendering;
#[cfg(test)]
mod tests;

use filesystem::*;
use rendering::*;

use super::{EvidenceDisposition, OperatorEvidenceProfile, OperatorEvidenceSlot};

const MANIFEST_FILE: &str = ".phase28-evidence-manifest";
const SUMMARY_FILE: &str = "summary.md";
static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const COMPLETION_PROVENANCE: &str = "generated_provenance: phase29-completion";

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
pub(crate) enum WorkflowStatus {
    Passed,
    Blocked,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PromotionFailurePoint {
    BeforeStagingSync,
    BeforeExchange,
    AfterExchange,
    DuringOldGenerationCleanup,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConsolidationOptions {
    pub(crate) maybe_failure: Option<PromotionFailurePoint>,
}

#[derive(Debug)]
pub(crate) enum GenerationError {
    InvalidInput(String),
    Io {
        action: String,
        source: io::Error,
    },
    Validation(Vec<String>),
    Injected(PromotionFailurePoint),
    RecoveryRequired {
        destination: Utf8PathBuf,
        retained_old_generation: Utf8PathBuf,
        detail: String,
    },
}

impl fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => formatter.write_str(message),
            Self::Io { action, source } => write!(formatter, "{action}: {source}"),
            Self::Validation(errors) => {
                write!(formatter, "generated operator evidence failed validation: {}", errors.join("; "))
            }
            Self::Injected(point) => write!(formatter, "injected promotion failure at {point:?}"),
            Self::RecoveryRequired {
                destination,
                retained_old_generation,
                detail,
            } => write!(
                formatter,
                "phase28 promotion needs recovery; destination={destination}; retained_old_generation={retained_old_generation}; {detail}"
            ),
        }
    }
}

impl std::error::Error for GenerationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

type GenerationResult<T> = Result<T, GenerationError>;

pub(crate) fn complete_operator_evidence(
    workspace_root: &Utf8Path,
    profile: OperatorEvidenceProfile,
    evidence_root: &Utf8Path,
    workflow_status: WorkflowStatus,
) -> GenerationResult<Vec<Utf8PathBuf>> {
    if !matches!(
        profile,
        OperatorEvidenceProfile::Phase25 | OperatorEvidenceProfile::Phase27
    ) {
        return Err(GenerationError::InvalidInput(
            "completion supports only phase25 and phase27 profiles".to_owned(),
        ));
    }

    let relative_root = normalize_repo_relative(evidence_root, "evidence root")?;
    let absolute_root = workspace_root.join(&relative_root);
    reject_symlink_managed_path(workspace_root, &absolute_root)?;
    fs::create_dir_all(absolute_root.as_std_path()).map_err(|source| {
        io_error(
            format!("failed to create evidence root {absolute_root}"),
            source,
        )
    })?;

    let disposition = match workflow_status {
        WorkflowStatus::Passed => EvidenceDisposition::Deferred,
        WorkflowStatus::Blocked | WorkflowStatus::Failed => EvidenceDisposition::Blocked,
    };
    let mut changed = Vec::new();
    for slot in OperatorEvidenceSlot::ALL {
        let path = absolute_root.join(slot.file_name());
        let desired = render_completion_slot(profile, slot, disposition, workflow_status);
        if !path.exists() {
            write_synced(&path, &desired)?;
            changed.push(path);
            continue;
        }

        let existing = fs::read_to_string(path.as_std_path()).map_err(|source| {
            io_error(
                format!("failed to read existing evidence slot {path}"),
                source,
            )
        })?;
        if !is_completion_owned(&existing) || existing == desired {
            continue;
        }

        replace_synced(&path, &desired)?;
        changed.push(path);
    }
    validate_completion_workflow_state(&absolute_root, workflow_status)?;
    sync_directory(&absolute_root)?;
    Ok(changed)
}

fn is_completion_owned(contents: &str) -> bool {
    contents.lines().any(|line| line == COMPLETION_PROVENANCE)
}

fn validate_completion_workflow_state(
    root: &Utf8Path,
    workflow_status: WorkflowStatus,
) -> GenerationResult<()> {
    if workflow_status != WorkflowStatus::Passed {
        return Ok(());
    }

    for slot in OperatorEvidenceSlot::ALL {
        let path = root.join(slot.file_name());
        let contents = fs::read_to_string(path.as_std_path()).map_err(|source| {
            io_error(
                format!("failed to verify completed evidence slot {path}"),
                source,
            )
        })?;
        if is_completion_owned(&contents)
            && contents
                .lines()
                .any(|line| line == "workflow_status: failed")
        {
            return Err(GenerationError::Validation(vec![format!(
                "passed workflow retains failed generator-owned slot {}",
                slot.file_name()
            )]));
        }
    }
    Ok(())
}

pub(crate) fn consolidate_phase28_evidence(
    workspace_root: &Utf8Path,
    phase27_root: &Utf8Path,
    evidence_root: &Utf8Path,
) -> GenerationResult<()> {
    consolidate_phase28_evidence_with_options(
        workspace_root,
        phase27_root,
        evidence_root,
        ConsolidationOptions::default(),
    )
}

pub(crate) fn consolidate_phase28_evidence_with_options(
    workspace_root: &Utf8Path,
    phase27_root: &Utf8Path,
    evidence_root: &Utf8Path,
    options: ConsolidationOptions,
) -> GenerationResult<()> {
    let relative_source = normalize_repo_relative(phase27_root, "phase27 root")?;
    let relative_destination = normalize_repo_relative(evidence_root, "evidence root")?;
    reject_related_roots(&relative_source, &relative_destination)?;

    let source = workspace_root.join(&relative_source);
    let destination = workspace_root.join(&relative_destination);
    reject_symlink_managed_path(workspace_root, &source)?;
    reject_symlink_managed_path(workspace_root, &destination)?;
    if !source.is_dir() {
        return Err(GenerationError::InvalidInput(format!(
            "phase27 root is not a directory: {relative_source}"
        )));
    }

    if destination.exists() {
        validate_managed_destination(&destination)?;
    }

    let source_categories = read_source_categories(&source)?;
    validate_source_categories(&source_categories)?;
    let staging = create_staging_directory(&destination)?;
    let generation_result = generate_phase28_staging(
        &staging,
        &relative_source,
        &source,
        &source_categories,
        options,
    );
    if let Err(error) = generation_result {
        let _ = fs::remove_dir_all(staging.as_std_path());
        return Err(error);
    }

    validate_staging(&staging, &relative_source, &source_categories)?;
    if options.maybe_failure == Some(PromotionFailurePoint::BeforeExchange) {
        let _ = fs::remove_dir_all(staging.as_std_path());
        return Err(GenerationError::Injected(
            PromotionFailurePoint::BeforeExchange,
        ));
    }

    promote_staging(&destination, &staging, options)
}
