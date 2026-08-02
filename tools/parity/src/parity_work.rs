use std::collections::BTreeMap;
use std::fs;

use crate::*;

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
    let maybe_open_plan = find_open_plan(&environment.workspace_dir)?;
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

fn find_open_plan(workspace: &Utf8Path) -> Result<Option<OpenPlan>> {
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
        if !path.is_dir() || path.join("RESULT.md").exists() || !path.join("PLAN.md").is_file() {
            continue;
        }
        let document = fs::read_to_string(path.join("PLAN.md").as_std_path())
            .with_context(|| format!("failed to read open parity plan {path}/PLAN.md"))?;
        let row_id = parse_plan_row(&document)?;
        let relative = path
            .strip_prefix(workspace)
            .context("open parity plan is outside the workspace")?;
        open_plans.push(OpenPlan {
            row_id,
            plan_path: format!("{relative}/PLAN.md"),
        });
    }
    if open_plans.len() > 1 {
        bail!("multiple open parity plans exist; close or reconcile them before selecting work");
    }
    Ok(open_plans.pop())
}

fn parse_plan_row(document: &str) -> Result<String> {
    let prefix = "- Parity row: `";
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
    bail!("open parity plan is missing parity-row metadata")
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
            "# Plan\n\n- Parity row: `CFG-001`\n",
        )
        .expect("plan");

        // Act
        let open_plan = find_open_plan(&workspace)
            .expect("open-plan scan")
            .expect("open plan");

        // Assert
        assert_eq!(open_plan.row_id, "CFG-001");
        assert_eq!(open_plan.plan_path, "docs/parity/work-plans/run/PLAN.md");
        fs::remove_dir_all(workspace.as_std_path()).expect("cleanup");
    }
}
