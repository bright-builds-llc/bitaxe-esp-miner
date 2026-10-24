//! Keeps the board-identity guard (ADR-0041) on the paths that admit hardware control.
const PRODUCTION_SOURCE: &str = include_str!("asic_adapter/production.rs");
const SELF_TEST_SOURCE: &str = include_str!("self_test_runtime.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const TPS546_BUS_SOURCE: &str = include_str!("safety_adapter/i2c_bus/tps546.rs");

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
