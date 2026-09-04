use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use super::{scan_candidates, ReceiveOnlyReader};

pub(crate) struct ReconnectingReceiveCapture {
    pub(crate) bytes: Vec<u8>,
    pub(crate) open_count: u16,
}

pub(crate) fn capture_reconnecting_receive_only(
    requested_port: &str,
    timeout: Duration,
) -> Result<ReconnectingReceiveCapture> {
    const MAX_CAPTURE_BYTES: usize = 64 * 1024;
    const POLL_INTERVAL: Duration = Duration::from_millis(50);

    let deadline = Instant::now() + timeout;
    let mut maybe_physical_identity = None;
    let mut maybe_reader: Option<ReceiveOnlyReader> = None;
    let mut bytes = Vec::new();
    let mut open_count = 0_u16;
    while Instant::now() < deadline {
        let candidates = match scan_candidates() {
            Ok(candidates) => candidates,
            Err(_) => {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
        };
        if maybe_physical_identity.is_none() {
            let requested = candidates
                .iter()
                .filter(|candidate| candidate.port == requested_port)
                .collect::<Vec<_>>();
            match requested.as_slice() {
                [candidate] => {
                    maybe_physical_identity = Some(candidate.physical_identity_digest.clone());
                }
                [] => {
                    thread::sleep(POLL_INTERVAL);
                    continue;
                }
                _ => bail!("multiple USB devices matched the reboot-loop port"),
            }
        }
        let expected = maybe_physical_identity
            .as_deref()
            .context("reboot-loop physical identity is absent")?;
        if candidates.iter().any(|candidate| {
            candidate.port == requested_port && candidate.physical_identity_digest != expected
        }) {
            bail!("reboot-loop physical identity drifted");
        }
        let matching = candidates
            .iter()
            .filter(|candidate| candidate.physical_identity_digest == expected)
            .collect::<Vec<_>>();
        let maybe_port = match matching.as_slice() {
            [] => None,
            [candidate] => Some(candidate.port.as_str()),
            _ => bail!("multiple USB profiles matched the reboot-loop connector"),
        };
        if maybe_reader
            .as_ref()
            .is_some_and(|reader| Some(reader.port()) != maybe_port)
        {
            maybe_reader = None;
        }
        if maybe_reader.is_none() {
            if let Some(port) = maybe_port {
                if let Ok(reader) = ReceiveOnlyReader::open(port) {
                    open_count = open_count.saturating_add(1);
                    maybe_reader = Some(reader);
                }
            }
        }
        if let Some(reader) = maybe_reader.as_mut() {
            match reader.read_available() {
                Ok(chunk) if bytes.len().saturating_add(chunk.len()) <= MAX_CAPTURE_BYTES => {
                    bytes.extend_from_slice(&chunk);
                }
                Ok(_) => bail!("reboot-loop capture exceeded its byte bound"),
                Err(_) => maybe_reader = None,
            }
        }
        thread::sleep(POLL_INTERVAL);
    }
    if maybe_physical_identity.is_none() {
        bail!("reboot-loop device never appeared");
    }
    Ok(ReconnectingReceiveCapture { bytes, open_count })
}
