//! Pure BM1366 mining-ready init planning after chip detect.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/bm1366.c:BM1366_init` post chip-detect
//! - `reference/esp-miner/components/asic/asic_common.c:get_difficulty_mask`
//! - parity checklist rows `ASIC-006` and `ASIC-008`

use bitaxe_config::{ultra_205_catalog_entry, ConfigValidationError};

use super::{
    command::{
        Bm1366AdapterAction, Bm1366Command, FrequencyPlan, NonceSpacePlan, RegisterWrite, MAX_BAUD,
    },
    frequency_voltage::{actual_frequency_mhz, transition_frequency_plan, Bm1366FrequencyPlan},
    init_plan::{Bm1366InitDecision, Bm1366InitPlan, Bm1366Preflight, FailClosedAction},
    observation::{AsicInitStatus, ChipAddress},
};

const NONCE_SPACE: f64 = 4_294_967_296.0;
const FREQ_MULT_MHZ: f64 = 25.0;
const INIT_COMMAND_ENCODING_FAILED: &str = "mining_ready_command_encoding_failed";
const MINING_READY_FREQUENCY_INVALID: &str = "mining_ready_frequency_invalid";
const FREQUENCY_TRANSITION_INVALID: &str = "frequency_transition_invalid";

/// Closed Ultra 205 BM1366 production frequencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1366MiningProfile {
    /// Lower-power first mining profile at 400 MHz.
    Conservative,
    /// Upstream Ultra BM1366 default at 485 MHz.
    UpstreamDefault,
}

impl Bm1366MiningProfile {
    /// Returns the profile target frequency.
    #[must_use]
    pub const fn frequency_mhz(self) -> u16 {
        match self {
            Self::Conservative => 400,
            Self::UpstreamDefault => 485,
        }
    }

