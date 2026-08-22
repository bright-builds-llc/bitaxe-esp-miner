//! Pure Stratum V2 channel, job, and share lifecycle.

use std::array;
use std::collections::{HashMap, VecDeque};
use std::fmt;

use bitaxe_asic::bm1366::{result::Bm1366NonceResult, work::Bm1366JobId};

use super::frame::Frame;
use super::messages::{
    ChannelKind, NewExtendedMiningJob, NewMiningJob, OpenExtendedMiningChannel,
    OpenStandardMiningChannel, ServerMessage, SetNewPrevHash, SetupConnection,
    SubmitSharesExtended, SubmitSharesStandard,
};
use super::work::{target_to_pdiff, V2MiningWork};
use super::{StratumV2Error, PENDING_JOB_CAPACITY};

const SUBMIT_TIMING_CAPACITY: usize = 32;
const SEEN_SHARE_CAPACITY: usize = 64;

#[derive(Clone, PartialEq)]
pub struct SessionConfig {
    pub endpoint_host: String,
    pub endpoint_port: u16,
    pub vendor: String,
    pub hardware_version: String,
    pub firmware: String,
    pub device_id: String,
    pub user_identity: String,
    pub nominal_hashrate: f32,
    pub channel_kind: ChannelKind,
    pub minimum_extranonce_size: u16,
}

impl fmt::Debug for SessionConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionConfig")
            .field("endpoint", &"redacted")
            .field("identity", &"redacted")
            .field("nominal_hashrate", &self.nominal_hashrate)
            .field("channel_kind", &self.channel_kind)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    Disconnected,
    AwaitingSetup,
    AwaitingChannel,
    Active,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFailure {
    SetupRejected,
    ChannelRejected,
    UnexpectedMessage,
    RequestMismatch,
    ChannelMismatch,
    ExtranonceExhausted,
    SequenceExhausted,
}

#[derive(Clone, PartialEq, Eq)]
pub enum SessionEvent {
    Outbound(Frame),
    ChannelReady { kind: ChannelKind, pdiff: u32 },
    Work(V2MiningWork),
    TargetUpdated { pdiff: u32 },
    ShareAccepted { accepted_count: u32 },
    ShareRejected,
    Failed(SessionFailure),
    Stopped,
}

impl fmt::Debug for SessionEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outbound(_) => formatter.write_str("SessionEvent::Outbound(redacted)"),
            Self::ChannelReady { kind, pdiff } => formatter
                .debug_struct("SessionEvent::ChannelReady")
                .field("kind", kind)
                .field("pdiff", pdiff)
                .finish(),
            Self::Work(_) => formatter.write_str("SessionEvent::Work(redacted)"),
            Self::TargetUpdated { pdiff } => formatter
                .debug_struct("SessionEvent::TargetUpdated")
                .field("pdiff", pdiff)
                .finish(),
            Self::ShareAccepted { accepted_count } => formatter
                .debug_struct("SessionEvent::ShareAccepted")
                .field("accepted_count", accepted_count)
                .finish(),
            Self::ShareRejected => formatter.write_str("SessionEvent::ShareRejected"),
            Self::Failed(category) => formatter
                .debug_tuple("SessionEvent::Failed")
                .field(category)
                .finish(),
            Self::Stopped => formatter.write_str("SessionEvent::Stopped"),
        }
    }
}

pub struct V2Session {
    config: SessionConfig,
    phase: SessionPhase,
    maybe_channel_id: Option<u32>,
    pool_target: [u8; 32],
    extranonce_prefix: Vec<u8>,
    extranonce_size: usize,
    next_extranonce: u64,
    next_asic_job_id: Bm1366JobId,
    next_sequence: u32,
    maybe_prev_hash: Option<SetNewPrevHash>,
    pending_standard: [Option<NewMiningJob>; PENDING_JOB_CAPACITY],
    pending_extended: [Option<NewExtendedMiningJob>; PENDING_JOB_CAPACITY],
    active_work: HashMap<Bm1366JobId, V2MiningWork>,
    pending_submits: VecDeque<u32>,
    seen_shares: VecDeque<ShareKey>,
}

