use bitaxe_config::catalog::{board_catalog, VerificationScope};
use serde::Deserialize;

use crate::{
    bm1366::crc::{crc16_false, crc5},
    bm1397::{
        frequency::{frequency_write_sequence, Bm1397FrequencyPlan},
        init::{
            frequency_transition_commands, initialization_commands, Bm1397InitConfig, CORE_COUNT,
            DEFAULT_ASIC_TIMEOUT_MS, DEFAULT_DIFFICULTY, DEFAULT_FREQUENCY_MHZ, DEFAULT_VOLTAGE_MV,
            HASH_DOMAINS, SMALL_CORE_COUNT,
        },
        protocol::{Bm1397Command, RegisterTarget, DEFAULT_BAUD, MAX_BAUD},
        result::{
            Bm1397JobContext, Bm1397ParsedResult, Bm1397Register, Bm1397ResultTracker,
            RECEIVE_PREAMBLE,
        },
        work::{
            encode_work_frame, Bm1397JobId, Bm1397Midstates, Bm1397WorkFields, Bm1397WorkPayload,
            BM1397_JOB_FRAME_LEN,
        },
        Bm1397ProtocolFault, BM1397_CHIP_ID,
    },
    dispatch::{dispatch_catalog_entry, AsicDispatch, DeferredAsicModel},
};

#[derive(Debug, Deserialize)]
struct Fixture {
    schema_version: String,
    provenance: ProvenanceFixture,
    profile: ProfileFixture,
    commands: CommandFixture,
    one_midstate_work: WorkFixture,
    four_midstate_work: WorkFixture,
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
    version_mask_placeholder: u32,
    default_baud: u32,
    default_baud_payload_hex: String,
    max_baud: u32,
    max_baud_payload_hex: String,
    init_payloads_hex: Vec<String>,
    frequency_mhz: u16,
    frequency_actual_mhz: u16,
    frequency_payload_hex: String,
    prefrequency_payload_hex: String,
}

#[derive(Debug, Deserialize)]
struct WorkFixture {
    job_id: u8,
    next_job_id: u8,
    starting_nonce_hex: String,
    nbits_hex: String,
    ntime_hex: String,
    merkle4_hex: String,
    midstate_bytes: Vec<u8>,
    expected_payload_hex: String,
}

