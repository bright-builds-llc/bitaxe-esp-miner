use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use bitaxe_stratum::v2::frame::{FrameHeader, FRAME_HEADER_LEN};
use noise_sv2::{Responder, AEAD_MAC_LEN};
use rand::rngs::OsRng;
use serde::Serialize;

use super::FixtureProgress;

const ACT_ONE_BYTES: usize = 64;
const MAX_CANDIDATES: usize = 3;
const POST_MATCH_OBSERVATION: Duration = Duration::from_millis(500);
const DIAGNOSTIC_PROOF_EXTENSION: u16 = 0xffff;
const DIAGNOSTIC_PROOF_MESSAGE: u8 = 0xff;
const ENCRYPTED_HEADER_LEN: usize = FRAME_HEADER_LEN + AEAD_MAC_LEN;

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct NoiseAuthCandidateProgress {
    pub(super) remote_port: u16,
    pub(super) act_one_bytes_received: u16,
    pub(super) act_one_read_category: &'static str,
}

pub(super) struct SelectedNoiseAuthCandidate {
    pub(super) stream: TcpStream,
    pub(super) act_one: [u8; ACT_ONE_BYTES],
}

pub(super) struct NoiseAuthInventory {
    pub(super) candidates: Vec<NoiseAuthCandidateProgress>,
    pub(super) unexpected_peer_count: u16,
    pub(super) candidate_overflow: bool,
    pub(super) selected_index: Option<usize>,
    pub(super) selected: Option<SelectedNoiseAuthCandidate>,
}

struct Candidate {
    stream: TcpStream,
    progress: NoiseAuthCandidateProgress,
    bytes: [u8; ACT_ONE_BYTES + 1],
    received: usize,
    terminal: bool,
}

impl Candidate {
    fn new(stream: TcpStream, remote_port: u16) -> Result<Self> {
        stream
            .set_nonblocking(true)
            .context("set Noise candidate nonblocking")?;
        Ok(Self {
            stream,
            progress: NoiseAuthCandidateProgress {
                remote_port,
                ..NoiseAuthCandidateProgress::default()
            },
            bytes: [0; ACT_ONE_BYTES + 1],
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
                Ok(0) => {
                    self.finish("eof");
                    return Ok(false);
                }
                Ok(count) => {
                    self.received += count;
                    self.progress.act_one_bytes_received =
                        self.received.min(ACT_ONE_BYTES).try_into().unwrap_or(64);
                    if self.received > ACT_ONE_BYTES {
                        self.finish("extra");
                        return Ok(false);
                    }
                    if self.received == ACT_ONE_BYTES {
                        match self.stream.read(&mut self.bytes[ACT_ONE_BYTES..]) {
                            Ok(0) => self.finish("eof"),
                            Ok(_) => self.finish("extra"),
                            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                                self.finish("complete");
                                return Ok(true);
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {
                                continue;
                            }
                            Err(_) => self.finish("io"),
                        }
                        return Ok(false);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    self.finish("io");
                    return Ok(false);
                }
            }
        }
    }

    fn finish(&mut self, category: &'static str) {
        self.progress.act_one_bytes_received =
            self.received.min(ACT_ONE_BYTES).try_into().unwrap_or(64);
        self.progress.act_one_read_category = category;
        self.terminal = true;
    }

    fn observation_end(&mut self) {
        if !self.terminal {
            self.finish("observation_end");
        }
    }

    fn timeout(&mut self) {
        if !self.terminal {
            self.finish("timeout");
        }
    }

    fn select(self) -> Result<SelectedNoiseAuthCandidate> {
        self.stream
            .set_nonblocking(false)
            .context("restore Noise candidate blocking")?;
        let mut act_one = [0; ACT_ONE_BYTES];
        act_one.copy_from_slice(&self.bytes[..ACT_ONE_BYTES]);
        Ok(SelectedNoiseAuthCandidate {
            stream: self.stream,
            act_one,
        })
    }
}

pub(super) fn inventory_noise_auth(
    listener: &TcpListener,
    accept_timeout: Duration,
    read_timeout: Duration,
    maybe_expected_peer: Option<IpAddr>,
) -> Result<NoiseAuthInventory> {
    let accept_deadline = Instant::now() + accept_timeout;
    let mut read_deadline = None;
    let mut candidates = Vec::<Candidate>::new();
    let mut unexpected_peer_count = 0_u16;
    let mut candidate_overflow = false;
    let mut selected_index = None;
    let mut selected_at = None;

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
                    read_deadline.get_or_insert_with(|| Instant::now() + read_timeout);
                    candidates.push(Candidate::new(stream, peer.port())?);
                }
                Ok((_stream, _peer)) => {
                    unexpected_peer_count = unexpected_peer_count.saturating_add(1);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error).context("accept Noise candidate"),
            }
        }

        for (index, candidate) in candidates.iter_mut().enumerate() {
            if candidate.poll()? && selected_index.is_none() {
                selected_index = Some(index);
                selected_at = Some(Instant::now());
            }
        }
        let now = Instant::now();
        if candidate_overflow
            || selected_at.is_some_and(|observed| now >= observed + POST_MATCH_OBSERVATION)
        {
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

    let progress = candidates
        .iter()
        .map(|candidate| candidate.progress.clone())
        .collect();
    let selected = match selected_index {
        Some(index) => candidates
            .into_iter()
            .nth(index)
            .map(Candidate::select)
            .transpose()?,
        None => None,
    };
    Ok(NoiseAuthInventory {
        candidates: progress,
        unexpected_peer_count,
        candidate_overflow,
        selected_index,
        selected,
    })
}

