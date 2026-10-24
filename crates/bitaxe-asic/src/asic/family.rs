//! What differs between the chip families the firmware drives.
//!
//! Reference: `reference/esp-miner/components/asic/bm1366.c` and `bm1370.c` (job-id steps and
//! result lookups), `components/asic/asic.c` (`ASIC_get_asic_job_frequency_ms`) and
//! `main/device_config.h` (`default_asic_timeout`).

use super::{ChipIdCountFrameDisposition, JobId, ProductionResult, ValidJobIds};
use crate::{bm1370::Bm1370ProtocolFault, error::Bm1366ProtocolFault};
use thiserror::Error;

/// Why one frame of the chip count failed the count, in the family's own terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ChipCountFrameFault {
    #[error(transparent)]
    Bm1366(#[from] Bm1366ProtocolFault),
    #[error(transparent)]
    Bm1370(#[from] Bm1370ProtocolFault),
}

/// A BM13xx chip family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsicFamily {
    Bm1366,
    Bm1370,
}

impl AsicFamily {
    /// The family of a catalog ASIC model, if the firmware drives it.
    #[must_use]
    pub fn from_model(model: &str) -> Option<Self> {
        match model {
            "BM1366" => Some(Self::Bm1366),
            "BM1370" => Some(Self::Bm1370),
            _ => None,
        }
    }

    #[must_use]
    pub const fn model(self) -> &'static str {
        match self {
            Self::Bm1366 => "BM1366",
            Self::Bm1370 => "BM1370",
        }
    }

    /// How far each new job's id advances, modulo 128.
    #[must_use]
    pub const fn job_id_step(self) -> u8 {
        match self {
            Self::Bm1366 => 8,
            Self::Bm1370 => 24,
        }
    }

    /// The first job's id: upstream advances from zero before sending.
    #[must_use]
    pub const fn first_job_id(self) -> JobId {
        JobId::new(0).advance(self)
    }

    /// How often a new job is sent: the chip's timeout over the chain length, rounded up to a
    /// power of two.
    #[must_use]
    pub const fn job_interval_ms(self, asic_count: u32) -> u32 {
        let timeout_ms = match self {
            Self::Bm1366 => 2_000,
            Self::Bm1370 => 500,
        };
        if asic_count <= 1 {
            return timeout_ms;
        }
        timeout_ms / asic_count.next_power_of_two()
    }

    /// Classifies one frame of the chip count: counted, skipped, or a failed count.
    pub fn classify_chip_id_count_frame(
        self,
        bytes: &[u8],
    ) -> Result<ChipIdCountFrameDisposition, ChipCountFrameFault> {
        match self {
            Self::Bm1366 => Ok(crate::bm1366::chip_detect::classify_chip_id_count_frame(
                bytes,
            )?),
            Self::Bm1370 => Ok(crate::bm1370::plan::classify_chip_id_count_frame(bytes)?),
        }
    }

    /// Classifies one complete receive frame for the production loop.
    #[must_use]
    pub fn classify_production_result(
        self,
        bytes: &[u8],
        valid_jobs: &ValidJobIds,
        address_interval: u16,
    ) -> ProductionResult {
        match self {
            Self::Bm1366 => crate::bm1366::result::classify_bm1366_production_result(
                bytes,
                valid_jobs,
                address_interval,
            ),
            Self::Bm1370 => crate::bm1370::result::classify_production_result(
                bytes,
                valid_jobs,
                address_interval,
            ),
        }
    }
}

#[cfg(test)]
mod tests;
