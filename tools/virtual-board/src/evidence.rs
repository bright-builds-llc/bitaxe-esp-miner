use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    process::Command,
};

#[derive(Serialize)]
struct PackageBinding {
    manifest_sha256: String,
    app_elf_sha256: String,
    source_commit: String,
}
#[derive(Serialize)]
struct RunEvidence<'a> {
    schema: &'static str,
    compiled_inputs_sha256: String,
    executable_sha256: String,
    backend: &'static str,
    scenario: &'a str,
    seed: u64,
    source_commit: String,
    source_dirty: bool,
    model_sha256: String,
    runtime_sha256: String,
    scenario_sha256: String,
    worker_control_sha256: String,
    fixture_sha256: String,
    cargo_lock_sha256: String,
    sdkconfig_sha256: String,
    sdkconfig_kind: &'static str,
    schema_versions: serde_json::Value,
    validator_sha256: String,
    package_binding: Option<PackageBinding>,
    result: Option<bitaxe_simulation::ScenarioResult>,
    error: Option<String>,
    passed: bool,
    hardware_qualified: bool,
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    if !output.status.success() {
        bail!("source identity unavailable");
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
fn collect_sources(base: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(base.join(relative))? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_symlink() {
            bail!("source symlink rejected");
        }
        let child = relative.join(entry.file_name());
        if ty.is_dir() {
            collect_sources(base, &child, files)?;
        } else if matches!(
            child.extension().and_then(|name| name.to_str()),
            Some("rs" | "toml")
        ) {
            files.push(child);
        }
    }
    Ok(())
}
fn source_digest(root: &Path, folder: &str) -> Result<String> {
    let mut files = Vec::new();
    collect_sources(root, Path::new(folder), &mut files)?;
    files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    let mut hash = Sha256::new();
    for path in files {
        let name = path.to_string_lossy();
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        let bytes = fs::read(root.join(path))?;
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Ok(hex(&hash.finalize()))
}
fn required_text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("missing package {key}"))
}
fn package_binding(path: &Path) -> Result<PackageBinding> {
    let bytes = fs::read(path)?;
    let manifest: Value = serde_json::from_slice(&bytes)?;
    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .context("missing artifacts")?;
    let matches: Vec<_> = artifacts
        .iter()
        .filter(|artifact| artifact.get("kind").and_then(Value::as_str) == Some("firmware_elf"))
        .collect();
    if matches.len() != 1 {
        bail!("package requires exactly one ELF");
    }
    let relative = Path::new(required_text(matches[0], "path")?);
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("package ELF must be a relative sibling");
    }
    let parent = path.parent().context("package parent")?.canonicalize()?;
    let elf_path = parent.join(relative);
    let canonical = elf_path.canonicalize()?;
    if !canonical.starts_with(&parent) {
        bail!("package ELF escapes package directory");
    }
    let elf = fs::read(canonical)?;
    let hash = digest(&elf);
    if hash != required_text(matches[0], "sha256")?
        || hash != required_text(&manifest, "app_elf_sha256")?
    {
        bail!("package ELF digest mismatch");
    }
    Ok(PackageBinding {
        manifest_sha256: digest(&bytes),
        app_elf_sha256: hash,
        source_commit: required_text(&manifest, "source_commit")?.to_owned(),
    })
}
fn private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
struct WriterLease {
    path: PathBuf,
}
impl Drop for WriterLease {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.path) {
            tracing::error!(error = %error, "writer lease cleanup failed");
        }
    }
}

pub fn run(
    root: &Path,
    evidence_dir: &Path,
    scenario: &str,
    seed: u64,
    maybe_manifest: Option<&Path>,
) -> Result<bool> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(evidence_dir)
        .context("evidence root must be fresh with an existing parent")?;
    let lease_path = evidence_dir.join("writer.json");
    private_file(
        &lease_path,
        format!(
            "{{\"pid\":{},\"schema\":\"virtual_writer_v1\"}}",
            std::process::id()
        )
        .as_bytes(),
    )?;
    let lease = WriterLease { path: lease_path };
    let envelope = make_evidence(root, scenario, seed, maybe_manifest);
    match envelope {
        Ok(result) => {
            let passed = result.passed;
            let bytes = serde_json::to_vec_pretty(&result)?;
            private_file(&evidence_dir.join("result.json"), &bytes)?;
            drop(lease);
            if evidence_dir.join("writer.json").exists() {
                bail!("live writer blocks finalization");
            }
            private_file(
                &evidence_dir.join("result.sha256"),
                digest(&bytes).as_bytes(),
            )?;
            File::open(evidence_dir)?.sync_all()?;
            Ok(passed)
        }
        Err(error) => {
            private_file(
                &evidence_dir.join("failure.json"),
                &serde_json::to_vec(
                    &serde_json::json!({"schema":"bitaxe_virtual_board_failure_v1","phase":"binding","error":error.to_string(),"passed":false}),
                )?,
            )?;
            Err(error)
        }
    }
}
fn make_evidence<'a>(
    root: &Path,
    scenario: &'a str,
    seed: u64,
    maybe_manifest: Option<&Path>,
) -> Result<RunEvidence<'a>> {
    let compiled = env!("BITAXE_VIRTUAL_COMPILED_INPUTS_JSON");
    let current = crate::snapshot::snapshot(root)?;
    if current != compiled {
        bail!("compiled input closure differs from current source");
    }
    let source_commit = git(root, &["rev-parse", "HEAD"])?;
    let source_dirty =
        !git(root, &["status", "--porcelain", "--untracked-files=normal"])?.is_empty();
    let model_sha256 = source_digest(root, "crates/bitaxe-virtual-board")?;
    let runtime_sha256 = source_digest(root, "crates/bitaxe-runtime")?;
    let scenario_sha256 = source_digest(root, "crates/bitaxe-simulation")?;
    let worker_control_sha256 = source_digest(root, "crates/bitaxe-worker-control")?;
    let fixture_sha256 = source_digest(root, "tools/stratum-v2-fixture")?;
    let cargo_lock_sha256 = digest(&fs::read(root.join("Cargo.lock"))?);
    let sdkconfig_sha256 = digest(&fs::read(root.join("firmware/bitaxe/sdkconfig.defaults"))?);
    let validator_sha256 = source_digest(root, "tools/virtual-board")?;
    let package_binding = maybe_manifest.map(package_binding).transpose()?;
    let outcome = bitaxe_simulation::run_scenario(scenario, seed);
    let (result, error, passed) = match outcome {
        Ok(result) => {
            let passed = result
                .checks
                .iter()
                .all(|check| check.status == bitaxe_simulation::CheckStatus::Passed)
                && !result.checks.is_empty();
            (Some(result), None, passed)
        }
        Err(error) => (None, Some(error.to_string()), false),
    };
    Ok(RunEvidence {
        schema: "bitaxe_virtual_board_run_v3",
        compiled_inputs_sha256: digest(compiled.as_bytes()),
        executable_sha256: digest(&fs::read(std::env::current_exe()?)?),
        backend: "host",
        scenario,
        seed,
        source_commit,
        source_dirty,
        model_sha256,
        runtime_sha256,
        scenario_sha256,
        worker_control_sha256,
        fixture_sha256,
        cargo_lock_sha256,
        sdkconfig_sha256,
        validator_sha256,
        sdkconfig_kind: "source_defaults",
        schema_versions: serde_json::json!({"model":bitaxe_virtual_board::MODEL_VERSION,"scenario":bitaxe_simulation::SCENARIO_VERSION,"runtime":"bitaxe_runtime_v1"}),
        package_binding,
        result,
        error,
        passed,
        hardware_qualified: false,
    })
}
