use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Command as ProcessCommand;

use crate::*;

use super::{read_authoritative_rows, rounded_ratio, validate_audit_path, Progress};

pub(crate) const PROGRESS_HISTORY_PATH: &str = "docs/parity/progress.jsonl";
const README_PATH: &str = "README.md";
const HISTORY_SCHEMA: &str = "parity-progress-v1";
const README_BEGIN: &str = "<!-- parity-progress:begin -->";
const README_END: &str = "<!-- parity-progress:end -->";
const README_BADGES_END: &str = "<!-- bright-builds-rules-readme-badges:end -->";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressRecord {
    schema_version: String,
    recorded_at: String,
    source_commit: String,
    reference_commit: String,
    checklist_sha256: String,
    previous_record_sha256: Option<String>,
    status_counts: BTreeMap<String, u64>,
    total: u64,
    active_total: u64,
    verified_total: u64,
    completion_basis_points: u64,
    selected_row: Option<String>,
    plan_path: Option<String>,
}

struct HistoryEntry {
    record: ProgressRecord,
    line: String,
}

pub(crate) fn run_sync_progress_command(
    args: &SyncProgressArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    require_full_commit(&args.source_commit)?;
    if !environment.source_commit_is_ancestor_of_head(&args.source_commit)? {
        bail!("progress source commit is not an ancestor of HEAD");
    }
    require_selection_pair(
        args.maybe_selected_row.as_deref(),
        args.maybe_plan.as_deref(),
    )?;

    let (checklist, rows) = read_authoritative_rows(environment)?;
    let progress = Progress::from_rows(&rows)?;
    validate_selection(environment, args, &rows)?;

    let history_path = environment.workspace_dir.join(PROGRESS_HISTORY_PATH);
    let mut history = load_history(&history_path, false)?;
    let checklist_sha256 = phase35_evidence::sha256_hex(checklist.as_bytes());
    if history
        .iter()
        .take(history.len().saturating_sub(1))
        .any(|entry| entry.record.checklist_sha256 == checklist_sha256)
    {
        bail!("progress history would return to an earlier checklist digest");
    }

    let appended = history
        .last()
        .is_none_or(|entry| entry.record.checklist_sha256 != checklist_sha256);
    if appended {
        let record = new_record(
            args,
            environment,
            &progress,
            checklist_sha256,
            history.last(),
        )?;
        let line = serde_json::to_string(&record).context("failed to serialize progress record")?;
        append_history(&history_path, &line)?;
        history.push(HistoryEntry { record, line });
    }

    validate_history(&history)?;
    sync_readme(&environment.workspace_dir, &progress)?;
    validate_progress_artifacts(&environment.workspace_dir, &checklist, &rows)
        .map_err(anyhow::Error::msg)?;

    Ok(format!(
        "progress_appended={appended} verified={} active={} completion={}",
        progress.verified_total,
        progress.active_total,
        progress.display_percent()
    ))
}

fn validate_selection(
    environment: &LocalEnvironment,
    args: &SyncProgressArgs,
    rows: &[ChecklistRow],
) -> Result<()> {
    if let Some(row_id) = &args.maybe_selected_row {
        if !rows.iter().any(|row| row.id == *row_id) {
            bail!("selected parity row `{row_id}` is absent from the checklist");
        }
    }
    if let Some(plan) = &args.maybe_plan {
        validate_audit_path(plan, "PLAN.md")?;
        let plan_path = environment.workspace_dir.join(plan);
        if !plan_path.is_file() {
            bail!("parity plan `{plan}` does not exist");
        }
    }
    Ok(())
}

fn new_record(
    args: &SyncProgressArgs,
    environment: &LocalEnvironment,
    progress: &Progress,
    checklist_sha256: String,
    maybe_previous: Option<&HistoryEntry>,
) -> Result<ProgressRecord> {
    Ok(ProgressRecord {
        schema_version: HISTORY_SCHEMA.to_owned(),
        recorded_at: git_commit_timestamp(environment, &args.source_commit)?,
        source_commit: args.source_commit.clone(),
        reference_commit: environment.reference_commit()?,
        checklist_sha256,
        previous_record_sha256: maybe_previous
            .map(|entry| phase35_evidence::sha256_hex(entry.line.as_bytes())),
        status_counts: progress.status_counts.clone(),
        total: progress.total,
        active_total: progress.active_total,
        verified_total: progress.verified_total,
        completion_basis_points: progress.completion_basis_points,
        selected_row: args.maybe_selected_row.clone(),
        plan_path: args.maybe_plan.as_ref().map(ToString::to_string),
    })
}