impl fmt::Debug for V2Session {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("V2Session")
            .field("phase", &self.phase)
            .field("channel", &self.maybe_channel_id.map(|_| "redacted"))
            .field("active_work_count", &self.active_work.len())
            .field("pending_submit_count", &self.pending_submits.len())
            .finish()
    }
}

impl V2Session {
    pub fn new(config: SessionConfig) -> Result<Self, StratumV2Error> {
        if !config.nominal_hashrate.is_finite() || config.nominal_hashrate <= 0.0 {
            return Err(StratumV2Error::InvalidField {
                field: "nominal_hashrate",
                reason: "must be finite and positive",
            });
        }
        if config.channel_kind == ChannelKind::Extended
            && !(1..=32).contains(&config.minimum_extranonce_size)
        {
            return Err(StratumV2Error::InvalidField {
                field: "minimum_extranonce_size",
                reason: "must be between one and 32 bytes",
            });
        }
        Ok(Self {
            config,
            phase: SessionPhase::Disconnected,
            maybe_channel_id: None,
            pool_target: [0; 32],
            extranonce_prefix: Vec::new(),
            extranonce_size: 0,
            next_extranonce: 0,
            next_asic_job_id: Bm1366JobId::new(0),
            next_sequence: 0,
            maybe_prev_hash: None,
            pending_standard: array::from_fn(|_| None),
            pending_extended: array::from_fn(|_| None),
            active_work: HashMap::new(),
            pending_submits: VecDeque::with_capacity(SUBMIT_TIMING_CAPACITY),
            seen_shares: VecDeque::with_capacity(SEEN_SHARE_CAPACITY),
        })
    }

    #[must_use]
    pub const fn phase(&self) -> SessionPhase {
        self.phase
    }

    pub fn start(&mut self) -> Result<SessionEvent, StratumV2Error> {
        if self.phase != SessionPhase::Disconnected {
            return Err(StratumV2Error::InvalidField {
                field: "session_phase",
                reason: "session has already started",
            });
        }
        let setup = SetupConnection {
            endpoint_host: self.config.endpoint_host.clone(),
            endpoint_port: self.config.endpoint_port,
            vendor: self.config.vendor.clone(),
            hardware_version: self.config.hardware_version.clone(),
            firmware: self.config.firmware.clone(),
            device_id: self.config.device_id.clone(),
            flags: self.config.channel_kind.setup_flags(),
        };
        self.phase = SessionPhase::AwaitingSetup;
        setup.encode().map(SessionEvent::Outbound)
    }

    pub fn handle(&mut self, message: ServerMessage) -> Result<Vec<SessionEvent>, StratumV2Error> {
        match message {
            ServerMessage::SetupConnectionSuccess(_)
                if self.phase == SessionPhase::AwaitingSetup =>
            {
                self.open_channel()
            }
            ServerMessage::SetupConnectionError(_) if self.phase == SessionPhase::AwaitingSetup => {
                Ok(self.fail(SessionFailure::SetupRejected))
            }
            ServerMessage::OpenStandardMiningChannelSuccess(success)
                if self.phase == SessionPhase::AwaitingChannel
                    && self.config.channel_kind == ChannelKind::Standard =>
            {
                if success.request_id != 1 {
                    return Ok(self.fail(SessionFailure::RequestMismatch));
                }
                self.activate_channel(
                    success.channel_id,
                    success.target,
                    success.extranonce_prefix,
                    0,
                )
            }
            ServerMessage::OpenExtendedMiningChannelSuccess(success)
                if self.phase == SessionPhase::AwaitingChannel
                    && self.config.channel_kind == ChannelKind::Extended =>
            {
                if success.request_id != 1 {
                    return Ok(self.fail(SessionFailure::RequestMismatch));
                }
                self.activate_channel(
                    success.channel_id,
                    success.target,
                    success.extranonce_prefix,
                    usize::from(success.extranonce_size),
                )
            }
            ServerMessage::OpenMiningChannelError(_)
                if self.phase == SessionPhase::AwaitingChannel =>
            {
                Ok(self.fail(SessionFailure::ChannelRejected))
            }
            ServerMessage::NewMiningJob(job) if self.phase == SessionPhase::Active => {
                self.handle_standard_job(job)
            }
            ServerMessage::NewExtendedMiningJob(job) if self.phase == SessionPhase::Active => {
                self.handle_extended_job(job)
            }
            ServerMessage::SetNewPrevHash(prev_hash) if self.phase == SessionPhase::Active => {
                self.handle_prev_hash(prev_hash)
            }
            ServerMessage::SetTarget(target) if self.phase == SessionPhase::Active => {
                if !self.is_channel(target.channel_id) {
                    return Ok(self.fail(SessionFailure::ChannelMismatch));
                }
                self.pool_target = target.maximum_target;
                Ok(vec![SessionEvent::TargetUpdated {
                    pdiff: target_to_pdiff(self.pool_target),
                }])
            }
            ServerMessage::SubmitSharesSuccess(success) if self.phase == SessionPhase::Active => {
                self.handle_submit_success(
                    success.channel_id,
                    success.last_sequence_number,
                    success.accepted_count,
                )
            }
            ServerMessage::SubmitSharesError(error) if self.phase == SessionPhase::Active => {
                self.handle_submit_error(error.channel_id, error.sequence_number)
            }
            _ => Ok(self.fail(SessionFailure::UnexpectedMessage)),
        }
    }

