//! Independent BM1366 endpoint oracle based on pinned upstream wire definitions.
//!
//! The literal discovery request is `bm1366.c:BM1366_init`; its reply is the
//! retained Phase 28.1 J3 hardware frame documented by the production parser.
//! CRC arithmetic here uses polynomial division, not the production codec.
use crate::ModelError;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const DISCOVERY_REQUEST: [u8; 7] = [0x55, 0xaa, 0x52, 0x05, 0x00, 0x00, 0x0a];
pub const DISCOVERY_REPLY: [u8; 11] = [0xaa, 0x55, 0x13, 0x66, 0, 0, 0, 0, 0, 0, 5];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonceFixture {
    pub work_payload: [u8; 82],
    pub header: [u8; 80],
    pub nonce: u32,
    /// Bitcoin comparison representation: reversed double-SHA digest.
    pub target_be: [u8; 32],
    pub provenance: String,
}
impl NonceFixture {
    /// Proves the injected nonce against independently reconstructed ASIC work.
    pub fn validate(&self) -> Result<[u8; 32], ModelError> {
        if self.provenance.is_empty() || self.work_payload[1] != 1 {
            return Err(ModelError::Invalid("nonce fixture provenance/midstates"));
        }
        let reconstructed = header_from_payload(&self.work_payload, self.nonce);
        if reconstructed != self.header {
            return Err(ModelError::Invalid("nonce fixture header/work mismatch"));
        }
        let mut hash: [u8; 32] = Sha256::digest(Sha256::digest(self.header)).into();
        hash.reverse();
        if hash > self.target_be {
            return Err(ModelError::Invalid("nonce does not meet target"));
        }
        Ok(hash)
    }
}

/// Reconstructs the Bitcoin header from ASIC byte fields without a miner codec.
pub fn header_from_payload(payload: &[u8; 82], nonce: u32) -> [u8; 80] {
    let mut header = [0; 80];
    header[..4].copy_from_slice(&payload[78..82]);
    for word in 0..8 {
        header[4 + word * 4..8 + word * 4]
            .copy_from_slice(&payload[46 + (7 - word) * 4..50 + (7 - word) * 4]);
        header[36 + word * 4..40 + word * 4]
            .copy_from_slice(&payload[14 + (7 - word) * 4..18 + (7 - word) * 4]);
    }
    header[68..72].copy_from_slice(&payload[10..14]);
    header[72..76].copy_from_slice(&payload[6..10]);
    header[76..].copy_from_slice(&nonce.to_le_bytes());
    header
}

