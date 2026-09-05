use super::*;

const TEST_MANIFEST: &str = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";

fn input_uat_command() -> InputUatCommand {
    InputUatCommand {
        board: BoardId::Ultra205,
        port: "/dev/cu.usbmodem101".to_owned(),
        manifest: Utf8PathBuf::from(TEST_MANIFEST),
        private_root: Utf8PathBuf::from(INPUT_UAT_PRIVATE_ROOT),
        plan: Utf8PathBuf::from(INPUT_UAT_PLAN),
        projection: Utf8PathBuf::from(INPUT_UAT_PROJECTION),
    }
}

fn write_input_uat_workspace(root: &Utf8Path) {
    write_manifest_at(root, TEST_MANIFEST, DEFAULT_ELF_NAME);
    write_fixture_file(
        root,
        INPUT_UAT_PLAN,
        "# Parity work plan\n- Run ID: `20260816T102741Z-UI-003`\n- Parity row: `UI-003`\n`attempt-002` is the sole effectful attempt\n",
    );
    write_fixture_file(
        root,
        INPUT_CORE_SOURCE,
        "pub const BUTTON_SAMPLE_MS: u64 = 10;\npub const BUTTON_DEBOUNCE_MS: u64 = 30;\npub const BUTTON_LONG_PRESS_MS: u64 = 2_000;\n",
    );
    write_fixture_file(
        root,
        INPUT_ADAPTER_SOURCE,
        "PinDriver::input(pin, Pull::Up)?\ninput_status=active owner=boot_button sampling_ms={BUTTON_SAMPLE_MS} active_low=true\ninput_event=short_click effect=screen_advance\n",
    );
    write_fixture_file(
        root,
        REFERENCE_INPUT_SOURCE,
        "#define LONG_PRESS_DURATION_MS 2000\n.pull_up_en = GPIO_PULLUP_ENABLE\ngpio_get_level(GPIO_BUTTON_BOOT) == 0\nLV_EVENT_SHORT_CLICKED\n",
    );
}

fn write_fixture_file(root: &Utf8Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture parent").as_std_path())
        .expect("create fixture parent");
    std::fs::write(path.as_std_path(), contents).expect("write fixture");
}

#[test]
fn exact_package_short_click_writes_closed_projection_after_cleanup() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let root = dir_path(&dir);
    write_input_uat_workspace(&root);
    let attestation = format!("{}\n", runtime_attestation_log());
    let split = attestation
        .find("firmware_commit")
        .expect("attestation field")
        .saturating_add(7);
    let environment = FakeFlashEnvironment::default()
        .with_workspace_dir(root.clone())
        .with_input_uat_chunks(vec![
            attestation.as_bytes()[..split].to_vec(),
            attestation.as_bytes()[split..].to_vec(),
            b"I (123) input: input_event=short_click effect=screen_advance\n".to_vec(),
        ]);

    // Act
    run_input_uat(&input_uat_command(), &environment).expect("input UAT");

    // Assert
    let projection_path = root.join(INPUT_UAT_PROJECTION);
    let evidence: InputUatEvidence =
        serde_json::from_slice(&std::fs::read(projection_path.as_std_path()).expect("projection"))
            .expect("input UAT evidence");
    assert_eq!(evidence.validate(), Ok(()));
    assert_eq!(environment.cleanup_calls(), 1);
    assert_eq!(environment.observed_flashes().len(), 5);
    assert!(root
        .join(INPUT_UAT_PRIVATE_ROOT)
        .join("short-click.required.json")
        .exists());
}

#[test]
fn operator_interruption_cleans_up_and_withholds_projection() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let root = dir_path(&dir);
    write_input_uat_workspace(&root);
    let attestation = format!("{}\n", runtime_attestation_log());
    let environment = FakeFlashEnvironment::default()
        .with_workspace_dir(root.clone())
        .with_input_uat_chunks(vec![attestation.into_bytes()])
        .with_input_uat_interrupted();

    // Act
    let error = run_input_uat(&input_uat_command(), &environment)
        .expect_err("interruption must stop")
        .to_string();

    // Assert
    assert_eq!(error, "input_uat=stopped reason=operator_interrupted");
    assert_eq!(environment.cleanup_calls(), 1);
    assert!(!root.join(INPUT_UAT_PROJECTION).exists());
}

#[test]
fn malformed_runtime_attestation_preserves_closed_detail_after_cleanup() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let root = dir_path(&dir);
    write_input_uat_workspace(&root);
    let malformed = format!(
        "{}\n",
        runtime_attestation_log().replace("board=205", "board=999")
    );
    let environment = FakeFlashEnvironment::default()
        .with_workspace_dir(root.clone())
        .with_input_uat_chunks(vec![malformed.into_bytes()]);

    // Act
    let error = run_input_uat(&input_uat_command(), &environment)
        .expect_err("malformed runtime attestation must stop")
        .to_string();

    // Assert
    assert_eq!(
        error,
        "input_uat=failed reason=runtime_attestation_malformed"
    );
    assert_eq!(environment.cleanup_calls(), 1);
    assert!(!root.join(INPUT_UAT_PROJECTION).exists());
}
