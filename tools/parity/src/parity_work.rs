use std::collections::BTreeMap;
use std::fs;

use crate::*;

mod closure;
mod history;
pub(crate) use history::{run_sync_progress_command, validate_progress_artifacts};

pub(crate) const WORK_PLANS_ROOT: &str = "docs/parity/work-plans";
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Progress {
    pub(crate) status_counts: BTreeMap<String, u64>,
    pub(crate) total: u64,
    pub(crate) active_total: u64,
    pub(crate) verified_total: u64,
    pub(crate) completion_basis_points: u64,
    pub(crate) completion_tenths_percent: u64,
}

impl Progress {
    pub(crate) fn from_rows(rows: &[ChecklistRow]) -> Result<Self> {
        let mut status_counts = BTreeMap::new();
        for row in rows {
            let status = normalize(&row.status);
            if !matches!(
                status.as_str(),
                "not-started" | "in-progress" | "implemented" | "verified" | "deferred"
            ) {
                bail!("unknown parity status `{}` for {}", row.status, row.id);
            }
            *status_counts.entry(status).or_insert(0) += 1;
        }

        let total = rows.len() as u64;
        let deferred = status_counts.get("deferred").copied().unwrap_or(0);
        let active_total = total
            .checked_sub(deferred)
            .context("deferred parity count exceeded total")?;
        if active_total == 0 {
            bail!("parity completion is undefined when every row is deferred");
        }
        let verified_total = status_counts.get("verified").copied().unwrap_or(0);
        let completion_basis_points = rounded_ratio(verified_total, active_total, 10_000)?;
        let completion_tenths_percent = rounded_ratio(verified_total, active_total, 1_000)?;

        Ok(Self {
            status_counts,
            total,
            active_total,
            verified_total,
            completion_basis_points,
            completion_tenths_percent,
        })
    }

    pub(super) fn display_percent(&self) -> String {
        format!(
            "{}.{}%",
            self.completion_tenths_percent / 10,
            self.completion_tenths_percent % 10
        )
    }
}