fn load_history(path: &Utf8Path, required: bool) -> Result<Vec<HistoryEntry>> {
    let document = match fs::read_to_string(path.as_std_path()) {
        Ok(document) => document,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !required => {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error).with_context(|| format!("failed to read {path}")),
    };
    if document.is_empty() {
        bail!("progress history is empty");
    }
    if !document.ends_with('\n') {
        bail!("progress history must end with a newline");
    }
    let mut history = Vec::new();
    for (index, line) in document.lines().enumerate() {
        if line.trim().is_empty() {
            bail!("progress history contains a blank line at {}", index + 1);
        }
        let record = serde_json::from_str(line)
            .with_context(|| format!("invalid progress record at line {}", index + 1))?;
        history.push(HistoryEntry {
            record,
            line: line.to_owned(),
        });
    }
    validate_history(&history)?;
    Ok(history)
}

fn validate_history(history: &[HistoryEntry]) -> Result<()> {
    if history.is_empty() {
        bail!("progress history contains no records");
    }
    let mut checklist_digests = BTreeSet::new();
    for (index, entry) in history.iter().enumerate() {
        let record = &entry.record;
        if record.schema_version != HISTORY_SCHEMA {
            bail!("unsupported progress history schema at line {}", index + 1);
        }
        require_full_commit(&record.source_commit)?;
        require_full_commit(&record.reference_commit)?;
        require_digest(&record.checklist_sha256, "checklist")?;
        let expected_previous = index
            .checked_sub(1)
            .map(|previous| phase35_evidence::sha256_hex(history[previous].line.as_bytes()));
        if record.previous_record_sha256 != expected_previous {
            bail!(
                "progress history predecessor mismatch at line {}",
                index + 1
            );
        }
        require_selection_pair(
            record.selected_row.as_deref(),
            record.plan_path.as_deref().map(Utf8Path::new),
        )?;
        if let Some(plan) = record.plan_path.as_deref() {
            validate_audit_path(Utf8Path::new(plan), "PLAN.md")?;
        }
        if !checklist_digests.insert(record.checklist_sha256.as_str()) {
            bail!("progress history contains a duplicate checklist digest");
        }
        validate_record_math(record)?;
    }
    Ok(())
}

fn validate_record_math(record: &ProgressRecord) -> Result<()> {
    let allowed = [
        "not-started",
        "in-progress",
        "implemented",
        "verified",
        "deferred",
    ];
    if record
        .status_counts
        .keys()
        .any(|status| !allowed.contains(&status.as_str()))
    {
        bail!("progress record contains an unknown status");
    }
    let total = record.status_counts.values().sum::<u64>();
    let deferred = record.status_counts.get("deferred").copied().unwrap_or(0);
    let active_total = total
        .checked_sub(deferred)
        .context("progress record deferred count exceeded total")?;
    let verified_total = record.status_counts.get("verified").copied().unwrap_or(0);
    let basis_points = rounded_ratio(verified_total, active_total, 10_000)?;
    if record.total != total
        || record.active_total != active_total
        || record.verified_total != verified_total
        || record.completion_basis_points != basis_points
    {
        bail!("progress record arithmetic mismatch");
    }
    Ok(())
}

fn append_history(path: &Utf8Path, line: &str) -> Result<()> {
    let parent = path
        .parent()
        .context("progress history path has no parent")?;
    fs::create_dir_all(parent.as_std_path())
        .with_context(|| format!("failed to create {parent}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_std_path())
        .with_context(|| format!("failed to append {path}"))?;
    writeln!(file, "{line}").with_context(|| format!("failed to append {path}"))
}

fn git_commit_timestamp(environment: &LocalEnvironment, commit: &str) -> Result<String> {
    let output = ProcessCommand::new("git")
        .args([
            "-C",
            environment.workspace_dir.as_str(),
            "show",
            "-s",
            "--format=%cI",
            commit,
        ])
        .output()
        .context("failed to read progress source commit timestamp")?;
    if !output.status.success() {
        bail!(
            "failed to read progress source commit timestamp: {}",
            command_stderr_or_status(&output)
        );
    }
    let timestamp = String::from_utf8(output.stdout)
        .context("progress source commit timestamp was not valid UTF-8")?;
    let trimmed = timestamp.trim();
    if trimmed.is_empty() {
        bail!("progress source commit timestamp was empty");
    }
    Ok(trimmed.to_owned())
}

fn require_full_commit(value: &str) -> Result<()> {
    if value.len() != 40 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        bail!("expected a full 40-character hexadecimal commit ID");
    }
    Ok(())
}

fn require_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        bail!("invalid {label} SHA-256 digest");
    }
    Ok(())
}

