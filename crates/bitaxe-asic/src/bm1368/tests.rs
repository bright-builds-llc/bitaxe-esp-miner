use bitaxe_config::catalog::{board_catalog, VerificationScope};
use serde::Deserialize;

use crate::{
    bm1366::{
        crc::{crc16_false, crc5},
        frequency_voltage::actual_frequency_mhz,
    },
    bm1368::{
        init::{
            frequency_ramp_commands, initialization_commands, Bm1368InitConfig, CORE_COUNT,
            DEFAULT_ASIC_TIMEOUT_MS, DEFAULT_DIFFICULTY, DEFAULT_FREQUENCY_MHZ, DEFAULT_VOLTAGE_MV,
            HASH_DOMAINS, SMALL_CORE_COUNT,
        },
        protocol::{Bm1368Command, RegisterTarget, DEFAULT_BAUD, MAX_BAUD},
        result::{
            parse_result_frame, Bm1368ParsedResult, Bm1368Register, Bm1368ValidJobIds,
            RECEIVE_PREAMBLE,
        },
        work::{
            encode_work_frame, Bm1368JobId, Bm1368WorkFields, Bm1368WorkPayload,
            BM1368_JOB_FRAME_LEN,
        },
        Bm1368ProtocolFault, BM1368_CHIP_ID,
    },
    dispatch::{dispatch_catalog_entry, AsicDispatch, DeferredAsicModel},
};

#[derive(Debug, Deserialize)]
struct Fixture {
    schema_version: String,
    provenance: ProvenanceFixture,
    profile: ProfileFixture,
    commands: CommandFixture,
    work: WorkFixture,
    job_result: JobResultFixture,
    register_result: RegisterResultFixture,
}

#[derive(Debug, Deserialize)]
struct ProvenanceFixture {
    reference_commit: String,
    source_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProfileFixture {
    chip_id: u16,
    default_frequency_mhz: u16,
    default_voltage_mv: u16,
    difficulty: u16,
    core_count: u16,
    small_core_count: u16,
    hash_domains: u8,
    default_asic_timeout_ms: u16,
}

#[derive(Debug, Deserialize)]
struct CommandFixture {
    version_mask: u32,
    version_mask_payload_hex: String,
    default_baud: u32,
    default_baud_payload_hex: String,
    max_baud: u32,
    max_baud_payload_hex: String,
    global_init_payloads_hex: Vec<String>,
    single_chip_init_payloads_hex: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WorkFixture {
    job_id: u8,
    next_job_id: u8,
    num_midstates: u8,
    starting_nonce_hex: String,
    nbits_hex: String,
    ntime_hex: String,
    merkle_root_byte: u8,
    prev_block_hash_byte: u8,
    version_hex: String,
    expected_payload_hex: String,
}

#[derive(Debug, Deserialize)]
struct JobResultFixture {
    body_hex: String,
    address_interval: u16,
    expected_job_id: u8,
    expected_nonce_hex: String,
    expected_asic_index: u8,
    expected_core_id: u8,
    expected_small_core_id: u8,
    expected_version_bits: u32,
}

#[derive(Debug, Deserialize)]
struct RegisterResultFixture {
    body_hex: String,
    address_interval: u16,
    expected_asic_index: u8,
    expected_asic_address: u8,
    expected_value: u32,
    expected_register: String,
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../../fixtures/bm1368/protocol-cases.json"))
        .expect("BM1368 protocol fixture should parse")
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("fixture hex should be UTF-8");
            u8::from_str_radix(text, 16).expect("fixture hex should decode")
        })
        .collect()
}

fn array4(value: &str) -> [u8; 4] {
    decode_hex(value)
        .try_into()
        .expect("fixture field should contain four bytes")
}

fn result_frame(body_hex: &str, is_job_response: bool) -> [u8; 11] {
    let body: [u8; 8] = decode_hex(body_hex)
        .try_into()
        .expect("result body should contain eight bytes");
    let response_bit = if is_job_response { 0x80 } else { 0x00 };
    let mut frame = [0; 11];
    frame[..2].copy_from_slice(&RECEIVE_PREAMBLE.to_be_bytes());
    frame[2..10].copy_from_slice(&body);
    frame[10] = (0..32)
        .map(|crc| response_bit | crc)
        .find(|candidate| {
            let mut residue = [0; 9];
            residue[..8].copy_from_slice(&body);
            residue[8] = *candidate;
            crc5(&residue) == 0
        })
        .expect("fixture should admit a CRC5 residue byte");
    frame
}