pub(super) fn rounded_ratio(numerator: u64, denominator: u64, scale: u64) -> Result<u64> {
    if denominator == 0 {
        bail!("cannot calculate parity completion with a zero denominator");
    }
    let scaled = numerator
        .checked_mul(scale)
        .context("parity completion calculation overflowed")?;
    let rounding = denominator / 2;
    scaled
        .checked_add(rounding)
        .context("parity completion rounding overflowed")
        .map(|value| value / denominator)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct Candidate {
    pub(crate) row_id: String,
    pub(crate) status: String,
    pub(crate) surface: String,
    pub(crate) checklist_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct OpenPlan {
    pub(crate) row_id: String,
    pub(crate) plan_path: String,
}

#[derive(Debug)]
struct OpenPlanDocument {
    open_plan: OpenPlan,
    document: String,
    terminal_closed: bool,
}
#[derive(Debug, Serialize)]
struct NextItemReport {
    maybe_open_plan: Option<OpenPlan>,
    candidates: Vec<Candidate>,
}

pub(crate) fn run_progress_command(
    args: &ProgressArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let (checklist, rows) = read_authoritative_rows(environment)?;
    let progress = Progress::from_rows(&rows)?;
    validate_progress_artifacts(&environment.workspace_dir, &checklist, &rows)
        .map_err(anyhow::Error::msg)?;
    render_progress(&progress, args.format)
}

pub(crate) fn run_next_item_command(
    args: &NextItemArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let (_, rows) = read_authoritative_rows(environment)?;
    let maybe_open_plan = find_open_plan(&environment.workspace_dir, &rows)?;
    let candidates = if maybe_open_plan.is_some() {
        Vec::new()
    } else {
        ranked_candidates(&rows)
    };
    let report = NextItemReport {
        maybe_open_plan,
        candidates,
    };
    match args.format {
        ReportFormat::Json => serde_json::to_string_pretty(&report)
            .context("failed to serialize next parity item report"),
        ReportFormat::Text => Ok(render_next_item_text(&report)),
    }
}

pub(super) fn read_authoritative_rows(
    environment: &LocalEnvironment,
) -> Result<(String, Vec<ChecklistRow>)> {
    environment.run_reference_guard()?;
    let checklist = environment
        .read_checklist(Utf8Path::new(PHASE35_CHECKLIST_PATH))
        .context("failed to read authoritative parity checklist")?;
    let rows = parse_checklist(&checklist)?;
    Ok((checklist, rows))
}

fn render_progress(progress: &Progress, format: ReportFormat) -> Result<String> {
    match format {
        ReportFormat::Json => {
            serde_json::to_string_pretty(progress).context("failed to serialize parity progress")
        }
        ReportFormat::Text => Ok(format!(
            "verified={} active={} total={} deferred={} completion={}",
            progress.verified_total,
            progress.active_total,
            progress.total,
            progress.status_counts.get("deferred").copied().unwrap_or(0),
            progress.display_percent()
        )),
    }
}

fn ranked_candidates(rows: &[ChecklistRow]) -> Vec<Candidate> {
    let mut candidates = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            candidate_rank(&row.status).map(|rank| {
                (
                    rank,
                    Candidate {
                        row_id: row.id.clone(),
                        status: normalize(&row.status),
                        surface: row.surface.clone(),
                        checklist_index: index,
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(rank, candidate)| (*rank, candidate.checklist_index));
    candidates
        .into_iter()
        .map(|(_, candidate)| candidate)
        .collect()
}

fn candidate_rank(status: &str) -> Option<u8> {
    match normalize(status).as_str() {
        "implemented" => Some(0),
        "in-progress" => Some(1),
        "not-started" => Some(2),
        _ => None,
    }
}

fn render_next_item_text(report: &NextItemReport) -> String {
    if let Some(open_plan) = &report.maybe_open_plan {
        return format!(
            "resume row={} plan={}",
            open_plan.row_id, open_plan.plan_path
        );
    }
    let mut output = String::from("candidates:\n");
    for candidate in &report.candidates {
        output.push_str(&format!(
            "- {} status={} index={} surface={}\n",
            candidate.row_id, candidate.status, candidate.checklist_index, candidate.surface
        ));
    }
    output
}

fn find_open_plan(workspace: &Utf8Path, rows: &[ChecklistRow]) -> Result<Option<OpenPlan>> {
    let root = workspace.join(WORK_PLANS_ROOT);
    if !root.exists() {
        return Ok(None);
    }
    let mut directories = fs::read_dir(root.as_std_path())
        .context("failed to read parity work-plan directory")?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    directories.sort_by_key(|entry| entry.file_name());
    let mut open_plans = Vec::new();
    for entry in directories {
        let path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| anyhow::anyhow!("work-plan path is not valid UTF-8"))?;
        if !path.is_dir() || !path.join("PLAN.md").is_file() {
            continue;
        }
        if closure::result_closes_plan(&path)? {
            continue;
        }
        let document = fs::read_to_string(path.join("PLAN.md").as_std_path())
            .with_context(|| format!("failed to read open parity plan {path}/PLAN.md"))?;
        let (row_id, initial_status) = parse_plan_metadata(&document)?;
        let maybe_row = rows.iter().find(|row| row.id == row_id);
        let Some(row) = maybe_row else {
            bail!("open parity plan references missing row {row_id}");
        };
        let current_status = normalize(&row.status);
        let terminal_closed = closure::closes_plan(&path, &document, &row_id, &initial_status)?;
        if current_status != initial_status {
            require_plan_status_advance(&initial_status, &current_status, &row_id)?;
            continue;
        }
        let relative = path
            .strip_prefix(workspace)
            .context("open parity plan is outside the workspace")?;
        open_plans.push(OpenPlanDocument {
            open_plan: OpenPlan {
                row_id,
                plan_path: format!("{relative}/PLAN.md"),
            },
            document,
            terminal_closed,
        });
    }
    reconcile_open_plans(open_plans)
}

fn reconcile_open_plans(mut open_plans: Vec<OpenPlanDocument>) -> Result<Option<OpenPlan>> {
    open_plans.sort_by(|left, right| left.open_plan.plan_path.cmp(&right.open_plan.plan_path));
    let mut lineages = BTreeMap::<String, Vec<OpenPlanDocument>>::new();
    for candidate in open_plans {
        let row_id = candidate.open_plan.row_id.clone();
        lineages.entry(row_id).or_default().push(candidate);
    }
    let mut active_plans = Vec::new();
    for mut lineage in lineages.into_values() {
        if lineage.iter().all(|candidate| candidate.terminal_closed) {
            continue;
        }
        for pair in lineage.windows(2) {
            let older = &pair[0].open_plan;
            let newer = &pair[1];
            let lineage_reference = format!("`{}`", older.plan_path);
            if !newer.document.contains(&lineage_reference) {
                bail!(
                    "multiple open parity plans for {} lack an explicit continuation lineage",
                    older.row_id
                );
            }
        }
        let latest = lineage.pop().expect("non-empty plan lineage");
        if !latest.terminal_closed {
            active_plans.push(latest.open_plan);
        }
    }
    if active_plans.len() > 1 {
        bail!(
            "multiple open parity plans span rows; close or reconcile them before selecting work"
        );
    }
    Ok(active_plans.pop())
}
fn parse_plan_metadata(document: &str) -> Result<(String, String)> {
    let row_id = parse_plan_metadata_value(document, "- Parity row: `", "parity-row")?;
    let initial_status = normalize(&parse_plan_metadata_value(
        document,
        "- Initial status: `",
        "initial-status",
    )?);
    if plan_status_rank(&initial_status).is_none() {
        bail!("open parity plan has non-actionable initial status {initial_status}");
    }
    Ok((row_id, initial_status))
}

fn parse_plan_metadata_value(document: &str, prefix: &str, label: &str) -> Result<String> {
    for line in document.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix('`'))
        {
            if !value.is_empty() {
                return Ok(value.to_owned());
            }
        }
    }
    bail!("open parity plan is missing {label} metadata")
}

fn require_plan_status_advance(initial: &str, current: &str, row_id: &str) -> Result<()> {
    if current == "deferred" {
        return Ok(());
    }
    let initial_rank = plan_status_rank(initial)
        .with_context(|| format!("open parity plan has invalid initial status {initial}"))?;
    let current_rank = plan_status_rank(current)
        .with_context(|| format!("parity row {row_id} has invalid current status {current}"))?;
    if current_rank <= initial_rank {
        bail!(
            "parity row {row_id} status regressed from plan initial status {initial} to {current}"
        );
    }
    Ok(())
}

fn plan_status_rank(status: &str) -> Option<u8> {
    match status {
        "not-started" => Some(0),
        "in-progress" => Some(1),
        "implemented" => Some(2),
        "verified" => Some(3),
        _ => None,
    }
}

pub(crate) fn validate_audit_path(path: &Utf8Path, expected_file: &str) -> Result<()> {
    if path.is_absolute()
        || path
            .components()
            .any(|component| component.as_str() == "..")
        || !path.starts_with(WORK_PLANS_ROOT)
        || path.file_name() != Some(expected_file)
    {
        bail!("audit path must be a relative work-plan path ending in {expected_file}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, status: &str) -> ChecklistRow {
        ChecklistRow {
            id: id.to_owned(),
            surface: id.to_owned(),
            reference_breadcrumb: "reference/source.c".to_owned(),
            rust_owned_target: "crate/src/lib.rs".to_owned(),
            rust_owned_target_markdown: "`crate/src/lib.rs`".to_owned(),
            status: status.to_owned(),
            evidence: "unit".to_owned(),
            notes: String::new(),
        }
    }

    #[test]
    fn progress_counts_only_verified_active_rows_as_complete() {
        // Arrange
        let mut rows = Vec::new();
        rows.extend((0..27).map(|index| row(&format!("V-{index}"), "verified")));
        rows.extend((0..67).map(|index| row(&format!("A-{index}"), "implemented")));
        rows.extend((0..5).map(|index| row(&format!("D-{index}"), "deferred")));

        // Act
        let progress = Progress::from_rows(&rows).expect("progress should calculate");

        // Assert
        assert_eq!(progress.total, 99);
        assert_eq!(progress.active_total, 94);
        assert_eq!(progress.verified_total, 27);
        assert_eq!(progress.completion_basis_points, 2872);
        assert_eq!(progress.display_percent(), "28.7%");
    }

    #[test]
    fn progress_rejects_an_all_deferred_checklist() {
        // Arrange
        let rows = vec![row("D-1", "deferred")];

        // Act
        let error = Progress::from_rows(&rows).expect_err("zero active rows must fail");

        // Assert
        assert!(error.to_string().contains("undefined"));
    }

    #[test]
    fn progress_rejects_unknown_statuses() {
        // Arrange
        let rows = vec![row("X-1", "done")];

        // Act
        let error = Progress::from_rows(&rows).expect_err("unknown status must fail");

        // Assert
        assert!(error.to_string().contains("unknown parity status"));
    }

    #[test]
    fn candidates_prefer_implemented_then_in_progress_then_not_started() {
        // Arrange
        let rows = vec![
            row("N-1", "not-started"),
            row("I-1", "implemented"),
            row("P-1", "in-progress"),
            row("I-2", "implemented"),
            row("V-1", "verified"),
        ];

        // Act
        let candidates = ranked_candidates(&rows);

        // Assert
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.row_id.as_str())
                .collect::<Vec<_>>(),
            vec!["I-1", "I-2", "P-1", "N-1"]
        );
    }

    #[test]
    fn next_item_resumes_the_single_open_plan() {
        // Arrange
        let workspace = Utf8PathBuf::from_path_buf(
            std::env::temp_dir().join(format!("bitaxe-parity-open-plan-{}", std::process::id())),
        )
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        let plan_root = workspace.join("docs/parity/work-plans/run");
        fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
        fs::write(
            plan_root.join("PLAN.md").as_std_path(),
            "# Plan\n\n- Parity row: `CFG-001`\n- Initial status: `in-progress`\n",
        )
        .expect("plan");
        let rows = vec![row("CFG-001", "in-progress")];

        // Act
        let open_plan = find_open_plan(&workspace, &rows)
            .expect("open-plan scan")
            .expect("open plan");

        // Assert
        assert_eq!(open_plan.row_id, "CFG-001");
        assert_eq!(open_plan.plan_path, "docs/parity/work-plans/run/PLAN.md");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn next_item_closes_non_verified_plan_after_status_advance() {
        // Arrange
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-completed-plan-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        let plan_root = workspace.join("docs/parity/work-plans/run");
        fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
        fs::write(
            plan_root.join("PLAN.md").as_std_path(),
            "# Plan\n\n- Parity row: `SYS-004`\n- Initial status: `in-progress`\n",
        )
        .expect("plan");
        let rows = vec![row("SYS-004", "implemented")];

        // Act
        let maybe_open_plan = find_open_plan(&workspace, &rows).expect("open-plan scan");

        // Assert
        assert_eq!(maybe_open_plan, None);
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn next_item_resumes_newest_explicitly_linked_same_row_plan() {
        // Arrange
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-linked-open-plans-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        let older_root = workspace.join("docs/parity/work-plans/20260101T000000Z-API-010");
        let newer_root = workspace.join("docs/parity/work-plans/20260102T000000Z-API-010");
        fs::create_dir_all(older_root.as_std_path()).expect("older plan root");
        fs::create_dir_all(newer_root.as_std_path()).expect("newer plan root");
        fs::write(
            older_root.join("PLAN.md").as_std_path(),
            "# Plan\n\n- Parity row: `API-010`\n- Initial status: `implemented`\n",
        )
        .expect("older plan");
        fs::write(
            newer_root.join("PLAN.md").as_std_path(),
            "# Plan\n\n- Parity row: `API-010`\n- Initial status: `implemented`\n\nContinues `docs/parity/work-plans/20260101T000000Z-API-010/PLAN.md`.\n",
        )
        .expect("newer plan");
        let rows = vec![row("API-010", "implemented")];

        // Act
        let open_plan = find_open_plan(&workspace, &rows)
            .expect("linked plans should reconcile")
            .expect("newest plan should remain open");

        // Assert
        assert_eq!(open_plan.row_id, "API-010");
        assert_eq!(
            open_plan.plan_path,
            "docs/parity/work-plans/20260102T000000Z-API-010/PLAN.md"
        );
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn next_item_rejects_open_plans_spanning_rows() {
        // Arrange
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-cross-row-open-plans-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        for (run, row_id) in [
            ("20260101T000000Z-API-010", "API-010"),
            ("20260102T000000Z-CFG-001", "CFG-001"),
        ] {
            let plan_root = workspace.join("docs/parity/work-plans").join(run);
            fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
            fs::write(
                plan_root.join("PLAN.md").as_std_path(),
                format!("# Plan\n\n- Parity row: `{row_id}`\n- Initial status: `implemented`\n"),
            )
            .expect("plan");
        }
        let rows = vec![row("API-010", "implemented"), row("CFG-001", "implemented")];

        // Act
        let error =
            find_open_plan(&workspace, &rows).expect_err("cross-row plans must fail closed");

        // Assert
        assert!(error.to_string().contains("span rows"));
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }

    #[test]
    fn next_item_rejects_row_status_regression_from_plan() {
        // Arrange
        let workspace = Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
            "bitaxe-parity-regressed-plan-{}",
            std::process::id()
        )))
        .expect("temporary path must be UTF-8");
        let _ = fs::remove_dir_all(workspace.as_std_path());
        let plan_root = workspace.join("docs/parity/work-plans/run");
        fs::create_dir_all(plan_root.as_std_path()).expect("plan root");
        fs::write(
            plan_root.join("PLAN.md").as_std_path(),
            "# Plan\n\n- Parity row: `SYS-004`\n- Initial status: `implemented`\n",
        )
        .expect("plan");
        let rows = vec![row("SYS-004", "in-progress")];

        // Act
        let error = find_open_plan(&workspace, &rows).expect_err("regression must fail closed");

        // Assert
        assert!(error.to_string().contains("status regressed"));
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }
}
