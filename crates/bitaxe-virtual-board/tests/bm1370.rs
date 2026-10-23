//! The BM1370 pure core against the independent oracle: init, chip detection, the baud switch,
//! one job and its share, all on the host.

use bitaxe_asic::asic::{AsicFamily, JobId, ProductionResult, ValidJobIds, WorkFields};
use bitaxe_asic::bm1370::{
    init::{initialization_plan, Bm1370InitConfig},
    protocol::Bm1370Command,
    result::parse_chip_id_response,
    work::encode_work_frame,
};
use bitaxe_virtual_board::asic::{header_from_payload, Bm1366, ChipModel, NonceFixture};

fn decode<const N: usize>(hex: &str) -> [u8; N] {
    assert_eq!(hex.len(), N * 2);
    let mut value = [0; N];
    for (index, byte) in value.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).expect("literal hex");
    }
    value
}

/// The Bitcoin genesis block as BM1370 work with job id 24.
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
    payload[0] = 24;
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
        nonce: 2_083_236_893,
        target_be: decode("00000000ffff0000000000000000000000000000000000000000000000000000"),
        provenance: "Bitcoin genesis block fixed public header and hash".to_owned(),
    }
}

fn fields_from(payload: &[u8; 82]) -> WorkFields {
    let array4 = |start: usize| -> [u8; 4] { payload[start..start + 4].try_into().expect("4") };
    WorkFields {
        starting_nonce: array4(2),
        nbits: array4(6),
        ntime: array4(10),
        merkle_root: payload[14..46].try_into().expect("32"),
        prev_block_hash: payload[46..78].try_into().expect("32"),
        version: array4(78),
    }
}

fn send(asic: &mut Bm1366, commands: &[Bm1370Command]) -> Vec<Vec<u8>> {
    commands
        .iter()
        .filter_map(|command| command.maybe_frame_bytes().expect("encodes"))
        .map(|frame| {
            asic.exchange(frame.as_slice())
                .expect("the oracle accepts the frame")
        })
        .collect()
}

/// A powered BM1370 oracle taken through the core's whole init plan and the baud switch.
fn initialized() -> Bm1366 {
    let mut asic = Bm1366 {
        model: ChipModel::Bm1370,
        power_enabled: true,
        reset_asserted: false,
        ..Bm1366::default()
    };
    let plan = initialization_plan(Bm1370InitConfig::gamma_single_chip()).expect("plan");
    send(&mut asic, &plan.before_chip_detect);
    send(&mut asic, &plan.after_chip_detect);
    send(&mut asic, &[Bm1370Command::SetMaxBaud]);
    asic.host_baud = 1_000_000;
    asic
}

#[test]
fn the_oracle_answers_the_chip_id_read_as_a_bm1370() {
    // Arrange
    let mut asic = Bm1366 {
        model: ChipModel::Bm1370,
        power_enabled: true,
        reset_asserted: false,
        ..Bm1366::default()
    };
    // Act
    let replies = send(&mut asic, &[Bm1370Command::ReadChipId]);
    // Assert
    assert!(parse_chip_id_response(&replies[0]).is_ok());
}

#[test]
fn the_whole_init_plan_leaves_the_chip_at_its_target_and_max_baud() {
    // Act
    let asic = initialized();
    // Assert
    assert_eq!((asic.frequency_mhz, asic.chip_baud), (525, 1_000_000));
}

#[test]
fn a_job_s_nonce_decodes_to_its_job_and_meets_the_target() {
    // Arrange
    let mut asic = initialized();
    let fixture = genesis_fixture();
    asic.install_nonce_fixture(fixture.clone())
        .expect("fixture");
    let job_id = AsicFamily::Bm1370.first_job_id();
    let frame = encode_work_frame(job_id, fields_from(&fixture.work_payload)).expect("frame");
    // Act
    let reply = asic.exchange(frame.as_slice()).expect("result");
    let result =
        AsicFamily::Bm1370.classify_production_result(&reply, &ValidJobIds::single(job_id), 256);
    // Assert
    let ProductionResult::JobNonce(nonce) = result else {
        panic!("expected a nonce result, got {result:?}");
    };
    assert_eq!((nonce.job_id, nonce.nonce), (JobId::new(24), fixture.nonce));
    assert_eq!(
        header_from_payload(&fixture.work_payload, nonce.nonce),
        fixture.header
    );
}
