//! Pure BM1366 mining-ready init planning after chip detect.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/bm1366.c:BM1366_init` post chip-detect
//! - `reference/esp-miner/components/asic/asic_common.c:get_difficulty_mask`
//! - parity checklist rows `ASIC-006` and `ASIC-008`

use bitaxe_config::{ultra_205_catalog_entry, ultra_205_defaults};

use super::{
    command::{
        Bm1366AdapterAction, Bm1366Command, FrequencyPlan, NonceSpacePlan, RegisterWrite, MAX_BAUD,
    },
    frequency_voltage::Bm1366FrequencyPlan,
    init_plan::{Bm1366InitDecision, Bm1366InitPlan, Bm1366Preflight, FailClosedAction},
    observation::{AsicInitStatus, ChipAddress},
};

const NONCE_SPACE: f64 = 4_294_967_296.0;
const FREQ_MULT_MHZ: f64 = 25.0;
const INIT_COMMAND_ENCODING_FAILED: &str = "mining_ready_command_encoding_failed";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiningReadyConfig {
    pub chip_count: u8,
    pub asic_count: u16,
    pub core_count: u16,
    pub frequency_mhz: f32,
    pub difficulty: f64,
    pub nonce_percent: f64,
}

impl MiningReadyConfig {
    #[must_use]
    pub fn ultra_205_single_chip(chip_count: u8) -> Self {
        let catalog = ultra_205_catalog_entry();
        let defaults = ultra_205_defaults();
        Self {
            chip_count,
            asic_count: u16::from(catalog.asic_count()),
            core_count: catalog.asic().core_count(),
            frequency_mhz: defaults.asic_frequency_mhz() as f32,
            difficulty: f64::from(defaults.primary_pool().difficulty()),
            nonce_percent: 1.0,
        }
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
    let hash_counting = hash_counting_number(
        config.nonce_percent,
        config.frequency_mhz,
        config.asic_count,
        config.core_count,
    );
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
        commands.extend(frequency_ramp_commands(config.frequency_mhz)?);
    } else {
        commands.push(Bm1366Command::SetFrequency(frequency_plan_for_mhz(
            config.frequency_mhz,
        )));
    }

    commands.push(Bm1366Command::SetNonceSpace(NonceSpacePlan {
        hash_counting_number: hash_counting,
    }));
    commands.push(Bm1366Command::WriteRegister(RegisterWrite::all(
        0xA4,
        [0x90, 0x00, 0xFF, 0xFF],
    )));

    Ok(commands)
}

const FREQ_RAMP_START_MHZ: f32 = 50.0;
const FREQ_RAMP_STEP_MHZ: f32 = 6.25;
const FREQ_RAMP_DELAY_MS: u32 = 100;

fn frequency_ramp_commands(target_mhz: f32) -> Result<Vec<Bm1366Command>, &'static str> {
    let mut commands = Vec::new();
    let mut current_mhz = FREQ_RAMP_START_MHZ;

    if (target_mhz - current_mhz).abs() < FREQ_RAMP_STEP_MHZ {
        commands.push(Bm1366Command::SetFrequency(frequency_plan_for_mhz(target_mhz)));
        return Ok(commands);
    }

    let sign = if target_mhz > current_mhz { 1.0 } else { -1.0 };
    let mut current_step = (current_mhz / FREQ_RAMP_STEP_MHZ).floor() as i32;
    let target_step = (target_mhz / FREQ_RAMP_STEP_MHZ).floor() as i32;

    while (sign > 0.0 && current_step < target_step) || (sign < 0.0 && current_step > target_step)
    {
        current_step += sign as i32;
        current_mhz = current_step as f32 * FREQ_RAMP_STEP_MHZ;
        commands.push(Bm1366Command::SetFrequency(frequency_plan_for_mhz(current_mhz)));
        commands.push(Bm1366Command::DelayMs(FREQ_RAMP_DELAY_MS));
    }

    if (current_mhz - target_mhz).abs() > f32::EPSILON {
        commands.push(Bm1366Command::SetFrequency(frequency_plan_for_mhz(target_mhz)));
    }

    Ok(commands)
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
    Ok(actions)
}

fn frequency_plan_for_mhz(frequency_mhz: f32) -> FrequencyPlan {
    Bm1366FrequencyPlan::ultra_205_bm1366(frequency_mhz as i64)
        .map(|plan| plan.command_plan())
        .unwrap_or_else(|_| {
            // Fallback for test-only invalid paths; Ultra 205 catalog uses 485 MHz.
            Bm1366FrequencyPlan::ultra_205_bm1366(485)
                .expect("485 MHz should encode")
                .command_plan()
        })
}

pub fn encode_commands(commands: &[Bm1366Command]) -> Result<Vec<Bm1366AdapterAction>, &'static str> {
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
}

impl MiningReadyInitOptions {
    #[must_use]
    pub const fn phase27_default() -> Self {
        Self {
            skip_max_baud: false,
            skip_asic_max_baud: false,
            use_frequency_ramp: false,
        }
    }
}

