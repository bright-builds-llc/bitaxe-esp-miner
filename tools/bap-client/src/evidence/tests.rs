use super::*;
use serde_json::json;

fn parent(mode: u32) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("dir");
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(mode)).expect("mode");
    dir
}

fn detector_report(dir: &Path, mode: u32, text: &str) -> PathBuf {
    let path = dir.join("detector.stdout");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(&path)
        .expect("report");
    file.write_all(text.as_bytes()).expect("write");
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("mode");
    path
}

const RUNTIME: &str =
    "port: /dev/cu.usbmodem1101\nusb_profile: serial_jtag_runtime\nrom_admitted: false\n";

#[test]
fn the_child_is_created_private_inside_a_private_parent() {
    // Arrange
    let dir = parent(0o700);
    let child = dir.path().join("attempt-001");
    // Act
    AttemptRoot::create(&child).expect("root");
    // Assert
    assert_eq!(
        fs::metadata(&child).expect("child").permissions().mode() & 0o777,
        0o700
    );
}

#[test]
fn a_permissive_parent_is_refused() {
    // Arrange
    let dir = parent(0o755);
    // Act
    let result = AttemptRoot::create(&dir.path().join("attempt-001"));
    // Assert
    assert_eq!(
        result.expect_err("refused").to_string(),
        "evidence_parent_not_private"
    );
}

#[test]
fn an_existing_child_is_never_reused() {
    // Arrange
    let dir = parent(0o700);
    let child = dir.path().join("attempt-001");
    fs::create_dir(&child).expect("existing");
    // Act
    let result = AttemptRoot::create(&child);
    // Assert
    assert_eq!(
        result.expect_err("refused").to_string(),
        "evidence_child_exists"
    );
}

#[test]
fn sealing_leaves_one_read_only_record() {
    // Arrange
    let dir = parent(0o700);
    let child = dir.path().join("attempt-001");
    let root = AttemptRoot::create(&child).expect("root");
    // Act
    root.seal(RECORD_FILE, &json!({"outcome": "complete"}))
        .expect("seal");
    // Assert
    let record = child.join(RECORD_FILE);
    assert_eq!(
        fs::metadata(&record).expect("record").permissions().mode() & 0o777,
        0o400
    );
    assert_eq!(
        fs::metadata(&child).expect("child").permissions().mode() & 0o777,
        0o500
    );
    assert_eq!(fs::read_dir(&child).expect("list").count(), 1);
    fs::set_permissions(&child, fs::Permissions::from_mode(0o700)).expect("cleanup");
}

#[test]
fn a_runtime_detector_report_admits_the_run() {
    // Arrange
    let dir = parent(0o700);
    let report = detector_report(dir.path(), 0o600, RUNTIME);
    // Act
    let profile = admit_detector(&report, &dir.path().join("attempt-001"));
    // Assert
    assert_eq!(profile.expect("admitted"), "serial_jtag_runtime");
}

#[test]
fn a_rom_or_ambiguous_detector_report_is_refused() {
    // Arrange
    let dir = parent(0o700);
    let rom = "port: /dev/cu.usbmodem1101\nusb_profile: rom_downloader\n";
    let report = detector_report(dir.path(), 0o600, rom);
    // Act
    let result = admit_detector(&report, &dir.path().join("attempt-001"));
    // Assert
    assert_eq!(
        result.expect_err("refused").to_string(),
        "detector_not_runtime"
    );
}

#[test]
fn a_readable_or_misplaced_detector_report_is_refused() {
    // Arrange
    let dir = parent(0o700);
    let report = detector_report(dir.path(), 0o644, RUNTIME);
    let elsewhere = parent(0o700);
    // Act
    let readable = admit_detector(&report, &dir.path().join("attempt-001"));
    let misplaced = admit_detector(&report, &elsewhere.path().join("attempt-001"));
    // Assert
    assert_eq!(
        readable.expect_err("refused").to_string(),
        "report_not_private"
    );
    assert_eq!(
        misplaced.expect_err("refused").to_string(),
        "report_not_sibling"
    );
}

#[test]
fn a_sealed_probe_is_readable_by_its_sibling_binding() {
    // Arrange
    let dir = parent(0o700);
    let probe_child = dir.path().join("attempt");
    AttemptRoot::create(&probe_child)
        .expect("root")
        .seal(RECORD_FILE, &json!({"outcome": "complete"}))
        .expect("seal");
    // Act
    let text = read_sealed_probe(&probe_child, &dir.path().join("binding"));
    // Assert
    assert!(text.expect("sealed").contains("complete"));
    fs::set_permissions(&probe_child, fs::Permissions::from_mode(0o700)).expect("cleanup");
}

#[test]
fn an_unsealed_probe_is_refused() {
    // Arrange
    let dir = parent(0o700);
    let probe_child = dir.path().join("attempt");
    AttemptRoot::create(&probe_child).expect("root");
    // Act
    let result = read_sealed_probe(&probe_child, &dir.path().join("binding"));
    // Assert
    assert!(result.is_err());
}
