//! Strict BM1370 receive-frame decoding: nonce results, register reads and chip identity.
//!
//! Reference: `reference/esp-miner/components/asic/bm1370.c` (`BM1370_process_work`,
//! `REGISTER_MAP`) and `components/asic/asic_common.c` (`receive_work`, `count_asic_chips`).

use crate::{
    bm1366::crc::crc5,
    bm1370::{work::Bm1370JobId, Bm1370ProtocolFault, BM1370_CHIP_ID, BM1370_RESULT_FRAME_LEN},
};

pub const RECEIVE_PREAMBLE: u16 = 0xaa55;
/// Bits 4-7 of a result's id byte carry the job id shifted left one; bits 0-3 the small core.
const RESULT_JOB_LOOKUP_MASK: u8 = 0xf0;
const RESULT_SMALL_CORE_MASK: u8 = 0x0f;
/// Byte 10's top bit marks a nonce result; clear, the frame is a register read.
const JOB_RESPONSE_FLAG: u8 = 0x80;

/// The registers a BM1370 reports back, as upstream's `REGISTER_MAP` names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1370Register {
    ErrorCount,
    Domain0Count,
    Domain1Count,
    Domain2Count,
    Domain3Count,
    TotalCount,
}

impl TryFrom<u8> for Bm1370Register {
    type Error = Bm1370ProtocolFault;

    fn try_from(register: u8) -> Result<Self, Self::Error> {
        match register {
            0x4c => Ok(Self::ErrorCount),
            0x88 => Ok(Self::Domain0Count),
            0x89 => Ok(Self::Domain1Count),
            0x8a => Ok(Self::Domain2Count),
            0x8b => Ok(Self::Domain3Count),
            0x8c => Ok(Self::TotalCount),
            _ => Err(Bm1370ProtocolFault::UnknownRegister { register }),
        }
    }
}

impl Bm1370Register {
    /// The register byte to read it with.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::ErrorCount => 0x4c,
            Self::Domain0Count => 0x88,
            Self::Domain1Count => 0x89,
            Self::Domain2Count => 0x8a,
            Self::Domain3Count => 0x8b,
            Self::TotalCount => 0x8c,
        }
    }
}

/// Valid transmitted job identifiers for one active BM1370 session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bm1370ValidJobIds([bool; 128]);

impl Bm1370ValidJobIds {
    #[must_use]
    pub const fn empty() -> Self {
        Self([false; 128])
    }

    pub fn insert(&mut self, job_id: Bm1370JobId) {
        self.0[usize::from(job_id.raw())] = true;
    }

    #[must_use]
    pub fn contains(&self, job_id: Bm1370JobId) -> bool {
        self.0[usize::from(job_id.raw())]
    }

    /// Forgets every job, as a pool's clean-jobs notice requires. Upstream clears every id that
    /// is a multiple of four, and every BM1370 id is one.
    pub fn clear(&mut self) {
        self.0 = [false; 128];
    }
}

impl Default for Bm1370ValidJobIds {
    fn default() -> Self {
        Self::empty()
    }
}

/// Decoded BM1370 nonce result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1370NonceResult {
    pub job_id: Bm1370JobId,
    /// The nonce as the header carries it: the wire bytes read little-endian.
    pub nonce: u32,
    pub asic_index: u8,
    pub core_id: u8,
    pub small_core_id: u8,
    /// The rolled version bits, already shifted into place, to OR into the job's version.
    pub version_bits: u32,
}

/// Decoded BM1370 register response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1370RegisterRead {
    pub register: Bm1370Register,
    pub asic_index: u8,
    pub asic_address: u8,
    pub value: u32,
}

/// Strictly decoded BM1370 receive-frame shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1370ParsedResult {
    JobNonce(Bm1370NonceResult),
    RegisterRead(Bm1370RegisterRead),
}

/// One chip's answer to the `CHIP_ID` read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1370ChipIdentity {
    pub core_num: u8,
    pub address: u8,
}

