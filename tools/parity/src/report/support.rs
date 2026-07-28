use crate::*;

pub(crate) fn row_haystack(row: &ChecklistRow) -> String {
    format!(
        "{} {} {} {} {} {}",
        row.id, row.surface, row.rust_owned_target, row.status, row.evidence, row.notes
    )
    .to_ascii_lowercase()
}

pub(crate) struct RequiredTerm {
    label: &'static str,
    needle: &'static str,
}

impl RequiredTerm {
    pub(crate) const fn new(label: &'static str, needle: &'static str) -> Self {
        Self { label, needle }
    }
}

pub(crate) fn missing_required_terms(
    row: &ChecklistRow,
    required_terms: &[RequiredTerm],
) -> Vec<&'static str> {
    let haystack = row_haystack(row);

    required_terms
        .iter()
        .filter(|term| !haystack.contains(term.needle))
        .map(|term| term.label)
        .collect()
}

pub(crate) fn format_required_terms(missing_terms: &[&'static str]) -> String {
    if missing_terms.is_empty() {
        return "required release evidence terms".to_owned();
    }

    missing_terms.join(", ")
}

pub(crate) fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub(crate) fn parse_utf8_path(value: &str) -> std::result::Result<Utf8PathBuf, String> {
    if value.trim().is_empty() {
        return Err("path must not be empty".to_owned());
    }

    Ok(Utf8PathBuf::from(value))
}

pub(crate) fn maybe_read_text(path: &Utf8Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path.as_std_path()) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {path}")),
    }
}

pub(crate) fn detect_workspace_dir() -> Result<Utf8PathBuf> {
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

pub(crate) fn detect_reference_guard_path(workspace_dir: &Utf8Path) -> Utf8PathBuf {
    let maybe_guard_path = env::var("BITAXE_REFERENCE_GUARD").ok();
    if let Some(guard_path) = maybe_guard_path {
        if !guard_path.trim().is_empty() {
            return Utf8PathBuf::from(guard_path);
        }
    }

    workspace_dir.join(DEFAULT_REFERENCE_GUARD_PATH)
}

pub(crate) fn command_stderr_or_status(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed = stderr.trim();
    if !trimmed.is_empty() {
        return trimmed.to_owned();
    }

    format!("process exited with status {}", output.status)
}
