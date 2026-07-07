//! BM1366 register IDs and command payload builders.
//!
//! Reference breadcrumb: `reference/esp-miner/components/asic/bm1366.c`,
//! parity checklist rows `ASIC-001`, `ASIC-002`, and `ASIC-006`.

use crate::Bm1366ProtocolFault;

pub const CHIP_ID_REGISTER: u8 = 0x00;
pub const ERROR_COUNT_REGISTER: u8 = 0x4c;
pub const DOMAIN0_COUNT_REGISTER: u8 = 0x88;
pub const DOMAIN1_COUNT_REGISTER: u8 = 0x89;
pub const DOMAIN2_COUNT_REGISTER: u8 = 0x8a;
pub const DOMAIN3_COUNT_REGISTER: u8 = 0x8b;
pub const TOTAL_COUNT_REGISTER: u8 = 0x8c;
pub const VERSION_MASK_REGISTER: u8 = 0xa4;
pub const HASH_COUNTING_REGISTER: u8 = 0x10;
pub const FREQUENCY_REGISTER: u8 = 0x08;
pub const MISC_CONTROL_REGISTER: u8 = 0x18;
pub const DIFFICULTY_MASK_REGISTER: u8 = 0x14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Bm1366Register {
    ChipId = CHIP_ID_REGISTER,
    ErrorCount = ERROR_COUNT_REGISTER,
    Domain0Count = DOMAIN0_COUNT_REGISTER,
    Domain1Count = DOMAIN1_COUNT_REGISTER,
    Domain2Count = DOMAIN2_COUNT_REGISTER,
    Domain3Count = DOMAIN3_COUNT_REGISTER,
    TotalCount = TOTAL_COUNT_REGISTER,
}

impl Bm1366Register {
    pub const fn address(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Bm1366Register {
    type Error = Bm1366ProtocolFault;

    fn try_from(register: u8) -> Result<Self, Self::Error> {
        match register {
            CHIP_ID_REGISTER => Ok(Self::ChipId),
            ERROR_COUNT_REGISTER => Ok(Self::ErrorCount),
            DOMAIN0_COUNT_REGISTER => Ok(Self::Domain0Count),
            DOMAIN1_COUNT_REGISTER => Ok(Self::Domain1Count),
            DOMAIN2_COUNT_REGISTER => Ok(Self::Domain2Count),
            DOMAIN3_COUNT_REGISTER => Ok(Self::Domain3Count),
            TOTAL_COUNT_REGISTER => Ok(Self::TotalCount),
            _ => Err(Bm1366ProtocolFault::UnknownRegister { register }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterPayload<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> RegisterPayload<N> {
    pub const fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    pub const fn bytes(self) -> [u8; N] {
        self.bytes
    }

    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.bytes
    }
}

pub const fn read_register_payload(register: u8) -> RegisterPayload<2> {
    RegisterPayload::new([0x00, register])
}

pub const fn write_register_payload(
    asic_address: u8,
    register: u8,
    value: [u8; 4],
) -> RegisterPayload<6> {
    RegisterPayload::new([
        asic_address,
        register,
        value[0],
        value[1],
        value[2],
        value[3],
    ])
}

pub const fn set_address_payload(chip_address: u8) -> RegisterPayload<2> {
    RegisterPayload::new([chip_address, 0x00])
}

pub const fn inactive_chain_payload() -> RegisterPayload<2> {
    RegisterPayload::new([0x00, 0x00])
}

pub const fn version_mask_payload(version_mask: u32) -> RegisterPayload<6> {
    let versions_to_roll = version_mask >> 13;
    RegisterPayload::new([
        0x00,
        VERSION_MASK_REGISTER,
        0x90,
        0x00,
        ((versions_to_roll >> 8) & 0xff) as u8,
        (versions_to_roll & 0xff) as u8,
    ])
}

pub const fn hash_counting_payload(hash_counting_number: u32) -> RegisterPayload<6> {
    RegisterPayload::new([
        0x00,
        HASH_COUNTING_REGISTER,
        ((hash_counting_number >> 24) & 0xff) as u8,
        ((hash_counting_number >> 16) & 0xff) as u8,
        ((hash_counting_number >> 8) & 0xff) as u8,
        (hash_counting_number & 0xff) as u8,
    ])
}

pub const fn frequency_payload(
    vdo_scale: u8,
    fb_divider: u8,
    refdiv: u8,
    postdiv: u8,
) -> RegisterPayload<6> {
    RegisterPayload::new([
        0x00,
        FREQUENCY_REGISTER,
        vdo_scale,
        fb_divider,
        refdiv,
        postdiv,
    ])
}

pub const fn misc_control_payload(value: [u8; 4]) -> RegisterPayload<6> {
    write_register_payload(0x00, MISC_CONTROL_REGISTER, value)
}

pub const fn difficulty_mask_payload(mask: [u8; 4]) -> RegisterPayload<6> {
    write_register_payload(0x00, DIFFICULTY_MASK_REGISTER, mask)
}