fn command_payload(command: Bm1368Command) -> Vec<u8> {
    let frame = command
        .maybe_frame_bytes()
        .expect("fixture command should encode")
        .expect("fixture command should write a frame");
    frame.as_slice()[4..frame.as_slice().len() - 1].to_vec()
}

#[test]
fn bm1368_fixture_provenance_and_profile_are_exact() {
    // Arrange
    let fixture = fixture();

    // Act
    let sources = fixture.provenance.source_paths;

    // Assert
    assert_eq!(fixture.schema_version, "bitaxe-bm1368-protocol-fixtures-v1");
    assert_eq!(
        fixture.provenance.reference_commit,
        "c1915b0a63bfabebdb95a515cedfee05146c1d50"
    );
    assert_eq!(sources.len(), 3);
    assert_eq!(fixture.profile.chip_id, BM1368_CHIP_ID);
    assert_eq!(fixture.profile.default_frequency_mhz, DEFAULT_FREQUENCY_MHZ);
    assert_eq!(fixture.profile.default_voltage_mv, DEFAULT_VOLTAGE_MV);
    assert_eq!(fixture.profile.difficulty, DEFAULT_DIFFICULTY);
    assert_eq!(fixture.profile.core_count, CORE_COUNT);
    assert_eq!(fixture.profile.small_core_count, SMALL_CORE_COUNT);
    assert_eq!(fixture.profile.hash_domains, HASH_DOMAINS);
    assert_eq!(
        fixture.profile.default_asic_timeout_ms,
        DEFAULT_ASIC_TIMEOUT_MS
    );
}

#[test]
fn bm1368_version_mask_and_baud_commands_match_golden_payloads() {
    // Arrange
    let fixture = fixture();

    // Act
    let version = command_payload(Bm1368Command::SetVersionMask(fixture.commands.version_mask));
    let default_baud = command_payload(Bm1368Command::SetDefaultBaud);
    let max_baud = command_payload(Bm1368Command::SetMaxBaud);

    // Assert
    assert_eq!(
        version,
        decode_hex(&fixture.commands.version_mask_payload_hex)
    );
    assert_eq!(
        default_baud,
        decode_hex(&fixture.commands.default_baud_payload_hex)
    );
    assert_eq!(max_baud, decode_hex(&fixture.commands.max_baud_payload_hex));
    assert_eq!(fixture.commands.default_baud, DEFAULT_BAUD);
    assert_eq!(fixture.commands.max_baud, MAX_BAUD);
}

#[test]
fn bm1368_init_plan_contains_exact_global_and_single_chip_writes() {
    // Arrange
    let fixture = fixture();

    // Act
    let commands = initialization_commands(Bm1368InitConfig::supra_single_chip())
        .expect("single-chip BM1368 init should plan");
    let payloads: Vec<Vec<u8>> = commands
        .iter()
        .filter_map(|command| match command {
            Bm1368Command::WriteRegister { .. } => Some(command_payload(*command)),
            _ => None,
        })
        .collect();

    // Assert
    let expected_global: Vec<Vec<u8>> = fixture
        .commands
        .global_init_payloads_hex
        .iter()
        .map(|value| decode_hex(value))
        .collect();
    let expected_single: Vec<Vec<u8>> = fixture
        .commands
        .single_chip_init_payloads_hex
        .iter()
        .map(|value| decode_hex(value))
        .collect();
    assert_eq!(&payloads[..expected_global.len()], expected_global);
    assert_eq!(&payloads[expected_global.len()..], expected_single);
    assert_eq!(
        commands
            .iter()
            .filter(|command| matches!(command, Bm1368Command::SetVersionMask(_)))
            .count(),
        5
    );
    assert!(commands.contains(&Bm1368Command::SetChipAddress(0)));
    assert!(commands.contains(&Bm1368Command::DelayMs(500)));
    assert!(matches!(
        commands.last(),
        Some(Bm1368Command::SetVersionMask(_))
    ));
}

