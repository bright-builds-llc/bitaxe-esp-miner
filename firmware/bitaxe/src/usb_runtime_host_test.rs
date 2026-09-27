//! Exercises the production USB write loop against deterministic ESP-IDF call outcomes.
#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]
extern crate self as esp_idf_sys;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
static ADMITTED: AtomicBool = AtomicBool::new(true);

pub const ESP_OK: i32 = 0;
pub const configTICK_RATE_HZ: u32 = 100;
pub struct usb_serial_jtag_driver_config_t {
    pub tx_buffer_size: usize,
    pub rx_buffer_size: usize,
}
struct State {
    now: u64,
    write_ms: u64,
    maybe_write_clock: Option<u64>,
    flush_ms: u64,
    flush_ticks: u32,
    writes: usize,
    emitted: Vec<u8>,
    revoke_on_write: bool,
    revoke_on_flush: bool,
    force_full: bool,
}
static STATE: Mutex<State> = Mutex::new(State {
    now: 0,
    write_ms: 11,
    maybe_write_clock: None,
    flush_ms: 40,
    flush_ticks: 0,
    writes: 0,
    emitted: Vec::new(),
    revoke_on_write: false,
    revoke_on_flush: false,
    force_full: false,
});
static EXCLUSIVE: Mutex<()> = Mutex::new(());
pub unsafe fn usb_serial_jtag_driver_install(_: *mut usb_serial_jtag_driver_config_t) -> i32 {
    ESP_OK
}
pub unsafe fn usb_serial_jtag_read_bytes(_: *mut std::ffi::c_void, _: u32, _: u32) -> i32 {
    0
}
pub unsafe fn usb_serial_jtag_write_bytes(
    bytes: *const std::ffi::c_void,
    size: usize,
    ticks: u32,
) -> i32 {
    assert_eq!(ticks, 0, "native queue admission must never block");
    let mut state = STATE.lock().expect("test state");
    state.now += state.write_ms;
    if let Some(now) = state.maybe_write_clock {
        state.now = now;
    }
    state.writes += 1;
    if state.revoke_on_write {
        ADMITTED.store(false, Ordering::Release);
    }
    if state.force_full {
        return 0;
    }
    state
        .emitted
        .extend_from_slice(std::slice::from_raw_parts(bytes.cast::<u8>(), size));
    size as i32
}
pub unsafe fn usb_serial_jtag_wait_tx_done(ticks: u32) -> i32 {
    let mut state = STATE.lock().expect("test state");
    state.flush_ticks = ticks;
    if state.revoke_on_flush {
        ADMITTED.store(false, Ordering::Release);
    }
    let budget = u64::from(ticks) * 10;
    let elapsed = state.flush_ms.min(if ticks == 0 { 1 } else { budget });
    state.now += elapsed;
    state.flush_ms -= elapsed;
    if state.flush_ms == 0 {
        ESP_OK
    } else {
        0x107
    }
}
mod runtime_uptime {
    pub fn millis() -> u64 {
        super::STATE.lock().expect("test state").now
    }
}
#[path = "bwg_worker_usb/trace.rs"]
mod trace;
#[path = "usb_runtime.rs"]
mod usb_runtime;

#[test]
fn maximum_record_can_flush_within_the_existing_total_write_budget() {
    // Arrange
    let _exclusive = EXCLUSIVE.lock().expect("exclusive driver fixture");
    *STATE.lock().expect("test state") = State {
        now: 0,
        write_ms: 11,
        maybe_write_clock: None,
        flush_ms: 40,
        flush_ticks: 0,
        writes: 0,
        emitted: Vec::new(),
        revoke_on_write: false,
        revoke_on_flush: false,
        force_full: false,
    };
    // Act
    let result = usb_runtime::write_if(&vec![b'x'; 66560], || true);
    // Assert
    assert!(
        result.is_ok(),
        "a complete record and its 40ms drain fit within 2000ms: {result:?}"
    );
    let state = STATE.lock().expect("test state");
    assert!(state.now <= 2000);
    assert_eq!(
        state.now, 1470,
        "all forty milliseconds of queued drain must finish"
    );
    assert_eq!(
        state.flush_ticks, 1,
        "drain polls keep cancellation observable"
    );
}

