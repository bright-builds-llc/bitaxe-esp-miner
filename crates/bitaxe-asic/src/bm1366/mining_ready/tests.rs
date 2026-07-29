use super::super::command::Bm1366Command;
use super::super::upstream_init_frames::{
    CHAIN_INACTIVE_FRAME, DIFFICULTY_1000_FRAME, DIFFICULTY_256_FRAME, FREQUENCY_485_FRAME,
    INIT135_FRAME, INIT136_FRAME, INIT138_FRAME, INIT139_FRAME, INIT171_FRAME, INIT4_FRAME,
    INIT5_FRAME, INIT795_FRAME, NONCE_SPACE_485_FRAME, PER_CHIP_18_FRAME, PER_CHIP_3C_FIRST_FRAME,
    PER_CHIP_3C_SECOND_FRAME, PER_CHIP_3C_THIRD_FRAME, PER_CHIP_A8_FRAME, REG28_MAX_BAUD_FRAME,
};
use super::*;

const FIRST_RAMP_FREQUENCY_FRAME: [u8; 11] = [
    0x55, 0xAA, 0x51, 0x09, 0x00, 0x08, 0x40, 0xBD, 0x02, 0x65, 0x01,
];
const FREQUENCY_400_FRAME: [u8; 11] = [
    0x55, 0xAA, 0x51, 0x09, 0x00, 0x08, 0x40, 0xA0, 0x02, 0x40, 0x11,
];
const FREQUENCY_50_FRAME: [u8; 11] = [
    0x55, 0xAA, 0x51, 0x09, 0x00, 0x08, 0x40, 0xA8, 0x02, 0x65, 0x0F,
];
const NONCE_SPACE_50_FRAME: [u8; 11] = [
    0x55, 0xAA, 0x51, 0x09, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x07,
];

fn frame_bytes(command: Bm1366Command) -> Vec<u8> {
    command
        .frame_bytes()
        .expect("command should encode")
        .into_vec()
}

fn frequency_frames(commands: &[Bm1366Command]) -> Vec<Vec<u8>> {
    commands
        .iter()
        .filter_map(|command| match command {
            Bm1366Command::SetFrequency(_) => Some(frame_bytes(*command)),
            _ => None,
        })
        .collect()
}

#[test]
#[ignore = "local fixture generation helper"]
fn dump_dynamic_init_frames_for_fixture_capture() {
    let config = MiningReadyConfig::ultra_205_single_chip(1);
    let commands = mining_ready_commands(config, MiningReadyInitOptions::production_default())
        .expect("commands should build");
    let frames: Vec<Vec<u8>> = commands.iter().copied().map(frame_bytes).collect();
    for (index, frame) in frames.iter().enumerate() {
        eprintln!("frames[{index}] = {frame:?}");
    }
}

#[test]
fn mining_ready_dynamic_init_frames_match_upstream_computed_values() {
    let config = MiningReadyConfig::ultra_205_single_chip(1);
    assert_eq!(config.difficulty, 256.0);
    let commands = mining_ready_commands(config, MiningReadyInitOptions::production_default())
        .expect("commands should build");
    let frames: Vec<Vec<u8>> = commands.iter().copied().map(frame_bytes).collect();

    assert_eq!(frames[6], DIFFICULTY_256_FRAME);
    assert_eq!(frames[15], FREQUENCY_485_FRAME);
    assert_eq!(frames[16], NONCE_SPACE_485_FRAME);
}

#[test]
fn difficulty_mask_for_256_matches_upstream_asic_family_rule() {
    let mask = difficulty_mask_value(256.0);
    // mask = (1<<8)-1 = 255 = 0x000000FF, reversed per byte
    assert_eq!(mask[0], reverse_bits(0x00));
    assert_eq!(mask[1], reverse_bits(0x00));
    assert_eq!(mask[2], reverse_bits(0x00));
    assert_eq!(mask[3], reverse_bits(0xFF));
    assert_eq!(
        frame_bytes(Bm1366Command::SetDifficultyMask(mask)),
        DIFFICULTY_256_FRAME
    );
}

#[test]
fn difficulty_1000_frame_still_matches_pool_mask_math() {
    let mask = difficulty_mask_value(1000.0);
    assert_eq!(
        frame_bytes(Bm1366Command::SetDifficultyMask(mask)),
        DIFFICULTY_1000_FRAME
    );
}