#[derive(Debug, Clone)]
pub struct Bm1366 {
    pub power_enabled: bool,
    pub reset_asserted: bool,
    pub host_baud: u32,
    pub chip_baud: u32,
    pub address: u8,
    pub dispatched_jobs: u64,
    pub frequency_mhz: u32,
    pub registers: BTreeMap<u8, [u8; 4]>,
    pub maybe_nonce_fixture: Option<NonceFixture>,
    pub maybe_last_payload: Option<[u8; 82]>,
}
impl Default for Bm1366 {
    fn default() -> Self {
        Self {
            power_enabled: false,
            reset_asserted: true,
            host_baud: 115200,
            chip_baud: 115200,
            address: 0,
            dispatched_jobs: 0,
            frequency_mhz: 0,
            registers: BTreeMap::new(),
            maybe_nonce_fixture: None,
            maybe_last_payload: None,
        }
    }
}
impl Bm1366 {
    pub fn reset(&mut self) {
        self.chip_baud = 115200;
        self.address = 0;
        self.frequency_mhz = 0;
        self.registers.clear();
        self.maybe_last_payload = None;
    }
    pub fn install_nonce_fixture(&mut self, fixture: NonceFixture) -> Result<(), ModelError> {
        fixture.validate()?;
        self.maybe_nonce_fixture = Some(fixture);
        Ok(())
    }
    pub fn exchange(&mut self, frame: &[u8]) -> Result<Vec<u8>, ModelError> {
        if !self.power_enabled || self.reset_asserted {
            return Err(ModelError::Unavailable("ASIC power/reset"));
        }
        if self.host_baud != self.chip_baud {
            return Err(ModelError::Unavailable("ASIC baud mismatch"));
        }
        if frame.len() < 7 || frame[..2] != [0x55, 0xaa] || usize::from(frame[3]) + 2 != frame.len()
        {
            return Err(ModelError::Invalid("ASIC framing"));
        }
        if frame[2] & 0x20 != 0 {
            return self.work(frame);
        }
        if crc5(&frame[2..frame.len() - 1]) != frame[frame.len() - 1] {
            return Err(ModelError::Invalid("ASIC command CRC"));
        }
        if !matches!(frame[2] & 0xf0, 0x40 | 0x50) {
            return Err(ModelError::Unsupported("ASIC command header"));
        }
        let data = &frame[4..frame.len() - 1];
        if data.len() < 2 {
            return Err(ModelError::Invalid("ASIC command length"));
        }
        if frame[2] & 0x10 == 0 && frame[2] & 0x0f != 0 && data[0] != self.address {
            return Err(ModelError::Unavailable("ASIC address"));
        }
        match frame[2] & 0x0f {
            0 if data.len() == 2 => {
                self.address = data[0];
                Ok(Vec::new())
            }
            1 if data.len() == 6 => {
                self.write_register(data[1], [data[2], data[3], data[4], data[5]])
            }
            2 if data.len() == 2 => self.read_register(data[1]),
            3 if data.len() == 2 => {
                self.maybe_last_payload = None;
                Ok(Vec::new())
            }
            _ => Err(ModelError::Unsupported("ASIC command")),
        }
    }
    fn write_register(&mut self, register: u8, value: [u8; 4]) -> Result<Vec<u8>, ModelError> {
        if !matches!(
            register,
            0x08 | 0x10 | 0x14 | 0x18 | 0x28 | 0x2c | 0x3c | 0x54 | 0x58 | 0xa4 | 0xa8
        ) {
            return Err(ModelError::Unsupported("ASIC write register"));
        }
        if register == 0x08 {
            let refdiv = u32::from(value[2]);
            if refdiv == 0 {
                return Err(ModelError::Invalid("ASIC PLL divider"));
            }
            self.frequency_mhz = 25 * u32::from(value[1])
                / refdiv
                / (u32::from(value[3] >> 4) + 1)
                / (u32::from(value[3] & 15) + 1);
        }
        if register == 0x18 && value == [0x00, 0x00, 0x7a, 0x31] {
            self.chip_baud = 115200;
        }
        if register == 0x28 && value == [0x11, 0x30, 0x02, 0x00] {
            self.chip_baud = 1000000;
        }
        self.registers.insert(register, value);
        Ok(Vec::new())
    }
    fn read_register(&self, register: u8) -> Result<Vec<u8>, ModelError> {
        if register == 0 {
            return Ok(DISCOVERY_REPLY.to_vec());
        }
        let value = if matches!(register, 0x4c | 0x88..=0x8c) {
            [0; 4]
        } else {
            *self
                .registers
                .get(&register)
                .ok_or(ModelError::Unsupported("ASIC read register"))?
        };
        let mut reply = [0xaa, 0x55, 0, 0, 0, 0, self.address, register, 0, 0, 0];
        reply[2..6].copy_from_slice(&value);
        reply[10] = response_crc(&reply[2..10], false)?;
        Ok(reply.to_vec())
    }
    fn work(&mut self, frame: &[u8]) -> Result<Vec<u8>, ModelError> {
        if frame.len() != 88 || frame[2] != 0x21 || frame[5] != 1 {
            return Err(ModelError::Unsupported("ASIC work layout"));
        }
        if crc16_false(&frame[2..86]) != u16::from_be_bytes([frame[86], frame[87]]) {
            return Err(ModelError::Invalid("ASIC job CRC"));
        }
        if self.frequency_mhz == 0 {
            return Err(ModelError::Unavailable("ASIC PLL not initialized"));
        }
        let mut payload = [0; 82];
        payload.copy_from_slice(&frame[4..86]);
        self.dispatched_jobs = self
            .dispatched_jobs
            .checked_add(1)
            .ok_or(ModelError::Invalid("ASIC job count overflow"))?;
        self.maybe_last_payload = Some(payload);
        let Some(fixture) = &self.maybe_nonce_fixture else {
            return Ok(Vec::new());
        };
        if fixture.work_payload != payload {
            return Ok(Vec::new());
        }
        fixture.validate()?;
        let mut reply = [0xaa, 0x55, 0, 0, 0, 0, 1, payload[0], 0, 0, 0];
        reply[2..6].copy_from_slice(&fixture.nonce.to_le_bytes());
        reply[10] = response_crc(&reply[2..10], true)?;
        Ok(reply.to_vec())
    }
}

fn polynomial_crc(bytes: &[u8], width: u32, polynomial: u32, initial: u32) -> u32 {
    let mut remainder = initial;
    let top = 1 << width;
    for byte in bytes {
        for bit in (0..8).rev() {
            remainder = (remainder << 1) ^ (u32::from(byte >> bit) & 1);
            if remainder & top != 0 {
                remainder ^= polynomial;
            }
        }
    }
    remainder
}
/// Independent endpoint CRC5 for x^5+x^2+1 (direct, augmented-input form).
pub fn crc5(bytes: &[u8]) -> u8 {
    // Feed each wire bit into a high-bit polynomial divider with feedback.
    let mut remainder = 31_u32;
    for byte in bytes {
        for bit in (0..8).rev() {
            let feedback = (remainder >> 4) ^ (u32::from(byte >> bit) & 1);
            remainder = (remainder << 1) & 31;
            if feedback & 1 != 0 {
                remainder ^= 5;
            }
        }
    }
    remainder as u8
}
pub fn crc16_false(bytes: &[u8]) -> u16 {
    // Augment with zero bits; pre-xor input into the upper register as CCITT.
    let mut remainder = 0xffff_u32;
    for byte in bytes {
        remainder ^= u32::from(*byte) << 8;
        remainder = polynomial_crc(&[0], 16, 0x11021, remainder);
    }
    remainder as u16
}
fn response_crc(body: &[u8], job: bool) -> Result<u8, ModelError> {
    let mut input = [0; 9];
    input[..8].copy_from_slice(body);
    for checksum in 0..32 {
        input[8] = checksum | if job { 0x80 } else { 0 };
        if crc5(&input) == 0 {
            return Ok(input[8]);
        }
    }
    Err(ModelError::Invalid("ASIC response CRC residue"))
}
