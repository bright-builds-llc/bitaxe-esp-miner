//! Pure BM1370 initialization, frequency, nonce-space and timing planning.
//!
//! Reference: `reference/esp-miner/components/asic/bm1370.c` (`BM1370_init`,
//! `BM1370_send_hash_frequency`, `BM1370_set_nonce_space`), `components/asic/pll.c`,
//! `components/asic/frequency_transition_bmXX.c`, `components/asic/asic.c` (job interval),
//! `main/tasks/power_management_task.c` (the 50 MHz start, the expected hashrate),
//! `components/stratum/include/utils.h` (the default version mask) and
//! `main/device_config.h` (`ASIC_BM1370`).

use crate::{
    bm1366::{
        command::FrequencyPlan, frequency_voltage::frequency_plan_with_bounds,
        mining_ready::difficulty_mask_value,
    },
    bm1370::{
        protocol::{Bm1370Command, RegisterTarget},
        Bm1370ProtocolFault,
    },
};

pub const DEFAULT_FREQUENCY_MHZ: u16 = 525;
pub const DEFAULT_VOLTAGE_MV: u16 = 1150;
pub const DEFAULT_DIFFICULTY: u16 = 256;
pub const CORE_COUNT: u16 = 128;
pub const SMALL_CORE_COUNT: u16 = 2040;
pub const HASH_DOMAINS: u8 = 4;
pub const DEFAULT_ASIC_TIMEOUT_MS: u32 = 500;
pub const DEFAULT_VERSION_MASK: u32 = 0x1fff_e000;
/// The frequency a chip runs at after reset, where every ramp starts.
pub const RESET_FREQUENCY_QUARTER_MHZ: u32 = 50 * 4;
/// The BM1370's PLL feedback-divider range.
pub const FB_DIVIDER_MIN: u16 = 160;
pub const FB_DIVIDER_MAX: u16 = 239;
const FREQUENCY_RAMP_STEP_QUARTER_MHZ: u32 = 25;
const FREQUENCY_RAMP_DELAY_MS: u32 = 100;
const NONCE_SPACE: f32 = 4_294_967_296.0;
const FREQ_MULT_MHZ: f64 = 25.0;
/// The BM1370's hash-counting errata: 134 per clock cycle, doubled to avoid duplicates.
const HASH_COUNTING_ERRATA: f64 = 2.0 * 134.0;

/// Closed pure configuration used to build a BM1370 init plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bm1370InitConfig {
    pub chip_count: u8,
    pub core_count: u16,
    pub difficulty: f64,
    pub target_frequency_mhz: u16,
    pub nonce_percent: f64,
    pub version_mask: u32,
}

impl Bm1370InitConfig {
    /// The Gamma 601's one-chip defaults (`ASIC_BM1370`, `config-601.cvs`).
    #[must_use]
    pub const fn gamma_single_chip() -> Self {
        Self {
            chip_count: 1,
            core_count: CORE_COUNT,
            difficulty: DEFAULT_DIFFICULTY as f64,
            target_frequency_mhz: DEFAULT_FREQUENCY_MHZ,
            nonce_percent: 1.0,
            version_mask: DEFAULT_VERSION_MASK,
        }
    }

    /// Address interval used to derive per-chip addresses and result identity.
    pub fn address_interval(self) -> Result<u16, Bm1370ProtocolFault> {
        if self.chip_count == 0 {
            return Err(Bm1370ProtocolFault::InvalidChipCount { chip_count: 0 });
        }
        Ok(256 / u16::from(self.chip_count))
    }
}

/// The init plan, split where the adapter must count the chips: after the `CHIP_ID` read,
/// before anything that addresses them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bm1370InitPlan {
    pub before_chip_detect: Vec<Bm1370Command>,
    pub after_chip_detect: Vec<Bm1370Command>,
}