#[test]
fn mining_ready_init_frames_match_upstream_fixtures() {
    let config = MiningReadyConfig::ultra_205_single_chip(1);
    let commands = mining_ready_commands(config, MiningReadyInitOptions::production_default())
        .expect("commands should build");

    let frames: Vec<Vec<u8>> = commands.iter().copied().map(frame_bytes).collect();

    assert_eq!(frames[0], INIT4_FRAME);
    assert_eq!(frames[1], INIT5_FRAME);
    assert_eq!(frames[2], CHAIN_INACTIVE_FRAME);
    // frames[3] = set chip address 0
    assert_eq!(frames[4], INIT135_FRAME);
    assert_eq!(frames[5], INIT136_FRAME);
    // frames[6] = difficulty mask (dynamic) — asserted in dynamic_init_frames test
    assert_eq!(frames[7], INIT138_FRAME);
    assert_eq!(frames[8], INIT139_FRAME);
    assert_eq!(frames[9], INIT171_FRAME);
    assert_eq!(frames[10], PER_CHIP_A8_FRAME);
    assert_eq!(frames[11], PER_CHIP_18_FRAME);
    assert_eq!(frames[12], PER_CHIP_3C_FIRST_FRAME);
    assert_eq!(frames[13], PER_CHIP_3C_SECOND_FRAME);
    assert_eq!(frames[14], PER_CHIP_3C_THIRD_FRAME);
    // frames[15] = frequency (PLL-derived) — asserted in dynamic_init_frames test
    // frames[16] = nonce space (computed) — asserted in dynamic_init_frames test
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
    let actions = max_baud_prelude_actions(MiningReadyInitOptions::production_default())
        .expect("prelude should encode");

    assert!(matches!(
        actions.first(),
        Some(Bm1366AdapterAction::WriteFrame(_))
    ));
    assert!(actions.contains(&Bm1366AdapterAction::WAIT_TX_DONE));
    assert!(actions
        .iter()
        .any(|action| matches!(action, Bm1366AdapterAction::UseMaxBaud { baud: 1_000_000 })));
    assert!(actions.contains(&Bm1366AdapterAction::ClearRx));
}

#[test]
fn max_baud_prelude_can_insert_post_host_delay() {
    let actions = max_baud_prelude_actions(MiningReadyInitOptions {
        post_max_baud_delay_ms: 2_000,
        ..MiningReadyInitOptions::production_default()
    })
    .expect("prelude should encode");

    assert!(actions.contains(&Bm1366AdapterAction::ClearRx));
    assert!(actions
        .iter()
        .any(|command| matches!(command, Bm1366AdapterAction::DelayMs(2_000))));
}

#[test]
fn conservative_profile_ramp_matches_golden_frequency_order() {
    // Arrange
    let profile = Bm1366MiningProfile::Conservative;

    // Act
    let commands = mining_ready_frequency_ramp_commands(profile).expect("ramp should build");
    let frames = frequency_frames(&commands);

    // Assert
    assert_eq!(commands.len(), 112);
    assert_eq!(frames.len(), 56);
    assert_eq!(frames.first(), Some(&FIRST_RAMP_FREQUENCY_FRAME.to_vec()));
    assert_eq!(frames.last(), Some(&FREQUENCY_400_FRAME.to_vec()));
    assert!(commands
        .chunks_exact(2)
        .all(|pair| matches!(pair[1], Bm1366Command::DelayMs(100))));
}

#[test]
fn upstream_default_profile_ramp_matches_golden_frequency_order() {
    // Arrange
    let profile = Bm1366MiningProfile::UpstreamDefault;

    // Act
    let commands = mining_ready_frequency_ramp_commands(profile).expect("ramp should build");
    let frames = frequency_frames(&commands);

    // Assert
    assert_eq!(commands.len(), 139);
    assert_eq!(frames.len(), 70);
    assert_eq!(frames.first(), Some(&FIRST_RAMP_FREQUENCY_FRAME.to_vec()));
    assert_eq!(frames.last(), Some(&FREQUENCY_485_FRAME.to_vec()));
    assert!(commands[..commands.len() - 1]
        .chunks_exact(2)
        .all(|pair| matches!(pair[1], Bm1366Command::DelayMs(100))));
}

#[test]
fn production_ramp_option_is_explicitly_enabled() {
    // Arrange
    let direct = MiningReadyInitOptions::production_default();

    // Act
    let ramped = MiningReadyInitOptions::production_with_frequency_ramp();

    // Assert
    assert!(!direct.use_frequency_ramp);
    assert!(ramped.use_frequency_ramp);
}