#[test]
fn bm1368_work_payload_and_frame_match_golden_layout() {
    // Arrange
    let fixture = fixture();
    let work = fixture.work;
    let fields = Bm1368WorkFields {
        starting_nonce: array4(&work.starting_nonce_hex),
        nbits: array4(&work.nbits_hex),
        ntime: array4(&work.ntime_hex),
        merkle_root: [work.merkle_root_byte; 32],
        prev_block_hash: [work.prev_block_hash_byte; 32],
        version: array4(&work.version_hex),
    };
    let job_id = Bm1368JobId::new(work.job_id);

    // Act
    let payload = Bm1368WorkPayload::new(job_id, fields);
    let frame = encode_work_frame(job_id, fields).expect("BM1368 work should encode");

    // Assert
    assert_eq!(work.num_midstates, 1);
    assert_eq!(
        payload.bytes().as_slice(),
        decode_hex(&work.expected_payload_hex)
    );
    assert_eq!(frame.as_slice().len(), BM1368_JOB_FRAME_LEN);
    assert_eq!(crc16_false(&frame.as_slice()[2..]), 0);
    assert_eq!(job_id.advance().raw(), work.next_job_id);
}

#[test]
fn bm1368_frequency_ramp_uses_reference_step_and_delay_counts() {
    // Arrange
    let target_frequency_mhz = DEFAULT_FREQUENCY_MHZ;

    // Act
    let commands = frequency_ramp_commands(target_frequency_mhz)
        .expect("default BM1368 frequency ramp should plan");
    let frequencies: Vec<f32> = commands
        .iter()
        .filter_map(|command| match command {
            Bm1368Command::SetFrequency(plan) => Some(actual_frequency_mhz(*plan)),
            _ => None,
        })
        .collect();
    let delay_count = commands
        .iter()
        .filter(|command| matches!(command, Bm1368Command::DelayMs(100)))
        .count();

    // Assert
    assert_eq!(frequencies.len(), 71);
    assert_eq!(delay_count, 70);
    assert!((frequencies[0] - 56.25).abs() < 0.01);
    assert!((frequencies[69] - 487.5).abs() < 0.01);
    assert!((frequencies[70] - 490.0).abs() < 0.01);
}

#[test]
fn bm1368_init_plan_rejects_zero_chips() {
    // Arrange
    let config = Bm1368InitConfig {
        chip_count: 0,
        ..Bm1368InitConfig::supra_single_chip()
    };

    // Act
    let result = initialization_commands(config);

    // Assert
    assert_eq!(
        result,
        Err(Bm1368ProtocolFault::InvalidChipCount { chip_count: 0 })
    );
}

#[test]
fn bm1368_job_result_matches_golden_decode() {
    // Arrange
    let fixture = fixture().job_result;
    let frame = result_frame(&fixture.body_hex, true);
    let valid_jobs = Bm1368ValidJobIds::single(Bm1368JobId::new(fixture.expected_job_id));

    // Act
    let parsed = parse_result_frame(&frame, &valid_jobs, fixture.address_interval)
        .expect("golden BM1368 result should parse");

    // Assert
    let Bm1368ParsedResult::JobNonce(result) = parsed else {
        panic!("expected BM1368 job result");
    };
    assert_eq!(result.job_id.raw(), fixture.expected_job_id);
    assert_eq!(
        result.nonce,
        u32::from_str_radix(&fixture.expected_nonce_hex, 16).expect("fixture nonce should decode")
    );
    assert_eq!(result.asic_index, fixture.expected_asic_index);
    assert_eq!(result.core_id, fixture.expected_core_id);
    assert_eq!(result.small_core_id, fixture.expected_small_core_id);
    assert_eq!(result.version_bits, fixture.expected_version_bits);
}

