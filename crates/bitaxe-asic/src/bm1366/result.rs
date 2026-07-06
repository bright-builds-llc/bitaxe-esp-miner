//! BM1366 result frame parsing.
//!
//! Reference breadcrumb: `reference/esp-miner/components/asic/bm1366.c:BM1366_process_work`
//! and `reference/esp-miner/components/asic/asic_common.c:receive_work`,
//! parity checklist rows `ASIC-004`, `ASIC-006`, and `ASIC-008`.

use crate::Bm1366ProtocolFault;

use super::{
    crc::crc5,
    registers::Bm1366Register,
    work::{Bm1366JobId, JOB_ID_MODULUS},
};

pub const BM1366_RECEIVE_PREAMBLE: u16 = 0xaa55;
pub const BM1366_RESULT_FRAME_LEN: usize = 11;
pub const RESULT_WORK_TIMEOUT_MS: u32 = 10_000;
pub const BM1366_NORMAL_CORE_COUNT: u8 = 112;
pub const BM1366_SMALL_CORE_IDS: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultFrameBytes([u8; BM1366_RESULT_FRAME_LEN]);

impl ResultFrameBytes {
    pub const fn new(bytes: [u8; BM1366_RESULT_FRAME_LEN]) -> Self {
        Self(bytes)
    }

    pub fn try_from_slice(bytes: &[u8]) -> Result<Self, Bm1366ProtocolFault> {
        if bytes.len() != BM1366_RESULT_FRAME_LEN {
            return Err(Bm1366ProtocolFault::InvalidLength {
                expected: BM1366_RESULT_FRAME_LEN,
                actual: bytes.len(),
            });
        }

        let mut frame = [0; BM1366_RESULT_FRAME_LEN];
        frame.copy_from_slice(bytes);
        Ok(Self::new(frame))
    }

    pub const fn bytes(&self) -> &[u8; BM1366_RESULT_FRAME_LEN] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bm1366ValidJobIds {
    valid: [bool; JOB_ID_MODULUS as usize],
}

impl Bm1366ValidJobIds {
    pub const fn empty() -> Self {
        Self {
            valid: [false; JOB_ID_MODULUS as usize],
        }
    }

    pub fn single(job_id: Bm1366JobId) -> Self {
        let mut valid_jobs = Self::empty();
        valid_jobs.insert(job_id);
        valid_jobs
    }

    pub fn insert(&mut self, job_id: Bm1366JobId) {
        self.valid[usize::from(job_id.lookup_key().raw())] = true;
    }

