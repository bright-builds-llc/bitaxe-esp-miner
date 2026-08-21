const STARTUP_SOURCE: &str = include_str!("startup.rs");
const ADAPTER_SOURCE: &str = include_str!("display_adapter.rs");
const RUNTIME_SOURCE: &str = include_str!("operator_sensor_runtime.rs");
const SNAPSHOT_SOURCE: &str = include_str!("runtime_snapshot.rs");
const COMMAND_SURFACE_SOURCE: &str = include_str!("runtime_snapshot/command_surface.rs");
const SCREEN_SNAPSHOT_SOURCE: &str = include_str!("runtime_snapshot/screen.rs");
const INPUT_SOURCE: &str = include_str!("input_adapter.rs");

#[test]
fn confirmed_configuration_precedes_first_panel_initialization() {
    // Arrange
    let function = "fn initialize_operator_runtime(";
    let start = STARTUP_SOURCE.find(function).expect("startup owner must exist");
    let source = &STARTUP_SOURCE[start..];

    // Act
    let confirmed_snapshot = source
        .find("settings_adapter::current_settings_snapshot()")
        .expect("confirmed settings read");
    let configuration = source
        .find("load_ultra205_display_configuration")
        .expect("strict display projection");
    let initialization = source
        .find("RuntimeDisplayOwner::initialize")
        .expect("configured display initialization");

    // Assert
    assert!(confirmed_snapshot < configuration);
    assert!(configuration < initialization);
    assert_eq!(source.matches("RuntimeDisplayOwner::initialize").count(), 1);
}

#[test]
fn one_retained_owner_carries_configuration_rendering_and_power() {
    // Arrange
    let owner = "pub struct RuntimeDisplayOwner";
    let start = ADAPTER_SOURCE.find(owner).expect("runtime owner must exist");
    let source = &ADAPTER_SOURCE[start..];

    // Act / Assert
    assert_eq!(ADAPTER_SOURCE.matches(owner).count(), 1);
    assert!(source.contains("configuration: Ultra205DisplayConfiguration"));
    assert!(source.contains("power_policy: DisplayPowerPolicy"));
    assert!(source.contains("pub fn render_runtime_screen"));
    assert!(source.contains("pub fn service_power"));
    assert_eq!(ADAPTER_SOURCE.matches(".init()").count(), 1);
}

#[test]
fn configuration_is_applied_only_during_initialization_before_flush() {
    // Arrange
    let render = "fn render_debug_text<I2C>(";
    let start = ADAPTER_SOURCE.find(render).expect("render helper must exist");
    let end = ADAPTER_SOURCE[start..]
        .find("fn set_display_power<I2C>(")
        .map(|offset| start + offset)
        .expect("render helper boundary");
    let source = &ADAPTER_SOURCE[start..end];

    // Act
    let initialization = source.find(".init()").expect("panel init");
    let inversion = source
        .find("set_invert(configuration.inverted())")
        .expect("panel inversion");
    let flush = source.find(".flush()").expect("frame flush");

    // Assert
    assert!(initialization < inversion);
    assert!(inversion < flush);
    assert!(source.contains("driver_rotation(configuration.rotation())"));
}

#[test]
fn runtime_uses_absolute_cadence_and_edge_only_power_service() {
    // Arrange
    let schedule = "let mut maybe_display_schedule";
    let start = RUNTIME_SOURCE.find(schedule).expect("display schedule must exist");
    let source = &RUNTIME_SOURCE[start..];

    // Act / Assert
    assert!(source.contains("PeriodicDeadline::new"));
    assert!(source.contains("schedule.advance_past"));
    assert!(source.contains(
        ".service_power(owner, &mut i2c_budget, uptime_ms, decision.priority_visible)"
    ));
    assert!(source.contains(".render_runtime_screen(owner, &mut i2c_budget, &decision.frame)"));
    assert!(source.contains("RuntimeI2cBudget::new(sensor_publish_deadline_ms)"));
    assert!(source.contains("display.maybe_last_frame.as_ref() == Some(&decision.frame)"));
    assert!(source.contains("take_pending_screen_advances()"));
    assert!(source.contains("display.owner.record_input_activity(uptime_ms)"));
    assert!(!source.contains("std::thread::Builder::new().name(\"display"));
}

