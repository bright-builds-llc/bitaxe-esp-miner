use super::*;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

struct Workspace {
    _temporary: tempfile::TempDir,
    root: camino::Utf8PathBuf,
}

fn workspace(tasks: &str) -> Workspace {
    let temporary = tempfile::tempdir().expect("temporary directory must be available");
    let root = Utf8Path::from_path(temporary.path())
        .expect("temporary path must be UTF-8")
        .to_owned();
    fs::write(root.join("TASKS.md").as_std_path(), tasks).expect("tasks must be written");
    let parent = root.join("private");
    fs::create_dir(parent.as_std_path()).expect("private parent must be created");
    #[cfg(unix)]
    fs::set_permissions(parent.as_std_path(), fs::Permissions::from_mode(0o700))
        .expect("private parent mode must be set");
    Workspace {
        _temporary: temporary,
        root,
    }
}

fn run_endurance(workspace: &Workspace, cycles: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_device-session"))
        .env("BUILD_WORKSPACE_DIRECTORY", workspace.root.as_str())
        .env("ESPFLASH_BIN", "/nonexistent/espflash")
        .args([
            "usb-reset-endurance",
            "--port",
            "/nonexistent/cu.usbmodem-fixture",
            "--expected-physical-sha256",
            &digest('c'),
            "--expected-firmware-commit",
            COMMIT,
            "--expected-app-elf-sha256",
            &digest('e'),
            "--cycles",
            cycles,
            "--private-root",
            "private/attempt",
            "--projection-output",
            "projection.json",
        ])
        .output()
        .expect("device-session CLI must launch")
}

#[test]
fn built_cli_refuses_reset_endurance_without_the_active_task_gate() {
    // Arrange
    let workspace = workspace(
        "## Active\n\n### task-usb-stuck-link-after-reset | 2026-10-05 | Stop wedging\n\nUSB reset endurance hardware: disabled.\n",
    );

    // Act
    let output = run_endurance(&workspace, "3");

    // Assert
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("usb_reset_endurance=blocked reason=active_contract_disabled_or_ambiguous")
    );
    assert!(!workspace.root.join("private/attempt").exists());
    assert!(!workspace.root.join("projection.json").exists());
}

#[test]
fn built_cli_rejects_out_of_bounds_cycles_before_reading_the_task_gate() {
    // Arrange
    let workspace = workspace("");

    // Act
    let output = run_endurance(&workspace, "501");

    // Assert
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("usb_reset_endurance=invalid reason=cycles_out_of_bounds"));
    assert!(!workspace.root.join("private/attempt").exists());
}

#[test]
fn built_cli_admitted_gate_without_espflash_stops_before_device_discovery() {
    // Arrange
    let workspace = workspace(
        "## Active\n\n### task-usb-stuck-link-after-reset | 2026-10-05 | Stop wedging\n\nUSB reset endurance hardware: enabled.\n",
    );

    // Act
    let output = run_endurance(&workspace, "1");

    // Assert
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("usb_reset_endurance=blocked reason=espflash_not_found"));
    assert!(!workspace.root.join("private/attempt").exists());
    assert!(!workspace.root.join("projection.json").exists());
    assert!(!stderr.contains("cu.usbmodem-fixture"));
}
