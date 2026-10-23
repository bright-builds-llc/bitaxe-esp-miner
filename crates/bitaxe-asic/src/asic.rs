//! The wire data the BM13xx mining chips share, and the facts that differ by chip family.
//!
//! The BM1366 and the BM1370 send the same 82-byte job payload and return the same result
//! fields, so work, job tables and nonce results are chip-neutral types here. What differs, such
//! as the job-id step, the job interval and how a result's id byte is unpacked, is
//! [`AsicFamily`]'s. The BM1366 names (`Bm1366JobId` and the rest) remain as aliases of these
//! types.
//!
//! Reference: `reference/esp-miner/components/asic/include/bm1366.h`, `include/bm1370.h`
//! (`BM13xx_job`) and `components/asic/asic_common.c` (`receive_work`).

pub mod family;

pub use family::AsicFamily;

pub use crate::bm1366::registers::Bm1366Register as Register;

/// Bytes in one job payload, before framing.
pub const JOB_PAYLOAD_LEN: usize = 82;
/// The job payload's fixed midstate count.
pub const NUM_MIDSTATES: u8 = 0x01;
/// Job ids live in 0..128.
pub const JOB_ID_MODULUS: u8 = 128;
/// Every family's transmitted job ids are multiples of eight; a result's id byte may carry
/// small-core bits below them.
pub const JOB_SLOT_MASK: u8 = 0xf8;

/// A job identifier, as transmitted in work and recovered from results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId {
    id: u8,
}

impl JobId {
    pub const fn new(id: u8) -> Self {
        Self {
            id: id % JOB_ID_MODULUS,
        }
    }

    pub const fn raw(self) -> u8 {
        self.id
    }

    /// The id of the next job a chip of `family` is sent.
    pub const fn advance(self, family: AsicFamily) -> Self {
        Self::new((self.id + family.job_id_step()) % JOB_ID_MODULUS)
    }

    /// The job-table slot: the id without any small-core bits.
    pub const fn lookup_key(self) -> Self {
        Self::new(self.id & JOB_SLOT_MASK)
    }

    /// The BM1366's small-core bits, packed below the job slot.
    pub const fn small_core_id(self) -> u8 {
        self.id & !JOB_SLOT_MASK
    }
}

impl From<JobId> for u8 {
    fn from(job_id: JobId) -> Self {
        job_id.raw()
    }
}

/// Pool-derived work fields encoded into one job, each already in wire byte order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkFields {
    pub starting_nonce: [u8; 4],
    pub nbits: [u8; 4],
    pub ntime: [u8; 4],
    pub merkle_root: [u8; 32],
    pub prev_block_hash: [u8; 32],
    pub version: [u8; 4],
}

/// The fixed-layout job payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkPayload {
    bytes: [u8; JOB_PAYLOAD_LEN],
}

impl WorkPayload {
    pub fn new(job_id: JobId, fields: WorkFields) -> Self {
        let mut bytes = [0; JOB_PAYLOAD_LEN];
        bytes[0] = job_id.raw();
        bytes[1] = NUM_MIDSTATES;
        bytes[2..6].copy_from_slice(&fields.starting_nonce);
        bytes[6..10].copy_from_slice(&fields.nbits);
        bytes[10..14].copy_from_slice(&fields.ntime);
        bytes[14..46].copy_from_slice(&fields.merkle_root);
        bytes[46..78].copy_from_slice(&fields.prev_block_hash);
        bytes[78..82].copy_from_slice(&fields.version);
        Self { bytes }
    }

    pub const fn bytes(&self) -> &[u8; JOB_PAYLOAD_LEN] {
        &self.bytes
    }
}

/// One production job: its id and its payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductionWorkPayload {
    job_id: JobId,
    payload: WorkPayload,
}

impl ProductionWorkPayload {
    #[must_use]
    pub fn new(job_id: JobId, fields: WorkFields) -> Self {
        Self {
            job_id,
            payload: WorkPayload::new(job_id, fields),
        }
    }

    #[must_use]
    pub const fn job_id(&self) -> JobId {
        self.job_id
    }

    #[must_use]
    pub const fn payload(&self) -> &WorkPayload {
        &self.payload
    }
}

/// The jobs sent and not yet retired, by job-table slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidJobIds {
    valid: [bool; JOB_ID_MODULUS as usize],
}

impl ValidJobIds {
    pub const fn empty() -> Self {
        Self {
            valid: [false; JOB_ID_MODULUS as usize],
        }
    }

    pub fn single(job_id: JobId) -> Self {
        let mut valid_jobs = Self::empty();
        valid_jobs.insert(job_id);
        valid_jobs
    }

    pub fn insert(&mut self, job_id: JobId) {
        self.valid[usize::from(job_id.lookup_key().raw())] = true;
    }

    pub fn contains(&self, job_id: JobId) -> bool {
        self.valid[usize::from(job_id.lookup_key().raw())]
    }

    /// Forgets every job, as a pool's clean-jobs notice requires.
    pub fn clear(&mut self) {
        self.valid = [false; JOB_ID_MODULUS as usize];
    }
}

impl Default for ValidJobIds {
    fn default() -> Self {
        Self::empty()
    }
}

/// A decoded nonce result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonceResult {
    pub job_id: JobId,
    /// The nonce as the header carries it: the wire bytes read little-endian.
    pub nonce: u32,
    pub asic_index: u8,
    pub core_id: u8,
    pub small_core_id: u8,
    /// The rolled version bits, already shifted into place.
    pub version_bits: u32,
}

/// A decoded register response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterRead {
    pub register: Register,
    pub asic_index: u8,
    pub asic_address: u8,
    pub value: u32,
}

/// The production loop's disposition of one complete receive frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionResult {
    JobNonce(NonceResult),
    RegisterRead(RegisterRead),
    Discarded(ResultDiscardReason),
}

/// Privacy-safe categories for a discarded receive frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultDiscardReason {
    InvalidLength,
    InvalidPreamble,
    InvalidCrc,
    JobLookup,
    Core,
    AddressInterval,
    RegisterResponse,
    ParserInvariant,
}

impl ResultDiscardReason {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::InvalidLength => "invalid_length",
            Self::InvalidPreamble => "invalid_preamble",
            Self::InvalidCrc => "invalid_crc",
            Self::JobLookup => "job_lookup",
            Self::Core => "core",
            Self::AddressInterval => "address_interval",
            Self::RegisterResponse => "register_response",
            Self::ParserInvariant => "parser_invariant",
        }
    }
}