#[test]
fn a_slow_drain_cannot_extend_the_record_deadline() {
    // Arrange
    let _exclusive = EXCLUSIVE.lock().expect("exclusive driver fixture");
    *STATE.lock().expect("test state") = State {
        now: 0,
        write_ms: 11,
        maybe_write_clock: None,
        flush_ms: 3000,
        flush_ticks: 0,
        writes: 0,
        emitted: Vec::new(),
        revoke_on_write: false,
        revoke_on_flush: false,
        force_full: false,
    };
    // Act
    let result = usb_runtime::write_if(&vec![b'x'; 66560], || true);
    // Assert
    assert!(result.is_err());
    assert!(STATE.lock().expect("test state").now <= 2000);
}

#[test]
fn revocation_stops_future_native_chunks_and_resynchronizes_before_new_output() {
    // Arrange
    let _exclusive = EXCLUSIVE.lock().expect("exclusive driver fixture");
    ADMITTED.store(true, Ordering::Release);
    *STATE.lock().expect("state") = State {
        now: 0,
        write_ms: 1,
        maybe_write_clock: None,
        flush_ms: 0,
        flush_ticks: 0,
        writes: 0,
        emitted: Vec::new(),
        revoke_on_write: true,
        revoke_on_flush: false,
        force_full: false,
    };
    // Act
    let result = usb_runtime::write_if(&vec![b'x'; 1024], || ADMITTED.load(Ordering::Acquire));
    // Assert
    let failure = result.expect_err("revoked output");
    let failure = failure
        .downcast_ref::<usb_runtime::WriteFailure>()
        .expect("closed failure");
    assert_eq!(failure.queued_bytes, 512);
    assert_eq!(STATE.lock().expect("state").writes, 1);
    // Act: a fresh session only appends a delimiter and its new complete record.
    ADMITTED.store(true, Ordering::Release);
    STATE.lock().expect("state").revoke_on_write = false;
    usb_runtime::resynchronize_if(&mut usb_runtime::measurement::Retained::new(), || {
        ADMITTED.load(Ordering::Acquire)
    })
    .expect("resynchronize");
    usb_runtime::write_if(b"{}\n", || true).expect("new record");
    // Assert
    let emitted = &STATE.lock().expect("state").emitted;
    assert_eq!(&emitted[512..], b"\n{}\n");
}

#[test]
fn a_full_native_queue_cannot_admit_a_retry_after_revocation() {
    // Arrange
    let _exclusive = EXCLUSIVE.lock().expect("exclusive driver fixture");
    ADMITTED.store(true, Ordering::Release);
    *STATE.lock().expect("state") = State {
        now: 0,
        write_ms: 1,
        maybe_write_clock: None,
        flush_ms: 0,
        flush_ticks: 0,
        writes: 0,
        emitted: Vec::new(),
        revoke_on_write: true,
        revoke_on_flush: false,
        force_full: true,
    };
    // Act
    let result = usb_runtime::write_if(b"{}\n", || ADMITTED.load(Ordering::Acquire));
    // Assert
    assert!(result.is_err());
    let state = STATE.lock().expect("state");
    assert_eq!(state.writes, 1);
    assert!(state.emitted.is_empty());
}

#[test]
fn complete_queued_record_cancelled_during_flush_is_not_a_partial_prefix() {
    // Arrange
    let _exclusive = EXCLUSIVE.lock().expect("exclusive driver fixture");
    ADMITTED.store(true, Ordering::Release);
    *STATE.lock().expect("state") = State {
        now: 0,
        write_ms: 1,
        maybe_write_clock: None,
        flush_ms: 20,
        flush_ticks: 0,
        writes: 0,
        emitted: Vec::new(),
        revoke_on_write: false,
        revoke_on_flush: true,
        force_full: false,
    };
    // Act
    let result = usb_runtime::write_if(b"{}\n", || ADMITTED.load(Ordering::Acquire));
    // Assert
    let error = result.expect_err("cancelled during bounded flush");
    let failure = error
        .downcast_ref::<usb_runtime::WriteFailure>()
        .expect("closed evidence");
    assert_eq!(failure.queued_bytes, failure.record_bytes);
    assert!(!usb_runtime::has_partial_output());
    assert_eq!(STATE.lock().expect("state").emitted, b"{}\n");
}

fn trace_fixture(revoke_on_write: bool) {
    ADMITTED.store(true, Ordering::Release);
    *STATE.lock().expect("test state") = State {
        now: 0,
        write_ms: 1,
        maybe_write_clock: None,
        flush_ms: 0,
        flush_ticks: 0,
        writes: 0,
        emitted: Vec::new(),
        revoke_on_write,
        revoke_on_flush: false,
        force_full: false,
    };
}

