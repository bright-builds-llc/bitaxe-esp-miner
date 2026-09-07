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
    let elapsed = state.flush_ms.min(budget);
    state.now += elapsed;
    state.flush_ms -= elapsed;
    if state.flush_ms == 0 {
        ESP_OK
    } else {
        -1
    }
}
mod runtime_uptime {
    pub fn millis() -> u64 {
        super::STATE.lock().expect("test state").now
    }
}
#[path = "usb_runtime.rs"]
mod usb_runtime;

#[test]
fn maximum_record_can_flush_within_the_existing_total_write_budget() {
    // Arrange
    let _exclusive = EXCLUSIVE.lock().expect("exclusive driver fixture");
    *STATE.lock().expect("test state") = State {
        now: 0,
        write_ms: 11,
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
    usb_runtime::resynchronize_if(|| ADMITTED.load(Ordering::Acquire)).expect("resynchronize");
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