#[derive(Debug, Deserialize)]
struct JobResultFixture {
    body_hex: String,
    address_interval: u16,
    job_id: u8,
    base_version: u32,
    version_mask: u32,
    expected_midstate_index: u8,
    expected_nonce_hex: String,
    expected_asic_index: u8,
    expected_core_id: u8,
    expected_small_core_id: u8,
    expected_rolled_version: u32,
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
    serde_json::from_str(include_str!("../../fixtures/bm1397/protocol-cases.json"))
        .expect("BM1397 protocol fixture should parse")
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

fn result_frame(body_hex: &str, is_job_response: bool) -> [u8; 9] {
    let body: [u8; 6] = decode_hex(body_hex)
        .try_into()
        .expect("result body should contain six bytes");
    let response_bit = if is_job_response { 0x80 } else { 0x00 };
    let mut frame = [0; 9];
    frame[..2].copy_from_slice(&RECEIVE_PREAMBLE.to_be_bytes());
    frame[2..8].copy_from_slice(&body);
    frame[8] = (0..32)
        .map(|crc| response_bit | crc)
        .find(|candidate| {
            let mut residue = [0; 7];
            residue[..6].copy_from_slice(&body);
            residue[6] = *candidate;
            crc5(&residue) == 0
        })
        .expect("fixture should admit a CRC5 residue byte");
    frame
}

fn command_payload(command: Bm1397Command) -> Vec<u8> {
    let frame = command
        .maybe_frame_bytes()
        .expect("fixture command should encode")
        .expect("fixture command should write a frame");
    frame.as_slice()[4..frame.as_slice().len() - 1].to_vec()
}

fn work_fields(work: &WorkFixture) -> Bm1397WorkFields {
    let midstates = match work.midstate_bytes.as_slice() {
        [byte] => Bm1397Midstates::One([*byte; 32]),
        [first, second, third, fourth] => {
            Bm1397Midstates::Four([[*first; 32], [*second; 32], [*third; 32], [*fourth; 32]])
        }
        _ => panic!("fixture should contain one or four midstate bytes"),
    };
    Bm1397WorkFields {
        starting_nonce: array4(&work.starting_nonce_hex),
        nbits: array4(&work.nbits_hex),
        ntime: array4(&work.ntime_hex),
        merkle4: array4(&work.merkle4_hex),
        midstates,
    }
}

#[test]
fn bm1397_fixture_provenance_and_profile_are_exact() {
    // Arrange
    let fixture = fixture();

    // Act
    let sources = fixture.provenance.source_paths;

    // Assert
    assert_eq!(fixture.schema_version, "bitaxe-bm1397-protocol-fixtures-v1");
    assert_eq!(
        fixture.provenance.reference_commit,
        "c1915b0a63bfabebdb95a515cedfee05146c1d50"
    );
    assert_eq!(sources.len(), 6);
    assert_eq!(fixture.profile.chip_id, BM1397_CHIP_ID);
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
fn bm1397_baud_and_version_placeholder_match_reference_behavior() {
    // Arrange
    let fixture = fixture();

    // Act
    let default_baud = command_payload(Bm1397Command::SetDefaultBaud);
    let max_baud = command_payload(Bm1397Command::SetMaxBaud);
    let version_frame =
        Bm1397Command::VersionMaskPlaceholder(fixture.commands.version_mask_placeholder)
            .maybe_frame_bytes();

    // Assert
    assert_eq!(
        default_baud,
        decode_hex(&fixture.commands.default_baud_payload_hex)
    );
    assert_eq!(max_baud, decode_hex(&fixture.commands.max_baud_payload_hex));
    assert_eq!(fixture.commands.default_baud, DEFAULT_BAUD);
    assert_eq!(fixture.commands.max_baud, MAX_BAUD);
    assert_eq!(version_frame, Ok(None));
}

#[test]
fn bm1397_init_plan_contains_exact_reference_register_payloads() {
    // Arrange
    let fixture = fixture();

    // Act
    let commands = initialization_commands(Bm1397InitConfig::max_single_chip())
        .expect("single-chip BM1397 init should plan");
    let payloads: Vec<Vec<u8>> = commands
        .iter()
        .filter_map(|command| match command {
            Bm1397Command::WriteRegister { .. }
            | Bm1397Command::SetDifficultyMask(_)
            | Bm1397Command::SetDefaultBaud => Some(command_payload(*command)),
            _ => None,
        })
        .take(fixture.commands.init_payloads_hex.len())
        .collect();
    let expected: Vec<Vec<u8>> = fixture
        .commands
        .init_payloads_hex
        .iter()
        .map(|value| decode_hex(value))
        .collect();

    // Assert
    assert_eq!(payloads, expected);
    assert_eq!(commands.first(), Some(&Bm1397Command::ReadChipId));
    assert_eq!(commands.len(), 612);
    assert!(commands.contains(&Bm1397Command::DelayMs(20)));
    assert!(commands.contains(&Bm1397Command::SetChainInactive));
    assert!(commands.contains(&Bm1397Command::SetChipAddress(0)));
}

#[test]
fn bm1397_frequency_write_sequence_matches_exact_pll_and_duplicate_writes() {
    // Arrange
    let fixture = fixture();
    let quarter_mhz = u32::from(fixture.commands.frequency_mhz) * 4;

    // Act
    let plan =
        Bm1397FrequencyPlan::for_quarter_mhz(quarter_mhz).expect("fixture frequency should plan");
    let commands =
        frequency_write_sequence(quarter_mhz).expect("fixture frequency sequence should plan");
    let payloads: Vec<Vec<u8>> = commands
        .iter()
        .filter_map(|command| match command {
            Bm1397Command::WriteRegister { .. } => Some(command_payload(*command)),
            _ => None,
        })
        .collect();

    // Assert
    assert_eq!(
        plan.actual_frequency_mhz(),
        fixture.commands.frequency_actual_mhz as f32
    );
    assert_eq!(payloads.len(), 4);
    assert_eq!(
        payloads[0],
        decode_hex(&fixture.commands.prefrequency_payload_hex)
    );
    assert_eq!(payloads[1], payloads[0]);
    assert_eq!(
        payloads[2],
        decode_hex(&fixture.commands.frequency_payload_hex)
    );
    assert_eq!(payloads[3], payloads[2]);
    assert_eq!(
        commands
            .iter()
            .filter(|command| matches!(command, Bm1397Command::DelayMs(10)))
            .count(),
        5
    );
}

#[test]
fn bm1397_default_frequency_transition_uses_sixty_steps_and_delays() {
    // Arrange
    let current_frequency_mhz = 50;
    let target_frequency_mhz = DEFAULT_FREQUENCY_MHZ;

    // Act
    let commands = frequency_transition_commands(current_frequency_mhz, target_frequency_mhz)
        .expect("default BM1397 frequency transition should plan");

    // Assert
    assert_eq!(commands.len(), 600);
    assert_eq!(
        commands
            .iter()
            .filter(|command| matches!(command, Bm1397Command::DelayMs(100)))
            .count(),
        60
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| matches!(command, Bm1397Command::WriteRegister { .. }))
            .count(),
        240
    );
}

#[test]
fn bm1397_init_plan_rejects_zero_chips() {
    // Arrange
    let config = Bm1397InitConfig {
        chip_count: 0,
        ..Bm1397InitConfig::max_single_chip()
    };

    // Act
    let result = initialization_commands(config);

    // Assert
    assert_eq!(
        result,
        Err(Bm1397ProtocolFault::InvalidChipCount { chip_count: 0 })
    );
}

#[test]
fn bm1397_one_midstate_payload_zeroes_unused_slots_and_matches_golden() {
    // Arrange
    let work = fixture().one_midstate_work;
    let job_id = Bm1397JobId::new(work.job_id);
    let fields = work_fields(&work);

    // Act
    let payload = Bm1397WorkPayload::new(job_id, fields);
    let frame = encode_work_frame(job_id, fields).expect("BM1397 work should encode");

    // Assert
    assert_eq!(
        payload.bytes().as_slice(),
        decode_hex(&work.expected_payload_hex)
    );
    assert!(payload.bytes()[50..].iter().all(|byte| *byte == 0));
    assert_eq!(frame.as_slice().len(), BM1397_JOB_FRAME_LEN);
    assert_eq!(crc16_false(&frame.as_slice()[2..]), 0);
    assert_eq!(job_id.advance().raw(), work.next_job_id);
}

#[test]
fn bm1397_four_midstate_payload_matches_golden_and_wraps_job_id() {
    // Arrange
    let work = fixture().four_midstate_work;
    let job_id = Bm1397JobId::new(work.job_id);

    // Act
    let payload = Bm1397WorkPayload::new(job_id, work_fields(&work));

    // Assert
    assert_eq!(
        payload.bytes().as_slice(),
        decode_hex(&work.expected_payload_hex)
    );
    assert_eq!(payload.bytes()[1], 4);
    assert_eq!(job_id.advance().raw(), work.next_job_id);
}

#[test]
fn bm1397_job_result_matches_golden_midstate_version_roll() {
    // Arrange
    let fixture = fixture().job_result;
    let frame = result_frame(&fixture.body_hex, true);
    let mut tracker = Bm1397ResultTracker::empty();
    tracker.insert(Bm1397JobContext {
        job_id: Bm1397JobId::new(fixture.job_id),
        base_version: fixture.base_version,
        version_mask: fixture.version_mask,
    });

    // Act
    let parsed = tracker
        .parse_result_frame(&frame, fixture.address_interval)
        .expect("golden BM1397 result should parse");

    // Assert
    let Bm1397ParsedResult::JobNonce(result) = parsed else {
        panic!("expected BM1397 job result");
    };
    assert_eq!(result.job_id.raw(), fixture.job_id);
    assert_eq!(result.midstate_index, fixture.expected_midstate_index);
    assert_eq!(
        result.nonce,
        u32::from_str_radix(&fixture.expected_nonce_hex, 16).expect("fixture nonce should decode")
    );
    assert_eq!(result.asic_index, fixture.expected_asic_index);
    assert_eq!(result.core_id, fixture.expected_core_id);
    assert_eq!(result.small_core_id, fixture.expected_small_core_id);
    assert_eq!(result.rolled_version, fixture.expected_rolled_version);
}

#[test]
fn bm1397_result_tracker_rejects_consecutive_duplicate_nonce() {
    // Arrange
    let fixture = fixture().job_result;
    let frame = result_frame(&fixture.body_hex, true);
    let mut tracker = Bm1397ResultTracker::empty();
    tracker.insert(Bm1397JobContext {
        job_id: Bm1397JobId::new(fixture.job_id),
        base_version: fixture.base_version,
        version_mask: fixture.version_mask,
    });
    tracker
        .parse_result_frame(&frame, fixture.address_interval)
        .expect("first BM1397 nonce should parse");

    // Act
    let duplicate = tracker.parse_result_frame(&frame, fixture.address_interval);

    // Assert
    assert_eq!(
        duplicate,
        Err(Bm1397ProtocolFault::DuplicateNonce { nonce: 0x3412_40c8 })
    );
}

#[test]
fn bm1397_register_result_matches_golden_decode() {
    // Arrange
    let fixture = fixture().register_result;
    let frame = result_frame(&fixture.body_hex, false);
    let mut tracker = Bm1397ResultTracker::empty();

    // Act
    let parsed = tracker
        .parse_result_frame(&frame, fixture.address_interval)
        .expect("golden BM1397 register result should parse");

    // Assert
    let Bm1397ParsedResult::RegisterRead(read) = parsed else {
        panic!("expected BM1397 register result");
    };
    assert_eq!(fixture.expected_register, "hashrate");
    assert_eq!(read.register, Bm1397Register::Hashrate);
    assert_eq!(read.asic_index, fixture.expected_asic_index);
    assert_eq!(read.asic_address, fixture.expected_asic_address);
    assert_eq!(read.value, fixture.expected_value);
}

#[test]
fn bm1397_result_parser_rejects_invalid_crc_job_and_address_interval() {
    // Arrange
    let fixture = fixture().job_result;
    let mut bad_crc = result_frame(&fixture.body_hex, true);
    bad_crc[8] ^= 0x01;
    let invalid_job = result_frame(&fixture.body_hex.replace("06", "0a"), true);
    let out_of_range_job = result_frame(&fixture.body_hex.replace("06", "86"), true);
    let mut tracker = Bm1397ResultTracker::empty();
    tracker.insert(Bm1397JobContext {
        job_id: Bm1397JobId::new(fixture.job_id),
        base_version: fixture.base_version,
        version_mask: fixture.version_mask,
    });

    // Act
    let crc = tracker.parse_result_frame(&bad_crc, fixture.address_interval);
    let job = tracker.parse_result_frame(&invalid_job, fixture.address_interval);
    let out_of_range = tracker.parse_result_frame(&out_of_range_job, fixture.address_interval);
    let interval = tracker.parse_result_frame(&result_frame(&fixture.body_hex, true), 0);

    // Assert
    assert_eq!(crc, Err(Bm1397ProtocolFault::BadCrc));
    assert_eq!(job, Err(Bm1397ProtocolFault::InvalidJobId { job_id: 8 }));
    assert_eq!(
        out_of_range,
        Err(Bm1397ProtocolFault::InvalidJobId { job_id: 132 })
    );
    assert_eq!(
        interval,
        Err(Bm1397ProtocolFault::InvalidAddressInterval {
            address_interval: 0
        })
    );
}

#[test]
fn bm1397_result_parser_rejects_length_preamble_and_unknown_register() {
    // Arrange
    let mut bad_preamble = result_frame("010203042004", false);
    bad_preamble[0] = 0x55;
    let unknown_register = result_frame("0102030420ff", false);
    let mut tracker = Bm1397ResultTracker::empty();

    // Act
    let length = tracker.parse_result_frame(&[0; 8], 16);
    let preamble = tracker.parse_result_frame(&bad_preamble, 16);
    let register = tracker.parse_result_frame(&unknown_register, 16);

    // Assert
    assert_eq!(
        length,
        Err(Bm1397ProtocolFault::InvalidLength {
            expected: 9,
            actual: 8
        })
    );
    assert_eq!(
        preamble,
        Err(Bm1397ProtocolFault::BadPreamble {
            expected: RECEIVE_PREAMBLE,
            actual: 0x5555
        })
    );
    assert_eq!(
        register,
        Err(Bm1397ProtocolFault::UnknownRegister { register: 0xff })
    );
}

#[test]
fn bm1397_catalog_dispatch_remains_deferred_without_hardware_scope() {
    // Arrange
    let entry = board_catalog()
        .iter()
        .copied()
        .find(|entry| entry.asic().model() == "BM1397")
        .expect("catalog should contain BM1397");

    // Act
    let dispatch = dispatch_catalog_entry(entry);

    // Assert
    let AsicDispatch::Deferred(deferred) = dispatch else {
        panic!("BM1397 must remain deferred without hardware evidence");
    };
    assert_eq!(deferred.model(), DeferredAsicModel::Bm1397);
    assert_eq!(deferred.scope(), VerificationScope::NotHardwareVerified);
}

#[test]
fn bm1397_single_register_target_encodes_fixture_address() {
    // Arrange
    let command = Bm1397Command::WriteRegister {
        target: RegisterTarget::Single { asic_address: 0x40 },
        register: 0x18,
        value: [0x00, 0x00, 0x7a, 0x31],
    };

    // Act
    let payload = command_payload(command);

    // Assert
    assert_eq!(payload, [0x40, 0x18, 0x00, 0x00, 0x7a, 0x31]);
}

#[test]
fn bm1397_frequency_transition_handles_noop_and_substep_target() {
    // Arrange
    let same = (425, 425);
    let substep = (425, 430);

    // Act
    let no_change =
        frequency_transition_commands(same.0, same.1).expect("equal frequency should plan");
    let short_change = frequency_transition_commands(substep.0, substep.1)
        .expect("sub-step frequency should plan");

    // Assert
    assert!(no_change.is_empty());
    assert_eq!(short_change.len(), 9);
    assert!(!short_change
        .iter()
        .any(|command| matches!(command, Bm1397Command::DelayMs(100))));
}
