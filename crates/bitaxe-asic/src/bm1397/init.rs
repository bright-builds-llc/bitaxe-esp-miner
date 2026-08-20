//! Pure BM1397 initialization and frequency-transition planning.

use crate::{
    bm1366::mining_ready::difficulty_mask_value,
    bm1397::{
        frequency::frequency_write_sequence,
        protocol::{Bm1397Command, RegisterTarget},
        Bm1397ProtocolFault,
    },
};

pub const DEFAULT_FREQUENCY_MHZ: u16 = 425;
pub const DEFAULT_VOLTAGE_MV: u16 = 1400;
pub const DEFAULT_DIFFICULTY: u16 = 256;
pub const CORE_COUNT: u16 = 168;
pub const SMALL_CORE_COUNT: u16 = 672;
pub const HASH_DOMAINS: u8 = 1;
pub const DEFAULT_ASIC_TIMEOUT_MS: u16 = 20;
const INITIAL_FREQUENCY_MHZ: u16 = 50;
const FREQUENCY_STEP_QUARTER_MHZ: u32 = 25;
const FREQUENCY_STEP_DELAY_MS: u32 = 100;

/// Closed pure configuration used to build a BM1397 init plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bm1397InitConfig {
    pub chip_count: u8,
    pub difficulty: f64,
    pub target_frequency_mhz: u16,
}

impl Bm1397InitConfig {
    /// Default one-chip Max-family pure configuration.
    #[must_use]
    pub const fn max_single_chip() -> Self {
        Self {
            chip_count: 1,
            difficulty: DEFAULT_DIFFICULTY as f64,
            target_frequency_mhz: DEFAULT_FREQUENCY_MHZ,
        }
    }

    /// Address interval used to derive per-chip addresses and result identity.
    pub fn address_interval(self) -> Result<u16, Bm1397ProtocolFault> {
        if self.chip_count == 0 {
            return Err(Bm1397ProtocolFault::InvalidChipCount { chip_count: 0 });
        }
        Ok(256 / u16::from(self.chip_count))
    }
}

/// Builds the complete deterministic BM1397 post-reset initialization plan.
pub fn initialization_commands(
    config: Bm1397InitConfig,
) -> Result<Vec<Bm1397Command>, Bm1397ProtocolFault> {
    let address_interval = config.address_interval()?;
    let mut commands = vec![
        Bm1397Command::ReadChipId,
        Bm1397Command::DelayMs(20),
        Bm1397Command::SetChainInactive,
    ];

    for chip_index in 0..config.chip_count {
        commands.push(Bm1397Command::SetChipAddress(chip_address(
            chip_index,
            address_interval,
        )?));
    }

    for (register, value) in [
        (0x80, [0x00, 0x00, 0x00, 0x00]),
        (0x84, [0x00, 0x00, 0x00, 0x00]),
        (0x20, [0x00, 0x00, 0x00, 0x01]),
        (0x3c, [0x80, 0x00, 0x80, 0x74]),
    ] {
        commands.push(Bm1397Command::WriteRegister {
            target: RegisterTarget::All,
            register,
            value,
        });
    }

    commands.push(Bm1397Command::SetDifficultyMask(difficulty_mask_value(
        config.difficulty,
    )));
    commands.push(Bm1397Command::WriteRegister {
        target: RegisterTarget::All,
        register: 0x68,
        value: [0xc0, 0x70, 0x01, 0x11],
    });
    commands.push(Bm1397Command::WriteRegister {
        target: RegisterTarget::All,
        register: 0x28,
        value: [0x06, 0x00, 0x00, 0x0f],
    });
    commands.push(Bm1397Command::SetDefaultBaud);
    commands.extend(frequency_transition_commands(
        INITIAL_FREQUENCY_MHZ,
        config.target_frequency_mhz,
    )?);
    Ok(commands)
}

/// Plans the shared 6.25-MHz transition and expands every BM1397 frequency
/// operation into its model-specific duplicated writes.
pub fn frequency_transition_commands(
    current_frequency_mhz: u16,
    target_frequency_mhz: u16,
) -> Result<Vec<Bm1397Command>, Bm1397ProtocolFault> {
    let mut current_quarter_mhz = u32::from(current_frequency_mhz) * 4;
    let target_quarter_mhz = u32::from(target_frequency_mhz) * 4;
    if current_quarter_mhz == target_quarter_mhz {
        return Ok(Vec::new());
    }

    if current_quarter_mhz.abs_diff(target_quarter_mhz) < FREQUENCY_STEP_QUARTER_MHZ {
        return frequency_write_sequence(target_quarter_mhz);
    }

    let increasing = target_quarter_mhz > current_quarter_mhz;
    let mut current_step = transition_step(current_quarter_mhz, increasing);
    let target_step = transition_step(target_quarter_mhz, increasing);
    let mut commands = Vec::new();

    while (increasing && current_step < target_step) || (!increasing && current_step > target_step)
    {
        if increasing {
            current_step += 1;
        } else {
            current_step -= 1;
        }
        current_quarter_mhz = current_step * FREQUENCY_STEP_QUARTER_MHZ;
        commands.extend(frequency_write_sequence(current_quarter_mhz)?);
        commands.push(Bm1397Command::DelayMs(FREQUENCY_STEP_DELAY_MS));
    }

    if current_quarter_mhz != target_quarter_mhz {
        commands.extend(frequency_write_sequence(target_quarter_mhz)?);
    }
    Ok(commands)
}

const fn transition_step(frequency_quarter_mhz: u32, increasing: bool) -> u32 {
    if increasing {
        frequency_quarter_mhz / FREQUENCY_STEP_QUARTER_MHZ
    } else {
        frequency_quarter_mhz.div_ceil(FREQUENCY_STEP_QUARTER_MHZ)
    }
}

fn chip_address(chip_index: u8, address_interval: u16) -> Result<u8, Bm1397ProtocolFault> {
    u8::try_from(u16::from(chip_index) * address_interval).map_err(|_| {
        Bm1397ProtocolFault::InvalidChipCount {
            chip_count: chip_index.saturating_add(1),
        }
    })
}
