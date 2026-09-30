use serde_json::Value;
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bitaxe-virtual-board-tool"))
}
fn source_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repository root")
}
fn run(root: &Path, scenario: &str) -> std::process::Output {
    command()
        .args([
            "run",
            "--scenario",
            scenario,
            "--seed",
            "11",
            "--evidence-dir",
        ])
        .arg(root)
        .arg("--workspace-root")
        .arg(source_root())
        .output()
        .expect("CLI process")
}
#[test]
fn list_is_actual_shared_scenario_catalog() {
    // Arrange
    let mut cli = command();
    cli.arg("list");
    // Act
    let output = cli.output().expect("CLI process");
    // Assert
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("JSON"),
        serde_json::to_value(bitaxe_simulation::scenario_names()).expect("catalog")
    );
}
#[test]
fn existing_evidence_root_is_never_reused_or_modified() {
    // Arrange
    let directory = TempDir::new().expect("temp");
    let marker = directory.path().join("sealed");
    fs::write(&marker, b"immutable").expect("marker");
    // Act
    let output = run(directory.path(), "healthy_lifecycle");
    // Assert
    assert!(!output.status.success());
    assert_eq!(fs::read(marker).expect("marker"), b"immutable");
    assert!(!directory.path().join("result.json").exists());
}
#[test]
fn unsupported_scenario_is_persisted_and_returns_failure() {
    // Arrange
    let directory = TempDir::new().expect("temp");
    let root = directory.path().join("attempt");
    // Act
    let output = run(&root, "does_not_exist");
    // Assert
    assert_eq!(output.status.code(), Some(2));
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("result.json")).expect("result")).expect("JSON");
    assert_eq!(result["passed"], false);
    assert!(result["error"].is_string());
    assert!(!root.join("writer.json").exists());
    assert!(root.join("result.sha256").exists());
}
#[test]
fn same_seed_preserves_semantic_journal_and_source_bindings() {
    // Arrange
    let directory = TempDir::new().expect("temp");
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    let scenario = bitaxe_simulation::scenario_names()
        .first()
        .expect("catalog");
    // Act
    let _first = run(&first, scenario);
    let _second = run(&second, scenario);
    // Assert
    let read = |root: &Path| -> Value {
        serde_json::from_slice(&fs::read(root.join("result.json")).expect("result")).expect("JSON")
    };
    assert_eq!(read(&first), read(&second));
    assert_eq!(read(&first)["hardware_qualified"], false);
}
#[test]
fn package_digest_mismatch_is_rejected_before_scenario() {
    // Arrange
    let directory = TempDir::new().expect("temp");
    let root = directory.path().join("attempt");
    let package = directory.path().join("package.json");
    fs::write(directory.path().join("firmware.elf"), b"candidate bytes").expect("ELF");
    fs::write(&package,serde_json::to_vec(&serde_json::json!({"source_commit":"synthetic","app_elf_sha256":"wrong","artifacts":[{"kind":"firmware_elf","path":"firmware.elf","sha256":"wrong"}]})).expect("JSON")).expect("package");
    // Act
    let output = command()
        .args(["run", "--scenario", "healthy_lifecycle", "--evidence-dir"])
        .arg(&root)
        .arg("--workspace-root")
        .arg(source_root())
        .arg("--manifest")
        .arg(package)
        .output()
        .expect("CLI");
    // Assert
    assert!(!output.status.success());
    assert!(root.join("failure.json").exists());
    assert!(!root.join("result.json").exists());
    assert!(!root.join("writer.json").exists());
}
#[cfg(unix)]
#[test]
fn evidence_permissions_keep_raw_scenario_output_private() {
    use std::os::unix::fs::PermissionsExt;
    // Arrange
    let directory = TempDir::new().expect("temp");
    let root = directory.path().join("attempt");
    // Act
    let _output = run(&root, "does_not_exist");
    // Assert
    assert_eq!(
        fs::metadata(&root).expect("root").permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(root.join("result.json"))
            .expect("result")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn v2_run_binds_actual_controller_fixture_and_cargo_lock_sources() {
    use sha2::{Digest, Sha256};
    // Arrange
    let directory = TempDir::new().expect("temp");
    let root = directory.path().join("attempt");
    // Act
    let _output = run(&root, "does_not_exist");
    // Assert
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("result.json")).expect("result")).expect("JSON");
    assert_eq!(result["schema"], "bitaxe_virtual_board_run_v3");
    for field in ["worker_control_sha256", "fixture_sha256"] {
        let digest = result[field].as_str().expect("digest");
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    let expected: String =
        Sha256::digest(fs::read(source_root().join("Cargo.lock")).expect("lock"))
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
    assert_eq!(result["cargo_lock_sha256"], expected);
}

#[test]
fn changed_compile_inputs_cannot_be_attested_by_an_old_executable() {
    // Arrange
    let directory = TempDir::new().expect("temp");
    let source = directory.path().join("source");
    fs::create_dir(&source).expect("source directory");
    fn copy_sources(from: &Path, to: &Path) {
        fs::create_dir_all(to).expect("source folder");
        for entry in fs::read_dir(from).expect("source entries") {
            let entry = entry.expect("entry");
            if entry.file_type().expect("type").is_dir() {
                copy_sources(&entry.path(), &to.join(entry.file_name()));
            } else if matches!(
                entry.path().extension().and_then(|ext| ext.to_str()),
                Some("rs" | "toml" | "bzl" | "mjs")
            ) || entry.file_name() == "BUILD.bazel"
            {
                fs::copy(entry.path(), to.join(entry.file_name())).expect("copy public source");
            }
        }
    }
    for root in [
        "crates/bitaxe-api",
        "crates/bitaxe-asic",
        "crates/bitaxe-config",
        "crates/bitaxe-core",
        "crates/bitaxe-safety",
        "crates/bitaxe-stratum",
        "crates/bitaxe-runtime",
        "crates/bitaxe-virtual-board",
        "crates/bitaxe-worker-control",
        "crates/bitaxe-simulation",
        "tools/stratum-v2-fixture",
        "tools/virtual-board",
    ] {
        copy_sources(&source_root().join(root), &source.join(root));
    }
    for file in [
        "Cargo.toml",
        "Cargo.lock",
        "BUILD.bazel",
        "MODULE.bazel",
        "MODULE.bazel.lock",
        ".bazelrc",
        ".bazelversion",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "firmware/bitaxe/sdkconfig.defaults",
    ] {
        fs::create_dir_all(source.join(file).parent().expect("parent")).expect("parent directory");
        fs::copy(source_root().join(file), source.join(file)).expect("source input");
    }
    for (index, changed) in [
        "crates/bitaxe-core/src/lib.rs",
        "tools/virtual-board/BUILD.bazel",
    ]
    .into_iter()
    .enumerate()
    {
        let path = source.join(changed);
        let original = fs::read(&path).expect("original input");
        fs::write(&path, "// changed after compilation\n").expect("changed input");
        let evidence = directory.path().join(format!("attempt-{index}"));
        // Act
        let output = command()
            .args([
                "run",
                "--scenario",
                "healthy-lifecycle",
                "--seed",
                "1",
                "--evidence-dir",
            ])
            .arg(&evidence)
            .arg("--workspace-root")
            .arg(&source)
            .output()
            .expect("CLI");
        // Assert
        assert!(!output.status.success());
        let failure =
            fs::read_to_string(evidence.join("failure.json")).expect("saved binding failure");
        assert!(failure.contains("compiled input closure differs"));
        assert!(!evidence.join("result.json").exists());
        assert!(!evidence.join("writer.json").exists());
        fs::write(path, original).expect("restore local fixture");
    }
}
