use std::env;
use std::io::{self, Write};
use std::process::Command as ProcessCommand;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand, ValueEnum};
use release_gate::{
    render_release_gate_report, validate_release_gate, ReleaseGateDocuments,
    DEFAULT_CARGO_ABOUT_PATH, DEFAULT_LICENSE_INVENTORY_PATH, DEFAULT_PROVENANCE_PATH,
};
use serde::Serialize;
use std::io::ErrorKind;

const BAZEL_REFERENCE_GUARD_TARGET: &str = "//scripts:verify_reference_clean";
const DEFAULT_REFERENCE_GUARD_PATH: &str = "scripts/verify-reference-clean.sh";
const DEFAULT_REFERENCE_DIR: &str = "reference/esp-miner";
const DEFAULT_OPENAPI_PATH: &str = "reference/esp-miner/main/http_server/openapi.yaml";
const DEFAULT_API_COMPARE_MANIFEST: &str = "tools/parity/fixtures/api/phase05-required-routes.json";
const DEFAULT_AXEOS_ROUTE_USAGE: &str = "tools/parity/fixtures/api/axeos-route-usage.json";

mod api_compare;
mod release_gate;

#[derive(Debug, Parser)]
#[command(name = "bitaxe-parity")]
#[command(about = "Report Bitaxe parity checklist status and evidence gaps.")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Report(ReportArgs),
    ApiCompare(ApiCompareArgs),
    ReleaseGate(ReleaseGateArgs),
}

#[derive(Debug, Parser)]
struct ReportArgs {
    #[arg(long, default_value = "docs/parity/checklist.md", value_parser = parse_utf8_path)]
    checklist: Utf8PathBuf,

    #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
    format: ReportFormat,

    #[arg(long = "fail-on-invalid-verified")]
    fail_on_invalid_verified: bool,
}

#[derive(Debug, Parser)]
struct ApiCompareArgs {
    #[arg(long, default_value = DEFAULT_OPENAPI_PATH, value_parser = parse_utf8_path)]
    openapi: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_API_COMPARE_MANIFEST, value_parser = parse_utf8_path)]
    route_manifest: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_AXEOS_ROUTE_USAGE, value_parser = parse_utf8_path)]
    static_usage: Utf8PathBuf,
}

#[derive(Debug, Parser)]
struct ReleaseGateArgs {
    #[arg(long, default_value = DEFAULT_LICENSE_INVENTORY_PATH, value_parser = parse_utf8_path)]
    license_inventory: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_PROVENANCE_PATH, value_parser = parse_utf8_path)]
    provenance: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_CARGO_ABOUT_PATH, value_parser = parse_utf8_path)]
    cargo_about: Utf8PathBuf,

    #[arg(long, value_name = "package-json", value_parser = parse_utf8_path)]
    manifest: Option<Utf8PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum ReportFormat {
    Text,
    Json,
}

#[derive(Debug)]
struct ReportRequest {
    checklist: Utf8PathBuf,
    format: ReportFormat,
    fail_on_invalid_verified: bool,
}

impl From<ReportArgs> for ReportRequest {
    fn from(args: ReportArgs) -> Self {
        Self {
            checklist: args.checklist,
            format: args.format,
            fail_on_invalid_verified: args.fail_on_invalid_verified,
        }
    }
}

#[derive(Debug, Serialize)]
struct ParityReport {
    reference_commit: String,
    rows: Vec<ChecklistRow>,
    validation_errors: Vec<ValidationError>,
}

