use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::Serialize;

const MAX_CANDIDATES: usize = 3;
const MAX_PAYLOAD_BYTES: usize = 65;
const POST_MATCH_OBSERVATION: Duration = Duration::from_millis(500);
const RECEIPT_ACK: u8 = 0xa5;
const EXPECTED: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
];

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct TcpPayloadCandidateProgress {
    pub(super) remote_port: u16,
    pub(super) payload_bytes_received: u16,
    pub(super) payload_read_category: &'static str,
    pub(super) payload_digest_match: bool,
    pub(super) extra_bytes_received: u16,
    pub(super) receipt_ack_sent: bool,
}

pub(super) struct TcpPayloadInventory {
    pub(super) candidates: Vec<TcpPayloadCandidateProgress>,
    pub(super) unexpected_peer_count: u16,
    pub(super) candidate_overflow: bool,
    pub(super) matched_index: Option<usize>,
}

struct Candidate {
    stream: TcpStream,
    progress: TcpPayloadCandidateProgress,
    bytes: [u8; MAX_PAYLOAD_BYTES],
    received: usize,
    terminal: bool,
}

impl Candidate {
    fn new(stream: TcpStream, remote_port: u16) -> Result<Self> {
        stream
            .set_nonblocking(true)
            .context("set payload candidate nonblocking")?;
        Ok(Self {
            stream,
            progress: TcpPayloadCandidateProgress {
                remote_port,
                ..TcpPayloadCandidateProgress::default()
            },
            bytes: [0; MAX_PAYLOAD_BYTES],
            received: 0,
            terminal: false,
        })
    }

    fn poll(&mut self) -> Result<bool> {
        if self.terminal {
            return Ok(false);
        }
        loop {
            match self.stream.read(&mut self.bytes[self.received..]) {
                Ok(0) => return self.finish_eof(),
                Ok(count) => {
                    self.received += count;
                    self.progress.payload_bytes_received =
                        self.received.min(64).try_into().unwrap_or(64);
                    if self.received > 64 {
                        self.progress.extra_bytes_received =
                            (self.received - 64).try_into().unwrap_or(u16::MAX);
                        self.progress.payload_read_category = "extra";
                        self.terminal = true;
                        return Ok(false);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    self.progress.payload_read_category = "io";
                    self.terminal = true;
                    return Ok(false);
                }
            }
        }
    }

    fn finish_eof(&mut self) -> Result<bool> {
        self.terminal = true;
        if self.received != EXPECTED.len() {
            self.progress.payload_read_category = "eof";
            return Ok(false);
        }
        self.progress.payload_digest_match = self.bytes[..EXPECTED.len()] == EXPECTED;
        if !self.progress.payload_digest_match {
            self.progress.payload_read_category = "mismatch";
            return Ok(false);
        }
        self.progress.payload_read_category = "complete";
        self.stream
            .set_nonblocking(false)
            .context("restore payload candidate blocking")?;
        self.stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .context("set payload receipt timeout")?;
        self.stream
            .write_all(&[RECEIPT_ACK])
            .context("write payload receipt")?;
        self.stream.flush().context("flush payload receipt")?;
        self.progress.receipt_ack_sent = true;
        Ok(true)
    }

    fn timeout(&mut self) {
        if self.terminal {
            return;
        }
        self.progress.payload_bytes_received = self.received.try_into().unwrap_or(u16::MAX);
        self.progress.payload_read_category = "timeout";
        self.terminal = true;
    }

    fn observation_end(&mut self) {
        if self.terminal {
            return;
        }
        self.progress.payload_bytes_received = self.received.try_into().unwrap_or(u16::MAX);
        self.progress.payload_read_category = "observation_end";
        self.terminal = true;
    }
}

pub(super) fn inventory_tcp_payload(
    listener: &TcpListener,
    accept_timeout: Duration,
    read_timeout: Duration,
    maybe_expected_peer: Option<IpAddr>,
) -> Result<TcpPayloadInventory> {
    let accept_deadline = Instant::now() + accept_timeout;
    let mut read_deadline = None;
    let mut candidates = Vec::<Candidate>::new();
    let mut unexpected_peer_count = 0_u16;
    let mut candidate_overflow = false;
    let mut matched_index = None;
    let mut matched_at = None;

    loop {
        loop {
            match listener.accept() {
                Ok((stream, peer))
                    if maybe_expected_peer.is_none_or(|expected| peer.ip() == expected) =>
                {
                    if candidates.len() >= MAX_CANDIDATES {
                        candidate_overflow = true;
                        drop(stream);
                        continue;
                    }
                    if read_deadline.is_none() {
                        read_deadline = Some(Instant::now() + read_timeout);
                    }
                    candidates.push(Candidate::new(stream, peer.port())?);
                }
                Ok((_stream, _peer)) => {
                    unexpected_peer_count = unexpected_peer_count.saturating_add(1);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error).context("accept payload candidate"),
            }
        }

        for (index, candidate) in candidates.iter_mut().enumerate() {
            if candidate.poll()? && matched_index.is_none() {
                matched_index = Some(index);
                matched_at = Some(Instant::now());
            }
        }
        if candidate_overflow {
            for candidate in &mut candidates {
                candidate.observation_end();
            }
            break;
        }
        let now = Instant::now();
        if matched_at.is_some_and(|observed| now >= observed + POST_MATCH_OBSERVATION) {
            for candidate in &mut candidates {
                candidate.observation_end();
            }
            break;
        }
        if let Some(deadline) = read_deadline {
            if now >= deadline {
                for candidate in &mut candidates {
                    candidate.timeout();
                }
                break;
            }
        } else if now >= accept_deadline {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    Ok(TcpPayloadInventory {
        candidates: candidates
            .into_iter()
            .map(|candidate| candidate.progress)
            .collect(),
        unexpected_peer_count,
        candidate_overflow,
        matched_index,
    })
}
