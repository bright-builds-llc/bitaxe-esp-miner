const MAIN_SOURCE: &str = include_str!("main.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const RUNTIME_SOURCE: &str = include_str!("self_test_runtime.rs");
const DOMAIN_SOURCE: &str = include_str!("self_test_runtime/domain_measurement.rs");
const BOARD_SOURCE: &str = include_str!("self_test_runtime/board.rs");
const MEASUREMENT_SOURCE: &str = include_str!("self_test_runtime/measurement.rs");
const GAMMA601_SOURCE: &str = include_str!("gamma601.rs");
const BOARD_IDENTITY_SOURCE: &str = include_str!("board_identity.rs");
const SETTINGS_SOURCE: &str = include_str!("settings_adapter/self_test.rs");
const INPUT_SOURCE: &str = include_str!("input_adapter.rs");
const HTTP_SOURCE: &str = include_str!("http_api.rs");
const BOOT_EVIDENCE_SOURCE: &str = include_str!("boot_evidence.rs");

#[test]
fn self_test_is_consume_before_use_and_mutually_exclusive_with_production() {
    // Arrange
    let admission = "settings_adapter::load_self_test_admission()";
    let self_test_start = "self_test_runtime::start(admission)";
    let production_start = "production_mining_session::start()";

    // Act
    let admission_index = STARTUP_SOURCE.find(admission).expect("admission exists");
    let self_test_index = STARTUP_SOURCE
        .find(self_test_start)
        .expect("self-test start exists");
    let production_index = STARTUP_SOURCE
        .find(production_start)
        .expect("production start exists");

    // Assert
    assert!(admission_index < self_test_index);
    assert!(admission_index < production_index);
    assert!(STARTUP_SOURCE.contains("if let Some(admission) = maybe_self_test_admission"));
    assert!(STARTUP_SOURCE.contains("} else {"));
    assert!(MAIN_SOURCE.contains("mod self_test_runtime;"));
    assert_eq!(RUNTIME_SOURCE.matches(".name(OWNER_THREAD_NAME").count(), 1);
}

#[test]
fn marker_receipt_and_button_boundaries_are_closed_and_private() {
    // Arrange
    let required_keys = [
        "selftestkind",
        "selftestlease",
        "selftestcase",
        "selftestrcpt",
        "selftestrcid",
    ];

    // Act
    let all_keys_present = required_keys
        .iter()
        .all(|key| SETTINGS_SOURCE.contains(key));

    // Assert
    assert!(all_keys_present);
    assert!(SETTINGS_SOURCE.contains("erase_admission_tuple(&partition)?"));
    assert!(SETTINGS_SOURCE.contains("tuple_replay"));
    assert!(INPUT_SOURCE.contains("self_test_runtime::request_cancel()"));
    assert!(RUNTIME_SOURCE.contains("HardwareSelfTestStage::AwaitingCancel"));
    assert!(STARTUP_SOURCE.contains("boot_evidence::register_self_test_receipt"));
    assert!(BOOT_EVIDENCE_SOURCE.contains("BOOT_EVIDENCE_INTERVAL_MS"));
    assert!(BOOT_EVIDENCE_SOURCE.contains("emit_due_self_test_receipt(now_ms)"));
    let replay_start = BOOT_EVIDENCE_SOURCE
        .find("fn emit_due_self_test_receipt")
        .expect("receipt replay exists");
    let replay_end = BOOT_EVIDENCE_SOURCE[replay_start..]
        .find("fn emit_provisioning_network_ready")
        .map(|offset| replay_start + offset)
        .expect("receipt replay boundary exists");
    let replay = &BOOT_EVIDENCE_SOURCE[replay_start..replay_end];
    assert!(replay.contains("log::info!(\"{marker}\")"));
    assert!(!replay.contains("log_buffer"));
    assert!(!HTTP_SOURCE.contains("/api/system/self-test"));
    for source in [RUNTIME_SOURCE, MEASUREMENT_SOURCE] {
        assert!(!source.contains("connect_pool"));
        assert!(!source.contains("mining.submit"));
    }
}

#[test]
fn self_test_uses_existing_safe_actuation_and_terminal_shutdown() {
    // Arrange
    let required = [
        "profile_for_board(crate::board_identity::catalog_entry())",
        "set_self_test_fan_duty",
        "execute_self_test_command",
        "try_read_self_test_result",
        "HardwareSafeStopPurpose::Terminal",
        "HARDWARE_SELF_TEST_MAX_C",
        "HARDWARE_SELF_TEST_RESTART_DELAY_MS",
        "request_hashrate_monitor_register_reads_tx",
        "HashrateMonitor::new(1, HARDWARE_SELF_TEST_DOMAIN_COUNT)",
        "HashrateRegister::DomainCount",
        "domain_rejected_counts",
        "self_test_metrics=",
    ];

    // Act
    let all_present = required
        .iter()
        .all(|token| [RUNTIME_SOURCE, DOMAIN_SOURCE, BOARD_SOURCE, MEASUREMENT_SOURCE].iter().any(|source| source.contains(token)));

    // Assert
    assert!(all_present);
    assert!(RUNTIME_SOURCE.contains("PlannedEvaluationFailure"));
    assert!(RUNTIME_SOURCE.contains("safe_stop_complete"));
    assert!(!RUNTIME_SOURCE.contains("small_core_id) % HARDWARE_SELF_TEST_DOMAIN_COUNT"));
    assert!(!MEASUREMENT_SOURCE.contains("small_core_id"));
    assert!(!DOMAIN_SOURCE.contains("small_core_id"));
}

#[test]
fn the_601_runs_its_self_test_only_where_mining_is_admitted_and_never_beside_a_worker() {
    // Arrange
    let (_, admission) = GAMMA601_SOURCE
        .split_once("fn maybe_self_test_admission()")
        .expect("the 601's admission exists");
    let (_, owners) = GAMMA601_SOURCE
        .split_once("fn start_owners()")
        .expect("the 601's owners exist");
    let position = |body: &str, token: &str| body.find(token).expect(token);
    // Act
    let refused_before_start = position(admission, "MINING_DISPATCH_ADMITTED")
        < position(admission, "return Some(admission)");
    let self_test_first = position(owners, "crate::self_test_runtime::start(admission)")
        < position(owners, "crate::production_mining_session::start()");
    // Assert
    assert!(refused_before_start);
    assert!(admission.contains("settings_adapter::clear_refused_self_test_flag()"));
    assert!(!admission.contains("SelfTestReceipt::Cancelled"));
    assert!(self_test_first);
    assert!(owners.contains("return Ok(true);"));
}

#[test]
fn the_601_self_test_powers_its_regulator_only_under_a_bounded_local_generation() {
    // Arrange
    let (_, gamma) = BOARD_SOURCE
        .split_once("#[cfg(feature = \"board-gamma-601\")]")
        .expect("the 601's board part exists");
    // Act
    let bounded = gamma.contains("revocation::begin_local()")
        && gamma.contains("revocation::admit_budget(generation, OUTPUT_ON_BOUND_MS)")
        && gamma.contains("revocation::begin_dispatch(revocation::stamp(Some(generation)), now_ms)");
    // Assert
    assert!(bounded);
    assert!(gamma.contains("const OUTPUT_ON_BOUND_MS: u64 = 240_000;"));
    assert!(gamma.contains("FanControllerOwner::Local(self.generation)"));
    assert!(gamma.contains("revocation::finish_shutdown(self.generation)"));
    assert!(RUNTIME_SOURCE.contains("authority.end();"));
}

#[test]
fn each_board_names_its_own_self_test_kind() {
    // Act
    let kinds = ["\"ultra205_full_v1\"", "\"gamma601_full_v1\""].map(|kind| BOARD_IDENTITY_SOURCE.contains(kind));
    // Assert
    assert_eq!(kinds, [true, true]);
    assert!(SETTINGS_SOURCE.contains("const KIND: &str = crate::board_identity::SELF_TEST_KIND;"));
}

#[test]
fn the_self_test_feeds_work_at_the_family_interval_and_drains_every_frame() {
    // Arrange
    let (_, service) = MEASUREMENT_SOURCE
        .split_once("fn service_diagnostic_work(")
        .expect("the diagnostic work service exists");
    // Act
    let family_cadence = MEASUREMENT_SOURCE
        .contains("crate::board_identity::ASIC_FAMILY.job_interval_ms(asic_count)")
        && service.contains("now_ms.saturating_add(work_dispatch_ms())");
    let drains = service.contains("for _ in 0..MAXIMUM_READS_PER_PASS")
        && service.contains("ProductionReadOutcome::Pending | ProductionReadOutcome::Discarded(_) => break");
    // Assert
    assert!(family_cadence);
    assert!(drains);
    assert!(!RUNTIME_SOURCE.contains("WORK_DISPATCH_MS"));
}

#[test]
fn the_measured_window_counts_nothing_from_the_warm_up() {
    // Arrange
    let (_, measuring) = RUNTIME_SOURCE
        .split_once("HardwareSelfTestStage::Measuring,")
        .expect("the measuring stage exists");
    let position = |token: &str| measuring.find(token).expect(token);
    // Act
    let reset_first = position("measurement.begin_window();") < position("let started_at_ms");
    // Assert
    assert!(reset_first);
    assert!(MEASUREMENT_SOURCE.contains("self.nonce_count = 0;"));
    assert!(DOMAIN_SOURCE.contains("self.sample_counts = [0; HARDWARE_SELF_TEST_DOMAIN_COUNT];"));
}
