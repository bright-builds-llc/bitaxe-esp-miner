const MAIN_SOURCE: &str = include_str!("main.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const RUNTIME_SOURCE: &str = include_str!("self_test_runtime.rs");
const SETTINGS_SOURCE: &str = include_str!("settings_adapter/self_test.rs");
const INPUT_SOURCE: &str = include_str!("input_adapter.rs");
const HTTP_SOURCE: &str = include_str!("http_api.rs");

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
    assert!(!HTTP_SOURCE.contains("/api/system/self-test"));
    assert!(!RUNTIME_SOURCE.contains("connect_pool"));
    assert!(!RUNTIME_SOURCE.contains("mining.submit"));
}

#[test]
fn self_test_uses_existing_safe_actuation_and_terminal_shutdown() {
    // Arrange
    let required = [
        "MiningHardwareProfilePreset::UpstreamDefault.profile()",
        "set_self_test_fan_duty",
        "execute_self_test_command",
        "try_read_self_test_result",
        "HardwareSafeStopPurpose::Terminal",
        "HARDWARE_SELF_TEST_MAX_C",
        "HARDWARE_SELF_TEST_RESTART_DELAY_MS",
    ];

    // Act
    let all_present = required.iter().all(|token| RUNTIME_SOURCE.contains(token));

    // Assert
    assert!(all_present);
    assert!(RUNTIME_SOURCE.contains("PlannedEvaluationFailure"));
    assert!(RUNTIME_SOURCE.contains("safe_stop_complete"));
}
