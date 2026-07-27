use crate::*;

#[derive(Debug)]
pub(crate) struct LocalEnvironment {
    pub(crate) workspace_dir: Utf8PathBuf,
    pub(crate) reference_guard_path: Utf8PathBuf,
}

impl LocalEnvironment {
    pub(crate) fn detect() -> Result<Self> {
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
        if path == Utf8Path::new(PHASE35_CHECKLIST_PATH) {
            return checklist_revision::read_authoritative_checklist(&self.workspace_dir)
                .map_err(anyhow::Error::msg);
        }
        let checklist_path = self.workspace_path(path);
        std::fs::read_to_string(checklist_path.as_std_path())
            .with_context(|| format!("failed to read checklist {checklist_path}"))
    }

    fn read_phase30_promotion_artifact(&self, path: &Utf8Path) -> Result<String> {
        let artifact_path = self.workspace_path(path);
        std::fs::read_to_string(artifact_path.as_std_path())
            .with_context(|| format!("failed to read Phase 30 promotion artifact {artifact_path}"))
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

    fn validate_checklist_targets(&self, rows: &[ChecklistRow]) -> Vec<ValidationError> {
        rows.iter()
            .flat_map(|row| {
                checklist_targets::validate_targets(
                    &self.workspace_dir,
                    &row.rust_owned_target_markdown,
                )
                .into_iter()
                .map(|message| ValidationError {
                    id: row.id.clone(),
                    message: format!("invalid Rust-owned target: {message}"),
                })
            })
            .collect()
    }
}

impl LocalEnvironment {
    pub(crate) fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        if path.is_absolute() {
            return path.to_owned();
        }

        self.workspace_dir.join(path)
    }

    pub(crate) fn current_git_head(&self) -> Result<String> {
        let output = ProcessCommand::new("git")
            .args(["-C", self.workspace_dir.as_str(), "rev-parse", "HEAD"])
            .output()
            .with_context(|| {
                format!(
                    "failed to read current git HEAD from {}",
                    self.workspace_dir
                )
            })?;

        if !output.status.success() {
            bail!(
                "failed to read current git HEAD from {}: {}",
                self.workspace_dir,
                command_stderr_or_status(&output)
            );
        }

        let commit = String::from_utf8(output.stdout)
            .context("current git HEAD output was not valid UTF-8")?;
        let trimmed = commit.trim();
        if trimmed.is_empty() {
            bail!("current git HEAD output was empty");
        }

        Ok(trimmed.to_owned())
    }

    pub(crate) fn source_commit_is_ancestor_of_head(&self, source_commit: &str) -> Result<bool> {
        let output = ProcessCommand::new("git")
            .args([
                "-C",
                self.workspace_dir.as_str(),
                "merge-base",
                "--is-ancestor",
                source_commit,
                "HEAD",
            ])
            .output()
            .with_context(|| {
                format!(
                    "failed to compare package source commit {source_commit} with HEAD in {}",
                    self.workspace_dir
                )
            })?;

        if output.status.success() {
            return Ok(true);
        }

        if output.status.code() == Some(1) {
            return Ok(false);
        }

        bail!(
            "failed to compare package source commit {source_commit} with HEAD in {}: {}",
            self.workspace_dir,
            command_stderr_or_status(&output)
        );
    }

    pub(crate) fn changed_paths_since(&self, source_commit: &str) -> Result<Vec<Utf8PathBuf>> {
        let output = ProcessCommand::new("git")
            .args([
                "-C",
                self.workspace_dir.as_str(),
                "diff",
                "--name-only",
                &format!("{source_commit}..HEAD"),
            ])
            .output()
            .with_context(|| {
                format!("failed to list paths changed since package source commit {source_commit}")
            })?;

        if !output.status.success() {
            bail!(
                "failed to list paths changed since package source commit {source_commit}: {}",
                command_stderr_or_status(&output)
            );
        }

        let stdout =
            String::from_utf8(output.stdout).context("git diff output was not valid UTF-8")?;
        Ok(stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(Utf8PathBuf::from)
            .collect())
    }
}
