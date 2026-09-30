use bitaxe_asic::bm1366::{
    packet::{CommandFrame, JobFrame},
    upstream_init_frames,
};
use bitaxe_virtual_board::{
    asic::{Bm1366, NonceFixture, DISCOVERY_REPLY, DISCOVERY_REQUEST},
    ModelError,
};

fn powered() -> Bm1366 {
    Bm1366 {
        power_enabled: true,
        reset_asserted: false,
        ..Bm1366::default()
    }
}
fn decode<const N: usize>(hex: &str) -> [u8; N] {
    assert_eq!(hex.len(), N * 2);
    let mut value = [0; N];
    for (index, byte) in value.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).expect("literal hex");
    }
    value
}
fn genesis_fixture() -> NonceFixture {
    let header = decode::<80>(concat!(
        "01000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a",
        "29ab5f49",
        "ffff001d",
        "1dac2b7c"
    ));
    let mut payload = [0; 82];
    payload[0] = 0x28;
    payload[1] = 1;
    payload[6..10].copy_from_slice(&header[72..76]);
    payload[10..14].copy_from_slice(&header[68..72]);
    payload[78..82].copy_from_slice(&header[..4]);
    for word in 0..8 {
        payload[14 + word * 4..18 + word * 4]
            .copy_from_slice(&header[36 + (7 - word) * 4..40 + (7 - word) * 4]);
        payload[46 + word * 4..50 + word * 4]
            .copy_from_slice(&header[4 + (7 - word) * 4..8 + (7 - word) * 4]);
    }
    NonceFixture {
        work_payload: payload,
        header,
        nonce: 2083236893,
        target_be: decode("00000000ffff0000000000000000000000000000000000000000000000000000"),
        provenance: "Bitcoin genesis block fixed public header and hash".to_owned(),
    }
}
#[test]
fn independent_endpoint_returns_retained_discovery_golden() {
    // Arrange
    let mut asic = powered();
    // Act
    let reply = asic.exchange(&DISCOVERY_REQUEST);
    // Assert
    assert_eq!(reply, Ok(DISCOVERY_REPLY.to_vec()));
}
#[test]
fn endpoint_rejects_every_single_bit_discovery_mutation() {
    // Arrange
    let mut rejected = 0;
    // Act
    for index in 0..DISCOVERY_REQUEST.len() {
        for bit in 0..8 {
            let mut frame = DISCOVERY_REQUEST;
            frame[index] ^= 1 << bit;
            if powered().exchange(&frame).is_err() {
                rejected += 1;
            }
        }
    }
    // Assert
    assert_eq!(rejected, DISCOVERY_REQUEST.len() * 8);
}
#[test]
fn power_reset_precondition_cannot_be_bypassed_by_valid_wire() {
    // Arrange
    let mut asic = Bm1366::default();
    // Act
    let result = asic.exchange(&DISCOVERY_REQUEST);
    // Assert
    assert_eq!(result, Err(ModelError::Unavailable("ASIC power/reset")));
}
#[test]
fn upstream_initialization_golden_frames_match_independent_crc() {
    // Arrange
    let mut asic = powered();
    let frames = [
        upstream_init_frames::INIT4_FRAME,
        upstream_init_frames::INIT5_FRAME,
        upstream_init_frames::INIT135_FRAME,
        upstream_init_frames::INIT136_FRAME,
        upstream_init_frames::INIT138_FRAME,
        upstream_init_frames::INIT139_FRAME,
        upstream_init_frames::INIT171_FRAME,
        upstream_init_frames::PER_CHIP_A8_FRAME,
        upstream_init_frames::PER_CHIP_18_FRAME,
        upstream_init_frames::PER_CHIP_3C_FIRST_FRAME,
        upstream_init_frames::PER_CHIP_3C_SECOND_FRAME,
        upstream_init_frames::PER_CHIP_3C_THIRD_FRAME,
        upstream_init_frames::DIFFICULTY_256_FRAME,
        upstream_init_frames::FREQUENCY_485_FRAME,
        upstream_init_frames::NONCE_SPACE_485_FRAME,
        upstream_init_frames::INIT795_FRAME,
    ];
    // Act
    for frame in frames {
        asic.exchange(&frame).expect("pinned reference golden");
    }
    // Assert
    assert_eq!(asic.frequency_mhz, 485);
}
#[test]
fn changing_baud_requires_host_adapter_transition() {
    // Arrange
    let mut asic = powered();
    asic.exchange(&upstream_init_frames::REG28_MAX_BAUD_FRAME)
        .expect("baud request");
    // Act
    let old = asic.exchange(&DISCOVERY_REQUEST);
    asic.host_baud = 1000000;
    let new = asic.exchange(&DISCOVERY_REQUEST);
    // Assert
    assert_eq!(old, Err(ModelError::Unavailable("ASIC baud mismatch")));
    assert_eq!(new, Ok(DISCOVERY_REPLY.to_vec()));
}
#[test]
fn unknown_register_cannot_succeed() {
    // Arrange
    let mut asic = powered();
    let frame = CommandFrame::new(0x51, &[0, 0xff, 0, 0, 0, 0]).expect("codec");
    // Act
    let result = asic.exchange(frame.bytes());
    // Assert
    assert_eq!(result, Err(ModelError::Unsupported("ASIC write register")));
}
#[test]
fn double_sha_oracle_matches_known_genesis_hash() {
    // Arrange
    let fixture = genesis_fixture();
    // Act
    let result = fixture.validate();
    // Assert
    assert_eq!(
        result,
        Ok(decode(
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
        ))
    );
}
#[test]
fn mutated_nonce_cannot_be_presented_as_proven_fixture() {
    // Arrange
    let mut fixture = genesis_fixture();
    fixture.nonce ^= 1;
    // Act
    let result = fixture.validate();
    // Assert
    assert_eq!(
        result,
        Err(ModelError::Invalid("nonce fixture header/work mismatch"))
    );
}
#[test]
fn mutated_header_and_matching_nonce_fail_independent_target() {
    // Arrange
    let mut fixture = genesis_fixture();
    fixture.nonce ^= 1;
    fixture.header[76..].copy_from_slice(&fixture.nonce.to_le_bytes());
    // Act
    let result = fixture.validate();
    // Assert
    assert_eq!(
        result,
        Err(ModelError::Invalid("nonce does not meet target"))
    );
}
#[test]
fn actual_work_packet_returns_only_proven_matching_nonce() {
    // Arrange
    let mut asic = powered();
    asic.exchange(&upstream_init_frames::FREQUENCY_485_FRAME)
        .expect("PLL");
    let fixture = genesis_fixture();
    let payload = fixture.work_payload;
    asic.install_nonce_fixture(fixture).expect("proved nonce");
    let frame = JobFrame::new(0x21, &payload).expect("codec");
    // Act
    let reply = asic.exchange(frame.bytes()).expect("work");
    // Assert
    assert_eq!(reply.len(), 11);
    assert_eq!(&reply[2..6], &2083236893_u32.to_le_bytes());
    assert_eq!(asic.dispatched_jobs, 1);
}
#[test]
fn work_mutation_without_matching_fixture_produces_no_nonce() {
    // Arrange
    let mut asic = powered();
    asic.exchange(&upstream_init_frames::FREQUENCY_485_FRAME)
        .expect("PLL");
    let fixture = genesis_fixture();
    let mut payload = fixture.work_payload;
    payload[14] ^= 1;
    asic.install_nonce_fixture(fixture).expect("proved nonce");
    let frame = JobFrame::new(0x21, &payload).expect("codec");
    // Act
    let reply = asic.exchange(frame.bytes());
    // Assert
    assert_eq!(reply, Ok(Vec::new()));
}
#[test]
fn all_work_packet_single_bit_mutations_are_rejected() {
    // Arrange
    let payload = genesis_fixture().work_payload;
    let frame = JobFrame::new(0x21, &payload).expect("codec");
    let mut rejected = 0;
    // Act
    for index in 0..88 {
        for bit in 0..8 {
            let mut mutated = frame.bytes().to_vec();
            mutated[index] ^= 1 << bit;
            let mut asic = powered();
            asic.exchange(&upstream_init_frames::FREQUENCY_485_FRAME)
                .expect("PLL");
            if asic.exchange(&mutated).is_err() {
                rejected += 1;
            }
        }
    }
    // Assert
    assert_eq!(rejected, 88 * 8);
}