/// Parses one complete BM1370 receive frame.
pub fn parse_result_frame(
    bytes: &[u8],
    valid_jobs: &Bm1370ValidJobIds,
    address_interval: u16,
) -> Result<Bm1370ParsedResult, Bm1370ProtocolFault> {
    check_frame(bytes)?;
    let address_interval = valid_address_interval(address_interval)?;
    if bytes[10] & JOB_RESPONSE_FLAG != 0 {
        return parse_job_result(bytes, valid_jobs, address_interval);
    }
    parse_register_read(bytes, address_interval)
}

/// Parses one chip's `CHIP_ID` answer; any chip but a BM1370 is refused.
pub fn parse_chip_id_response(bytes: &[u8]) -> Result<Bm1370ChipIdentity, Bm1370ProtocolFault> {
    check_frame(bytes)?;
    let chip_id = u16::from_be_bytes([bytes[2], bytes[3]]);
    if chip_id != BM1370_CHIP_ID {
        return Err(Bm1370ProtocolFault::UnexpectedChipId { chip_id });
    }
    Ok(Bm1370ChipIdentity {
        core_num: bytes[4],
        address: bytes[5],
    })
}

fn check_frame(bytes: &[u8]) -> Result<(), Bm1370ProtocolFault> {
    if bytes.len() != BM1370_RESULT_FRAME_LEN {
        return Err(Bm1370ProtocolFault::InvalidLength {
            expected: BM1370_RESULT_FRAME_LEN,
            actual: bytes.len(),
        });
    }
    let preamble = u16::from_be_bytes([bytes[0], bytes[1]]);
    if preamble != RECEIVE_PREAMBLE {
        return Err(Bm1370ProtocolFault::BadPreamble {
            expected: RECEIVE_PREAMBLE,
            actual: preamble,
        });
    }
    if crc5(&bytes[2..]) != 0 {
        return Err(Bm1370ProtocolFault::BadCrc);
    }
    Ok(())
}

fn parse_job_result(
    bytes: &[u8],
    valid_jobs: &Bm1370ValidJobIds,
    address_interval: u16,
) -> Result<Bm1370ParsedResult, Bm1370ProtocolFault> {
    let nonce_bytes = [bytes[2], bytes[3], bytes[4], bytes[5]];
    let nonce_be = u32::from_be_bytes(nonce_bytes);
    let result_id = bytes[7];
    let job_id = Bm1370JobId::new((result_id & RESULT_JOB_LOOKUP_MASK) >> 1);
    if !valid_jobs.contains(job_id) {
        return Err(Bm1370ProtocolFault::InvalidJobId {
            job_id: job_id.raw(),
        });
    }
    // Seven core bits address all 128 BM1370 cores, so every value is a valid core.
    let core_id = ((nonce_be >> 25) & 0x7f) as u8;
    let asic_address = ((nonce_be >> 17) & 0xff) as u8;
    Ok(Bm1370ParsedResult::JobNonce(Bm1370NonceResult {
        job_id,
        nonce: u32::from_le_bytes(nonce_bytes),
        asic_index: (u16::from(asic_address) / address_interval) as u8,
        core_id,
        small_core_id: result_id & RESULT_SMALL_CORE_MASK,
        version_bits: u32::from(u16::from_be_bytes([bytes[8], bytes[9]])) << 13,
    }))
}

fn parse_register_read(
    bytes: &[u8],
    address_interval: u16,
) -> Result<Bm1370ParsedResult, Bm1370ProtocolFault> {
    let asic_address = bytes[6];
    Ok(Bm1370ParsedResult::RegisterRead(Bm1370RegisterRead {
        register: Bm1370Register::try_from(bytes[7])?,
        asic_index: (u16::from(asic_address) / address_interval) as u8,
        asic_address,
        value: u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]),
    }))
}

fn valid_address_interval(address_interval: u16) -> Result<u16, Bm1370ProtocolFault> {
    if address_interval == 0 || address_interval > 256 {
        return Err(Bm1370ProtocolFault::InvalidAddressInterval { address_interval });
    }
    Ok(address_interval)
}
