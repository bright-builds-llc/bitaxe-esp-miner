use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

use bitaxe_api::BuildProvenance;

const PACKAGE_MANIFEST_FILE_NAME: &str = "bitaxe-ultra205-package.json";
const FACTORY_IMAGE_FILE_NAME: &str = "bitaxe-ultra205-factory.bin";
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct ReleaseEvidenceManifest {
    pub(crate) schema_version: u32,
    pub(crate) semantic_version: String,
    pub(crate) source_commit: String,
    pub(crate) reference_commit: String,
    pub(crate) app_elf_sha256: String,
    pub(crate) build_identity: ReleaseEvidenceBuildIdentity,
    #[serde(default)]
    pub(crate) artifacts: Vec<ReleaseEvidenceArtifact>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct ReleaseEvidenceBuildIdentity {
    pub(crate) label: String,
    pub(crate) channel: String,
    pub(crate) source_dirty: bool,
    pub(crate) release_tag: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct ReleaseEvidenceArtifact {
    pub(crate) path: String,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ReleaseEvidenceFlashEvidence {
    pub(crate) command_kind: String,
    pub(crate) board: String,
    pub(crate) firmware_commit: String,
    pub(crate) reference_commit: String,
    pub(crate) manifest_path: Utf8PathBuf,
    pub(crate) trusted_output: bool,
    pub(crate) observed_firmware_commit: String,
    pub(crate) observed_reference_commit: String,
    pub(crate) flash_image_path: Option<Utf8PathBuf>,
    pub(crate) flash_command: Option<String>,
    pub(crate) log_path: Utf8PathBuf,
    pub(crate) monitor_log_path: Utf8PathBuf,
}

#[derive(Debug)]
pub(crate) struct ReleaseEvidenceDocuments {
    pub(crate) manifest: ReleaseEvidenceManifest,
    pub(crate) current_git_head: String,
    pub(crate) allow_post_source_evidence_commits: bool,
    pub(crate) source_commit_is_ancestor_of_head: bool,
    pub(crate) post_source_changed_paths: Vec<Utf8PathBuf>,
    pub(crate) evidence_root: Utf8PathBuf,
    pub(crate) maybe_flash_evidence_json_path: Option<Utf8PathBuf>,
    pub(crate) maybe_flash_evidence: Option<ReleaseEvidenceFlashEvidence>,
    pub(crate) maybe_redaction_review: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ReleaseEvidenceReport {
    pub(crate) validation_errors: Vec<String>,
}

impl ReleaseEvidenceReport {
    pub(crate) fn passed(&self) -> bool {
        self.validation_errors.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct RawReleaseEvidenceFlashEvidence {
    command_kind: String,
    board: String,
    firmware_commit: String,
    reference_commit: String,
    manifest_path: String,
    trusted_output: bool,
    observed_firmware_commit: String,
    observed_reference_commit: String,
    flash_image_path: Option<String>,
    flash_command: Option<String>,
    log_path: String,
    monitor_log_path: String,
}

pub(crate) fn parse_release_evidence_manifest_json(
    manifest_json: &str,
    manifest_path: &Utf8Path,
) -> Result<ReleaseEvidenceManifest> {
    serde_json::from_str(manifest_json)
        .with_context(|| format!("package manifest {manifest_path} is not valid JSON"))
}

pub(crate) fn parse_flash_evidence_json(
    flash_evidence_json: &str,
    flash_evidence_path: &Utf8Path,
) -> Result<ReleaseEvidenceFlashEvidence> {
    let raw: RawReleaseEvidenceFlashEvidence = serde_json::from_str(flash_evidence_json)
        .with_context(|| format!("flash evidence {flash_evidence_path} is not valid JSON"))?;

    Ok(ReleaseEvidenceFlashEvidence {
        command_kind: raw.command_kind,
        board: raw.board,
        firmware_commit: raw.firmware_commit,
        reference_commit: raw.reference_commit,
        manifest_path: Utf8PathBuf::from(raw.manifest_path),
        trusted_output: raw.trusted_output,
        observed_firmware_commit: raw.observed_firmware_commit,
        observed_reference_commit: raw.observed_reference_commit,
        flash_image_path: raw.flash_image_path.map(Utf8PathBuf::from),
        flash_command: raw.flash_command,
        log_path: Utf8PathBuf::from(raw.log_path),
        monitor_log_path: Utf8PathBuf::from(raw.monitor_log_path),
    })
}

pub(crate) fn validate_release_evidence(
    documents: &ReleaseEvidenceDocuments,
    require_redaction_passed: bool,
) -> ReleaseEvidenceReport {
    let mut validation_errors = Vec::new();

    validate_manifest_identity(&mut validation_errors, documents);
    validate_current_commit(&mut validation_errors, documents);
    validate_redaction_review(&mut validation_errors, documents, require_redaction_passed);

    if let Some(flash_evidence) = &documents.maybe_flash_evidence {
        validate_flash_evidence(&mut validation_errors, documents, flash_evidence);
    }

    ReleaseEvidenceReport { validation_errors }
}

fn validate_manifest_identity(
    validation_errors: &mut Vec<String>,
    documents: &ReleaseEvidenceDocuments,
) {
    let manifest = &documents.manifest;
    if manifest.schema_version != 3 {
        validation_errors.push("package manifest schema_version must be 3".to_owned());
        return;
    }
    let provenance = match BuildProvenance::new(
        &manifest.semantic_version,
        &manifest.source_commit,
        manifest.build_identity.source_dirty,
        manifest.build_identity.release_tag.as_deref(),
        &manifest.reference_commit,
    ) {
        Ok(provenance) => provenance,
        Err(error) => {
            validation_errors.push(format!("package build identity is invalid: {error}"));
            return;
        }
    };
    let identity = provenance.build_identity();
    if manifest.build_identity.label != identity.build_label()
        || manifest.build_identity.channel != identity.build_channel().as_str()
    {
        validation_errors.push("package build identity fields are contradictory".to_owned());
    }
    if identity.source_dirty() {
        validation_errors.push("dirty package cannot qualify release evidence".to_owned());
    }
    let valid_app_hash = manifest.app_elf_sha256.len() == 64
        && manifest
            .app_elf_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && manifest.app_elf_sha256.bytes().any(|byte| byte != b'0');
    if !valid_app_hash {
        validation_errors
            .push("package app_elf_sha256 must be a nonzero lowercase SHA-256".to_owned());
    }
}

pub(crate) fn render_release_evidence_report(
    documents: &ReleaseEvidenceDocuments,
    report: &ReleaseEvidenceReport,
) -> String {
    if report.passed() {
        let redaction_status = if documents
            .maybe_redaction_review
            .as_deref()
            .is_some_and(redaction_review_passed)
        {
            "passed"
        } else {
            "not-required"
        };

        return format!(
            "release_evidence_status: passed\nsource_commit: {}\nreference_commit: {}\nevidence_root: {}\nredaction_status: {}\n",
            documents.manifest.source_commit,
            documents.manifest.reference_commit,
            documents.evidence_root,
            redaction_status
        );
    }

    let mut output = String::from("release_evidence_status: failed\nvalidation_errors:\n");
    for error in &report.validation_errors {
        output.push_str("- ");
        output.push_str(error);
        output.push('\n');
    }
    output
}

fn validate_current_commit(
    validation_errors: &mut Vec<String>,
    documents: &ReleaseEvidenceDocuments,
) {
    if documents.current_git_head == documents.manifest.source_commit {
        return;
    }

    if documents.allow_post_source_evidence_commits {
        validate_post_source_evidence_commits(validation_errors, documents);
        return;
    }

    validation_errors.push("current git HEAD does not match package source_commit".to_owned());
}

fn validate_post_source_evidence_commits(
    validation_errors: &mut Vec<String>,
    documents: &ReleaseEvidenceDocuments,
) {
    if !documents.source_commit_is_ancestor_of_head {
        validation_errors
            .push("package source_commit is not an ancestor of current git HEAD".to_owned());
        return;
    }

    let disallowed_paths = documents
        .post_source_changed_paths
        .iter()
        .filter(|path| !is_allowed_post_source_evidence_path(path))
        .map(|path| path.as_str())
        .collect::<Vec<_>>();

    if disallowed_paths.is_empty() {
        return;
    }

    validation_errors.push(format!(
        "post-source commits include non-evidence path(s): {}",
        disallowed_paths.join(", ")
    ));
}

fn is_allowed_post_source_evidence_path(path: &Utf8Path) -> bool {
    let Some(normalized_path) = normalize_path(path) else {
        return false;
    };

    if normalized_path.starts_with('/') {
        return false;
    }

    matches!(
        normalized_path.as_str(),
        ".planning/REQUIREMENTS.md"
            | ".planning/ROADMAP.md"
            | ".planning/STATE.md"
            | "docs/parity/checklist.md"
            | "docs/release/license-inventory.md"
            | "docs/release/provenance-manifest.md"
            | "docs/release/ultra-205.md"
    ) || normalized_path
        .starts_with(".planning/phases/16-current-commit-release-evidence-completion/")
        || normalized_path
            == "docs/parity/evidence/phase-16-current-commit-release-evidence-completion.md"
        || normalized_path.starts_with(
            "docs/parity/evidence/phase-16-current-commit-release-evidence-completion/",
        )
}

fn validate_redaction_review(
    validation_errors: &mut Vec<String>,
    documents: &ReleaseEvidenceDocuments,
    require_redaction_passed: bool,
) {
    if !require_redaction_passed {
        return;
    }

    let Some(review) = documents.maybe_redaction_review.as_deref() else {
        validation_errors.push("redaction review is missing".to_owned());
        return;
    };

    if redaction_review_passed(review) {
        return;
    }

    validation_errors.push("redaction review has not passed".to_owned());
}

fn validate_flash_evidence(
    validation_errors: &mut Vec<String>,
    documents: &ReleaseEvidenceDocuments,
    flash_evidence: &ReleaseEvidenceFlashEvidence,
) {
    if flash_evidence.command_kind != "flash-monitor" {
        validation_errors.push("flash evidence command_kind must be flash-monitor".to_owned());
    }

    if flash_evidence.board != "205" {
        validation_errors.push("flash evidence board must be 205".to_owned());
    }

    if !flash_evidence.trusted_output {
        validation_errors.push("flash evidence is not trusted".to_owned());
    }

    if flash_evidence.firmware_commit != documents.manifest.source_commit {
        validation_errors.push("flash evidence firmware_commit mismatch".to_owned());
    }

    if flash_evidence.reference_commit != documents.manifest.reference_commit {
        validation_errors.push("flash evidence reference_commit mismatch".to_owned());
    }

    if flash_evidence.observed_reference_commit != documents.manifest.reference_commit {
        validation_errors.push("observed reference commit mismatch".to_owned());
    }

    if !observed_firmware_commit_matches(
        &documents.manifest.source_commit,
        &flash_evidence.observed_firmware_commit,
    ) {
        validation_errors.push("observed firmware commit mismatch".to_owned());
    }

    if flash_evidence.manifest_path.file_name() != Some(PACKAGE_MANIFEST_FILE_NAME) {
        validation_errors.push(format!(
            "flash evidence manifest_path must end with {PACKAGE_MANIFEST_FILE_NAME}"
        ));
    }

    if !flash_evidence_references_factory_image(flash_evidence) {
        validation_errors.push(format!(
            "flash evidence must reference {FACTORY_IMAGE_FILE_NAME}"
        ));
    }

    validate_evidence_path(
        validation_errors,
        &documents.evidence_root,
        &flash_evidence.log_path,
    );
    validate_evidence_path(
        validation_errors,
        &documents.evidence_root,
        &flash_evidence.monitor_log_path,
    );

    if let Some(flash_evidence_json_path) = &documents.maybe_flash_evidence_json_path {
        validate_evidence_path(
            validation_errors,
            &documents.evidence_root,
            flash_evidence_json_path,
        );
    }
}

fn observed_firmware_commit_matches(source_commit: &str, observed_commit: &str) -> bool {
    observed_commit == source_commit
}

fn flash_evidence_references_factory_image(flash_evidence: &ReleaseEvidenceFlashEvidence) -> bool {
    flash_evidence
        .flash_image_path
        .as_ref()
        .is_some_and(|path| path.as_str().contains(FACTORY_IMAGE_FILE_NAME))
        || flash_evidence
            .flash_command
            .as_deref()
            .is_some_and(|command| command.contains(FACTORY_IMAGE_FILE_NAME))
}

fn validate_evidence_path(
    validation_errors: &mut Vec<String>,
    evidence_root: &Utf8Path,
    evidence_path: &Utf8Path,
) {
    if path_is_under_root(evidence_path, evidence_root) {
        return;
    }

    validation_errors.push("evidence path is outside Phase 16 root".to_owned());
}

fn path_is_under_root(evidence_path: &Utf8Path, evidence_root: &Utf8Path) -> bool {
    let Some(normalized_path) = normalize_path(evidence_path) else {
        return false;
    };
    let Some(normalized_root) = normalize_path(evidence_root) else {
        return false;
    };

    normalized_path == normalized_root
        || normalized_path
            .strip_prefix(&normalized_root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn normalize_path(path: &Utf8Path) -> Option<String> {
    let raw = path.as_str().replace('\\', "/");
    let is_absolute = raw.starts_with('/');
    let mut parts = Vec::new();

    for part in raw.split('/') {
        match part {
            "" | "." => {}
            ".." => return None,
            segment => parts.push(segment),
        }
    }

    let normalized = parts.join("/");
    if is_absolute {
        return Some(format!("/{normalized}"));
    }

    Some(normalized)
}

fn redaction_review_passed(review: &str) -> bool {
    review.contains("redaction_status: passed")
}

#[cfg(test)]
mod tests;