    /// Revalidates the profile target against the Ultra 205 BM1366 catalog.
    pub fn frequency_plan(self) -> Result<Bm1366FrequencyPlan, ConfigValidationError> {
        Bm1366FrequencyPlan::ultra_205_bm1366(i64::from(self.frequency_mhz()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiningReadyConfig {
    pub chip_count: u8,
    pub asic_count: u16,
    pub core_count: u16,
    pub difficulty: f64,
    pub nonce_percent: f64,
    profile: Bm1366MiningProfile,
}

impl MiningReadyConfig {
    #[must_use]
    pub fn ultra_205_single_chip(chip_count: u8) -> Self {
        Self::ultra_205_profile(chip_count, Bm1366MiningProfile::UpstreamDefault)
    }

    /// Builds a mining-ready config for one of the closed production profiles.
    #[must_use]
    pub fn ultra_205_profile(chip_count: u8, profile: Bm1366MiningProfile) -> Self {
        let catalog = ultra_205_catalog_entry();
        Self {
            chip_count,
            asic_count: u16::from(catalog.asic_count()),
            core_count: catalog.asic().core_count(),
            // Ticket mask follows ASIC family difficulty (upstream BM1366_init /
            // device_config.h ASIC_BM1366.difficulty=256). Pool stratumdiff remains
            // a Stratum/share concern and must not drive reg 0x14.
            difficulty: f64::from(catalog.asic().difficulty()),
            nonce_percent: 1.0,
            profile,
        }
    }

    /// Returns the selected production profile.
    #[must_use]
    pub const fn profile(self) -> Bm1366MiningProfile {
        self.profile
    }

    /// Returns the selected profile frequency.
    #[must_use]
    pub const fn frequency_mhz(self) -> u16 {
        self.profile.frequency_mhz()
    }

    #[must_use]
    pub fn address_interval(self) -> u16 {
        256_u16 / u16::from(self.chip_count)
    }
}

#[must_use]
pub fn next_power_of_two(value: u32) -> u32 {
    if value <= 1 {
        return 1;
    }
    let mut power = 1_u32;
    while power < value {
        power <<= 1;
    }
    power
}

/// BM1366 per-job dispatch interval in milliseconds for a given chain length.
///
/// Reference: reference/esp-miner/components/asic/asic.c:149-158
/// (ASIC_get_asic_job_frequency_ms) and
/// reference/esp-miner/main/device_config.h:93 (BM1366 default_asic_timeout = 2000).
#[must_use]
pub fn bm1366_job_interval_ms(asic_count: u32) -> u32 {
    const BM1366_DEFAULT_ASIC_TIMEOUT_MS: u32 = 2000;
    if asic_count == 0 {
        return BM1366_DEFAULT_ASIC_TIMEOUT_MS;
    }
    BM1366_DEFAULT_ASIC_TIMEOUT_MS / next_power_of_two(asic_count)
}

#[must_use]
pub fn hash_counting_number(
    nonce_percent: f64,
    frequency_mhz: f32,
    asic_count: u16,
    core_count: u16,
) -> u32 {
    let cores_up = next_power_of_two(u32::from(core_count));
    let asic_count_up = next_power_of_two(u32::from(asic_count));
    let hcn_space = NONCE_SPACE / f64::from(cores_up) / f64::from(asic_count_up);
    let hcn_max = hcn_space * FREQ_MULT_MHZ / f64::from(frequency_mhz) * 0.5;
    let hcn_frac = nonce_percent * hcn_max;
    hcn_frac as u32
}

fn reverse_bits(byte: u8) -> u8 {
    let mut reversed = 0_u8;
    let mut num = byte;
    for _ in 0..8 {
        reversed <<= 1;
        reversed |= num & 1;
        num >>= 1;
    }
    reversed
}

#[must_use]
pub fn difficulty_mask_value(difficulty: f64) -> [u8; 4] {
    let diff_int = difficulty.ceil() as u32;
    let mut power = 0_u32;
    let mut value = diff_int;
    while value > 1 {
        value >>= 1;
        power += 1;
    }
    let mask = (1_u32 << power) - 1;
    [
        reverse_bits(((mask >> 24) & 0xff) as u8),
        reverse_bits(((mask >> 16) & 0xff) as u8),
        reverse_bits(((mask >> 8) & 0xff) as u8),
        reverse_bits((mask & 0xff) as u8),
    ]
}

pub fn mining_ready_commands(
    config: MiningReadyConfig,
    options: MiningReadyInitOptions,
) -> Result<Vec<Bm1366Command>, &'static str> {
    let final_frequency_plan = config
        .profile()
        .frequency_plan()
        .map_err(|_| MINING_READY_FREQUENCY_INVALID)?
        .command_plan();
    let difficulty_mask = difficulty_mask_value(config.difficulty);
    let address_interval = config.address_interval();

    let mut commands = vec![
        Bm1366Command::WriteRegister(RegisterWrite::all(0xA8, [0x00, 0x07, 0x00, 0x00])),
        Bm1366Command::WriteRegister(RegisterWrite::all(0x18, [0xFF, 0x0F, 0xC1, 0x00])),
        Bm1366Command::SetChainInactive,
    ];

    for chip_index in 0..config.chip_count {
        let address = u8::try_from(chip_index as u16 * address_interval)
            .map_err(|_| "mining_ready_chip_address_overflow")?;
        commands.push(Bm1366Command::SetChipAddress(ChipAddress::new(address)));
    }

    commands.extend([
        Bm1366Command::WriteRegister(RegisterWrite::all(0x3C, [0x80, 0x00, 0x85, 0x40])),
        Bm1366Command::WriteRegister(RegisterWrite::all(0x3C, [0x80, 0x00, 0x80, 0x20])),
        Bm1366Command::SetDifficultyMask(difficulty_mask),
        Bm1366Command::WriteRegister(RegisterWrite::all(0x54, [0x00, 0x00, 0x00, 0x03])),
        Bm1366Command::WriteRegister(RegisterWrite::all(0x58, [0x02, 0x11, 0x11, 0x11])),
        Bm1366Command::WriteRegister(RegisterWrite::single(
            ChipAddress::new(0),
            0x2C,
            [0x00, 0x7C, 0x00, 0x03],
        )),
    ]);

    for chip_index in 0..config.chip_count {
        let address = ChipAddress::new(
            u8::try_from(chip_index as u16 * address_interval)
                .map_err(|_| "mining_ready_chip_address_overflow")?,
        );
        commands.extend([
            Bm1366Command::WriteRegister(RegisterWrite::single(
                address,
                0xA8,
                [0x00, 0x07, 0x01, 0xF0],
            )),
            Bm1366Command::WriteRegister(RegisterWrite::single(
                address,
                0x18,
                [0xF0, 0x00, 0xC1, 0x00],
            )),
            Bm1366Command::WriteRegister(RegisterWrite::single(
                address,
                0x3C,
                [0x80, 0x00, 0x85, 0x40],
            )),
            Bm1366Command::WriteRegister(RegisterWrite::single(
                address,
                0x3C,
                [0x80, 0x00, 0x80, 0x20],
            )),
            Bm1366Command::WriteRegister(RegisterWrite::single(
                address,
                0x3C,
                [0x80, 0x00, 0x82, 0xAA],
            )),
        ]);
    }

    if options.use_frequency_ramp {
        commands.extend(mining_ready_frequency_ramp_commands(config.profile())?);
    } else {
        commands.push(Bm1366Command::SetFrequency(final_frequency_plan));
    }

    let hash_counting = hash_counting_number(
        config.nonce_percent,
        actual_frequency_mhz(final_frequency_plan),
        config.asic_count,
        config.core_count,
    );
    commands.push(Bm1366Command::SetNonceSpace(NonceSpacePlan {
        hash_counting_number: hash_counting,
    }));
    commands.push(Bm1366Command::WriteRegister(RegisterWrite::all(
        0xA4,
        [0x90, 0x00, 0xFF, 0xFF],
    )));

    Ok(commands)
}

const QUARTERS_PER_MHZ: u32 = 4;
const FREQ_RAMP_START_QUARTER_MHZ: u32 = 50 * QUARTERS_PER_MHZ;
const FREQ_RAMP_STEP_QUARTER_MHZ: u32 = 25;
const FREQ_RAMP_DELAY_MS: u32 = 100;

/// Plans the upstream-aligned 50-MHz-to-profile frequency ramp.
pub fn mining_ready_frequency_ramp_commands(
    profile: Bm1366MiningProfile,
) -> Result<Vec<Bm1366Command>, &'static str> {
    frequency_transition_commands(
        FREQ_RAMP_START_QUARTER_MHZ,
        u32::from(profile.frequency_mhz()) * QUARTERS_PER_MHZ,
    )
}

fn frequency_transition_commands(
    start_quarter_mhz: u32,
    target_quarter_mhz: u32,
) -> Result<Vec<Bm1366Command>, &'static str> {
    let mut commands = Vec::new();

    if start_quarter_mhz == target_quarter_mhz {
        return Ok(commands);
    }

    let frequency_distance = start_quarter_mhz.abs_diff(target_quarter_mhz);
    if frequency_distance < FREQ_RAMP_STEP_QUARTER_MHZ {
        commands.push(Bm1366Command::SetFrequency(transition_plan(
            target_quarter_mhz,
        )?));
        return Ok(commands);
    }

    let increasing = target_quarter_mhz > start_quarter_mhz;
    let mut current_step = transition_step(start_quarter_mhz, increasing);
    let target_step = transition_step(target_quarter_mhz, increasing);

    while (increasing && current_step < target_step) || (!increasing && current_step > target_step)
    {
        if increasing {
            current_step += 1;
        } else {
            current_step -= 1;
        }

        let current_quarter_mhz = current_step * FREQ_RAMP_STEP_QUARTER_MHZ;
        commands.push(Bm1366Command::SetFrequency(transition_plan(
            current_quarter_mhz,
        )?));
        commands.push(Bm1366Command::DelayMs(FREQ_RAMP_DELAY_MS));
    }

    let current_quarter_mhz = current_step * FREQ_RAMP_STEP_QUARTER_MHZ;
    if current_quarter_mhz != target_quarter_mhz {
        commands.push(Bm1366Command::SetFrequency(transition_plan(
            target_quarter_mhz,
        )?));
    }

    Ok(commands)
}

const fn transition_step(frequency_quarter_mhz: u32, increasing: bool) -> u32 {
    if increasing {
        frequency_quarter_mhz / FREQ_RAMP_STEP_QUARTER_MHZ
    } else {
        frequency_quarter_mhz.div_ceil(FREQ_RAMP_STEP_QUARTER_MHZ)
    }
}

fn transition_plan(frequency_quarter_mhz: u32) -> Result<FrequencyPlan, &'static str> {
    transition_frequency_plan(frequency_quarter_mhz).ok_or(FREQUENCY_TRANSITION_INVALID)
}

