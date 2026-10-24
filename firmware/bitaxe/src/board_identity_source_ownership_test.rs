//! Keeps the board-identity guard (ADR-0041) on the paths that admit hardware control.
const PRODUCTION_SOURCE: &str = include_str!("asic_adapter/production.rs");
const SELF_TEST_SOURCE: &str = include_str!("self_test_runtime.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const TPS546_BUS_SOURCE: &str = include_str!("safety_adapter/i2c_bus/tps546.rs");
const GAMMA601_SOURCE: &str = include_str!("gamma601.rs");
const GAMMA601_ACTUATION_SOURCE: &str = include_str!("gamma601/actuation.rs");
const SAFETY_ADAPTER_SOURCE: &str = include_str!("safety_adapter.rs");

fn body_of<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature present");
    let rest = &source[start..];
    let end = rest.find("\n}\n").expect("function end");
    &rest[..end]
}

#[test]
fn the_asic_production_handle_requires_a_permitted_board() {
    // Act
    let body = body_of(
        PRODUCTION_SOURCE,
        "pub fn production_handle_available() -> bool {",
    );
    // Assert
    assert!(body.contains("crate::board_identity::hardware_control_permitted()"));
}

#[test]
fn the_self_test_preflight_checks_the_board_before_waiting() {
    // Arrange
    let body = body_of(SELF_TEST_SOURCE, "fn wait_for_safety_preflight(");
    // Act
    let maybe_guard = body.find("crate::board_identity::hardware_control_permitted()");
    let maybe_wait = body.find("SAFETY_PREFLIGHT_TIMEOUT_MS");
    // Assert
    assert!(maybe_guard
        .zip(maybe_wait)
        .is_some_and(|(guard, wait)| guard < wait));
}

#[test]
fn the_board_is_read_after_the_settings_and_before_the_hardware() {
    // Arrange
    let settings_body = body_of(STARTUP_SOURCE, "fn initialize_boot_identity_and_settings(");
    let run_body = body_of(STARTUP_SOURCE, "fn run_startup(");
    let position = |body: &str, needle: &str| body.find(needle).expect(needle);
    // Act
    let settings = position(
        settings_body,
        "settings_adapter::initialize_current_settings_snapshot()",
    );
    let identity = position(
        settings_body,
        "crate::board_identity::initialize(settings_read)",
    );
    let boot_settings = position(run_body, "initialize_boot_identity_and_settings()");
    let hardware = position(run_body, "initialize_hardware(");
    // Assert
    assert!(settings < identity && boot_settings < hardware);
}

#[test]
fn the_regulator_enable_carries_the_isolation_gate_s_literal_in_test_builds_only() {
    // Arrange
    let gate = TPS546_BUS_SOURCE
        .find(r#"#[cfg(any(feature = "gamma601-vout-check", feature = "gamma601-mining"))]
impl Tps546Bus"#)
        .expect("the enabling writes are gated to the test variants");
    // Act
    let body = body_of(&TPS546_BUS_SOURCE[gate..], "pub(crate) fn operation_on(");
    // Assert
    // `scripts/verify-firmware-variant-isolation.mjs` refuses this literal in production ELFs.
    assert!(body.contains(r#".context("tps546_operation_on_enable")"#));
}

#[test]
fn the_601_safe_boot_thread_is_its_board_s_only_actuation_consumer() {
    // Arrange
    let board_service = "pub(crate) fn service_next_board_actuation_request(";
    let gate = SAFETY_ADAPTER_SOURCE
        .find(board_service)
        .and_then(|at| SAFETY_ADAPTER_SOURCE[..at].rfind("#[cfg("))
        .expect("the board service is gated");
    // Act
    let calls = GAMMA601_SOURCE
        .matches("safety_adapter::service_next_board_actuation_request(")
        .count();
    // Assert
    assert!(SAFETY_ADAPTER_SOURCE[gate..].starts_with(r#"#[cfg(feature = "board-gamma-601")]"#));
    assert_eq!(calls, 1);
    assert!(!GAMMA601_SOURCE.contains("service_next_safety_actuation_request("));
}

#[test]
fn the_601_applies_fan_commands_by_the_shared_authority_and_regulator_commands_only_when_mining() {
    // Arrange
    let production_arm = body_of(GAMMA601_ACTUATION_SOURCE, "#[cfg(not(feature = \"gamma601-mining\"))]\nfn apply_regulator(");
    // Act / Assert
    assert!(GAMMA601_ACTUATION_SOURCE.contains("safety_adapter::decide_fan_duty(command)"));
    assert!(production_arm.contains("SafetyActuationReply::Refused"));
    assert!(!production_arm.contains("regulator::"));
}
