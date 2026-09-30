//! Physical scheduling adapter for the shared bounded runtime wait.
use std::{thread, time::Duration};

pub(super) struct ProductionClock;

impl bitaxe_runtime::clock::Clock for ProductionClock {
    fn now_ms(&self) -> u64 {
        crate::runtime_uptime::millis()
    }

    fn sleep_ms(&mut self, duration_ms: u64) {
        thread::sleep(Duration::from_millis(duration_ms));
    }
}