#[test]
fn bm1368_register_result_matches_golden_decode() {
    // Arrange
    let fixture = fixture().register_result;
    let frame = result_frame(&fixture.body_hex, false);

    // Act
    let parsed = parse_result_frame(
        &frame,
        &Bm1368ValidJobIds::empty(),
        fixture.address_interval,
    )
    .expect("golden BM1368 register result should parse");

    // Assert
    let Bm1368ParsedResult::RegisterRead(read) = parsed else {
        panic!("expected BM1368 register result");
    };
    assert_eq!(fixture.expected_register, "total_count");
    assert_eq!(read.register, Bm1368Register::TotalCount);
    assert_eq!(read.asic_index, fixture.expected_asic_index);
    assert_eq!(read.asic_address, fixture.expected_asic_address);
    assert_eq!(read.value, fixture.expected_value);
}

#[test]
fn bm1368_result_parser_rejects_invalid_crc_job_core_and_address_interval() {
    // Arrange
    let fixture = fixture().job_result;
    let valid_jobs = Bm1368ValidJobIds::single(Bm1368JobId::new(fixture.expected_job_id));
    let mut bad_crc = result_frame(&fixture.body_hex, true);
    bad_crc[10] ^= 0x01;
    let invalid_job = result_frame(&fixture.body_hex.replace("3f", "5f"), true);
    let invalid_core = result_frame("a0401234013f0003", true);

    // Act
    let crc = parse_result_frame(&bad_crc, &valid_jobs, fixture.address_interval);
    let job = parse_result_frame(&invalid_job, &valid_jobs, fixture.address_interval);
    let core = parse_result_frame(&invalid_core, &valid_jobs, fixture.address_interval);
    let interval = parse_result_frame(&result_frame(&fixture.body_hex, true), &valid_jobs, 0);

    // Assert
    assert_eq!(crc, Err(Bm1368ProtocolFault::BadCrc));
    assert_eq!(job, Err(Bm1368ProtocolFault::InvalidJobId { job_id: 40 }));
    assert_eq!(
        core,
        Err(Bm1368ProtocolFault::InvalidCoreId { core_id: 80 })
    );
    assert_eq!(
        interval,
        Err(Bm1368ProtocolFault::InvalidAddressInterval {
            address_interval: 0
        })
    );
}

#[test]
fn bm1368_result_parser_rejects_length_preamble_and_unknown_register() {
    // Arrange
    let mut bad_preamble = result_frame("01020304208c0000", false);
    bad_preamble[0] = 0x55;
    let unknown_register = result_frame("0102030420ff0000", false);

    // Act
    let length = parse_result_frame(&[0; 10], &Bm1368ValidJobIds::empty(), 16);
    let preamble = parse_result_frame(&bad_preamble, &Bm1368ValidJobIds::empty(), 16);
    let register = parse_result_frame(&unknown_register, &Bm1368ValidJobIds::empty(), 16);

    // Assert
    assert_eq!(
        length,
        Err(Bm1368ProtocolFault::InvalidLength {
            expected: 11,
            actual: 10
        })
    );
    assert_eq!(
        preamble,
        Err(Bm1368ProtocolFault::BadPreamble {
            expected: RECEIVE_PREAMBLE,
            actual: 0x5555
        })
    );
    assert_eq!(
        register,
        Err(Bm1368ProtocolFault::UnknownRegister { register: 0xff })
    );
}

#[test]
fn bm1368_catalog_dispatch_remains_deferred_without_hardware_scope() {
    // Arrange
    let entry = board_catalog()
        .iter()
        .copied()
        .find(|entry| entry.asic().model() == "BM1368")
        .expect("catalog should contain BM1368");

    // Act
    let dispatch = dispatch_catalog_entry(entry);

    // Assert
    let AsicDispatch::Deferred(deferred) = dispatch else {
        panic!("BM1368 must remain deferred without hardware evidence");
    };
    assert_eq!(deferred.model(), DeferredAsicModel::Bm1368);
    assert_eq!(deferred.scope(), VerificationScope::NotHardwareVerified);
}

#[test]
fn bm1368_single_register_target_encodes_fixture_address() {
    // Arrange
    let command = Bm1368Command::WriteRegister {
        target: RegisterTarget::Single { asic_address: 0x40 },
        register: 0x18,
        value: [0xf0, 0x00, 0xc1, 0x00],
    };

    // Act
    let payload = command_payload(command);

    // Assert
    assert_eq!(payload, [0x40, 0x18, 0xf0, 0x00, 0xc1, 0x00]);
}