/// Builds the complete deterministic BM1370 post-reset initialization plan, in upstream's
/// order. The max-baud switch follows it separately (`Bm1370Command::SetMaxBaud`).
pub fn initialization_plan(
    config: Bm1370InitConfig,
) -> Result<Bm1370InitPlan, Bm1370ProtocolFault> {
    let address_interval = config.address_interval()?;
    let mut before_chip_detect = vec![Bm1370Command::SetVersionMask(config.version_mask); 3];
    before_chip_detect.push(Bm1370Command::ReadChipId);

    let mut after = vec![Bm1370Command::SetVersionMask(config.version_mask)];
    after.extend(write_all(&[
        (0xa8, [0x00, 0x07, 0x00, 0x00]),
        (0x18, [0xf0, 0x00, 0xc1, 0x00]),
    ]));
    after.push(Bm1370Command::SetChainInactive);
    for chip_index in 0..config.chip_count {
        after.push(Bm1370Command::SetChipAddress(chip_address(
            chip_index,
            address_interval,
        )?));
    }
    after.extend(write_all(&[
        (0x3c, [0x80, 0x00, 0x8b, 0x00]),
        (0x3c, [0x80, 0x00, 0x80, 0x0c]),
    ]));
    after.push(Bm1370Command::SetDifficultyMask(difficulty_mask_value(
        config.difficulty,
    )));
    after.extend(write_all(&[(0x58, [0x00, 0x01, 0x11, 0x11])]));
    for chip_index in 0..config.chip_count {
        let asic_address = chip_address(chip_index, address_interval)?;
        for (register, value) in [
            (0xa8, [0x00, 0x07, 0x01, 0xf0]),
            (0x18, [0xf0, 0x00, 0xc1, 0x00]),
            (0x3c, [0x80, 0x00, 0x8b, 0x00]),
            (0x3c, [0x80, 0x00, 0x80, 0x0c]),
            (0x3c, [0x80, 0x00, 0x82, 0xaa]),
        ] {
            after.push(Bm1370Command::WriteRegister {
                target: RegisterTarget::Single { asic_address },
                register,
                value,
            });
        }
    }
    after.extend(write_all(&[
        (0xb9, [0x00, 0x00, 0x44, 0x80]),
        (0x54, [0x00, 0x00, 0x00, 0x02]),
        (0xb9, [0x00, 0x00, 0x44, 0x80]),
        (0x3c, [0x80, 0x00, 0x8d, 0xee]),
    ]));
    let target_quarter_mhz = u32::from(config.target_frequency_mhz) * 4;
    after.extend(frequency_ramp_commands(
        RESET_FREQUENCY_QUARTER_MHZ,
        target_quarter_mhz,
    )?);
    // Upstream sizes the initial nonce space from the requested frequency, not the achieved one.
    after.push(Bm1370Command::SetNonceSpace(hash_counting_number(
        config.nonce_percent,
        f32::from(config.target_frequency_mhz),
        u16::from(config.chip_count),
        config.core_count,
    )?));
    Ok(Bm1370InitPlan {
        before_chip_detect,
        after_chip_detect: after,
    })
}

/// The stepped ramp between two frequencies: whole 6.25 MHz steps 100 ms apart, then the exact
/// target. Under one step away, the target is set directly; at the target, nothing is sent.
pub fn frequency_ramp_commands(
    from_quarter_mhz: u32,
    to_quarter_mhz: u32,
) -> Result<Vec<Bm1370Command>, Bm1370ProtocolFault> {
    let step = FREQUENCY_RAMP_STEP_QUARTER_MHZ;
    if from_quarter_mhz == to_quarter_mhz {
        return Ok(Vec::new());
    }
    if from_quarter_mhz.abs_diff(to_quarter_mhz) < step {
        return Ok(vec![Bm1370Command::SetFrequency(frequency_plan(
            to_quarter_mhz,
        )?)]);
    }
    let rising = to_quarter_mhz > from_quarter_mhz;
    let (mut current_step, target_step) = if rising {
        (from_quarter_mhz / step, to_quarter_mhz / step)
    } else {
        (
            from_quarter_mhz.div_ceil(step),
            to_quarter_mhz.div_ceil(step),
        )
    };
    let mut commands = Vec::new();
    let mut current_quarter_mhz = from_quarter_mhz;
    while current_step != target_step {
        current_step = if rising {
            current_step + 1
        } else {
            current_step - 1
        };
        current_quarter_mhz = current_step * step;
        commands.push(Bm1370Command::SetFrequency(frequency_plan(
            current_quarter_mhz,
        )?));
        commands.push(Bm1370Command::DelayMs(FREQUENCY_RAMP_DELAY_MS));
    }
    if current_quarter_mhz != to_quarter_mhz {
        commands.push(Bm1370Command::SetFrequency(frequency_plan(to_quarter_mhz)?));
    }
    Ok(commands)
}

