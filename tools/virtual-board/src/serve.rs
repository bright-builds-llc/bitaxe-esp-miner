//! A real-time virtual device on stdin/stdout for an external serial host.
//!
//! stdout carries only `bwg-worker-serial/0.2` bytes; diagnostics go to stderr. The
//! capability and the deployment trust that verifies leases are runtime inputs, not
//! compiled in, and their digests are recorded in the optional outcome file, which
//! holds categories and counts only.

use anyhow::{Context, Result};
use bitaxe_simulation::{PortInput, StdioConfig, StdioDevice, StdioFault, StdioPort};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::{Read, StdoutLock, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

const OUTCOME_SCHEMA: &str = "bitaxe_virtual_serial_outcome_v2";
/// The device loop's idle cadence, like the firmware writer's output wait.
const STEP: Duration = Duration::from_millis(10);

/// What the device is served with.
pub struct ServeInputs<'a> {
    pub seed: u64,
    pub capability: &'a Path,
    pub trust: &'a Path,
    pub maybe_fault: Option<Fault>,
    pub maybe_outcome: Option<&'a Path>,
}

/// Faults a host-facing scenario may inject.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Fault {
    /// The first restoration of a started lease fails its cooling proof.
    RejectRestore,
}

/// Serves one logical session; `Ok(true)` when it was admitted, restored and closed cleanly.
pub fn serve(inputs: &ServeInputs<'_>) -> Result<bool> {
    let capability_bytes = std::fs::read(inputs.capability).context("capability unreadable")?;
    let capability: Value =
        serde_json::from_slice(&capability_bytes).context("capability is not JSON")?;
    let trust_json = std::fs::read_to_string(inputs.trust).context("trust unreadable")?;
    let host = Rc::new(RefCell::new(Host {
        input: spawn_stdin_reader(),
        started: Instant::now(),
        stdout: std::io::stdout().lock(),
        closed: false,
        gone: false,
    }));
    let config = StdioConfig {
        seed: inputs.seed,
        capability,
        trust_json: trust_json.clone(),
        maybe_fault: inputs
            .maybe_fault
            .map(|Fault::RejectRestore| StdioFault::RejectRestore),
    };
    let mut device = StdioDevice::new(config, Box::new(SharedHost(Rc::clone(&host))))?;
    loop {
        let (chunk, closed, wall_ms) = host.borrow_mut().next();
        let output = if closed {
            device.input_closed(wall_ms)
        } else {
            device.step(&chunk, wall_ms)
        };
        let mut host = host.borrow_mut();
        // A host that stopped reading is gone; end the session as the transport would.
        if !host.write(&output) && !closed {
            drop(host);
            device.input_closed(wall_ms);
            break;
        }
        if closed {
            break;
        }
    }
    let outcome = device.outcome();
    if let Some(path) = inputs.maybe_outcome {
        let record = json!({
            "schema": OUTCOME_SCHEMA,
            "seed": inputs.seed,
            "capability_sha256": hex(&Sha256::digest(&capability_bytes)),
            "trust_sha256": hex(&Sha256::digest(trust_json.as_bytes())),
            "fault": inputs.maybe_fault.map(|Fault::RejectRestore| "reject_restore"),
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

/// stdin and stdout, read by the device loop between steps and by the device's link
/// and writer while its owner actuates.
struct Host {
    input: mpsc::Receiver<Option<Vec<u8>>>,
    started: Instant,
    stdout: StdoutLock<'static>,
    closed: bool,
    gone: bool,
}

impl Host {
    fn wall_ms(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// The next stdin chunk within one idle step: bytes, end of input, and the instant.
    fn next(&mut self) -> (Vec<u8>, bool, u64) {
        if self.closed || self.gone {
            return (Vec::new(), true, self.wall_ms());
        }
        let (chunk, closed) = match self.input.recv_timeout(STEP) {
            Ok(Some(chunk)) => (chunk, false),
            Ok(None) | Err(RecvTimeoutError::Disconnected) => (Vec::new(), true),
            Err(RecvTimeoutError::Timeout) => (Vec::new(), false),
        };
        self.closed |= closed;
        (chunk, closed, self.wall_ms())
    }

    fn write(&mut self, bytes: &[u8]) -> bool {
        if self.gone {
            return false;
        }
        if bytes.is_empty() {
            return true;
        }
        self.gone = self
            .stdout
            .write_all(bytes)
            .and_then(|()| self.stdout.flush())
            .is_err();
        !self.gone
    }
}

struct SharedHost(Rc<RefCell<Host>>);

impl StdioPort for SharedHost {
    fn input_until(&mut self, wall_ms: u64) -> PortInput {
        let mut host = self.0.borrow_mut();
        let mut input = PortInput::default();
        loop {
            if host.closed {
                input.closed = true;
                return input;
            }
            let remaining = wall_ms.saturating_sub(host.wall_ms());
            let received = if remaining == 0 {
                host.input.try_recv().map_err(|error| match error {
                    mpsc::TryRecvError::Empty => RecvTimeoutError::Timeout,
                    mpsc::TryRecvError::Disconnected => RecvTimeoutError::Disconnected,
                })
            } else {
                host.input.recv_timeout(Duration::from_millis(remaining))
            };
            match received {
                Ok(Some(chunk)) => input.bytes.extend(chunk),
                Ok(None) | Err(RecvTimeoutError::Disconnected) => host.closed = true,
                Err(RecvTimeoutError::Timeout) if remaining == 0 => return input,
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }

    fn output(&mut self, bytes: &[u8]) -> bool {
        self.0.borrow_mut().write(bytes)
    }
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
