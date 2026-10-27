//! Keeps the board-identity guard (ADR-0041) on the paths that admit hardware control.
const PRODUCTION_SOURCE: &str = include_str!("asic_adapter/production.rs");
const SELF_TEST_SOURCE: &str = include_str!("self_test_runtime.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");
const TPS546_BUS_SOURCE: &str = include_str!("safety_adapter/i2c_bus/tps546.rs");
const GAMMA601_SOURCE: &str = include_str!("gamma601.rs");
const GAMMA601_ACTUATION_SOURCE: &str = include_str!("gamma601/actuation.rs");
const SAFETY_ADAPTER_SOURCE: &str = include_str!("safety_adapter.rs");
const BOARD_601_BOOT_GATE_SOURCE: &str = include_str!("asic_adapter/board_601.rs");
const BOARD_IDENTITY_SOURCE: &str = include_str!("board_identity.rs");
const LIVE_HARDWARE_SOURCE: &str = include_str!("production_mining_session/live_hardware.rs");
const DEVICE_PROFILE_SOURCE: &str = include_str!("settings_adapter/device_profile.rs");
const CHIP_CHECK_SOURCE: &str = include_str!("gamma601/chip_check.rs");
const WORKER_SESSION_SOURCE: &str = include_str!("bwg_worker_session.rs");
const BOOT_GUARD_SOURCE: &str = include_str!("boot_guard.rs");
const CHECK_POWER_SOURCE: &str = include_str!("gamma601/check_power.rs");
const BUILD_SCRIPT_SOURCE: &str = include_str!("../build.rs");

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
fn the_regulator_enable_carries_the_board_gate_s_literal() {
    // Act
    let body = body_of(TPS546_BUS_SOURCE, "pub(crate) fn operation_on(");
    // Assert
    // `scripts/verify-firmware-board-isolation.mjs` requires this literal in the 601's ELF and
    // refuses it in the 205's.
    assert!(body.contains(r#".context("tps546_operation_on_enable")"#));
}

#[test]
fn only_the_601_builds_but_the_bounded_checks_may_mine() {
    // Arrange
    let (_, emit) = BUILD_SCRIPT_SOURCE
        .split_once("fn emit_gamma601_dispatch_cfg()")
        .expect("the dispatch cfg is emitted");
    // Act
    let checks_excluded = [
        "CARGO_FEATURE_GAMMA601_VOUT_CHECK",
        "CARGO_FEATURE_GAMMA601_CHIP_CHECK",
        "CARGO_FEATURE_GAMMA601_HASH_CHECK",
    ]
    .iter()
    .all(|feature| emit.contains(feature));
    // Assert
    assert!(checks_excluded);
    assert!(emit.contains(
        r#"if env::var_os("CARGO_FEATURE_BOARD_GAMMA_601").is_some() && !check_variant {"#
    ));
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
    let check_arm = body_of(GAMMA601_ACTUATION_SOURCE, "#[cfg(not(gamma601_dispatch))]\nfn apply_regulator(");
    // Act / Assert
    assert!(GAMMA601_ACTUATION_SOURCE.contains("safety_adapter::decide_fan_duty(command)"));
    assert!(check_arm.contains("SafetyActuationReply::Refused"));
    assert!(!check_arm.contains("regulator::"));
}

#[test]
fn the_601_boot_gate_confirms_only_with_the_peripherals_retained() {
    // Act
    let confirms = BOARD_601_BOOT_GATE_SOURCE.matches("BootMiningBaseline::Confirmed").count();
    let retained = BOARD_601_BOOT_GATE_SOURCE
        .find("production::store_board_601_peripherals(uart, reset);")
        .zip(BOARD_601_BOOT_GATE_SOURCE.find("Ok(BootMiningBaseline::Confirmed)"));
    // Assert
    assert_eq!(confirms, 1);
    assert!(retained.is_some_and(|(store, confirm)| store < confirm));
}

#[test]
fn the_601_worker_baseline_also_needs_the_output_confirmed_off() {
    // Act
    let gate = body_of(GAMMA601_SOURCE, "pub(crate) fn initialize_hardware(");
    let confirm = body_of(GAMMA601_SOURCE, "fn confirm_output_off(");
    // Assert
    assert!(gate.contains(".map(confirm_output_off)"));
    assert!(confirm.contains("SafeBootReport::off_confirmed"));
    assert!(confirm.contains("BootMiningBaseline::Unconfirmed"));
}

#[test]
fn the_601_reset_interlock_asks_the_executor_s_reset_owner_first() {
    // Arrange
    let body = body_of(GAMMA601_SOURCE, "pub(crate) fn reset_held() -> bool {");
    // Act
    let maybe_owner = body.find("asic_adapter::production::executor_holds_reset()");
    let maybe_pin = body.find("gpio_get_level");
    // Assert
    assert!(maybe_owner.zip(maybe_pin).is_some_and(|(owner, pin)| owner < pin));
}

#[test]
fn the_asic_production_handle_requires_this_build_to_admit_dispatch() {
    // Act
    let body = body_of(
        PRODUCTION_SOURCE,
        "pub fn production_handle_available() -> bool {",
    );
    // Assert
    assert!(body.contains("crate::board_identity::MINING_DISPATCH_ADMITTED"));
}

#[test]
fn the_601_builds_refuse_live_changes_and_admit_dispatch_unless_a_bounded_check() {
    // Arrange
    let gamma = &BOARD_IDENTITY_SOURCE[BOARD_IDENTITY_SOURCE
        .find(r#"#[cfg(feature = "board-gamma-601")]
mod built"#)
        .expect("the 601 build's facts")..];
    // Act / Assert
    assert!(gamma.contains("DISPATCH_ADMITTED: bool = cfg!(gamma601_dispatch);"));
    assert!(gamma.contains("LIVE_HARDWARE_CHANGES: bool = false;"));
}

#[test]
fn the_live_path_returns_before_any_change_when_the_board_has_none() {
    // Arrange
    let body = body_of(LIVE_HARDWARE_SOURCE, "pub(super) fn service_live_hardware_settings(");
    // Act
    let maybe_guard = body.find("!crate::board_identity::LIVE_HARDWARE_CHANGES");
    let maybe_change = body.find("apply_live_core_voltage");
    // Assert
    assert!(maybe_guard.zip(maybe_change).is_some_and(|(guard, change)| guard < change));
}

#[test]
fn the_saved_profile_is_checked_against_the_built_board() {
    // Act / Assert
    assert!(DEVICE_PROFILE_SOURCE.contains("from_device_settings_for_board("));
    assert!(DEVICE_PROFILE_SOURCE.contains("crate::board_identity::catalog_entry()"));
}

#[test]
fn the_chip_check_never_initializes_or_hashes() {
    // Act
    let sources = [CHIP_CHECK_SOURCE, CHECK_POWER_SOURCE];
    // Assert
    // Only the hash check's own stage may run the init or send work (ADR-0041 items 8 and 9).
    for source in sources {
        assert!(!source.contains("mining_ready_actions"));
        assert!(!source.contains("SendDiagnosticWork"));
    }
}

#[test]
fn the_chip_check_runs_nothing_after_its_detection() {
    // Act
    let body = body_of(CHIP_CHECK_SOURCE, "fn run_check(");
    // Assert
    assert!(body.contains("check_power::run_powered(owner, report, &SPEC, |_| Ok(()))"));
}

#[test]
fn a_board_without_admitted_mining_refuses_a_worker_start_before_its_budget() {
    // Arrange
    let body = body_of(WORKER_SESSION_SOURCE, "    fn start(");
    // Act
    let maybe_refusal = body.find("if !crate::board_identity::MINING_DISPATCH_ADMITTED {");
    let maybe_budget = body.find("crate::worker_acceptance_budget::admit(");
    // Assert
    assert!(maybe_refusal
        .zip(maybe_budget)
        .is_some_and(|(refusal, budget)| refusal < budget));
}

#[test]
fn each_board_embeds_its_own_worker_capability() {
    // Act
    let ultra = BOARD_IDENTITY_SOURCE
        .find("KnownBoard::Ultra205;")
        .zip(BOARD_IDENTITY_SOURCE.find("../bwg/ultra205-capability.json"));
    let gamma = BOARD_IDENTITY_SOURCE
        .find("KnownBoard::Gamma601;")
        .zip(BOARD_IDENTITY_SOURCE.find("../bwg/gamma601-capability.json"));
    // Assert
    // Each capability sits inside its board's `built` module, which follows its board fact.
    assert!(ultra.is_some_and(|(board, capability)| board < capability));
    assert!(gamma.is_some_and(|(board, capability)| board < capability));
    assert!(ultra.zip(gamma).is_some_and(|((_, ultra_file), (gamma_board, _))| ultra_file < gamma_board));
}

#[test]
fn the_601_overheat_build_never_mines_on_boot() {
    // Act
    let fact = BOARD_IDENTITY_SOURCE.contains(
        r#"pub(crate) const MINES_ON_BOOT: bool = !cfg!(feature = "gamma601-overheat-trip-test");"#,
    );
    let applied = BOOT_GUARD_SOURCE
        .contains("return start_mining_on_boot && crate::board_identity::MINES_ON_BOOT;");
    // Assert
    assert!(fact && applied);
}
