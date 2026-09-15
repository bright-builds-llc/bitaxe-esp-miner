//! Exercises the actual USB link preparation factory without any native USB operations.
#![allow(dead_code)]
#[path = "bwg_worker_usb/prepared_link.rs"]
mod prepared_link;
#[path = "prepared_thread.rs"]
mod prepared_thread;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
static TEST_LOCK: Mutex<()> = Mutex::new(());
static READS: AtomicU32 = AtomicU32::new(0);
mod link {
    pub fn run() {
        super::READS.fetch_add(1, super::Ordering::Relaxed);
    }
}
