//! Strict BM1397 result/register decoding and duplicate suppression.

use crate::{
    bm1366::crc::crc5,
    bm1397::{
        work::{Bm1397JobId, JOB_ID_MODULUS},
        Bm1397ProtocolFault, BM1397_RESULT_FRAME_LEN,
    },
};

pub const RECEIVE_PREAMBLE: u16 = 0xaa55;
const RESULT_JOB_LOOKUP_MASK: u8 = 0xfc;
const RESULT_MIDSTATE_INDEX_MASK: u8 = 0x03;

/// Reference result-register classifications used by BM1397 chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1397Register {
    Hashrate,
    ErrorCount,
}

impl TryFrom<u8> for Bm1397Register {
    type Error = Bm1397ProtocolFault;

    fn try_from(register: u8) -> Result<Self, Self::Error> {
        match register {
            0x04 => Ok(Self::Hashrate),
            0x4c => Ok(Self::ErrorCount),
            _ => Err(Bm1397ProtocolFault::UnknownRegister { register }),
        }
    }
}

/// Version context retained for one valid transmitted job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1397JobContext {
    pub job_id: Bm1397JobId,
    pub base_version: u32,
    pub version_mask: u32,
}

/// Decoded BM1397 nonce result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1397NonceResult {
    pub job_id: Bm1397JobId,
    pub midstate_index: u8,
    pub nonce: u32,
    pub asic_index: u8,
    pub core_id: u8,
    pub small_core_id: u8,
    pub rolled_version: u32,
}

/// Decoded BM1397 register response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1397RegisterRead {
    pub register: Bm1397Register,
    pub asic_index: u8,
    pub asic_address: u8,
    pub value: u32,
}

/// Strictly decoded BM1397 receive-frame shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1397ParsedResult {
    JobNonce(Bm1397NonceResult),
    RegisterRead(Bm1397RegisterRead),
}

/// Stateful valid-job registry and previous-nonce filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bm1397ResultTracker {
    jobs: [Option<Bm1397JobContext>; 128],
    maybe_previous_nonce: Option<[u8; 4]>,
}

impl Bm1397ResultTracker {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            jobs: [None; 128],
            maybe_previous_nonce: None,
        }
    }

    pub fn insert(&mut self, context: Bm1397JobContext) {
        self.jobs[usize::from(context.job_id.raw())] = Some(context);
    }

    /// Parses one complete frame and updates duplicate state only after an
    /// otherwise valid nonce result.
    pub fn parse_result_frame(
        &mut self,
        bytes: &[u8],
        address_interval: u16,
    ) -> Result<Bm1397ParsedResult, Bm1397ProtocolFault> {
        if bytes.len() != BM1397_RESULT_FRAME_LEN {
            return Err(Bm1397ProtocolFault::InvalidLength {
                expected: BM1397_RESULT_FRAME_LEN,
                actual: bytes.len(),
            });
        }

        let preamble = u16::from_be_bytes([bytes[0], bytes[1]]);
        if preamble != RECEIVE_PREAMBLE {
            return Err(Bm1397ProtocolFault::BadPreamble {
                expected: RECEIVE_PREAMBLE,
                actual: preamble,
            });
        }
        if crc5(&bytes[2..]) != 0 {
            return Err(Bm1397ProtocolFault::BadCrc);
        }

        let address_interval = valid_address_interval(address_interval)?;
        if (bytes[8] & 0x80) != 0 {
            return self.parse_job_result(bytes, address_interval);
        }
        parse_register_read(bytes, address_interval)
    }

    fn parse_job_result(
        &mut self,
        bytes: &[u8],
        address_interval: u16,
    ) -> Result<Bm1397ParsedResult, Bm1397ProtocolFault> {
        let nonce_bytes = [bytes[2], bytes[3], bytes[4], bytes[5]];
        let nonce = u32::from_le_bytes(nonce_bytes);
        let result_id = bytes[7];
        let raw_job_id = result_id & RESULT_JOB_LOOKUP_MASK;
        if raw_job_id >= JOB_ID_MODULUS {
            return Err(Bm1397ProtocolFault::InvalidJobId { job_id: raw_job_id });
        }
        let job_id = Bm1397JobId::new(raw_job_id);
        let maybe_context = self.jobs[usize::from(job_id.raw())];
        let Some(context) = maybe_context else {
            return Err(Bm1397ProtocolFault::InvalidJobId {
                job_id: job_id.raw(),
            });
        };
        if self.maybe_previous_nonce == Some(nonce_bytes) {
            return Err(Bm1397ProtocolFault::DuplicateNonce { nonce });
        }

        let midstate_index = result_id & RESULT_MIDSTATE_INDEX_MASK;
        let mut rolled_version = context.base_version;
        for _ in 0..midstate_index {
            rolled_version = increment_bitmask(rolled_version, context.version_mask);
        }

        let nonce_be = u32::from_be_bytes(nonce_bytes);
        let asic_address = ((nonce_be >> 17) & 0xff) as u8;
        let asic_index = (u16::from(asic_address) / address_interval) as u8;
        let core_id = ((nonce_be >> 25) & 0x7f) as u8;
        let small_core_id = result_id & 0x0f;
        self.maybe_previous_nonce = Some(nonce_bytes);

        Ok(Bm1397ParsedResult::JobNonce(Bm1397NonceResult {
            job_id,
            midstate_index,
            nonce,
            asic_index,
            core_id,
            small_core_id,
            rolled_version,
        }))
    }
}

impl Default for Bm1397ResultTracker {
    fn default() -> Self {
        Self::empty()
    }
}

fn parse_register_read(
    bytes: &[u8],
    address_interval: u16,
) -> Result<Bm1397ParsedResult, Bm1397ProtocolFault> {
    let value = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    let asic_address = bytes[6];
    let register = Bm1397Register::try_from(bytes[7])?;
    let asic_index = (u16::from(asic_address) / address_interval) as u8;
    Ok(Bm1397ParsedResult::RegisterRead(Bm1397RegisterRead {
        register,
        asic_index,
        asic_address,
        value,
    }))
}

fn valid_address_interval(address_interval: u16) -> Result<u16, Bm1397ProtocolFault> {
    if address_interval == 0 || address_interval > 256 {
        return Err(Bm1397ProtocolFault::InvalidAddressInterval { address_interval });
    }
    Ok(address_interval)
}

#[must_use]
fn increment_bitmask(value: u32, mask: u32) -> u32 {
    if mask == 0 {
        return value;
    }

    let least_set = mask & mask.wrapping_neg();
    let carry = (value & mask).wrapping_add(least_set);
    let overflow = carry & !mask;
    let mut new_value = (value & !mask) | (carry & mask);
    if overflow > 0 {
        new_value = increment_bitmask(new_value, overflow << 1);
    }
    new_value
}
