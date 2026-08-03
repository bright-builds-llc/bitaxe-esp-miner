use std::env;
use std::fs;
use std::process::Command;

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;

use crate::EXPECTED_REFERENCE_COMMIT;

const BUILD_IDENTITY_PATHS: &str = "scripts/build-identity-pathspecs.txt";
const REFERENCE_DIR: &str = "reference/esp-miner";

pub(crate) fn build_identity_status(workspace_dir: &Utf8PathBuf) -> Result<String> {
    let source_commit = git_text(workspace_dir, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    require_lower_hex_commit("source", &source_commit)?;

    let pathspec_text = fs::read_to_string(workspace_dir.join(BUILD_IDENTITY_PATHS))
        .context("missing build identity pathspec contract")?;
    let pathspecs: Vec<_> = pathspec_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    if pathspecs.is_empty() {
        bail!("empty build identity pathspec contract");
    }

    let mut status_args = vec!["status", "--porcelain=v1", "--untracked-files=all", "--"];
    status_args.extend(pathspecs.iter().copied());
    let source_dirty = !git_text(workspace_dir, &status_args)?.is_empty();

    let tags = git_text(workspace_dir, &["tag", "--points-at", "HEAD"])?;
    let matching_tags: Vec<_> = tags.lines().filter(|tag| is_release_tag(tag)).collect();
    if matching_tags.len() > 1 {
        bail!("multiple release tags at HEAD");
    }
    let release_tag = matching_tags.first().copied().unwrap_or("unavailable");

    let firmware_manifest = fs::read_to_string(workspace_dir.join("firmware/bitaxe/Cargo.toml"))
        .context("semantic version unavailable")?;
    let semantic_version = package_version(&firmware_manifest)?;
    let reference_commit = verify_reference(workspace_dir)?;

    Ok(format!(
        "STABLE_BITAXE_SOURCE_COMMIT {source_commit}\nSTABLE_BITAXE_SOURCE_DIRTY {source_dirty}\nSTABLE_BITAXE_RELEASE_TAG {release_tag}\nSTABLE_BITAXE_SEMANTIC_VERSION {semantic_version}\nSTABLE_BITAXE_REFERENCE_COMMIT {reference_commit}\n"
    ))
}

pub(crate) fn verify_reference(workspace_dir: &Utf8PathBuf) -> Result<String> {
    let reference_dir = workspace_dir.join(REFERENCE_DIR);
    if !reference_dir.is_dir() {
        bail!("reference missing or not initialized: {REFERENCE_DIR}");
    }
    let submodule_status = git_text(
        workspace_dir,
        &["submodule", "status", "--recursive", REFERENCE_DIR],
    )?;
    if submodule_status
        .lines()
        .any(|line| matches!(line.as_bytes().first(), Some(b'-' | b'+' | b'U')))
    {
        bail!("reference submodule state invalid: {REFERENCE_DIR}");
    }
    let actual_commit = git_text(&reference_dir, &["rev-parse", "HEAD"])?;
    if actual_commit != EXPECTED_REFERENCE_COMMIT {
        bail!(
            "reference commit mismatch: expected {EXPECTED_REFERENCE_COMMIT}, found {actual_commit}"
        );
    }
    if !git_text(
        &reference_dir,
        &["status", "--porcelain", "--untracked-files=all"],
    )?
    .is_empty()
    {
        bail!("reference dirty: {actual_commit}");
    }
    Ok(actual_commit)
}

pub(crate) fn detect_workspace_dir() -> Result<Utf8PathBuf> {
    if let Ok(workspace_dir) = env::var("BUILD_WORKSPACE_DIRECTORY") {
        if !workspace_dir.is_empty() {
            return Ok(Utf8PathBuf::from(workspace_dir));
        }
    }

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .context("failed to detect workspace directory with git rev-parse --show-toplevel")?;
    if !output.status.success() {
        bail!(
            "failed to detect workspace directory: {}",
            command_stderr_or_status(&output)
        );
    }

    let workspace_dir = String::from_utf8(output.stdout)
        .context("workspace directory output was not valid UTF-8")?;
    let trimmed = workspace_dir.trim();
    if trimmed.is_empty() {
        bail!("workspace directory output was empty");
    }
    Ok(Utf8PathBuf::from(trimmed))
}

fn git_text(workspace_dir: &Utf8PathBuf, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(workspace_dir)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            command_stderr_or_status(&output)
        );
    }
    String::from_utf8(output.stdout)
        .context("git output was not valid UTF-8")
        .map(|value| value.trim().to_owned())
}

fn require_lower_hex_commit(label: &str, value: &str) -> Result<()> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("{label} commit must be a full lowercase hash");
    }
    Ok(())
}

fn is_release_tag(value: &str) -> bool {
    let Some(version) = value.strip_prefix('v') else {
        return false;
    };
    let parts: Vec<_> = version.split('.').collect();
    (parts.len() == 2 || parts.len() == 3)
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn package_version(manifest: &str) -> Result<&str> {
    let mut in_package = false;
    for line in manifest.lines().map(str::trim) {
        if line == "[package]" {
            in_package = true;
            continue;
        }
        if line.starts_with('[') {
            in_package = false;
            continue;
        }
        if !in_package {
            continue;
        }
        let Some(value) = line
            .strip_prefix("version = \"")
            .and_then(|value| value.strip_suffix('"'))
        else {
            continue;
        };
        if value.is_empty() || !value.contains('.') {
            bail!("invalid semantic version");
        }
        return Ok(value);
    }
    bail!("semantic version unavailable")
}

fn command_stderr_or_status(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed_stderr = stderr.trim();
    if !trimmed_stderr.is_empty() {
        return trimmed_stderr.to_owned();
    }
    format!("exit status {}", output.status)
}
