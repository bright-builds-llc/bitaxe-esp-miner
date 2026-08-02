use crate::*;

mod support;
mod validation;

pub(crate) use support::*;
pub(crate) use validation::*;

#[derive(Debug, Serialize)]
pub(crate) struct ParityReport {
    pub(crate) reference_commit: String,
    pub(crate) rows: Vec<ChecklistRow>,
    pub(crate) validation_errors: Vec<ValidationError>,
}

impl ParityReport {
    #[cfg(test)]
    pub(crate) fn new(reference_commit: String, rows: Vec<ChecklistRow>) -> Self {
        Self::new_with_phase30_artifact(
            reference_commit,
            rows,
            &Phase30PromotionArtifactState::Unavailable(
                "structured Phase 30 evidence artifact was not loaded".to_owned(),
            ),
        )
    }

    pub(crate) fn new_with_phase30_artifact(
        reference_commit: String,
        rows: Vec<ChecklistRow>,
        phase30_artifact: &Phase30PromotionArtifactState,
    ) -> Self {
        let validation_errors = validate_rows_with_phase30_artifact(&rows, phase30_artifact);

        Self {
            reference_commit,
            rows,
            validation_errors,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChecklistRow {
    pub(crate) id: String,
    pub(crate) surface: String,
    pub(crate) reference_breadcrumb: String,
    pub(crate) rust_owned_target: String,
    #[serde(skip)]
    pub(crate) rust_owned_target_markdown: String,
    pub(crate) status: String,
    pub(crate) evidence: String,
    pub(crate) notes: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ValidationError {
    pub(crate) id: String,
    pub(crate) message: String,
}

#[derive(Debug)]
pub(crate) struct Phase30PromotionArtifact {
    pub(crate) fields: BTreeMap<String, String>,
}

impl Phase30PromotionArtifact {
    pub(crate) fn maybe_field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub(crate) fn has_exact_field(&self, key: &str, value: &str) -> bool {
        self.maybe_field(key) == Some(value)
    }
}

#[derive(Debug)]
pub(crate) enum Phase30PromotionArtifactState {
    Available(Phase30PromotionArtifact),
    Unavailable(String),
    Malformed(String),
}

pub(crate) fn parse_phase30_promotion_artifact(
    document: &str,
) -> std::result::Result<Phase30PromotionArtifact, String> {
    let mut fields = BTreeMap::new();

    for line in document.lines() {
        let trimmed = line.trim();
        let Some((key, value)) = trimmed.split_once(": ") else {
            continue;
        };
        if key.is_empty()
            || !key
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
        {
            continue;
        }
        if fields.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("duplicate structured Phase 30 field {key}"));
        }
    }

    let artifact = Phase30PromotionArtifact { fields };
    validate_phase30_artifact_closed_fields(&artifact)?;
    Ok(artifact)
}

pub(crate) fn validate_phase30_artifact_closed_fields(
    artifact: &Phase30PromotionArtifact,
) -> std::result::Result<(), String> {
    require_phase30_field_value(
        artifact,
        "phase30_disposition",
        &["no_promotion_no_eligible_evidence", "promoted"],
    )?;
    require_phase30_field_value(artifact, "new_evidence_input", &["none", "explicit"])?;
    require_phase30_field_value(artifact, "archived_lineage_verification", &["gaps_found"])?;
    require_phase30_field_value(
        artifact,
        "eligible_share_outcome",
        &["none", "accepted", "rejected"],
    )?;
    require_phase30_field_value(artifact, "hardware_accessed", &["false", "true"])?;
    require_phase30_field_value(artifact, "credentials_accessed", &["false", "true"])?;
    require_phase30_field_value(artifact, "raw_artifacts_committed", &["no"])?;

    match artifact.maybe_field("phase30_disposition") {
        Some("no_promotion_no_eligible_evidence") => {
            require_phase30_field_value(artifact, "new_evidence_input", &["none"])?;
            require_phase30_field_value(artifact, "eligible_share_outcome", &["none"])?;
            require_phase30_field_value(artifact, "hardware_accessed", &["false"])?;
            require_phase30_field_value(artifact, "credentials_accessed", &["false"])?;
        }
        Some("promoted") => {
            require_phase30_field_value(artifact, "new_evidence_input", &["explicit"])?;
            require_phase30_field_value(
                artifact,
                "eligible_share_outcome",
                &["accepted", "rejected"],
            )?;
            require_phase30_field_value(artifact, "hardware_accessed", &["true"])?;
            for (key, value) in [
                ("current_source_gate", "passed"),
                ("detector_gate", "passed"),
                ("same_chain_gate", "passed"),
                ("provenance_gate", "passed"),
                ("redaction_status", "passed"),
            ] {
                require_phase30_field_value(artifact, key, &[value])?;
            }
        }
        _ => return Err("invalid phase30_disposition".to_owned()),
    }

    Ok(())
}

pub(crate) fn require_phase30_field_value(
    artifact: &Phase30PromotionArtifact,
    key: &str,
    allowed_values: &[&str],
) -> std::result::Result<(), String> {
    let Some(value) = artifact.maybe_field(key) else {
        return Err(format!("missing structured Phase 30 field {key}"));
    };
    if allowed_values.contains(&value) {
        return Ok(());
    }

    Err(format!(
        "invalid structured Phase 30 value for {key}: expected {}",
        allowed_values.join(" or ")
    ))
}

pub(crate) trait ReportEnvironment {
    fn run_reference_guard(&self) -> Result<()>;
    fn read_checklist(&self, path: &Utf8Path) -> Result<String>;
    fn read_phase30_promotion_artifact(&self, path: &Utf8Path) -> Result<String>;
    fn reference_commit(&self) -> Result<String>;
    fn validate_checklist_targets(&self, _rows: &[ChecklistRow]) -> Vec<ValidationError> {
        Vec::new()
    }
    fn validate_reference_inventory(
        &self,
        _rows: &[ChecklistRow],
        _reference_commit: &str,
    ) -> Vec<ValidationError> {
        Vec::new()
    }
    fn validate_progress_artifacts(
        &self,
        _checklist: &str,
        _rows: &[ChecklistRow],
    ) -> Vec<ValidationError> {
        Vec::new()
    }
}

pub(crate) fn run_report(
    environment_request: &ReportRequest,
    environment: &impl ReportEnvironment,
) -> Result<String> {
    if let Err(error) = environment.run_reference_guard() {
        bail!("reference guard blocked parity report generation: {error:#}");
    }

    let checklist = environment
        .read_checklist(&environment_request.checklist)
        .with_context(|| format!("failed to load {}", environment_request.checklist))?;
    let rows = parse_checklist(&checklist)?;
    let phase30_artifact = load_phase30_promotion_artifact(&rows, environment);
    let reference_commit = environment.reference_commit()?;
    let mut report =
        ParityReport::new_with_phase30_artifact(reference_commit, rows, &phase30_artifact);
    report
        .validation_errors
        .extend(environment.validate_checklist_targets(&report.rows));
    report
        .validation_errors
        .extend(environment.validate_reference_inventory(&report.rows, &report.reference_commit));
    report
        .validation_errors
        .extend(environment.validate_progress_artifacts(&checklist, &report.rows));

    if environment_request.fail_on_invalid_verified && !report.validation_errors.is_empty() {
        bail!(
            "invalid parity checklist:\n{}",
            format_validation_errors(&report.validation_errors)
        );
    }

    render_report(&report, environment_request.format)
}

pub(crate) fn load_phase30_promotion_artifact(
    rows: &[ChecklistRow],
    environment: &impl ReportEnvironment,
) -> Phase30PromotionArtifactState {
    let has_verified_phase30_row = rows
        .iter()
        .any(|row| is_phase30_promotion_row(row) && normalize(&row.status) == "verified");
    if !has_verified_phase30_row {
        return Phase30PromotionArtifactState::Unavailable(
            "no verified Phase 30 row requested promotion admission".to_owned(),
        );
    }

    let document = match environment
        .read_phase30_promotion_artifact(Utf8Path::new(DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH))
    {
        Ok(document) => document,
        Err(error) => {
            return Phase30PromotionArtifactState::Unavailable(format!(
                "structured Phase 30 evidence artifact is missing or unreadable: {error:#}"
            ));
        }
    };

    match parse_phase30_promotion_artifact(&document) {
        Ok(artifact) => Phase30PromotionArtifactState::Available(artifact),
        Err(error) => Phase30PromotionArtifactState::Malformed(error),
    }
}

pub(crate) fn parse_checklist(checklist: &str) -> Result<Vec<ChecklistRow>> {
    let mut rows = Vec::new();

    for (line_index, line) in checklist.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }

        let raw_cells: Vec<String> = trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().to_owned())
            .collect();
        let cells = raw_cells
            .iter()
            .map(|cell| clean_markdown_cell(cell))
            .collect::<Vec<_>>();