impl ParityReport {
    fn new(reference_commit: String, rows: Vec<ChecklistRow>) -> Self {
        let validation_errors = validate_rows(&rows);

        Self {
            reference_commit,
            rows,
            validation_errors,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct ChecklistRow {
    id: String,
    surface: String,
    reference_breadcrumb: String,
    rust_owned_target: String,
    status: String,
    evidence: String,
    notes: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct ValidationError {
    id: String,
    message: String,
}

trait ReportEnvironment {
    fn run_reference_guard(&self) -> Result<()>;
    fn read_checklist(&self, path: &Utf8Path) -> Result<String>;
    fn reference_commit(&self) -> Result<String>;
}

#[derive(Debug)]
struct LocalEnvironment {
    workspace_dir: Utf8PathBuf,
    reference_guard_path: Utf8PathBuf,
}

impl LocalEnvironment {
    fn detect() -> Result<Self> {
        let workspace_dir = detect_workspace_dir()?;
        let reference_guard_path = detect_reference_guard_path(&workspace_dir);

        Ok(Self {
            workspace_dir,
            reference_guard_path,
        })
    }
}

impl ReportEnvironment for LocalEnvironment {
    fn run_reference_guard(&self) -> Result<()> {
        let output = ProcessCommand::new("bash")
            .arg(self.reference_guard_path.as_std_path())
            .env("BUILD_WORKSPACE_DIRECTORY", self.workspace_dir.as_str())
            .output()
            .with_context(|| {
                format!(
                    "failed to run reference guard {BAZEL_REFERENCE_GUARD_TARGET} at {}",
                    self.reference_guard_path
                )
            })?;

        if output.status.success() {
            return Ok(());
        }

        bail!(
            "reference guard {BAZEL_REFERENCE_GUARD_TARGET} failed: {}",
            command_stderr_or_status(&output)
        );
    }

    fn read_checklist(&self, path: &Utf8Path) -> Result<String> {
        let checklist_path = self.workspace_path(path);
        std::fs::read_to_string(checklist_path.as_std_path())
            .with_context(|| format!("failed to read checklist {checklist_path}"))
    }

    fn reference_commit(&self) -> Result<String> {
        let reference_dir = self.workspace_dir.join(DEFAULT_REFERENCE_DIR);
        let output = ProcessCommand::new("git")
            .args(["-C", reference_dir.as_str(), "rev-parse", "HEAD"])
            .output()
            .with_context(|| format!("failed to read reference commit from {reference_dir}"))?;

        if !output.status.success() {
            bail!(
                "failed to read reference commit from {reference_dir}: {}",
                command_stderr_or_status(&output)
            );
        }

        let commit = String::from_utf8(output.stdout)
            .context("reference commit output was not valid UTF-8")?;
        let trimmed = commit.trim();
        if trimmed.is_empty() {
            bail!("reference commit output was empty");
        }

        Ok(trimmed.to_owned())
    }
}

impl LocalEnvironment {
    fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        if path.is_absolute() {
            return path.to_owned();
        }

        self.workspace_dir.join(path)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let environment = LocalEnvironment::detect()?;

    let output = match cli.command {
        CliCommand::Report(args) => {
            let request = ReportRequest::from(args);
            run_report(&request, &environment)?
        }
        CliCommand::ApiCompare(args) => run_api_compare_command(args, &environment)?,
        CliCommand::ReleaseGate(args) => run_release_gate_command(args, &environment)?,
    };

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{output}")?;

    Ok(())
}

fn run_release_gate_command(
    args: ReleaseGateArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let license_inventory_path = environment.workspace_path(&args.license_inventory);
    let provenance_path = environment.workspace_path(&args.provenance);
    let cargo_about_path = environment.workspace_path(&args.cargo_about);
    let maybe_manifest_path = args
        .manifest
        .as_ref()
        .map(|manifest| environment.workspace_path(manifest));

    let license_inventory_markdown = std::fs::read_to_string(license_inventory_path.as_std_path())
        .with_context(|| format!("failed to read license inventory {license_inventory_path}"))?;
    let provenance_markdown = std::fs::read_to_string(provenance_path.as_std_path())
        .with_context(|| format!("failed to read provenance manifest {provenance_path}"))?;
    let maybe_cargo_about_html = read_optional_text(&cargo_about_path)?;
    let maybe_manifest_json = if let Some(manifest_path) = &maybe_manifest_path {
        read_optional_text(manifest_path)?
    } else {
        None
    };

    let documents = ReleaseGateDocuments {
        license_inventory_path: args.license_inventory,
        license_inventory_markdown,
        provenance_path: args.provenance,
        provenance_markdown,
        cargo_about_path: args.cargo_about,
        maybe_cargo_about_html,
        maybe_manifest_path: args.manifest,
        maybe_manifest_json,
    };
    let report = validate_release_gate(&documents);
    let output = render_release_gate_report(&report);

    if !report.passed() {
        bail!("release gate failed:\n{output}");
    }

    Ok(output)
}

fn run_api_compare_command(args: ApiCompareArgs, environment: &LocalEnvironment) -> Result<String> {
    let openapi_path = environment.workspace_path(&args.openapi);
    let route_manifest_path = environment.workspace_path(&args.route_manifest);
    let static_usage_path = environment.workspace_path(&args.static_usage);

    let openapi_yaml = std::fs::read_to_string(openapi_path.as_std_path())
        .with_context(|| format!("failed to read OpenAPI contract {openapi_path}"))?;
    let route_manifest_json = std::fs::read_to_string(route_manifest_path.as_std_path())
        .with_context(|| format!("failed to read API compare manifest {route_manifest_path}"))?;
    let static_usage_json = std::fs::read_to_string(static_usage_path.as_std_path())
        .with_context(|| format!("failed to read AxeOS route usage fixture {static_usage_path}"))?;

    let request = api_compare::ApiCompareRequest {
        openapi_yaml: &openapi_yaml,
        route_manifest_json: &route_manifest_json,
        static_usage_json: &static_usage_json,
    };
    let loader = api_compare::WorkspaceFixtureLoader::new(environment.workspace_dir.clone());
    let report = api_compare::run_api_compare(&request, &loader)?;
    let output = api_compare::render_api_compare_report(&report);

    if report.has_validation_errors() {
        bail!("api compare failed:\n{output}");
    }

    Ok(output)
}

fn run_report(
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
    let reference_commit = environment.reference_commit()?;
    let report = ParityReport::new(reference_commit, rows);

    if environment_request.fail_on_invalid_verified && !report.validation_errors.is_empty() {
        bail!(
            "invalid verified parity claims:\n{}",
            format_validation_errors(&report.validation_errors)
        );
    }

    render_report(&report, environment_request.format)
}

fn parse_checklist(checklist: &str) -> Result<Vec<ChecklistRow>> {
    let mut rows = Vec::new();

    for (line_index, line) in checklist.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }

        let cells: Vec<String> = trimmed
            .trim_matches('|')
            .split('|')
            .map(clean_markdown_cell)
            .collect();

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
            status: cells[4].clone(),
            evidence: cells[5].clone(),
            notes: cells[6].clone(),
        });
    }

