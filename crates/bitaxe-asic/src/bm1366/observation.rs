//! Semantic BM1366 observations and initialization status.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/bm1366.c:BM1366_process_work`
//! - `reference/esp-miner/components/asic/asic_common.c:count_asic_chips`
//! - parity checklist rows `ASIC-003`, `ASIC-004`, and `ASIC-008`

use crate::Bm1366ProtocolFault;

use super::result::{Bm1366NonceResult, Bm1366RegisterRead};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChipId(u16);

impl ChipId {
    #[must_use]
    pub const fn new(chip_id: u16) -> Self {
        Self(chip_id)
    }

    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsicIndex(u8);

impl AsicIndex {
    #[must_use]
    pub const fn new(index: u8) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChipAddress(u8);

impl ChipAddress {
    #[must_use]
    pub const fn new(address: u8) -> Self {
        Self(address)
    }

    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1366Observation {
    ChipId {
        chip_id: ChipId,
        asic_index: AsicIndex,
    },
    RegisterRead(Bm1366RegisterRead),
    JobNonce(Bm1366NonceResult),
    ProtocolFault(Bm1366ProtocolFault),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicInitStatus {
    PreflightMissing { reason: &'static str },
    ChipDetectOnly,
    ChipDetectedNoMining { chips: u8 },
    InitializedNoMining,
    FailClosed { reason: &'static str },
}