        if is_header_or_separator(&cells) {
            continue;
        }

        if cells.len() != 7 {
            bail!(
                "invalid checklist row at line {}: expected 7 columns, found {}",
                line_index + 1,
                cells.len()
            );
        }

        rows.push(ChecklistRow {
            id: cells[0].clone(),
            surface: cells[1].clone(),
            reference_breadcrumb: cells[2].clone(),
            rust_owned_target: cells[3].clone(),
            rust_owned_target_markdown: raw_cells[3].clone(),
            status: cells[4].clone(),
            evidence: cells[5].clone(),
            notes: cells[6].clone(),
        });
    }

    Ok(rows)
}

#[cfg(test)]
pub(crate) fn validate_rows(rows: &[ChecklistRow]) -> Vec<ValidationError> {
    validate_rows_with_phase30_artifact(
        rows,
        &Phase30PromotionArtifactState::Unavailable(
            "structured Phase 30 evidence artifact was not loaded".to_owned(),
        ),
    )
}

pub(crate) fn validate_rows_with_phase30_artifact(
    rows: &[ChecklistRow],
    phase30_artifact: &Phase30PromotionArtifactState,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if let Err(message) = v12_admission::validate_closed_phase31_contract() {
        errors.push(ValidationError {
            id: "PHASE-31".to_owned(),
            message,
        });
    }

    for row in rows {
        if normalize(&row.status) != "verified" {
            continue;
        }

        if normalize(&row.evidence) == "pending" {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "verified rows require non-pending evidence".to_owned(),
            });
        }

        if is_safety_critical(row) && !has_hardware_evidence(row) {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "safety-critical verified rows require hardware-smoke or hardware-regression evidence".to_owned(),
            });
        }

        if is_active_safety_control(row) && !has_evidence_token(row, "hardware-regression") {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "active safety-control verified row requires hardware-regression evidence"
                    .to_owned(),
            });
        }

        errors.extend(validate_live_asic_mining_verified_row(row));
        errors.extend(validate_release_ota_verified_row(row));
        errors.extend(validate_deferred_scope_verified_row(row));
        errors.extend(validate_phase26_telemetry_verified_row(row));
        errors.extend(validate_phase28_hardware_promotion_row(
            row,
            phase30_artifact,
        ));
        errors.extend(validate_phase30_promotion_row(row, phase30_artifact));
    }

    errors
}