    pub fn contains(&self, job_id: Bm1366JobId) -> bool {
        self.valid[usize::from(job_id.lookup_key().raw())]
    }
}

impl Default for Bm1366ValidJobIds {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1366ParsedResult {
    JobNonce(Bm1366NonceResult),
    RegisterRead(Bm1366RegisterRead),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1366NonceResult {
    pub job_id: Bm1366JobId,
    pub nonce: u32,
    pub asic_index: u8,
    pub core_id: u8,
    pub small_core_id: u8,
    pub version_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1366RegisterRead {
    pub register: Bm1366Register,
    pub asic_index: u8,
    pub asic_address: u8,
    pub value: u32,
}

pub fn parse_bm1366_result_frame(
    bytes: &[u8],
    valid_jobs: &Bm1366ValidJobIds,
    address_interval: u16,
) -> Result<Bm1366ParsedResult, Bm1366ProtocolFault> {
    let frame = ResultFrameBytes::try_from_slice(bytes)?;
    validate_result_frame(frame)?;

    if (frame.bytes()[10] & 0x80) != 0 {
        return parse_job_result(frame, valid_jobs, address_interval);
    }

    parse_register_read(frame, address_interval)
}

fn validate_result_frame(frame: ResultFrameBytes) -> Result<(), Bm1366ProtocolFault> {
    let bytes = frame.bytes();
    let actual_preamble = u16::from_be_bytes([bytes[0], bytes[1]]);

    if actual_preamble != BM1366_RECEIVE_PREAMBLE {
        return Err(Bm1366ProtocolFault::BadPreamble {
            expected: BM1366_RECEIVE_PREAMBLE,
            actual: actual_preamble,
        });
    }

    if crc5(&bytes[2..]) != 0 {
        return Err(Bm1366ProtocolFault::BadCrc);
    }

    Ok(())
}

fn parse_job_result(
    frame: ResultFrameBytes,
    valid_jobs: &Bm1366ValidJobIds,
    address_interval: u16,
) -> Result<Bm1366ParsedResult, Bm1366ProtocolFault> {
    let bytes = frame.bytes();
    let nonce_be = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    let id = bytes[7];
    let raw_lookup_key = id & 0xf8;

    if raw_lookup_key >= JOB_ID_MODULUS {
        return Err(Bm1366ProtocolFault::InvalidJobId {
            job_id: raw_lookup_key,
        });
    }

    let job_id = Bm1366JobId::new(raw_lookup_key);
    if !valid_jobs.contains(job_id) {
        return Err(Bm1366ProtocolFault::InvalidJobId {
            job_id: job_id.raw(),
        });
    }

    let core_id = ((nonce_be >> 25) & 0x7f) as u8;
    if core_id >= BM1366_NORMAL_CORE_COUNT {
        return Err(Bm1366ProtocolFault::InvalidCoreId { core_id });
    }

    let address_interval = valid_address_interval(address_interval)?;
    let asic_index = (u16::from(((nonce_be >> 17) & 0xff) as u8) / address_interval) as u8;
    let small_core_id = id & 0x07;
    let version_be = u16::from_be_bytes([bytes[8], bytes[9]]);
    let version_bits = (u32::from(version_be)) << 13;

    Ok(Bm1366ParsedResult::JobNonce(Bm1366NonceResult {
        job_id,
        nonce: nonce_be,
        asic_index,
        core_id,
        small_core_id,
        version_bits,
    }))
}

fn parse_register_read(
    frame: ResultFrameBytes,
    address_interval: u16,
) -> Result<Bm1366ParsedResult, Bm1366ProtocolFault> {
    let bytes = frame.bytes();
    let value = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
    let asic_address = bytes[6];
    let register = Bm1366Register::try_from(bytes[7])?;
    let address_interval = valid_address_interval(address_interval)?;
    let asic_index = (u16::from(asic_address) / address_interval) as u8;

    Ok(Bm1366ParsedResult::RegisterRead(Bm1366RegisterRead {
        register,
        asic_index,
        asic_address,
        value,
    }))
}

fn valid_address_interval(address_interval: u16) -> Result<u16, Bm1366ProtocolFault> {
    if address_interval == 0 {
        return Err(Bm1366ProtocolFault::ChipCountMismatch {
            expected: 1,
            actual: 0,
        });
    }

    Ok(address_interval)
}

#[cfg(test)]
mod tests {
    use crate::{
        bm1366::{crc::crc5, registers::Bm1366Register, work::Bm1366JobId},
        Bm1366ProtocolFault,
    };

    use super::{
        parse_bm1366_result_frame, Bm1366ParsedResult, Bm1366ValidJobIds, BM1366_RECEIVE_PREAMBLE,
        BM1366_RESULT_FRAME_LEN,
    };

    fn crc_residue_byte(body: [u8; 8], is_job_response: bool) -> u8 {
        let response_bit = if is_job_response { 0x80 } else { 0x00 };

        for crc in 0..32 {
            let candidate = response_bit | crc;
            let mut residue_input = [0; 9];
            residue_input[..8].copy_from_slice(&body);
            residue_input[8] = candidate;

            if crc5(&residue_input) == 0 {
                return candidate;
            }
        }

        panic!("fixture body must have a CRC5 residue byte");
    }

    fn result_frame(body: [u8; 8], is_job_response: bool) -> [u8; 11] {
        let mut frame = [0; BM1366_RESULT_FRAME_LEN];
        frame[0..2].copy_from_slice(&BM1366_RECEIVE_PREAMBLE.to_be_bytes());
        frame[2..10].copy_from_slice(&body);
        frame[10] = crc_residue_byte(body, is_job_response);
        frame
    }

    fn job_body(result_id: u8, nonce_be: u32, version_be: u16) -> [u8; 8] {
        let mut body = [0; 8];
        body[0..4].copy_from_slice(&nonce_be.to_be_bytes());
        body[4] = 0x01;
        body[5] = result_id;
        body[6..8].copy_from_slice(&version_be.to_be_bytes());
        body
    }

    fn register_body(register: u8) -> [u8; 8] {
        [0x01, 0x02, 0x03, 0x04, 0x20, register, 0x00, 0x00]
    }

    #[test]
    fn bm1366_result_valid_job_response_parses_nonce_observation() {
        // Arrange
        let nonce_be = (2_u32 << 25) | (0x20_u32 << 17) | 0x1234;
        let frame = result_frame(job_body(0x2f, nonce_be, 0x0003), true);
        let valid_jobs = Bm1366ValidJobIds::single(Bm1366JobId::new(0x28));

        // Act
        let parsed = parse_bm1366_result_frame(&frame, &valid_jobs, 16)
            .expect("valid result frame should parse");

        // Assert
        assert!(matches!(parsed, Bm1366ParsedResult::JobNonce(_)));
    }

    #[test]
    fn bm1366_result_job_id_decodes_lookup_key_and_small_core() {
        // Arrange
        let nonce_be = (2_u32 << 25) | (0x20_u32 << 17) | 0x1234;
        let frame = result_frame(job_body(0x2f, nonce_be, 0x0003), true);
        let valid_jobs = Bm1366ValidJobIds::single(Bm1366JobId::new(0x28));

        // Act
        let parsed = parse_bm1366_result_frame(&frame, &valid_jobs, 16)
            .expect("valid result frame should parse");

        // Assert
        let Bm1366ParsedResult::JobNonce(result) = parsed else {
            panic!("expected job nonce");
        };
        assert_eq!(result.job_id.raw(), 0x28);
        assert_eq!(result.small_core_id, 7);
    }

    #[test]
    fn bm1366_result_core_id_rejects_values_outside_normal_range() {
        // Arrange
        let nonce_be = (112_u32 << 25) | (0x20_u32 << 17) | 0x1234;
        let frame = result_frame(job_body(0x28, nonce_be, 0x0003), true);
        let valid_jobs = Bm1366ValidJobIds::single(Bm1366JobId::new(0x28));

        // Act
        let parsed = parse_bm1366_result_frame(&frame, &valid_jobs, 16);

        // Assert
        assert_eq!(
            parsed,
            Err(Bm1366ProtocolFault::InvalidCoreId { core_id: 112 })
        );
    }

    #[test]
    fn bm1366_result_register_response_maps_known_registers() {
        // Arrange
        let frame = result_frame(register_body(0x8c), false);
        let valid_jobs = Bm1366ValidJobIds::empty();

        // Act
        let parsed = parse_bm1366_result_frame(&frame, &valid_jobs, 16)
            .expect("valid register frame should parse");

        // Assert
        let Bm1366ParsedResult::RegisterRead(read) = parsed else {
            panic!("expected register read");
        };
        assert_eq!(read.register, Bm1366Register::TotalCount);
        assert_eq!(read.value, 0x01020304);
        assert_eq!(read.asic_index, 2);
    }

    #[test]
    fn bm1366_result_register_response_rejects_unknown_register() {
        // Arrange
        let frame = result_frame(register_body(0xff), false);
        let valid_jobs = Bm1366ValidJobIds::empty();

        // Act
        let parsed = parse_bm1366_result_frame(&frame, &valid_jobs, 16);

        // Assert
        assert_eq!(
            parsed,
            Err(Bm1366ProtocolFault::UnknownRegister { register: 0xff })
        );
    }

    #[test]
    fn bm1366_result_malformed_frames_fail_closed() {
        // Arrange
        let valid_jobs = Bm1366ValidJobIds::single(Bm1366JobId::new(0x28));
        let frame = result_frame(job_body(0x28, 0x0400_0000, 0x0003), true);
        let mut bad_preamble = frame;
        bad_preamble[0] = 0x00;
        let mut bad_crc = frame;
        bad_crc[10] ^= 0x01;

        // Act
        let timeout = parse_bm1366_result_frame(&[], &valid_jobs, 16);
        let partial = parse_bm1366_result_frame(&frame[..10], &valid_jobs, 16);
        let preamble = parse_bm1366_result_frame(&bad_preamble, &valid_jobs, 16);
        let crc = parse_bm1366_result_frame(&bad_crc, &valid_jobs, 16);

        // Assert
        assert_eq!(
            timeout,
            Err(Bm1366ProtocolFault::InvalidLength {
                expected: BM1366_RESULT_FRAME_LEN,
                actual: 0
            })
        );
        assert_eq!(
            partial,
            Err(Bm1366ProtocolFault::InvalidLength {
                expected: BM1366_RESULT_FRAME_LEN,
                actual: 10
            })
        );
        assert!(matches!(
            preamble,
            Err(Bm1366ProtocolFault::BadPreamble { .. })
        ));
        assert_eq!(crc, Err(Bm1366ProtocolFault::BadCrc));
    }
}
