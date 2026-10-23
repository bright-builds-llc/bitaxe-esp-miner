//! BM1370 receive-frame decoding against the golden fixture.

use super::{decode_hex, fixture, gamma_jobs, hex_at, u64_at};
use crate::bm1370::{
    result::{
        parse_chip_id_response, parse_result_frame, Bm1370ChipIdentity, Bm1370ParsedResult,
        Bm1370Register, Bm1370ValidJobIds,
    },
    Bm1370ProtocolFault,
};

#[test]
fn a_nonce_result_decodes_its_job_core_and_version() {
    // Arrange
    let fixture = fixture();
    let frame = hex_at(&fixture, "/job_result/frame_hex");
    // Act
    let parsed = parse_result_frame(&frame, &gamma_jobs(), 16).expect("result");
    // Assert
    let Bm1370ParsedResult::JobNonce(result) = parsed else {
        panic!("expected a nonce result");
    };
    let case = &fixture["job_result"];
    assert_eq!(u64::from(result.job_id.raw()), case["expected_job_id"]);
    assert_eq!(
        result.nonce.to_be_bytes().to_vec(),
        decode_hex(case["expected_nonce_hex"].as_str().expect("nonce"))
    );
    assert_eq!(
        [result.asic_index, result.core_id, result.small_core_id].map(u64::from),
        [
            "expected_asic_index",
            "expected_core_id",
            "expected_small_core_id"
        ]
        .map(|name| case[name].as_u64().expect(name))
    );
    assert_eq!(
        u64::from(result.version_bits),
        case["expected_version_bits"]
    );
}

#[test]
fn the_highest_core_is_a_valid_core() {
    // Arrange
    let frame = hex_at(&fixture(), "/job_result_highest_core/frame_hex");
    // Act
    let parsed = parse_result_frame(&frame, &gamma_jobs(), 256).expect("result");
    // Assert
    let Bm1370ParsedResult::JobNonce(result) = parsed else {
        panic!("expected a nonce result");
    };
    assert_eq!(
        (result.core_id, result.asic_index, result.small_core_id),
        (127, 0, 0)
    );
}

#[test]
fn a_result_for_an_unsent_job_is_refused() {
    // Arrange
    let frame = hex_at(&fixture(), "/job_result/frame_hex");
    // Act
    let refusal = parse_result_frame(&frame, &Bm1370ValidJobIds::empty(), 16);
    // Assert
    assert_eq!(
        refusal,
        Err(Bm1370ProtocolFault::InvalidJobId { job_id: 24 })
    );
}

#[test]
fn a_cleared_job_table_refuses_its_old_results() {
    // Arrange
    let frame = hex_at(&fixture(), "/job_result/frame_hex");
    let mut jobs = gamma_jobs();
    // Act
    jobs.clear();
    // Assert
    assert!(parse_result_frame(&frame, &jobs, 16).is_err());
}

#[test]
fn a_register_read_decodes_its_register_and_value() {
    // Arrange
    let fixture = fixture();
    let frame = hex_at(&fixture, "/register_result/frame_hex");
    // Act
    let parsed = parse_result_frame(&frame, &gamma_jobs(), 16).expect("register read");
    // Assert
    let Bm1370ParsedResult::RegisterRead(read) = parsed else {
        panic!("expected a register read");
    };
    assert_eq!(read.register, Bm1370Register::TotalCount);
    assert_eq!(
        [
            u64::from(read.asic_index),
            u64::from(read.asic_address),
            u64::from(read.value)
        ],
        [
            "expected_asic_index",
            "expected_asic_address",
            "expected_value"
        ]
        .map(|name| u64_at(&fixture, &format!("/register_result/{name}")))
    );
}

#[test]
fn every_mapped_register_round_trips_its_code() {
    // Act
    let codes = [0x4c, 0x88, 0x89, 0x8a, 0x8b, 0x8c]
        .map(|code| Bm1370Register::try_from(code).expect("mapped").code());
    // Assert
    assert_eq!(codes, [0x4c, 0x88, 0x89, 0x8a, 0x8b, 0x8c]);
}

#[test]
fn an_unknown_register_is_refused() {
    // Arrange
    let frame = hex_at(&fixture(), "/unknown_register_frame_hex");
    // Act
    let refusal = parse_result_frame(&frame, &gamma_jobs(), 16);
    // Assert
    assert_eq!(
        refusal,
        Err(Bm1370ProtocolFault::UnknownRegister { register: 0x50 })
    );
}

#[test]
fn a_damaged_frame_is_refused() {
    // Arrange
    let good = hex_at(&fixture(), "/register_result/frame_hex");
    let mut bad_crc = good.clone();
    bad_crc[10] ^= 0x01;
    let mut bad_preamble = good.clone();
    bad_preamble[0] = 0x55;
    // Act
    let refusals = [&bad_crc[..], &bad_preamble[..], &good[..10]]
        .map(|frame| parse_result_frame(frame, &gamma_jobs(), 16).err());
    // Assert
    assert_eq!(
        refusals,
        [
            Some(Bm1370ProtocolFault::BadCrc),
            Some(Bm1370ProtocolFault::BadPreamble {
                expected: 0xaa55,
                actual: 0x5555
            }),
            Some(Bm1370ProtocolFault::InvalidLength {
                expected: 11,
                actual: 10
            }),
        ]
    );
}

#[test]
fn an_out_of_range_address_interval_is_refused() {
    // Arrange
    let frame = hex_at(&fixture(), "/register_result/frame_hex");
    // Act
    let refusals =
        [0, 257].map(|interval| parse_result_frame(&frame, &gamma_jobs(), interval).err());
    // Assert
    assert_eq!(
        refusals,
        [0, 257].map(
            |address_interval| Some(Bm1370ProtocolFault::InvalidAddressInterval {
                address_interval
            })
        )
    );
}

#[test]
fn a_bm1370_answers_the_chip_id_read() {
    // Act
    let identity = parse_chip_id_response(&hex_at(&fixture(), "/chip_id/frame_hex"));
    // Assert
    assert_eq!(
        identity,
        Ok(Bm1370ChipIdentity {
            core_num: 0x80,
            address: 0x00,
        })
    );
}

#[test]
fn another_chip_s_id_is_refused() {
    // Act
    let refusal = parse_chip_id_response(&hex_at(&fixture(), "/chip_id/wrong_chip_frame_hex"));
    // Assert
    assert_eq!(
        refusal,
        Err(Bm1370ProtocolFault::UnexpectedChipId { chip_id: 0x1368 })
    );
}
