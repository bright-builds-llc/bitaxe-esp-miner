//! Strict BM1368 result and register-response decoding.

use crate::{
    bm1366::crc::crc5,
    bm1368::{work::Bm1368JobId, Bm1368ProtocolFault, BM1368_RESULT_FRAME_LEN},
};

pub const RECEIVE_PREAMBLE: u16 = 0xaa55;
pub const NORMAL_CORE_COUNT: u8 = 80;
pub const SMALL_CORE_COUNT: u8 = 16;
const RESULT_JOB_LOOKUP_MASK: u8 = 0xf0;

/// Reference result-register classifications shared by BM1368 chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1368Register {
    ErrorCount,
    Domain0Count,
    Domain1Count,
    Domain2Count,
    Domain3Count,
    TotalCount,
}

impl TryFrom<u8> for Bm1368Register {
    type Error = Bm1368ProtocolFault;

    fn try_from(register: u8) -> Result<Self, Self::Error> {
        match register {
            0x4c => Ok(Self::ErrorCount),
            0x88 => Ok(Self::Domain0Count),
            0x89 => Ok(Self::Domain1Count),
            0x8a => Ok(Self::Domain2Count),
            0x8b => Ok(Self::Domain3Count),
            0x8c => Ok(Self::TotalCount),
            _ => Err(Bm1368ProtocolFault::UnknownRegister { register }),
        }
    }
}

/// Valid transmitted job identifiers for one active BM1368 session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bm1368ValidJobIds([bool; 128]);

impl Bm1368ValidJobIds {
    #[must_use]
    pub const fn empty() -> Self {
        Self([false; 128])
    }

    #[must_use]
    pub fn single(job_id: Bm1368JobId) -> Self {
        let mut jobs = Self::empty();
        jobs.insert(job_id);
        jobs
    }

    pub fn insert(&mut self, job_id: Bm1368JobId) {
        self.0[usize::from(job_id.raw())] = true;
    }

    #[must_use]
    pub fn contains(&self, job_id: Bm1368JobId) -> bool {
        self.0[usize::from(job_id.raw())]
    }
}

impl Default for Bm1368ValidJobIds {
    fn default() -> Self {
        Self::empty()
    }
}

/// Decoded BM1368 nonce result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1368NonceResult {
    pub job_id: Bm1368JobId,
    pub nonce: u32,
    pub asic_index: u8,
    pub core_id: u8,
    pub small_core_id: u8,
    pub version_bits: u32,
}

/// Decoded BM1368 register response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1368RegisterRead {
    pub register: Bm1368Register,
    pub asic_index: u8,
    pub asic_address: u8,
    pub value: u32,
}

/// Strictly decoded BM1368 receive-frame shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1368ParsedResult {
    JobNonce(Bm1368NonceResult),
    RegisterRead(Bm1368RegisterRead),
}

/// Parses one complete BM1368 receive frame.
pub fn parse_result_frame(
    bytes: &[u8],
    valid_jobs: &Bm1368ValidJobIds,
    address_interval: u16,
) -> Result<Bm1368ParsedResult, Bm1368ProtocolFault> {
    if bytes.len() != BM1368_RESULT_FRAME_LEN {
        return Err(Bm1368ProtocolFault::InvalidLength {
            expected: BM1368_RESULT_FRAME_LEN,
            actual: bytes.len(),
        });
    }

    let preamble = u16::from_be_bytes([bytes[0], bytes[1]]);
    if preamble != RECEIVE_PREAMBLE {
        return Err(Bm1368ProtocolFault::BadPreamble {
            expected: RECEIVE_PREAMBLE,
            actual: preamble,
        });
    }
    if crc5(&bytes[2..]) != 0 {
        return Err(Bm1368ProtocolFault::BadCrc);
    }

    let address_interval = valid_address_interval(address_interval)?;
    if (bytes[10] & 0x80) != 0 {
        return parse_job_result(bytes, valid_jobs, address_interval);
    }
    parse_register_read(bytes, address_interval)
}

fn parse_job_result(
    bytes: &[u8],
    valid_jobs: &Bm1368ValidJobIds,
    address_interval: u16,
) -> Result<Bm1368ParsedResult, Bm1368ProtocolFault> {
    let nonce_bytes = [bytes[2], bytes[3], bytes[4], bytes[5]];
    let nonce_be = u32::from_be_bytes(nonce_bytes);
    let result_id = bytes[7];
    let job_id = Bm1368JobId::new((result_id & RESULT_JOB_LOOKUP_MASK) >> 1);
    if !valid_jobs.contains(job_id) {
        return Err(Bm1368ProtocolFault::InvalidJobId {
            job_id: job_id.raw(),
        });
    }

    let core_id = ((nonce_be >> 25) & 0x7f) as u8;
    if core_id >= NORMAL_CORE_COUNT {
        return Err(Bm1368ProtocolFault::InvalidCoreId { core_id });
    }

    let asic_address = ((nonce_be >> 17) & 0xff) as u8;
    let asic_index = (u16::from(asic_address) / address_interval) as u8;
    let small_core_id = result_id & 0x0f;
    let version_bits = u32::from(u16::from_be_bytes([bytes[8], bytes[9]])) << 13;

    Ok(Bm1368ParsedResult::JobNonce(Bm1368NonceResult {
        job_id,
        nonce: u32::from_le_bytes(nonce_bytes),
        asic_index,
        core_id,
        small_core_id,
        version_bits,
    }))
}

fn parse_register_read(
    bytes: &[u8],
    address_interval: u16,
) -> Result<Bm1368ParsedResult, Bm1368ProtocolFault> {
    let value = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    let asic_address = bytes[6];
    let register = Bm1368Register::try_from(bytes[7])?;
    let asic_index = (u16::from(asic_address) / address_interval) as u8;
    Ok(Bm1368ParsedResult::RegisterRead(Bm1368RegisterRead {
        register,
        asic_index,
        asic_address,
        value,
    }))
}

fn valid_address_interval(address_interval: u16) -> Result<u16, Bm1368ProtocolFault> {
    if address_interval == 0 || address_interval > 256 {
        return Err(Bm1368ProtocolFault::InvalidAddressInterval { address_interval });
    }
    Ok(address_interval)
}