/// Plans the upstream-aligned profile-to-50-MHz ASIC shutdown commands.
///
/// The final nonce-space value is calculated from the actual 50 MHz PLL
/// output. GPIO reset remains an adapter action and is not included here.
pub fn safe_shutdown_commands(
    config: MiningReadyConfig,
) -> Result<Vec<Bm1366Command>, &'static str> {
    let profile_frequency_quarter_mhz =
        u32::from(config.profile().frequency_mhz()) * QUARTERS_PER_MHZ;
    let shutdown_frequency_plan = transition_plan(FREQ_RAMP_START_QUARTER_MHZ)?;
    let mut commands =
        frequency_transition_commands(profile_frequency_quarter_mhz, FREQ_RAMP_START_QUARTER_MHZ)?;

    let hash_counting = hash_counting_number(
        config.nonce_percent,
        actual_frequency_mhz(shutdown_frequency_plan),
        config.asic_count,
        config.core_count,
    );
    commands.push(Bm1366Command::SetNonceSpace(NonceSpacePlan {
        hash_counting_number: hash_counting,
    }));
    Ok(commands)
}

/// Encodes the ASIC shutdown commands and waits for UART transmission.
///
/// This action slice deliberately omits reset GPIO control so coordinators can
/// execute frequency/nonce and reset-low as distinct shutdown steps.
pub fn safe_shutdown_command_actions(
    config: MiningReadyConfig,
) -> Result<Vec<Bm1366AdapterAction>, &'static str> {
    let mut actions = encode_commands(&safe_shutdown_commands(config)?)?;
    actions.push(Bm1366AdapterAction::WAIT_TX_DONE);
    Ok(actions)
}