    Ok(rows)
}

fn validate_rows(rows: &[ChecklistRow]) -> Vec<ValidationError> {
    let mut errors = Vec::new();

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
    }

    errors
}

fn render_report(report: &ParityReport, format: ReportFormat) -> Result<String> {
    match format {
        ReportFormat::Json => {
            serde_json::to_string_pretty(report).context("failed to serialize parity report")
        }
        ReportFormat::Text => Ok(render_text_report(report)),
    }
}

fn render_text_report(report: &ParityReport) -> String {
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

fn format_validation_errors(errors: &[ValidationError]) -> String {
    let mut output = String::new();

    for error in errors {
        output.push_str(&format!("- {}: {}\n", error.id, error.message));
    }

    output
}

fn is_header_or_separator(cells: &[String]) -> bool {
    let Some(first_cell) = cells.first() else {
        return false;
    };

    first_cell == "ID" || cells.iter().all(|cell| cell.chars().all(is_separator_char))
}

fn is_separator_char(character: char) -> bool {
    matches!(character, '-' | ':' | ' ')
}

fn clean_markdown_cell(cell: &str) -> String {
    cell.trim().replace('`', "")
}

fn is_safety_critical(row: &ChecklistRow) -> bool {
    let haystack = format!(
        "{} {} {} {}",
        row.id, row.surface, row.rust_owned_target, row.notes
    )
    .to_ascii_lowercase();

    haystack.contains("safety-critical")
        || row.id.starts_with("PWR-")
        || row.id.starts_with("THR-")
        || row.id.starts_with("SELF-")
        || [
            "voltage",
            "fan",
            "thermal",
            "power",
            "self-test hardware",
            "hardware-control",
            "runtime input",
            "runtime display",
        ]
        .iter()
        .any(|term| haystack.contains(term))
        || haystack.contains("asic initialization")
        || (row.id.starts_with("ASIC") && haystack.contains("initialization"))
}

fn has_hardware_evidence(row: &ChecklistRow) -> bool {
    matches!(
        normalize(&row.evidence).as_str(),
        "hardware-smoke" | "hardware-regression"
    )
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn parse_utf8_path(value: &str) -> std::result::Result<Utf8PathBuf, String> {
    if value.trim().is_empty() {
        return Err("path must not be empty".to_owned());
    }

    Ok(Utf8PathBuf::from(value))
}

fn read_optional_text(path: &Utf8Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path.as_std_path()) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {path}")),
    }
}