    pub fn observe_nonce(
        &mut self,
        result: Bm1366NonceResult,
    ) -> Result<Option<SessionEvent>, StratumV2Error> {
        if self.phase != SessionPhase::Active {
            return Err(StratumV2Error::InvalidField {
                field: "session_phase",
                reason: "share observed outside active channel",
            });
        }
        let lookup = result.job_id.lookup_key();
        let maybe_work = self.active_work.get(&lookup);
        let Some(work) = maybe_work else {
            return Err(StratumV2Error::InvalidField {
                field: "asic_job_id",
                reason: "share has no active work context",
            });
        };
        let key = ShareKey {
            asic_job_id: lookup,
            nonce: result.nonce,
            version_bits: result.version_bits,
        };
        if self.seen_shares.contains(&key) || !work.qualifies(result)? {
            return Ok(None);
        }
        let sequence = self.next_sequence;
        self.next_sequence =
            self.next_sequence
                .checked_add(1)
                .ok_or(StratumV2Error::InvalidField {
                    field: "sequence_number",
                    reason: "is exhausted",
                })?;
        let frame = match &work.maybe_extranonce {
            Some(extranonce) => SubmitSharesExtended {
                channel_id: work.channel_id,
                sequence_number: sequence,
                job_id: work.job_id,
                nonce: result.nonce,
                ntime: u32::from_le_bytes(work.fields.ntime),
                version: work.rolled_version(result),
                extranonce: extranonce.clone(),
            }
            .encode()?,
            None => SubmitSharesStandard {
                channel_id: work.channel_id,
                sequence_number: sequence,
                job_id: work.job_id,
                nonce: result.nonce,
                ntime: u32::from_le_bytes(work.fields.ntime),
                version: work.rolled_version(result),
            }
            .encode()?,
        };
        push_bounded(&mut self.seen_shares, key, SEEN_SHARE_CAPACITY);
        push_bounded(&mut self.pending_submits, sequence, SUBMIT_TIMING_CAPACITY);
        Ok(Some(SessionEvent::Outbound(frame)))
    }

    pub fn stop(&mut self) -> SessionEvent {
        self.clear_work();
        self.phase = SessionPhase::Stopped;
        SessionEvent::Stopped
    }

    fn open_channel(&mut self) -> Result<Vec<SessionEvent>, StratumV2Error> {
        let maximum_target = [0xff; 32];
        let frame = match self.config.channel_kind {
            ChannelKind::Standard => OpenStandardMiningChannel {
                request_id: 1,
                user_identity: self.config.user_identity.clone(),
                nominal_hashrate: self.config.nominal_hashrate,
                maximum_target,
            }
            .encode()?,
            ChannelKind::Extended => OpenExtendedMiningChannel {
                request_id: 1,
                user_identity: self.config.user_identity.clone(),
                nominal_hashrate: self.config.nominal_hashrate,
                maximum_target,
                minimum_extranonce_size: self.config.minimum_extranonce_size,
            }
            .encode()?,
        };
        self.phase = SessionPhase::AwaitingChannel;
        Ok(vec![SessionEvent::Outbound(frame)])
    }

