//! Pure BM1368 initialization, frequency, nonce-space, and baud planning.

use crate::{
    bm1366::{
        frequency_voltage::{actual_frequency_mhz, frequency_plan_for_quarter_mhz},
        mining_ready::{difficulty_mask_value, hash_counting_number},
    },
    bm1368::{
        protocol::{Bm1368Command, RegisterTarget},
        Bm1368ProtocolFault,
    },
};

pub const DEFAULT_FREQUENCY_MHZ: u16 = 490;
pub const DEFAULT_VOLTAGE_MV: u16 = 1166;
pub const DEFAULT_DIFFICULTY: u16 = 256;
pub const CORE_COUNT: u16 = 80;
pub const SMALL_CORE_COUNT: u16 = 1276;
pub const HASH_DOMAINS: u8 = 4;
pub const DEFAULT_ASIC_TIMEOUT_MS: u16 = 500;
pub const DEFAULT_VERSION_MASK: u32 = 0x1fff_e000;
const FREQUENCY_RAMP_START_QUARTER_MHZ: u32 = 50 * 4;
const FREQUENCY_RAMP_STEP_QUARTER_MHZ: u32 = 25;
const FREQUENCY_RAMP_DELAY_MS: u32 = 100;

/// Closed pure configuration used to build a BM1368 init plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bm1368InitConfig {
    pub chip_count: u8,
    pub asic_count: u16,
    pub core_count: u16,
    pub difficulty: f64,
    pub target_frequency_mhz: u16,
    pub nonce_percent: f64,
    pub version_mask: u32,
}

impl Bm1368InitConfig {
    /// Default one-chip Supra-family pure configuration.
    #[must_use]
    pub const fn supra_single_chip() -> Self {
        Self {
            chip_count: 1,
            asic_count: 1,
            core_count: CORE_COUNT,
            difficulty: DEFAULT_DIFFICULTY as f64,
            target_frequency_mhz: DEFAULT_FREQUENCY_MHZ,
            nonce_percent: 1.0,
            version_mask: DEFAULT_VERSION_MASK,
        }
    }

    /// Address interval used to derive per-chip addresses and result identity.
    pub fn address_interval(self) -> Result<u16, Bm1368ProtocolFault> {
        if self.chip_count == 0 {
            return Err(Bm1368ProtocolFault::InvalidChipCount { chip_count: 0 });
        }
        Ok(256 / u16::from(self.chip_count))
    }
}

/// Builds the complete deterministic BM1368 post-reset initialization plan.
pub fn initialization_commands(
    config: Bm1368InitConfig,
) -> Result<Vec<Bm1368Command>, Bm1368ProtocolFault> {
    let address_interval = config.address_interval()?;
    let mut commands = Vec::new();

    for _ in 0..4 {
        commands.push(Bm1368Command::SetVersionMask(config.version_mask));
    }
    commands.push(Bm1368Command::ReadChipId);
    commands.push(Bm1368Command::SetChainInactive);

    for (register, value) in [
        (0xa8, [0x00, 0x07, 0x00, 0x00]),
        (0x18, [0xff, 0x0f, 0xc1, 0x00]),
        (0x3c, [0x80, 0x00, 0x8b, 0x00]),
        (0x3c, [0x80, 0x00, 0x80, 0x18]),
        (0x14, [0x00, 0x00, 0x00, 0xff]),
        (0x54, [0x00, 0x00, 0x00, 0x03]),
        (0x58, [0x02, 0x11, 0x11, 0x11]),
    ] {
        commands.push(Bm1368Command::WriteRegister {
            target: RegisterTarget::All,
            register,
            value,
        });
    }

    for chip_index in 0..config.chip_count {
        commands.push(Bm1368Command::SetChipAddress(chip_address(
            chip_index,
            address_interval,
        )?));
    }

    for chip_index in 0..config.chip_count {
        let asic_address = chip_address(chip_index, address_interval)?;
        for (register, value) in [
            (0xa8, [0x00, 0x07, 0x01, 0xf0]),
            (0x18, [0xf0, 0x00, 0xc1, 0x00]),
            (0x3c, [0x80, 0x00, 0x8b, 0x00]),
            (0x3c, [0x80, 0x00, 0x80, 0x18]),
            (0x3c, [0x80, 0x00, 0x82, 0xaa]),
        ] {
            commands.push(Bm1368Command::WriteRegister {
                target: RegisterTarget::Single { asic_address },
                register,
                value,
            });
        }
        commands.push(Bm1368Command::DelayMs(500));
    }

    commands.push(Bm1368Command::SetDifficultyMask(difficulty_mask_value(
        config.difficulty,
    )));
    commands.extend(frequency_ramp_commands(config.target_frequency_mhz)?);

    let final_plan = frequency_plan(config.target_frequency_mhz)?;
    commands.push(Bm1368Command::SetNonceSpace(hash_counting_number(
        config.nonce_percent,
        actual_frequency_mhz(final_plan),
        config.asic_count,
        config.core_count,
    )));
    commands.push(Bm1368Command::SetVersionMask(config.version_mask));
    Ok(commands)
}

/// Plans the shared 6.25-MHz stepped ramp from 50 MHz to the target.
pub fn frequency_ramp_commands(
    target_frequency_mhz: u16,
) -> Result<Vec<Bm1368Command>, Bm1368ProtocolFault> {
    let target_quarter_mhz = u32::from(target_frequency_mhz) * 4;
    let mut commands = Vec::new();
    let mut current_step = FREQUENCY_RAMP_START_QUARTER_MHZ / FREQUENCY_RAMP_STEP_QUARTER_MHZ;
    let target_step = target_quarter_mhz / FREQUENCY_RAMP_STEP_QUARTER_MHZ;

    while current_step < target_step {
        current_step += 1;
        let frequency_quarter_mhz = current_step * FREQUENCY_RAMP_STEP_QUARTER_MHZ;
        commands.push(Bm1368Command::SetFrequency(frequency_plan_quarters(
            frequency_quarter_mhz,
        )?));
        commands.push(Bm1368Command::DelayMs(FREQUENCY_RAMP_DELAY_MS));
    }

    if current_step * FREQUENCY_RAMP_STEP_QUARTER_MHZ != target_quarter_mhz {
        commands.push(Bm1368Command::SetFrequency(frequency_plan_quarters(
            target_quarter_mhz,
        )?));
    }
    Ok(commands)
}

fn frequency_plan(
    target_frequency_mhz: u16,
) -> Result<crate::bm1366::command::FrequencyPlan, Bm1368ProtocolFault> {
    frequency_plan_quarters(u32::from(target_frequency_mhz) * 4)
}

fn frequency_plan_quarters(
    frequency_quarter_mhz: u32,
) -> Result<crate::bm1366::command::FrequencyPlan, Bm1368ProtocolFault> {
    frequency_plan_for_quarter_mhz(frequency_quarter_mhz).ok_or(
        Bm1368ProtocolFault::InvalidFrequency {
            frequency_quarter_mhz,
        },
    )
}

fn chip_address(chip_index: u8, address_interval: u16) -> Result<u8, Bm1368ProtocolFault> {
    u8::try_from(u16::from(chip_index) * address_interval).map_err(|_| {
        Bm1368ProtocolFault::InvalidChipCount {
            chip_count: chip_index.saturating_add(1),
        }
    })
}