fn require_selection_pair(
    maybe_selected_row: Option<&str>,
    maybe_plan: Option<&Utf8Path>,
) -> Result<()> {
    if maybe_selected_row.is_some() != maybe_plan.is_some() {
        bail!("selected row and plan path must be supplied together");
    }
    Ok(())
}

fn sync_readme(workspace: &Utf8Path, progress: &Progress) -> Result<()> {
    let path = workspace.join(README_PATH);
    let document =
        fs::read_to_string(path.as_std_path()).with_context(|| format!("failed to read {path}"))?;
    let block = render_readme_block(progress);
    let updated = replace_or_insert_readme_block(&document, &block)?;
    if updated != document {
        fs::write(path.as_std_path(), updated)
            .with_context(|| format!("failed to update {path}"))?;
    }
    Ok(())
}

fn render_readme_block(progress: &Progress) -> String {
    format!(
        "{README_BEGIN}\n\n## Parity progress\n\n**Parity: {} of {} active checklist items verified ({}).**\n\nSee the [parity checklist](docs/parity/checklist.md) and [progress history](docs/parity/progress.jsonl).\n\n{README_END}",
        progress.verified_total,
        progress.active_total,
        progress.display_percent()
    )
}

fn replace_or_insert_readme_block(document: &str, block: &str) -> Result<String> {
    match (document.find(README_BEGIN), document.find(README_END)) {
        (Some(begin), Some(end)) => {
            if document[begin + README_BEGIN.len()..].contains(README_BEGIN)
                || document[end + README_END.len()..].contains(README_END)
                || end < begin
            {
                bail!("README parity progress markers are malformed or duplicated");
            }
            let end = end + README_END.len();
            Ok(format!(
                "{}{}{}",
                &document[..begin],
                block,
                &document[end..]
            ))
        }
        (None, None) => {
            let marker_end = document
                .find(README_BADGES_END)
                .context("README managed badge end marker is missing")?
                + README_BADGES_END.len();
            Ok(format!(
                "{}\n\n{}{}",
                &document[..marker_end],
                block,
                &document[marker_end..]
            ))
        }
        _ => bail!("README parity progress markers are incomplete"),
    }
}

pub(crate) fn validate_progress_artifacts(
    workspace: &Utf8Path,
    checklist: &str,
    rows: &[ChecklistRow],
) -> std::result::Result<(), String> {
    let history_path = workspace.join(PROGRESS_HISTORY_PATH);
    let history = load_history(&history_path, true).map_err(|error| error.to_string())?;
    let latest = history
        .last()
        .ok_or_else(|| "progress history contains no records".to_owned())?;
    let progress = Progress::from_rows(rows).map_err(|error| error.to_string())?;
    let checklist_sha256 = phase35_evidence::sha256_hex(checklist.as_bytes());
    if latest.record.checklist_sha256 != checklist_sha256
        || latest.record.status_counts != progress.status_counts
        || latest.record.total != progress.total
        || latest.record.active_total != progress.active_total
        || latest.record.verified_total != progress.verified_total
        || latest.record.completion_basis_points != progress.completion_basis_points
    {
        return Err("latest progress record does not match the active checklist".to_owned());
    }

    let readme = fs::read_to_string(workspace.join(README_PATH).as_std_path())
        .map_err(|error| format!("failed to read README parity progress: {error}"))?;
    let expected = render_readme_block(&progress);
    let begin = readme
        .find(README_BEGIN)
        .ok_or_else(|| "README parity progress block is missing".to_owned())?;
    let end = readme[begin..]
        .find(README_END)
        .map(|offset| begin + offset + README_END.len())
        .ok_or_else(|| "README parity progress end marker is missing".to_owned())?;
    if readme[begin..end] != expected {
        return Err("README parity progress block does not match progress history".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