    fn activate_channel(
        &mut self,
        channel_id: u32,
        target: [u8; 32],
        extranonce_prefix: Vec<u8>,
        extranonce_size: usize,
    ) -> Result<Vec<SessionEvent>, StratumV2Error> {
        if self.config.channel_kind == ChannelKind::Extended && !(1..=32).contains(&extranonce_size)
        {
            return Ok(self.fail(SessionFailure::ChannelRejected));
        }
        self.maybe_channel_id = Some(channel_id);
        self.pool_target = target;
        self.extranonce_prefix = extranonce_prefix;
        self.extranonce_size = extranonce_size;
        self.phase = SessionPhase::Active;
        Ok(vec![SessionEvent::ChannelReady {
            kind: self.config.channel_kind,
            pdiff: target_to_pdiff(target),
        }])
    }

    fn handle_standard_job(
        &mut self,
        job: NewMiningJob,
    ) -> Result<Vec<SessionEvent>, StratumV2Error> {
        if self.config.channel_kind != ChannelKind::Standard || !self.is_channel(job.channel_id) {
            return Ok(self.fail(SessionFailure::ChannelMismatch));
        }
        if job.maybe_min_ntime.is_some() && self.maybe_prev_hash.is_some() {
            return self.standard_work(job);
        }
        let slot = job.job_id as usize % PENDING_JOB_CAPACITY;
        self.pending_standard[slot] = Some(job);
        Ok(Vec::new())
    }

    fn handle_extended_job(
        &mut self,
        job: NewExtendedMiningJob,
    ) -> Result<Vec<SessionEvent>, StratumV2Error> {
        if self.config.channel_kind != ChannelKind::Extended || !self.is_channel(job.channel_id) {
            return Ok(self.fail(SessionFailure::ChannelMismatch));
        }
        if job.maybe_min_ntime.is_some() && self.maybe_prev_hash.is_some() {
            return self.extended_work(job);
        }
        let slot = job.job_id as usize % PENDING_JOB_CAPACITY;
        self.pending_extended[slot] = Some(job);
        Ok(Vec::new())
    }

    fn handle_prev_hash(
        &mut self,
        prev_hash: SetNewPrevHash,
    ) -> Result<Vec<SessionEvent>, StratumV2Error> {
        if !self.is_channel(prev_hash.channel_id) {
            return Ok(self.fail(SessionFailure::ChannelMismatch));
        }
        let first = self.maybe_prev_hash.is_none();
        self.maybe_prev_hash = Some(prev_hash.clone());
        let mut events = Vec::new();
        match self.config.channel_kind {
            ChannelKind::Standard => {
                let jobs = take_matching_jobs(&mut self.pending_standard, prev_hash.job_id, first);
                for job in jobs {
                    events.extend(self.standard_work(job)?);
                }
            }
            ChannelKind::Extended => {
                let jobs = take_matching_jobs(&mut self.pending_extended, prev_hash.job_id, first);
                for job in jobs {
                    events.extend(self.extended_work(job)?);
                }
            }
        }
        Ok(events)
    }

    fn standard_work(&mut self, job: NewMiningJob) -> Result<Vec<SessionEvent>, StratumV2Error> {
        let prev_hash = self
            .maybe_prev_hash
            .as_ref()
            .ok_or(StratumV2Error::InvalidField {
                field: "prev_hash",
                reason: "is unavailable",
            })?
            .clone();
        let asic_job_id = self.take_asic_job_id();
        let work = V2MiningWork::standard(&job, &prev_hash, self.pool_target, asic_job_id)?;
        Ok(vec![self.publish_work(work)])
    }

    fn extended_work(
        &mut self,
        job: NewExtendedMiningJob,
    ) -> Result<Vec<SessionEvent>, StratumV2Error> {
        let prev_hash = self
            .maybe_prev_hash
            .as_ref()
            .ok_or(StratumV2Error::InvalidField {
                field: "prev_hash",
                reason: "is unavailable",
            })?
            .clone();
        let extranonce = self.take_extranonce()?;
        let extranonce_prefix = self.extranonce_prefix.clone();
        let asic_job_id = self.take_asic_job_id();
        let work = V2MiningWork::extended(
            &job,
            &prev_hash,
            &extranonce_prefix,
            extranonce,
            self.pool_target,
            asic_job_id,
        )?;
        Ok(vec![self.publish_work(work)])
    }