/// The PLL register plan for a frequency, searched within the BM1370's divider range.
pub fn frequency_plan(frequency_quarter_mhz: u32) -> Result<FrequencyPlan, Bm1370ProtocolFault> {
    frequency_plan_with_bounds(frequency_quarter_mhz, FB_DIVIDER_MIN, FB_DIVIDER_MAX).ok_or(
        Bm1370ProtocolFault::InvalidFrequency {
            frequency_quarter_mhz,
        },
    )
}

/// The hash-counting number (register 0x10) that sizes each core's nonce space, less the
/// BM1370's errata.
pub fn hash_counting_number(
    nonce_percent: f64,
    frequency_mhz: f32,
    asic_count: u16,
    core_count: u16,
) -> Result<u32, Bm1370ProtocolFault> {
    let admitted = (f64::MIN_POSITIVE..=1.0).contains(&nonce_percent)
        && frequency_mhz.is_finite()
        && frequency_mhz > 0.0;
    if !admitted {
        return Err(Bm1370ProtocolFault::InvalidNonceSpace);
    }
    let hcn_space = NONCE_SPACE
        / next_power_of_two(u32::from(core_count)) as f32
        / next_power_of_two(u32::from(asic_count)) as f32;
    let hcn_max = f64::from(hcn_space) * FREQ_MULT_MHZ / f64::from(frequency_mhz) * 0.5;
    let hcn = nonce_percent * (hcn_max - HASH_COUNTING_ERRATA);
    if !(0.0..=f64::from(u32::MAX)).contains(&hcn) {
        return Err(Bm1370ProtocolFault::InvalidNonceSpace);
    }
    Ok(hcn as u32)
}

/// How often a new job is sent: the chip's timeout over the chain length, rounded up to a
/// power of two.
#[must_use]
pub fn job_interval_ms(asic_count: u32) -> u32 {
    if asic_count == 0 {
        return DEFAULT_ASIC_TIMEOUT_MS;
    }
    DEFAULT_ASIC_TIMEOUT_MS / next_power_of_two(asic_count)
}

/// The hashrate the chain should reach at a frequency, in GH/s.
#[must_use]
pub fn expected_hashrate_ghs(frequency_mhz: f64, asic_count: u16) -> f64 {
    frequency_mhz * f64::from(SMALL_CORE_COUNT) * f64::from(asic_count) / 1000.0
}

fn write_all(writes: &[(u8, [u8; 4])]) -> impl Iterator<Item = Bm1370Command> + '_ {
    writes
        .iter()
        .map(|&(register, value)| Bm1370Command::WriteRegister {
            target: RegisterTarget::All,
            register,
            value,
        })
}

fn chip_address(chip_index: u8, address_interval: u16) -> Result<u8, Bm1370ProtocolFault> {
    u8::try_from(u16::from(chip_index) * address_interval).map_err(|_| {
        Bm1370ProtocolFault::InvalidChipCount {
            chip_count: chip_index.saturating_add(1),
        }
    })
}

const fn next_power_of_two(value: u32) -> u32 {
    if value <= 1 {
        1
    } else {
        value.next_power_of_two()
    }
}