#[test]
fn real_partial_native_write_is_retained_after_successor_admission_traffic() {
    // Arrange
    use bitaxe_worker_control::serial::trace::{
        SerialTrace, SerialTraceCorrelation, SerialTraceStage,
    };
    let _exclusive = EXCLUSIVE.lock().expect("exclusive driver fixture");
    trace_fixture(true);
    let retained = SerialTrace::new();
    let request = SerialTraceCorrelation {
        epoch: 1,
        request_sequence: 19,
    };
    // Act
    let result = usb_runtime::write_observed_if(
        &[b'x'; 1024],
        || ADMITTED.load(Ordering::Acquire),
        |observed| trace::observe_in(&retained, request, observed),
    );
    for time in 10..110 {
        retained.record(
            SerialTraceCorrelation {
                epoch: 2,
                request_sequence: 1,
            },
            SerialTraceStage::Validated,
            time,
            300,
            0,
        );
    }
    // Assert
    assert!(result.is_err());
    let old = retained.snapshot().previous.expect("old epoch retained");
    assert_eq!(old.events.len(), 2);
    assert_eq!(old.events[0].stage, SerialTraceStage::WriterQueued);
    assert_eq!(old.events[1].stage, SerialTraceStage::WriterAbandoned);
    assert_eq!(old.events[1].queued_bytes, 512);
    assert_eq!(old.events[1].wire_bytes, 1024);
    assert_eq!(STATE.lock().expect("state").writes, 1);
    usb_runtime::resynchronize_if(&mut usb_runtime::measurement::Retained::new(), || true)
        .expect("fixture partial-line cleanup");
}

#[test]
fn real_native_tx_completion_remains_visible_after_a_fresh_hello() {
    // Arrange
    use bitaxe_worker_control::serial::trace::{
        SerialTrace, SerialTraceCorrelation, SerialTraceStage,
    };
    let _exclusive = EXCLUSIVE.lock().expect("exclusive driver fixture");
    trace_fixture(false);
    let retained = SerialTrace::new();
    let request = SerialTraceCorrelation {
        epoch: 1,
        request_sequence: 23,
    };
    // Act
    usb_runtime::write_observed_if(
        b"{}\n",
        || ADMITTED.load(Ordering::Acquire),
        |observed| trace::observe_in(&retained, request, observed),
    )
    .expect("native complete");
    retained.record(
        SerialTraceCorrelation {
            epoch: 2,
            request_sequence: 0,
        },
        SerialTraceStage::Hello,
        10,
        300,
        0,
    );
    // Assert
    let old = retained.snapshot().previous.expect("completed prior epoch");
    assert_eq!(old.events.len(), 2);
    assert_eq!(old.events[1].stage, SerialTraceStage::WriterCompleted);
    assert_eq!(old.events[1].queued_bytes, 3);
    assert_eq!(old.events[1].wire_bytes, 3);
}

#[test]
fn sub_tick_drain_readiness_does_not_timeout_at_1991_ms() {
    let _exclusive = EXCLUSIVE.lock().expect("exclusive fixture");
    trace_fixture(false);
    {
        let mut state = STATE.lock().expect("state");
        state.write_ms = 1991;
        state.flush_ms = 8;
    }
    let result = usb_runtime::write_if(&[b'x'; 92], || true);
    assert!(
        result.is_ok(),
        "ready below the unchanged 2000ms deadline: {result:?}"
    );
    assert_eq!(STATE.lock().expect("state").now, 1999);
}
#[test]
fn queued_92_bytes_with_slow_drain_is_not_delivery() {
    let _exclusive = EXCLUSIVE.lock().expect("exclusive fixture");
    trace_fixture(false);
    STATE.lock().expect("state").flush_ms = 3000;
    let error = usb_runtime::write_if(&[b'x'; 92], || true).expect_err("drain timeout");
    let failure = error
        .downcast_ref::<usb_runtime::WriteFailure>()
        .expect("native failure");
    assert_eq!(failure.queued_bytes, 92);
    assert_eq!(failure.elapsed_ms, 2000);
}

