use super::*;

const PLAN_DOCUMENT: &str = include_str!(
    "../../../../docs/parity/work-plans/20260830T142327Z-NATIVE-USB-RECOVERY-TRANSITION/PLAN.md"
);

#[test]
fn native_usb_transition_writes_only_closed_projection_and_no_device_write() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let workspace = dir_path(&dir);
    let manifest = write_manifest_at(&workspace, TRANSITION_MANIFEST, DEFAULT_ELF_NAME);
    let plan = workspace.join(TRANSITION_PLAN);
    std::fs::create_dir_all(plan.parent().expect("plan parent").as_std_path())
        .expect("create plan parent");
    std::fs::write(plan.as_std_path(), PLAN_DOCUMENT).expect("write plan");
    std::fs::write(
        workspace.join("TASKS.md").as_std_path(),
        format!("### {TRANSITION_TASK}\n"),
    )
    .expect("write tasks");
    let command = VerifyNativeUsbTransitionCommand {
        board: BoardId::Ultra205,
        port: "/dev/cu.usbmodem-test".to_owned(),
        manifest: Utf8PathBuf::from(TRANSITION_MANIFEST),
        plan: Utf8PathBuf::from(TRANSITION_PLAN),
        private_root: Utf8PathBuf::from(TRANSITION_PRIVATE_ROOT),
        projection: Utf8PathBuf::from(TRANSITION_PROJECTION),
        redact_evidence: true,
    };
    let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace.clone());

    // Act
    run_verify_native_usb_transition(&command, &environment).expect("no-write transition");

    // Assert
    assert!(environment.executed_commands().is_empty());
    assert!(environment.generated_nvs_partitions().is_empty());
    assert_eq!(
        environment.device_effect_state(),
        UsbDeviceEffectState::None
    );
    assert_eq!(environment.cleanup_calls(), 1);
    assert!(!workspace.join(TRANSITION_PROJECTION).exists());
    let projection = std::fs::read_to_string(
        workspace
            .join(TRANSITION_PRIVATE_ROOT)
            .join("transition-result.private.json")
            .as_std_path(),
    )
    .expect("private transition result");
    assert!(projection.contains("\"device_write_observed\": false"));
    assert!(projection.contains("\"restoration_complete\": false"));
    assert!(projection.contains("\"terminal_category\": \"complete\""));
    assert!(projection.contains(&format!(
        "\"evaluator_sha256\": \"{}\"",
        transition_evaluator_sha256()
    )));
    for forbidden in ["usbmodem", "location_id", "physical_identity_digest"] {
        assert!(!projection.contains(forbidden));
    }
    assert!(manifest.exists());
}

#[test]
fn native_usb_transition_sources_exclude_every_write_surface() {
    // Arrange
    let verifier_sources = include_str!("../native_usb_transition.rs");

    // Act / Assert
    for forbidden in [
        "write-bin",
        "write_flash",
        "erase_flash",
        "generate_nvs_partition",
        "wifi_credentials",
        "pool_credentials",
        "mining-campaign",
    ] {
        assert!(
            !verifier_sources.contains(forbidden),
            "no-write verifier contains forbidden surface {forbidden}"
        );
    }
}

#[test]
fn transition_failure_stages_and_categories_are_closed() {
    // Arrange
    let counts = ProfileObservationCounts {
        absent: 4,
        same_worker: 2,
        same_serial_jtag: 0,
        same_unknown: 0,
        physical_mismatch: 0,
    };
    let error = anyhow::anyhow!("same_worker_after_commit: retained Worker profile");

    // Act
    let category = closed_transition_category(&error);
    let outcome = transition_failure_outcome(category, counts);

    // Assert
    assert_eq!(category, "same_worker_after_commit");
    assert!(outcome.ready_received);
    assert!(outcome.committed_received);
    assert!(!outcome.bus_reset_observed);
    assert_eq!(outcome.profile_counts, counts);
    assert_eq!(
        closed_transition_category(&anyhow::anyhow!("raw private detail")),
        "recovery_required"
    );
}
