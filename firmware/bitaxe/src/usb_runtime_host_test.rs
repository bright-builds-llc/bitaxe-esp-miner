//! Exercises the production USB write loop against deterministic ESP-IDF call outcomes.
#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]
extern crate self as esp_idf_sys;
use std::sync::Mutex;

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
}
static STATE: Mutex<State> = Mutex::new(State {
    now: 0,
    write_ms: 11,
    flush_ms: 40,
    flush_ticks: 0,
});
static EXCLUSIVE: Mutex<()> = Mutex::new(());
pub unsafe fn usb_serial_jtag_driver_install(_: *mut usb_serial_jtag_driver_config_t) -> i32 {
    ESP_OK
}
pub unsafe fn usb_serial_jtag_read_bytes(_: *mut std::ffi::c_void, _: u32, _: u32) -> i32 {
    0
}
pub unsafe fn usb_serial_jtag_write_bytes(_: *const std::ffi::c_void, size: usize, _: u32) -> i32 {
    let mut state = STATE.lock().expect("test state");
    state.now += state.write_ms;
    size as i32
}
pub unsafe fn usb_serial_jtag_wait_tx_done(ticks: u32) -> i32 {
    let mut state = STATE.lock().expect("test state");
    state.flush_ticks = ticks;
    let budget = u64::from(ticks) * 10;
    state.now += state.flush_ms.min(budget);
    if budget >= state.flush_ms {
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
    };
    // Act
    let result = usb_runtime::write(&vec![b'x'; 66560]);
    // Assert
    assert!(
        result.is_ok(),
        "a complete record and its 40ms drain fit within 2000ms: {result:?}"
    );
    let state = STATE.lock().expect("test state");
    assert!(state.now <= 2000);
    assert!(state.flush_ticks >= 4);
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
    };
    // Act
    let result = usb_runtime::write(&vec![b'x'; 66560]);
    // Assert
    assert!(result.is_err());
    assert!(STATE.lock().expect("test state").now <= 2000);
}
