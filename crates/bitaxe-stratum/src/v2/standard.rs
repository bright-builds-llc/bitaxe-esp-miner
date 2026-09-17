//! Fixed, single-job Standard session used by the serial qualification owners.
//! This layer narrows the reusable protocol without granting device authority.
use super::{
    frame::Frame,
    messages::{ChannelKind, MessageType, ServerMessage, SubmitSharesStandard},
    session::{SessionConfig, SessionEvent, V2Session},
    work::V2MiningWork,
    StratumV2Error,
};
use bitaxe_asic::bm1366::{result::Bm1366NonceResult, work::Bm1366JobId};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const BASE_VERSION: u32 = 0x2000_0000;
pub const VERSION_MASK: u32 = 0x1fff_e000;
pub const NBITS: u32 = 0x207f_ffff;
pub const TARGET: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xc0, 0xff, 0x3f, 0, 0,
    0, 0, 0,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejected {
    Protocol,
    Channel,
    Job,
    Nonce,
    Target,
    Capacity,
    Acknowledgement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Acknowledgement {
    pub channel_id: u32,
    pub last_sequence: u32,
    pub accepted_count: u32,
    pub shares_sum: u64,
    pub matched_submit_count: u32,
}
pub enum StandardEvent {
    Send(Frame),
    Setup,
    Channel,
    Job,
    Target,
    Work {
        work: V2MiningWork,
        commitment: String,
    },
    Accepted(Acknowledgement),
}
/// One retained job and at most16 submitted candidates; no work regeneration.
pub struct StandardSession {
    session: V2Session,
    phase: u8,
    maybe_channel: Option<u32>,
    maybe_job: Option<u32>,
    maybe_work: Option<V2MiningWork>,
    commitment: Sha256,
    pending: BTreeMap<u32, SubmitSharesStandard>,
    written: BTreeSet<u32>,
    submitted: usize,
    dispatched: bool,
}
impl StandardSession {
    pub fn new(config: SessionConfig) -> Result<Self, StratumV2Error> {
        if config.channel_kind != ChannelKind::Standard {
            return Err(StratumV2Error::InvalidField {
                field: "channel",
                reason: "standard required",
            });
        }
        Ok(Self {
            session: V2Session::new(config)?,
            phase: 0,
            maybe_channel: None,
            maybe_job: None,
            maybe_work: None,
            commitment: Sha256::new(),
            pending: BTreeMap::new(),
            written: BTreeSet::new(),
            submitted: 0,
            dispatched: false,
        })
    }
    pub fn start(&mut self) -> Result<Frame, Rejected> {
        let SessionEvent::Outbound(frame) = self.session.start().map_err(|_| Rejected::Protocol)?
        else {
            return Err(Rejected::Protocol);
        };
        // The qualified Standard profile also requests version rolling. The
        // historical general-session flags remain unchanged for old callers.
        let mut payload = frame.payload().to_vec();
        payload
            .get_mut(5..9)
            .ok_or(Rejected::Protocol)?
            .copy_from_slice(&5u32.to_le_bytes());
        Frame::new(0, MessageType::SetupConnection as u8, payload).map_err(|_| Rejected::Protocol)
    }
    pub fn receive(&mut self, frame: &Frame) -> Result<Vec<StandardEvent>, Rejected> {
        let kind =
            MessageType::try_from(frame.header.message_type).map_err(|_| Rejected::Protocol)?;
        let extension = match kind {
            MessageType::SetupConnectionSuccess | MessageType::OpenStandardMiningChannelSuccess => {
                0
            }
            MessageType::NewMiningJob
            | MessageType::SetTarget
            | MessageType::SetNewPrevHash
            | MessageType::SubmitSharesSuccess => 0x8000,
            _ => return Err(Rejected::Protocol),
        };
        if frame.header.extension_type != extension {
            return Err(Rejected::Protocol);
        }
        let message = ServerMessage::decode(frame).map_err(|_| Rejected::Protocol)?;
        let mut result = Vec::new();
        let mut maybe_ack = None;
        match (&message, self.phase) {
            (ServerMessage::SetupConnectionSuccess(s), 0)
                if s.used_version == 2 && s.flags == 0 =>
            {
                result.push(StandardEvent::Setup)
            }
            (ServerMessage::OpenStandardMiningChannelSuccess(s), 1)
                if s.request_id == 1
                    && s.channel_id != 0
                    && s.target == TARGET
                    && s.extranonce_prefix.is_empty() =>
            {
                self.maybe_channel = Some(s.channel_id);
                result.push(StandardEvent::Channel);
            }
            (ServerMessage::NewMiningJob(j), 2)
                if Some(j.channel_id) == self.maybe_channel
                    && j.job_id != 0
                    && j.maybe_min_ntime.is_none()
                    && j.version == BASE_VERSION =>
            {
                self.maybe_job = Some(j.job_id);
                result.push(StandardEvent::Job);
            }
            (ServerMessage::SetTarget(t), 3)
                if Some(t.channel_id) == self.maybe_channel && t.maximum_target == TARGET =>
            {
                result.push(StandardEvent::Target)
            }
            (ServerMessage::SetNewPrevHash(p), 4)
                if Some(p.channel_id) == self.maybe_channel
                    && Some(p.job_id) == self.maybe_job
                    && p.nbits == NBITS
                    && p.min_ntime != 0 => {}
            (ServerMessage::SubmitSharesSuccess(a), 5) => {
                if Some(a.channel_id) != self.maybe_channel
                    || a.accepted_count != 1
                    || a.shares_sum != 1024
                    || !self.pending.contains_key(&a.last_sequence_number)
                    || self.pending.range(..=a.last_sequence_number).count() != 1
                    || !self.written.contains(&a.last_sequence_number)
                {
                    return Err(Rejected::Acknowledgement);
                }
                maybe_ack = Some(Acknowledgement {
                    channel_id: a.channel_id,
                    last_sequence: a.last_sequence_number,
                    accepted_count: a.accepted_count,
                    shares_sum: a.shares_sum,
                    matched_submit_count: 1,
                });
            }
            _ => return Err(Rejected::Protocol),
        }
        if (1..=4).contains(&self.phase) {
            let encoded = frame.encode();
            self.commitment.update((encoded.len() as u32).to_le_bytes());
            self.commitment.update(encoded);
        }
        let events = self
            .session
            .handle(message)
            .map_err(|_| Rejected::Protocol)?;
        for event in events {
            match event {
                SessionEvent::Outbound(frame) => result.push(StandardEvent::Send(frame)),
                SessionEvent::Work(work) => {
                    if self.maybe_work.is_some() || self.phase != 4 {
                        return Err(Rejected::Job);
                    }
                    let commitment = self
                        .commitment
                        .clone()
                        .finalize()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect();
                    self.maybe_work = Some(work.clone());
                    result.push(StandardEvent::Work { work, commitment });
                }
                SessionEvent::ChannelReady { .. }
                | SessionEvent::TargetUpdated { .. }
                | SessionEvent::ShareAccepted { .. } => {}
                _ => return Err(Rejected::Protocol),
            }
        }
        if let Some(ack) = maybe_ack {
            self.pending.remove(&ack.last_sequence);
            self.written.remove(&ack.last_sequence);
            result.push(StandardEvent::Accepted(ack));
        } else {
            self.phase += 1;
        }
        Ok(result)
    }
    pub fn maybe_work(&self) -> Option<&V2MiningWork> {
        self.maybe_work.as_ref()
    }
    pub fn dispatched(&mut self, id: Bm1366JobId) -> Result<(), Rejected> {
        if self.dispatched || self.maybe_work.as_ref().is_none_or(|w| w.asic_job_id != id) {
            return Err(Rejected::Job);
        }
        self.dispatched = true;
        Ok(())
    }
    pub fn nonce(
        &mut self,
        result: Bm1366NonceResult,
    ) -> Result<Option<(Frame, SubmitSharesStandard)>, Rejected> {
        if !self.dispatched || result.version_bits & !VERSION_MASK != 0 {
            return Err(Rejected::Nonce);
        }
        if self.submitted >= 16 {
            return Err(Rejected::Capacity);
        }
        let maybe_event = self
            .session
            .observe_nonce(result)
            .map_err(|_| Rejected::Nonce)?;
        let Some(SessionEvent::Outbound(frame)) = maybe_event else {
            return Ok(None);
        };
        let super::messages::ClientMessage::SubmitSharesStandard(submit) =
            super::messages::ClientMessage::decode(&frame).map_err(|_| Rejected::Protocol)?
        else {
            return Err(Rejected::Protocol);
        };
        self.pending.insert(submit.sequence_number, submit);
        self.submitted += 1;
        Ok(Some((frame, submit)))
    }
    pub fn written(&mut self, sequence: u32) -> Result<(), Rejected> {
        if !self.pending.contains_key(&sequence) || !self.written.insert(sequence) {
            Err(Rejected::Acknowledgement)
        } else {
            Ok(())
        }
    }
    pub fn stop(&mut self) {
        self.session.stop();
        self.pending.clear();
        self.written.clear();
        self.dispatched = false;
    }
}

#[cfg(test)]
mod tests;