pub(super) fn run_noise_auth_fixture(
    listener: &TcpListener,
    accept_timeout: Duration,
    read_timeout: Duration,
    maybe_expected_peer: Option<IpAddr>,
    authority_private: [u8; 32],
    authority_public: [u8; 32],
    progress: &mut FixtureProgress,
) -> Result<()> {
    let inventory =
        inventory_noise_auth(listener, accept_timeout, read_timeout, maybe_expected_peer)?;
    progress.connection_accepted = !inventory.candidates.is_empty();
    progress.peer_matched = progress.connection_accepted;
    progress.unexpected_peer_count = inventory.unexpected_peer_count;
    progress.exact_peer_connection_count =
        inventory.candidates.len().try_into().unwrap_or(u16::MAX);
    progress.candidate_overflow = inventory.candidate_overflow;
    progress.noise_candidates = inventory.candidates.clone();
    let Some(mut selected) = inventory.selected else {
        bail!("no complete Noise act-one candidate");
    };
    if inventory.selected_index.is_none()
        || inventory.candidates.len() != 1
        || inventory.candidate_overflow
    {
        bail!("Noise connection ownership is ambiguous");
    }
    selected
        .stream
        .set_read_timeout(Some(read_timeout))
        .context("set Noise read timeout")?;
    selected
        .stream
        .set_write_timeout(Some(read_timeout))
        .context("set Noise write timeout")?;
    progress.act_one_bytes_received = 64;
    progress.act_one_read_category = "complete";
    progress.act_one_received = true;
    let mut rng = OsRng;
    let mut responder = Responder::from_authority_kp_with_rng(
        &authority_public,
        &authority_private,
        Duration::from_secs(u32::MAX as u64),
        &mut rng,
    )
    .map_err(|_| anyhow::anyhow!("create Noise responder"))?;
    progress.responder_created = true;
    let (act_two, mut codec) = responder
        .step_1_with_now_rng(selected.act_one, 0, &mut rng)
        .map_err(|_| anyhow::anyhow!("produce Noise act two"))?;
    progress.act_two_created = true;
    selected
        .stream
        .write_all(&act_two)
        .context("write Noise act two")?;
    progress.act_two_sent = true;
    let mut encrypted_header = vec![0_u8; ENCRYPTED_HEADER_LEN];
    selected
        .stream
        .read_exact(&mut encrypted_header)
        .context("read encrypted diagnostic proof")?;
    codec
        .decrypt(&mut encrypted_header)
        .map_err(|_| anyhow::anyhow!("decrypt diagnostic proof"))?;
    let header = FrameHeader::parse(&encrypted_header)?;
    if header.extension_type != DIAGNOSTIC_PROOF_EXTENSION
        || header.message_type != DIAGNOSTIC_PROOF_MESSAGE
        || header.payload_len != 0
    {
        bail!("encrypted diagnostic proof mismatch");
    }
    progress.client_authenticated = true;
    progress.noise_authenticated = true;
    progress.encrypted_proof_exact = true;
    Ok(())
}