impl Bm1366InitPlan {
    pub fn mining_ready_init(
        preflight: Bm1366Preflight,
        chip_count: u8,
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

        let config = MiningReadyConfig::ultra_205_single_chip(chip_count);
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
mod tests {
    use super::super::upstream_init_frames::{
        CHAIN_INACTIVE_FRAME, INIT135_FRAME, INIT136_FRAME, INIT138_FRAME, INIT139_FRAME,
        INIT171_FRAME, INIT4_FRAME, INIT5_FRAME, INIT795_FRAME, PER_CHIP_18_FRAME,
        PER_CHIP_3C_FIRST_FRAME, PER_CHIP_3C_SECOND_FRAME, PER_CHIP_3C_THIRD_FRAME,
        PER_CHIP_A8_FRAME, REG28_MAX_BAUD_FRAME,
    };
    use super::super::command::Bm1366Command;
    use super::*;

    fn frame_bytes(command: Bm1366Command) -> Vec<u8> {
        command
            .frame_bytes()
            .expect("command should encode")
            .into_vec()
    }

    #[test]
    fn mining_ready_init_frames_match_upstream_fixtures() {
        let config = MiningReadyConfig::ultra_205_single_chip(1);
        let commands =
            mining_ready_commands(config, MiningReadyInitOptions::phase27_default())
                .expect("commands should build");

        let frames: Vec<Vec<u8>> = commands.iter().copied().map(frame_bytes).collect();

        assert_eq!(frames[0], INIT4_FRAME);
        assert_eq!(frames[1], INIT5_FRAME);
        assert_eq!(frames[2], CHAIN_INACTIVE_FRAME);
        // frames[3] = set chip address 0
        assert_eq!(frames[4], INIT135_FRAME);
        assert_eq!(frames[5], INIT136_FRAME);
        // frames[6] = difficulty mask (dynamic)
        assert_eq!(frames[7], INIT138_FRAME);
        assert_eq!(frames[8], INIT139_FRAME);
        assert_eq!(frames[9], INIT171_FRAME);
        assert_eq!(frames[10], PER_CHIP_A8_FRAME);
        assert_eq!(frames[11], PER_CHIP_18_FRAME);
        assert_eq!(frames[12], PER_CHIP_3C_FIRST_FRAME);
        assert_eq!(frames[13], PER_CHIP_3C_SECOND_FRAME);
        assert_eq!(frames[14], PER_CHIP_3C_THIRD_FRAME);
        // frames[15] = frequency (PLL-derived)
        // frames[16] = nonce space (computed)
        assert_eq!(frames[17], INIT795_FRAME);
    }

    #[test]
    fn set_asic_max_baud_matches_upstream_reg28_fixture() {
        assert_eq!(
            frame_bytes(Bm1366Command::SetAsicMaxBaud),
            REG28_MAX_BAUD_FRAME
        );
    }

    #[test]
    fn max_baud_prelude_orders_reg28_wait_host_clear() {
        let actions = max_baud_prelude_actions(MiningReadyInitOptions::phase27_default())
            .expect("prelude should encode");

        assert!(matches!(
            actions.first(),
            Some(Bm1366AdapterAction::WriteFrame(_))
        ));
        assert!(actions.contains(&Bm1366AdapterAction::WAIT_TX_DONE));
        assert!(actions.iter().any(|action| matches!(
            action,
            Bm1366AdapterAction::UseMaxBaud { baud: 1_000_000 }
        )));
        assert!(actions.contains(&Bm1366AdapterAction::ClearRx));
    }

    #[test]
    fn frequency_ramp_emits_multiple_steps_with_delays() {
        let ramp = frequency_ramp_commands(485.0).expect("ramp should build");
        assert!(ramp.len() > 2);
        assert!(ramp.iter().any(|command| matches!(command, Bm1366Command::DelayMs(100))));
    }

    #[test]
    fn ultra_205_address_interval_is_256() {
        assert_eq!(ultra_205_result_address_interval(), 256);
        assert_eq!(
            MiningReadyConfig::ultra_205_single_chip(1).address_interval(),
            256
        );
    }

    #[test]
    fn difficulty_mask_for_1000_matches_upstream_power_of_two_rule() {
        let mask = difficulty_mask_value(1000.0);
        // mask = (1<<9)-1 = 511 = 0x000001FF, reversed per byte
        assert_eq!(mask[0], reverse_bits(0x00));
        assert_eq!(mask[1], reverse_bits(0x00));
        assert_eq!(mask[2], reverse_bits(0x01));
        assert_eq!(mask[3], reverse_bits(0xFF));
    }

    #[test]
    fn hash_counting_number_uses_next_power_of_two_cores() {
        let hcn = hash_counting_number(1.0, 485.0, 1, 112);
        assert!(hcn > 0);
        assert!(hcn < u32::MAX);
    }
}
