//! A real-time virtual device on stdin/stdout for an external serial host.
//!
//! stdout carries only `bwg-worker-serial/0.2` bytes; diagnostics go to stderr. The
//! capability is a runtime input, not compiled in, and its digest is recorded in the
//! optional outcome file, which holds categories and counts only.

use anyhow::{Context, Result};
use bitaxe_simulation::StdioDevice;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

const OUTCOME_SCHEMA: &str = "bitaxe_virtual_serial_outcome_v1";
/// The device loop's idle cadence, like the firmware writer's output wait.
const STEP: Duration = Duration::from_millis(10);

/// Serves one logical session; `Ok(true)` when it was admitted and closed cleanly.
pub fn serve(seed: u64, capability_path: &Path, maybe_outcome: Option<&Path>) -> Result<bool> {
    let capability_bytes = std::fs::read(capability_path).context("capability unreadable")?;
    let capability: Value =
        serde_json::from_slice(&capability_bytes).context("capability is not JSON")?;
    let mut device = StdioDevice::new(seed, capability)?;
    let input = spawn_stdin_reader();
    let started = Instant::now();
    let mut stdout = std::io::stdout().lock();
    loop {
        let (chunk, closed) = match input.recv_timeout(STEP) {
            Ok(Some(chunk)) => (chunk, false),
            Ok(None) | Err(RecvTimeoutError::Disconnected) => (Vec::new(), true),
            Err(RecvTimeoutError::Timeout) => (Vec::new(), false),
        };
        let wall_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let output = if closed {
            device.input_closed(wall_ms)
        } else {
            device.step(&chunk, wall_ms)
        };
        // A host that stopped reading is gone; end the session as the transport would.
        let host_gone = !output.is_empty()
            && stdout
                .write_all(&output)
                .and_then(|()| stdout.flush())
                .is_err();
        if host_gone && !closed {
            device.input_closed(wall_ms);
        }
        if closed || host_gone {
            break;
        }
    }
    let outcome = device.outcome();
    if let Some(path) = maybe_outcome {
        let record = json!({
            "schema": OUTCOME_SCHEMA,
            "seed": seed,
            "capability_sha256": hex(&Sha256::digest(&capability_bytes)),
            "clean": outcome.clean(),
            "outcome": outcome,
            "hardware_qualified": false,
        });
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .context("outcome file must be new")?;
        file.write_all(&serde_json::to_vec_pretty(&record)?)?;
        file.sync_all()?;
    }
    Ok(outcome.clean())
}

/// Reads stdin on its own thread; `None` reports end of input.
fn spawn_stdin_reader() -> mpsc::Receiver<Option<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin().lock();
        let mut buffer = [0u8; 4096];
        loop {
            let message = match stdin.read(&mut buffer) {
                Ok(0) | Err(_) => None,
                Ok(count) => Some(buffer[..count].to_vec()),
            };
            let end = message.is_none();
            // The device loop dropped its receiver only after it already stopped.
            if sender.send(message).is_err() || end {
                return;
            }
        }
    });
    receiver
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