pub(crate) fn render_report(report: &ParityReport, format: ReportFormat) -> Result<String> {
    match format {
        ReportFormat::Json => {
            serde_json::to_string_pretty(report).context("failed to serialize parity report")
        }
        ReportFormat::Text => Ok(render_text_report(report)),
    }
}

pub(crate) fn render_text_report(report: &ParityReport) -> String {
    let mut output = String::new();
    output.push_str(&format!("reference_commit: {}\n", report.reference_commit));
    output.push_str("rows:\n");

    for row in &report.rows {
        output.push_str(&format!(
            "- {} | status={} | evidence={}\n  reference_breadcrumb: {}\n  rust_owned_target: {}\n  notes: {}\n",
            row.id,
            row.status,
            row.evidence,
            row.reference_breadcrumb,
            row.rust_owned_target,
            row.notes
        ));
    }

    if report.validation_errors.is_empty() {
        output.push_str("validation_errors: none\n");
    } else {
        output.push_str("validation_errors:\n");
        output.push_str(&format_validation_errors(&report.validation_errors));
    }

    output
}

pub(crate) fn format_validation_errors(errors: &[ValidationError]) -> String {
    let mut output = String::new();

    for error in errors {
        output.push_str(&format!("- {}: {}\n", error.id, error.message));
    }

    output
}

pub(crate) fn is_header_or_separator(cells: &[String]) -> bool {
    let Some(first_cell) = cells.first() else {
        return false;
    };

    first_cell == "ID" || cells.iter().all(|cell| cell.chars().all(is_separator_char))
}

pub(crate) fn is_separator_char(character: char) -> bool {
    matches!(character, '-' | ':' | ' ')
}
