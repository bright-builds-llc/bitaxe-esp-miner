//! BWG-007 seams: the clock-reset stimulus lives only in the pure crate, and every
//! native deadline keeps reading the real uptime clock.
const CRATE_STIMULUS: &str =
    include_str!("../../../crates/bitaxe-worker-control/src/clock_stimulus.rs");
const CRATE_CLOCK_ROUTE: &str =
    include_str!("../../../crates/bitaxe-worker-control/src/controller/clock.rs");
const USB_SOURCE: &str = include_str!("bwg_worker_usb.rs");
const LINK_SOURCE: &str = include_str!("bwg_worker_usb/link.rs");
const WRITER_SOURCE: &str = include_str!("bwg_worker_usb/writer.rs");
const SESSION_SOURCE: &str = include_str!("bwg_worker_session.rs");
const NVS_SOURCE: &str = include_str!("bwg_worker_nvs.rs");
const NATIVE_LEASE_SOURCE: &str = include_str!("production_mining_session/bwg.rs");
const OWNER_LOOP_SOURCE: &str = include_str!("production_mining_session/owner_loop.rs");
const REVOCATION_SOURCE: &str = include_str!("production_mining_session/revocation.rs");
const REVOCATION_GLOBAL_SOURCE: &str =
    include_str!("production_mining_session/revocation/global.rs");
const MAIN_SOURCE: &str = include_str!("main.rs");

const FIRMWARE_SOURCES: [&str; 10] = [
    USB_SOURCE,
    LINK_SOURCE,
    WRITER_SOURCE,
    SESSION_SOURCE,
    NVS_SOURCE,
    NATIVE_LEASE_SOURCE,
    OWNER_LOOP_SOURCE,
    REVOCATION_SOURCE,
    REVOCATION_GLOBAL_SOURCE,
    MAIN_SOURCE,
];

const STIMULUS_SYMBOLS: [&str; 6] = [
    "clock_stimulus",
    "ClockStimulus",
    "clock_discontinuity_stimulus",
    "confirm_clock_stimulus",
    "OFFSET_MILLISECONDS",
    "ARMED_FOR_MILLISECONDS",
];

fn run_owner_source() -> &'static str {
    let start = USB_SOURCE
        .find("fn run_owner<V>(")
        .expect("owner loop must exist");
    let end = USB_SOURCE[start..]
        .find("fn process_frame<V>(")
        .map(|offset| start + offset)
        .expect("owner loop boundary must exist");
    &USB_SOURCE[start..end]
}

#[test]
fn the_clock_stimulus_is_defined_only_in_the_pure_crate() {
    // Arrange
    let crate_sources = [CRATE_STIMULUS, CRATE_CLOCK_ROUTE];

    // Act
    let leaked: Vec<&str> = STIMULUS_SYMBOLS
        .into_iter()
        .filter(|symbol| FIRMWARE_SOURCES.iter().any(|source| source.contains(symbol)))
        .collect();

    // Assert
    assert!(leaked.is_empty(), "firmware names stimulus symbols: {leaked:?}");
    assert!(crate_sources[0].contains("pub(crate) struct ClockStimulus"));
    assert!(crate_sources[1].contains("self.clock_stimulus.sample("));
}

#[test]
fn only_the_worker_tick_samples_the_stimulus() {
    // Arrange
    let route = CRATE_CLOCK_ROUTE;

    // Act
    let samples = route.matches(".sample(").count();
    let tick = &route[route.find("pub fn tick(").expect("tick")..];
    let tick = &tick[..tick.find("\n    }\n").expect("tick end")];

    // Assert
    assert_eq!(samples, 1);
    assert!(tick.contains("self.clock_stimulus.sample("));
}

#[test]
fn the_owner_loop_ticks_the_worker_from_real_uptime_and_names_the_failure() {
    // Arrange
    let owner = run_owner_source();

    // Act
    let tick = &owner[owner
        .find("Err(mpsc::RecvTimeoutError::Timeout)")
        .expect("timeout branch")..];

    // Assert
    assert!(owner.contains("let now = crate::runtime_uptime::millis();"));
    assert!(tick.contains("if let Err(error) = worker.tick(now) {"));
    assert!(tick.contains("diagnostic(error.category());"));
    assert!(WRITER_SOURCE.contains("| \"monotonic_reset\""));
}

#[test]
fn heartbeat_and_link_deadlines_read_real_uptime() {
    // Arrange
    let heartbeat = "let now = crate::runtime_uptime::millis();\n                    if !link.liveness.heartbeat(now) {";

    // Act
    let link_reads_uptime = LINK_SOURCE.contains(heartbeat);

    // Assert
    assert!(link_reads_uptime);
    assert!(LINK_SOURCE.contains("revocation::check_deadline(now);"));
    assert!(!LINK_SOURCE.contains("WorkerControl"));
}

#[test]
fn native_revocation_and_lease_deadlines_read_real_uptime() {
    // Arrange
    let monotonic_reset_revocation = concat!(
        "RestorationReason::ControlFailed | RestorationReason::MonotonicReset => {\n",
        "                    RevocationReason::ControlFailed\n",
    );

    // Act
    let session_stamp =
        "revocation::revoke_reason_at(generation, crate::runtime_uptime::millis(), cause);";

    // Assert
    assert!(SESSION_SOURCE.contains(monotonic_reset_revocation));
    assert!(SESSION_SOURCE.contains(session_stamp));
    assert!(OWNER_LOOP_SOURCE.contains("let revocation_now = crate::runtime_uptime::millis();"));
    assert!(NATIVE_LEASE_SOURCE.contains(".saturating_sub(crate::runtime_uptime::millis())"));
    for source in [
        NATIVE_LEASE_SOURCE,
        OWNER_LOOP_SOURCE,
        REVOCATION_SOURCE,
        REVOCATION_GLOBAL_SOURCE,
    ] {
        assert!(!source.contains(".tick("));
        assert!(!source.contains("WorkerControl"));
        assert!(!source.contains("clock_discontinuities_detected"));
    }
}