#[test]
fn drain_readiness_at_deadline_passes_but_after_deadline_fails() {
    // Arrange
    let _exclusive = EXCLUSIVE.lock().expect("exclusive fixture");
    for (ready_ms, succeeds) in [(1999, true), (2000, true), (2001, false), (3000, false)] {
        trace_fixture(false);
        STATE.lock().expect("state").flush_ms = ready_ms - 1;
        // Act
        let result = usb_runtime::write_if(&[b'x'; 92], || true);
        // Assert
        assert_eq!(result.is_ok(), succeeds, "ready at {ready_ms}: {result:?}");
        assert!(STATE.lock().expect("state").now <= 2000);
    }
}

#[test]
fn production_failed_bootstrap_retention_survives_hello_and_confirmed_replay() {
    // Arrange
    use usb_runtime::measurement::{Category, RecordKind};
    let _exclusive = EXCLUSIVE.lock().expect("exclusive fixture");
    trace_fixture(false);
    let mut retained = usb_runtime::measurement::Retained::new();
    STATE.lock().expect("state").flush_ms = 3000;
    // Act
    usb_runtime::write_measured_if(
        &mut retained,
        &[b'x'; 92],
        || true,
        |_| {},
        Category::BootstrapDiagnostic,
        RecordKind::StartupProgress,
    )
    .expect_err("native timeout");
    STATE.lock().expect("state").flush_ms = 0;
    usb_runtime::write_measured_if(
        &mut retained,
        b"{}\n",
        || true,
        |_| {},
        Category::Hello,
        RecordKind::Protocol,
    )
    .expect("hello completion");
    let bootstrap = retained.marker().expect("retained bootstrap");
    usb_runtime::write_measured_if(
        &mut retained,
        bootstrap.as_bytes(),
        || true,
        |_| {},
        Category::DiagnosticReplay,
        RecordKind::TxObservation,
    )
    .expect("replay drain");
    let failure = retained.marker().expect("retained failure");
    // Assert
    assert!(bootstrap.contains("category=bootstrap_diagnostic record_kind=startup_progress outcome=failed stage=flush_timeout"));
    assert!(bootstrap
        .contains("end_ms=2000 record_bytes=92 queued_bytes=92 queue_calls=1 queue_positive=1"));
    assert!(bootstrap.contains("drain_success=0"));
    assert!(failure.contains("slot=first_failure category=bootstrap_diagnostic"));
    assert!(failure.contains("actual_failures=1 replay_attempts=1 replay_completed=1 flags=0"));
}

#[test]
fn regressed_clock_stops_native_output_without_retry_or_drain() {
    // Arrange
    use usb_runtime::measurement::{Category, RecordKind, Retained};
    let _exclusive = EXCLUSIVE.lock().expect("exclusive fixture");
    trace_fixture(false);
    {
        let mut state = STATE.lock().expect("state");
        state.now = 100;
        state.maybe_write_clock = Some(50);
    }
    let mut retained = Retained::new();
    // Act
    let result = usb_runtime::write_measured_if(
        &mut retained,
        &[b'x'; 1024],
        || true,
        |_| {},
        Category::BootstrapDiagnostic,
        RecordKind::BootIdentity,
    );
    // Assert
    assert!(result.is_err());
    assert_eq!(STATE.lock().expect("state").writes, 1);
    let marker = retained.marker().expect("first bootstrap");
    assert!(marker.contains("drain_calls=0"));
    assert!(marker.contains("flags=2"));
    STATE.lock().expect("state").maybe_write_clock = None;
    usb_runtime::resynchronize_if(&mut retained, || true).expect("fixture partial cleanup");
}

#[test]
fn trace_callback_delay_cannot_change_confirmed_terminal_timestamp() {
    // Arrange
    use usb_runtime::measurement::{Category, RecordKind, Retained};
    let _exclusive = EXCLUSIVE.lock().expect("exclusive fixture");
    trace_fixture(false);
    STATE.lock().expect("state").flush_ms = 1999;
    let mut retained = Retained::new();
    // Act
    usb_runtime::write_measured_if(
        &mut retained,
        &[b'x'; 92],
        || true,
        |observation| {
            if observation.stage == usb_runtime::WriteObservationStage::Completed {
                STATE.lock().expect("state").now += 100;
            }
        },
        Category::BootstrapDiagnostic,
        RecordKind::StartupProgress,
    )
    .expect("on-time drain");
    // Assert
    assert_eq!(STATE.lock().expect("state").now, 2100);
    assert!(retained
        .marker()
        .expect("bootstrap")
        .contains("end_ms=2000"));
}
