//! Exercises the production reconnect preparation/activation seam with synthetic subscriptions.
#![allow(dead_code)]
#[path = "wifi_adapter/reconnect/preparation.rs"]
mod preparation;
#[path = "prepared_thread.rs"]
mod prepared_thread;
use bitaxe_core::wifi_reconnect::WifiReconnectEvent;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Mutex};
static TEST_LOCK: Mutex<()> = Mutex::new(());
static RUNS: AtomicU32 = AtomicU32::new(0);
static SINK: Mutex<Option<(mpsc::Sender<WifiReconnectEvent>, mpsc::Sender<()>)>> = Mutex::new(None);
fn run(receiver: mpsc::Receiver<WifiReconnectEvent>) {
    RUNS.fetch_add(1, Ordering::Relaxed);
    let (events, exited) = SINK
        .lock()
        .expect("sink")
        .as_ref()
        .expect("installed sink")
        .clone();
    for event in receiver {
        events.send(event).expect("event receiver");
    }
    exited.send(()).expect("exit receiver");
}