#[test]
fn mining_ready_profiles_select_validated_400_and_485_mhz_plans() {
    // Arrange
    let conservative = MiningReadyConfig::ultra_205_profile(1, Bm1366MiningProfile::Conservative);
    let upstream_default =
        MiningReadyConfig::ultra_205_profile(1, Bm1366MiningProfile::UpstreamDefault);

    // Act
    let conservative_plan = conservative.profile().frequency_plan();
    let upstream_default_plan = upstream_default.profile().frequency_plan();

    // Assert
    assert_eq!(conservative.frequency_mhz(), 400);
    assert_eq!(upstream_default.frequency_mhz(), 485);
    assert!(conservative_plan.is_ok());
    assert!(upstream_default_plan.is_ok());
}

#[test]
fn safe_shutdown_orders_50_mhz_nonce_reset_wait_and_reset_hold() {
    // Arrange
    let profiles = [
        Bm1366MiningProfile::Conservative,
        Bm1366MiningProfile::UpstreamDefault,
    ];

    for profile in profiles {
        let config = MiningReadyConfig::ultra_205_profile(1, profile);

        // Act
        let commands = safe_shutdown_commands(config).expect("shutdown commands should build");
        let command_actions =
            safe_shutdown_command_actions(config).expect("shutdown command actions should encode");
        let full_actions = safe_shutdown_actions(config).expect("shutdown actions should encode");
        let frames = frequency_frames(&commands);

        // Assert
        assert_eq!(frames.last(), Some(&FREQUENCY_50_FRAME.to_vec()));
        assert_eq!(
            commands.last().copied().map(frame_bytes),
            Some(NONCE_SPACE_50_FRAME.to_vec())
        );
        assert!(matches!(
            commands.get(commands.len() - 2),
            Some(Bm1366Command::DelayMs(100))
        ));
        assert_eq!(
            command_actions.last(),
            Some(&Bm1366AdapterAction::WAIT_TX_DONE)
        );
        assert_eq!(
            full_actions.last(),
            Some(&Bm1366AdapterAction::HOLD_RESET_LOW)
        );
        assert_eq!(
            &full_actions[..full_actions.len() - 1],
            command_actions.as_slice()
        );
    }
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
fn hash_counting_number_uses_actual_pll_frequency_for_nonce_space_frame() {
    let config = MiningReadyConfig::ultra_205_single_chip(1);
    let commands = mining_ready_commands(config, MiningReadyInitOptions::production_default())
        .expect("commands should build");
    let plan = config
        .profile()
        .frequency_plan()
        .expect("profile frequency should validate")
        .command_plan();
    let expected_hcn = hash_counting_number(
        config.nonce_percent,
        actual_frequency_mhz(plan),
        config.asic_count,
        config.core_count,
    );
    let nonce_space = commands
        .iter()
        .find_map(|command| match command {
            Bm1366Command::SetNonceSpace(plan) => Some(plan.hash_counting_number),
            _ => None,
        })
        .expect("nonce space command should exist");
    assert_eq!(nonce_space, expected_hcn);
    assert_eq!(expected_hcn, 0x000d_3224);
}

#[test]
fn hash_counting_number_uses_next_power_of_two_cores() {
    let hcn = hash_counting_number(1.0, 485.0, 1, 112);
    assert!(hcn > 0);
    assert!(hcn < u32::MAX);
}

#[test]
fn bm1366_job_interval_single_chip_is_2000_ms() {
    // Arrange
    let asic_count = 1_u32;

    // Act
    let interval_ms = bm1366_job_interval_ms(asic_count);

    // Assert
    assert_eq!(interval_ms, 2000);
}

#[test]
fn bm1366_job_interval_two_chips_is_1000_ms() {
    // Arrange
    let asic_count = 2_u32;

    // Act
    let interval_ms = bm1366_job_interval_ms(asic_count);

    // Assert
    assert_eq!(interval_ms, 1000);
}

#[test]
fn bm1366_job_interval_five_chips_rounds_up_to_eight_divisor() {
    // Arrange
    let asic_count = 5_u32;

    // Act
    let interval_ms = bm1366_job_interval_ms(asic_count);

    // Assert
    assert_eq!(interval_ms, 250);
}

#[test]
fn bm1366_job_interval_zero_chips_guards_as_single_chip() {
    // Arrange
    let asic_count = 0_u32;

    // Act
    let interval_ms = bm1366_job_interval_ms(asic_count);

    // Assert
    assert_eq!(interval_ms, 2000);
}