fn detect_workspace_dir() -> Result<Utf8PathBuf> {
    if let Ok(workspace_dir) = env::var("BUILD_WORKSPACE_DIRECTORY") {
        if !workspace_dir.trim().is_empty() {
            return Ok(Utf8PathBuf::from(workspace_dir));
        }
    }

    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to detect workspace root with git rev-parse --show-toplevel")?;

    if !output.status.success() {
        bail!(
            "failed to detect workspace root: {}",
            command_stderr_or_status(&output)
        );
    }

    let stdout = String::from_utf8(output.stdout).context("workspace path was not valid UTF-8")?;
    let workspace_dir = stdout.trim();
    if workspace_dir.is_empty() {
        bail!("workspace path output was empty");
    }

    Ok(Utf8PathBuf::from(workspace_dir))
}

fn detect_reference_guard_path(workspace_dir: &Utf8Path) -> Utf8PathBuf {
    let maybe_guard_path = env::var("BITAXE_REFERENCE_GUARD").ok();
    if let Some(guard_path) = maybe_guard_path {
        if !guard_path.trim().is_empty() {
            return Utf8PathBuf::from(guard_path);
        }
    }

    workspace_dir.join(DEFAULT_REFERENCE_GUARD_PATH)
}

fn command_stderr_or_status(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed = stderr.trim();
    if !trimmed.is_empty() {
        return trimmed.to_owned();
    }

    format!("process exited with status {}", output.status)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use anyhow::{anyhow, Result};
    use camino::{Utf8Path, Utf8PathBuf};

    use super::*;

    const CHECKLIST: &str = r#"
# Parity Checklist

## Foundation

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| WF-001 | Read-only reference submodule | `reference/esp-miner` | `scripts/verify-reference-clean.sh` | implemented | pending | Guard exists. |
"#;

    #[test]
    fn parses_markdown_checklist_rows() {
        // Arrange
        let checklist = CHECKLIST;

        // Act
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Assert
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "WF-001");
        assert_eq!(rows[0].status, "implemented");
        assert_eq!(rows[0].evidence, "pending");
        assert_eq!(rows[0].reference_breadcrumb, "reference/esp-miner");
        assert_eq!(
            rows[0].rust_owned_target,
            "scripts/verify-reference-clean.sh"
        );
    }

    #[test]
    fn json_output_includes_reference_commit() {
        // Arrange
        let rows = parse_checklist(CHECKLIST).expect("checklist should parse");
        let report = ParityReport::new("abc123".to_owned(), rows);

        // Act
        let output = render_report(&report, ReportFormat::Json).expect("json should render");
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("output should be valid json");

        // Assert
        assert_eq!(parsed["reference_commit"], "abc123");
        assert_eq!(parsed["rows"][0]["id"], "WF-001");
    }

    #[test]
    fn verified_rows_with_pending_evidence_are_invalid() {
        // Arrange
        let checklist = CHECKLIST.replace("implemented | pending", "verified | pending");
        let rows = parse_checklist(&checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("pending evidence"));
    }

    #[test]
    fn safety_critical_verified_rows_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-003 | Core voltage control | `reference/esp-miner/main/power/vcore.c` | `firmware/bitaxe` | verified | unit | Safety-critical. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("hardware-smoke"));
        assert!(errors[0].message.contains("hardware-regression"));
    }

    #[test]
    fn safety_critical_notes_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-001 | ASIC reset behavior | `reference/esp-miner/main/power/asic_reset.c` | `firmware/bitaxe` | verified | unit | Safety-critical; requires hardware evidence. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].id, "PWR-001");
        assert!(errors[0].message.contains("hardware-smoke"));
        assert!(errors[0].message.contains("hardware-regression"));
    }

    #[test]
    fn safety_critical_self_test_verified_rows_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SELF-001 | Self-test lifecycle | `reference/esp-miner/main/self_test/self_test.c` | `crates/bitaxe-safety`, `firmware/bitaxe` | verified | unit | Self-test hardware requires Ultra 205 hardware smoke before verification. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].id, "SELF-001");
        assert!(errors[0].message.contains("hardware-smoke"));
        assert!(errors[0].message.contains("hardware-regression"));
    }

    #[test]
    fn safety_critical_runtime_input_display_verified_rows_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| UI-003 | Input behavior | `reference/esp-miner/main/input.c` | `firmware/bitaxe` | verified | workflow | Runtime input and runtime display hardware-control rows require hardware-smoke evidence. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].id, "UI-003");
        assert!(errors[0].message.contains("hardware-smoke"));
        assert!(errors[0].message.contains("hardware-regression"));
    }

    #[test]
    fn safety_critical_implemented_rows_do_not_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| THR-003 | PID behavior | `reference/esp-miner/main/thermal/PID.c` | `crates/bitaxe-safety/src/thermal.rs` | implemented | unit | Pure PID behavior is covered by unit tests; hardware fan and thermal verification remains separate. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn missing_reference_guard_failure_blocks_report_output() {
        // Arrange
        let env = FakeEnvironment::failing_guard("reference missing or not initialized");
        let request = ReportRequest {
            checklist: Utf8PathBuf::from("docs/parity/checklist.md"),
            format: ReportFormat::Text,
            fail_on_invalid_verified: true,
        };

        // Act
        let result = run_report(&request, &env);

        // Assert
        assert!(result.is_err());
        assert!(result
            .expect_err("report should fail")
            .to_string()
            .contains("reference missing"));
        assert!(!env.read_called.get());
    }

    #[test]
    fn dirty_reference_guard_failure_blocks_report_output() {
        // Arrange
        let env = FakeEnvironment::failing_guard("reference dirty");
        let request = ReportRequest {
            checklist: Utf8PathBuf::from("docs/parity/checklist.md"),
            format: ReportFormat::Text,
            fail_on_invalid_verified: true,
        };

        // Act
        let result = run_report(&request, &env);

        // Assert
        assert!(result.is_err());
        assert!(result
            .expect_err("report should fail")
            .to_string()
            .contains("reference dirty"));
        assert!(!env.read_called.get());
    }

    struct FakeEnvironment {
        maybe_guard_error: Option<&'static str>,
        read_called: Cell<bool>,
    }

    impl FakeEnvironment {
        fn failing_guard(message: &'static str) -> Self {
            Self {
                maybe_guard_error: Some(message),
                read_called: Cell::new(false),
            }
        }
    }

    impl ReportEnvironment for FakeEnvironment {
        fn run_reference_guard(&self) -> Result<()> {
            if let Some(message) = self.maybe_guard_error {
                return Err(anyhow!(message));
            }

            Ok(())
        }

        fn read_checklist(&self, _path: &Utf8Path) -> Result<String> {
            self.read_called.set(true);
            Ok(CHECKLIST.to_owned())
        }

        fn reference_commit(&self) -> Result<String> {
            Ok("abc123".to_owned())
        }
    }
}
