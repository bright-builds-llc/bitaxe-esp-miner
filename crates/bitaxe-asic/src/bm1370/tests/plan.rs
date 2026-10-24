//! The BM1370's bring-up actions against the golden fixture.

use super::{decode_hex, fixture, hex_at, u64_at};
use crate::{
    asic::{AsicFamily, ChipIdCountFrameDisposition},
    bm1366::{command::Bm1366AdapterAction, observation::AsicInitStatus},
    bm1370::{
        init::Bm1370InitConfig,
        plan::{chip_detect_actions, classify_chip_id_count_frame, mining_ready_actions},
        Bm1370ProtocolFault,
    },
};

fn written(actions: &[Bm1366AdapterAction]) -> Vec<Vec<u8>> {
    actions
        .iter()
        .filter_map(|action| match action {
            Bm1366AdapterAction::WriteFrame(frame) => Some(frame.as_ref().to_vec()),
            _ => None,
        })
        .collect()
}

fn golden_init_frames() -> (Vec<Vec<u8>>, usize) {
    let fixture = fixture();
    let frames = fixture["init"]["frames_hex"]
        .as_array()
        .expect("frames")
        .iter()
        .map(|frame| decode_hex(frame.as_str().expect("frame hex")))
        .collect();
    let split = u64_at(&fixture, "/init/chip_detect_after_frame") as usize;
    (frames, split)
}

#[test]
fn chip_detection_flushes_resets_then_counts_at_the_default_baud() {
    // Arrange
    let (golden, split) = golden_init_frames();
    let frame = |index: usize| {
        Bm1366AdapterAction::WriteFrame(crate::bm1366::packet::FrameBytes::from_fixed(
            &golden[index],
        ))
    };
    // Act
    let actions = chip_detect_actions(Bm1370InitConfig::gamma_single_chip()).expect("plan");
    // Assert
    assert_eq!(split, 4);
    assert_eq!(
        actions,
        [
            Bm1366AdapterAction::PublishStatus(AsicInitStatus::ChipDetectOnly),
            Bm1366AdapterAction::ClearRx,
            Bm1366AdapterAction::DelayMs(100),
            Bm1366AdapterAction::RESET_PULSE,
            Bm1366AdapterAction::UseDefaultBaud { baud: 115_200 },
            Bm1366AdapterAction::DelayMs(100),
            frame(0),
            frame(1),
            frame(2),
            frame(3),
            Bm1366AdapterAction::WAIT_TX_DONE,
            Bm1366AdapterAction::ReadChipId {
                expected_chips: 1,
                timeout_ms: 1_000
            },
        ]
    );
}

#[test]
fn chip_detection_writes_upstream_s_frames_before_the_count() {
    // Arrange
    let (golden, split) = golden_init_frames();
    // Act
    let actions = chip_detect_actions(Bm1370InitConfig::gamma_single_chip()).expect("plan");
    // Assert
    assert_eq!(written(&actions), golden[..split]);
}

#[test]
fn the_mining_ready_init_writes_upstream_s_frames_then_the_max_baud_switch() {
    // Arrange
    let (golden, split) = golden_init_frames();
    let mut expected = golden[split..].to_vec();
    expected.push(hex_at(&fixture(), "/init/max_baud_frame_hex"));
    // Act
    let actions = mining_ready_actions(Bm1370InitConfig::gamma_single_chip()).expect("plan");
    // Assert
    assert_eq!(written(&actions), expected);
}

#[test]
fn the_mining_ready_init_ends_at_the_max_baud_with_a_clear_receive_buffer() {
    // Act
    let actions = mining_ready_actions(Bm1370InitConfig::gamma_single_chip()).expect("plan");
    // Assert
    assert_eq!(
        actions[actions.len() - 4..],
        [
            Bm1366AdapterAction::WAIT_TX_DONE,
            Bm1366AdapterAction::UseMaxBaud { baud: 1_000_000 },
            Bm1366AdapterAction::ClearRx,
            Bm1366AdapterAction::PublishStatus(AsicInitStatus::InitializedNoMining),
        ]
    );
}

#[test]
fn every_ramp_step_is_followed_by_its_100_ms_wait() {
    // Arrange
    let steps = u64_at(&fixture(), "/frequency/ramp_50_to_525_writes") as usize;
    // Act
    let actions = mining_ready_actions(Bm1370InitConfig::gamma_single_chip()).expect("plan");
    // Assert
    let waits: Vec<usize> = actions
        .iter()
        .enumerate()
        .filter(|(_, action)| **action == Bm1366AdapterAction::DelayMs(100))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(waits.len(), steps);
    assert!(waits
        .iter()
        .all(|&index| matches!(actions[index - 1], Bm1366AdapterAction::WriteFrame(_))));
}

#[test]
fn the_chip_count_accepts_a_bm1370_and_skips_another_chip() {
    // Arrange
    let fixture = fixture();
    let frames = [
        hex_at(&fixture, "/chip_id/frame_hex"),
        hex_at(&fixture, "/chip_id/wrong_chip_frame_hex"),
    ];
    // Act
    let dispositions = frames.map(|frame| classify_chip_id_count_frame(&frame));
    // Assert
    assert_eq!(
        dispositions,
        [
            Ok(ChipIdCountFrameDisposition::Accept),
            Ok(ChipIdCountFrameDisposition::SoftRetry)
        ]
    );
}

#[test]
fn the_chip_count_skips_a_corrupted_frame_and_fails_a_short_one() {
    // Arrange
    let mut corrupted = hex_at(&fixture(), "/chip_id/frame_hex");
    corrupted[10] ^= 0x01;
    // Act
    let dispositions = [
        classify_chip_id_count_frame(&corrupted),
        classify_chip_id_count_frame(&corrupted[..10]),
    ];
    // Assert
    assert_eq!(
        dispositions,
        [
            Ok(ChipIdCountFrameDisposition::SoftRetry),
            Err(Bm1370ProtocolFault::InvalidLength {
                expected: 11,
                actual: 10
            })
        ]
    );
}

#[test]
fn each_family_counts_only_its_own_chip() {
    // Arrange
    let bm1370_frame = hex_at(&fixture(), "/chip_id/frame_hex");
    // Act
    let dispositions = [AsicFamily::Bm1366, AsicFamily::Bm1370]
        .map(|family| family.classify_chip_id_count_frame(&bm1370_frame));
    // Assert
    assert_eq!(
        dispositions,
        [
            Ok(ChipIdCountFrameDisposition::SoftRetry),
            Ok(ChipIdCountFrameDisposition::Accept)
        ]
    );
}