    fn take_extranonce(&mut self) -> Result<Vec<u8>, StratumV2Error> {
        let mut extranonce = vec![0; self.extranonce_size];
        let counter = self.next_extranonce.to_be_bytes();
        let copy_len = extranonce.len().min(counter.len());
        let destination_start = extranonce.len() - copy_len;
        extranonce[destination_start..].copy_from_slice(&counter[counter.len() - copy_len..]);
        self.next_extranonce =
            self.next_extranonce
                .checked_add(1)
                .ok_or(StratumV2Error::InvalidField {
                    field: "extranonce",
                    reason: "counter is exhausted",
                })?;
        Ok(extranonce)
    }

    fn take_asic_job_id(&mut self) -> Bm1366JobId {
        let current = self.next_asic_job_id;
        self.next_asic_job_id = current.advance();
        current
    }

    fn publish_work(&mut self, work: V2MiningWork) -> SessionEvent {
        self.active_work.clear();
        self.seen_shares.clear();
        self.active_work
            .insert(work.asic_job_id.lookup_key(), work.clone());
        SessionEvent::Work(work)
    }

    fn handle_submit_success(
        &mut self,
        channel_id: u32,
        last_sequence: u32,
        accepted_count: u32,
    ) -> Result<Vec<SessionEvent>, StratumV2Error> {
        if !self.is_channel(channel_id) {
            return Ok(self.fail(SessionFailure::ChannelMismatch));
        }
        let matched = self.pending_submits.contains(&last_sequence);
        if !matched {
            return Ok(self.fail(SessionFailure::RequestMismatch));
        }
        while self
            .pending_submits
            .front()
            .is_some_and(|sequence| *sequence <= last_sequence)
        {
            self.pending_submits.pop_front();
        }
        Ok(vec![SessionEvent::ShareAccepted { accepted_count }])
    }

    fn handle_submit_error(
        &mut self,
        channel_id: u32,
        sequence: u32,
    ) -> Result<Vec<SessionEvent>, StratumV2Error> {
        if !self.is_channel(channel_id) {
            return Ok(self.fail(SessionFailure::ChannelMismatch));
        }
        let maybe_index = self
            .pending_submits
            .iter()
            .position(|pending| *pending == sequence);
        let Some(index) = maybe_index else {
            return Ok(self.fail(SessionFailure::RequestMismatch));
        };
        self.pending_submits.remove(index);
        Ok(vec![SessionEvent::ShareRejected])
    }

    fn is_channel(&self, channel_id: u32) -> bool {
        self.maybe_channel_id == Some(channel_id)
    }

    fn fail(&mut self, category: SessionFailure) -> Vec<SessionEvent> {
        self.clear_work();
        self.phase = SessionPhase::Failed;
        vec![SessionEvent::Failed(category)]
    }

    fn clear_work(&mut self) {
        self.active_work.clear();
        self.pending_submits.clear();
        self.seen_shares.clear();
        self.pending_standard.fill(None);
        self.pending_extended.fill(None);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShareKey {
    asic_job_id: Bm1366JobId,
    nonce: u32,
    version_bits: u32,
}

fn push_bounded<T>(queue: &mut VecDeque<T>, value: T, capacity: usize) {
    if queue.len() == capacity {
        queue.pop_front();
    }
    queue.push_back(value);
}

fn take_matching_jobs<T>(
    pending: &mut [Option<T>; PENDING_JOB_CAPACITY],
    job_id: u32,
    first_prev_hash: bool,
) -> Vec<T>
where
    T: JobIdentity,
{
    let mut selected_jobs = Vec::new();
    for maybe_job in pending {
        let selected = maybe_job
            .as_ref()
            .is_some_and(|job| job.job_id() == job_id || first_prev_hash);
        if selected {
            selected_jobs.push(maybe_job.take().expect("selected pending job must exist"));
        }
    }
    selected_jobs
}

trait JobIdentity {
    fn job_id(&self) -> u32;
}

impl JobIdentity for NewMiningJob {
    fn job_id(&self) -> u32 {
        self.job_id
    }
}

impl JobIdentity for NewExtendedMiningJob {
    fn job_id(&self) -> u32 {
        self.job_id
    }
}

#[cfg(test)]
mod tests;