#[test]
fn gpio0_input_has_one_pullup_owner_and_closed_effect_logs() {
    // Arrange / Act / Assert
    assert_eq!(STARTUP_SOURCE.matches("input_adapter::start(pins.gpio0)").count(), 1);
    assert!(INPUT_SOURCE.contains("PinDriver::input(pin, Pull::Up)"));
    assert!(INPUT_SOURCE.contains("Duration::from_millis(BUTTON_SAMPLE_MS)"));
    assert_eq!(INPUT_SOURCE.matches(".spawn(move || run(driver))").count(), 1);
    assert!(INPUT_SOURCE.contains("ButtonSelfTestState::Inactive"));
    assert!(INPUT_SOURCE.contains("cancel_identify_if_active_at(now_ms)"));
    assert!(COMMAND_SURFACE_SOURCE.contains("mutate_command_visible_state_with_result("));
    assert!(COMMAND_SURFACE_SOURCE.contains("ButtonIdentifyCancellation::StateUnavailable"));
    assert!(INPUT_SOURCE.contains("effect=self_test_cancel_requested"));
    assert!(INPUT_SOURCE.contains("effect=self_test_cancel status=not_ready"));
    assert!(!INPUT_SOURCE.contains("ssid="));
    assert!(!INPUT_SOURCE.contains("ipv4="));
    assert!(!INPUT_SOURCE.contains("password="));
}

#[test]
fn screen_projection_is_read_only_and_keeps_private_values_out_of_logs() {
    // Arrange
    let function = "pub fn collect_screen_snapshot(";
    let start = SCREEN_SNAPSHOT_SOURCE
        .find(function)
        .expect("private screen projection must exist");
    let end = SCREEN_SNAPSHOT_SOURCE[start..]
        .find("fn screen_pool_host(")
        .map(|offset| start + offset)
        .expect("screen projection boundary");
    let source = &SCREEN_SNAPSHOT_SOURCE[start..end];

    // Act / Assert
    assert!(source.contains("let command = screen_command_projection(now_ms)"));
    assert!(source.contains("SafeTelemetrySnapshot::from_observations"));
    assert!(source.contains("runtime_health_adapter::collect()"));
    assert!(source.contains("wifi_adapter::current_wifi_snapshot()"));
    assert!(source.contains("ultra_205_catalog_entry()"));
    assert!(source.contains("screen_pool_host(command.fallback_active)"));
    assert!(!source.contains("collect_operator_snapshot_candidate"));
    assert!(!source.contains("complete_api_snapshot"));
    assert!(!source.contains("ApiSnapshot"));
    assert!(!source.contains("RuntimeTelemetryProjection"));
    assert!(!source.contains("publish_operator_snapshot("));
    assert!(!source.contains("maybe_drain_pending_runtime_sample_marker"));
    assert!(!source.contains("retain_completed_operator_snapshot"));
    assert!(!source.contains("log::"));
}

#[test]
fn production_publication_supplies_work_received_without_private_payloads() {
    // Arrange
    let function = "pub fn publish_production_session_snapshot(";
    let start = SNAPSHOT_SOURCE
        .find(function)
        .expect("production publication must exist");
    let end = SNAPSHOT_SOURCE[start..]
        .find("/// Publishes a monitor sample")
        .map(|offset| start + offset)
        .expect("production publication boundary");
    let source = &SNAPSHOT_SOURCE[start..end];

    // Act / Assert
    assert!(source.contains("snapshot.job_transition.pool_notify_count"));
    assert!(source.contains("state.work_received"));
    assert!(!source.contains("pool_host"));
    assert!(!source.contains("credentials"));
}

#[test]
fn display_failure_disables_only_display_and_preserves_sensor_loop() {
    // Arrange
    let disable = "fn disable_runtime_display(";
    let start = RUNTIME_SOURCE.find(disable).expect("display failure boundary");
    let end = RUNTIME_SOURCE[start..]
        .find("fn project_observations(")
        .map(|offset| start + offset)
        .expect("display failure boundary end");
    let source = &RUNTIME_SOURCE[start..end];

    // Act / Assert
    assert!(source.contains("*maybe_display = None"));
    assert!(!source.contains("park_forever"));
    assert!(RUNTIME_SOURCE.contains("reduce_sensor_sweep("));
    assert!(RUNTIME_SOURCE.contains("replace_observations_from_producer"));
    assert!(RUNTIME_SOURCE.contains(
        "i2c_budget.outcome() != safety_adapter::RuntimeI2cBudgetOutcome::BudgetExhausted"
    ));
}