/// Returns the complete ordered ASIC shutdown plan ending in reset-low.
pub fn safe_shutdown_actions(
    config: MiningReadyConfig,
) -> Result<Vec<Bm1366AdapterAction>, &'static str> {
    let mut actions = safe_shutdown_command_actions(config)?;
    actions.push(Bm1366AdapterAction::HOLD_RESET_LOW);
    Ok(actions)
}

pub fn max_baud_prelude_actions(
    options: MiningReadyInitOptions,
) -> Result<Vec<Bm1366AdapterAction>, &'static str> {
    if options.skip_max_baud {
        return Ok(Vec::new());
    }

    let mut actions = Vec::new();
    if !options.skip_asic_max_baud {
        actions.extend(encode_commands(&[Bm1366Command::SetAsicMaxBaud])?);
        actions.push(Bm1366AdapterAction::WAIT_TX_DONE);
    }
    actions.push(Bm1366AdapterAction::UseMaxBaud { baud: MAX_BAUD });
    actions.push(Bm1366AdapterAction::ClearRx);
    if options.post_max_baud_delay_ms > 0 {
        actions.extend(encode_commands(&[Bm1366Command::DelayMs(
            options.post_max_baud_delay_ms,
        )])?);
    }
    Ok(actions)
}

pub fn encode_commands(
    commands: &[Bm1366Command],
) -> Result<Vec<Bm1366AdapterAction>, &'static str> {
    let mut actions = Vec::with_capacity(commands.len());
    for command in commands {
        let encoded = command
            .adapter_actions()
            .map_err(|_| INIT_COMMAND_ENCODING_FAILED)?;
        actions.extend(encoded);
    }
    Ok(actions)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningReadyInitOptions {
    pub skip_max_baud: bool,
    pub skip_asic_max_baud: bool,
    pub use_frequency_ramp: bool,
    pub post_max_baud_delay_ms: u32,
}

impl MiningReadyInitOptions {
    #[must_use]
    pub const fn production_default() -> Self {
        Self {
            skip_max_baud: false,
            skip_asic_max_baud: false,
            use_frequency_ramp: false,
            post_max_baud_delay_ms: 0,
        }
    }

    /// Enables the upstream-aligned 50-MHz-to-profile production ramp.
    #[must_use]
    pub const fn production_with_frequency_ramp() -> Self {
        Self {
            use_frequency_ramp: true,
            ..Self::production_default()
        }
    }
}

impl Bm1366InitPlan {
    pub fn mining_ready_init(
        preflight: Bm1366Preflight,
        chip_count: u8,
        options: MiningReadyInitOptions,
    ) -> Bm1366InitDecision {
        Self::mining_ready_init_for_profile(
            preflight,
            chip_count,
            Bm1366MiningProfile::UpstreamDefault,
            options,
        )
    }

    /// Plans mining-ready initialization for the selected production profile.
    pub fn mining_ready_init_for_profile(
        preflight: Bm1366Preflight,
        chip_count: u8,
        profile: Bm1366MiningProfile,
        options: MiningReadyInitOptions,
    ) -> Bm1366InitDecision {
        if let Err(reason) = preflight.validate_board_and_config() {
            return Bm1366InitDecision::preflight_missing(reason, FailClosedAction::HoldResetLow);
        }

        if chip_count == 0 {
            return Bm1366InitDecision::fail_closed(
                "mining_ready_zero_chips",
                FailClosedAction::HoldResetLow,
            );
        }

        let config = MiningReadyConfig::ultra_205_profile(chip_count, profile);
        let commands = match mining_ready_commands(config, options) {
            Ok(commands) => commands,
            Err(reason) => {
                return Bm1366InitDecision::fail_closed(reason, FailClosedAction::HoldResetLow);
            }
        };

        let mut actions = match encode_commands(&commands) {
            Ok(actions) => actions,
            Err(reason) => {
                return Bm1366InitDecision::fail_closed(reason, FailClosedAction::HoldResetLow);
            }
        };

        match max_baud_prelude_actions(options) {
            Ok(prelude) => actions.extend(prelude),
            Err(reason) => {
                return Bm1366InitDecision::fail_closed(reason, FailClosedAction::HoldResetLow);
            }
        }

        actions.push(Bm1366AdapterAction::PublishStatus(
            AsicInitStatus::InitializedNoMining,
        ));

        Bm1366InitDecision::mining_ready_success(actions)
    }
}

#[must_use]
pub fn ultra_205_result_address_interval() -> u16 {
    let catalog = ultra_205_catalog_entry();
    256_u16 / u16::from(catalog.asic_count())
}

#[cfg(test)]
mod tests;
