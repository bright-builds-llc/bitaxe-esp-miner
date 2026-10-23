use bitaxe_config::catalog::{board_catalog, VerificationScope};
use serde_json::Value;

use crate::{
    asic::{AsicFamily, JobId, ValidJobIds, WorkFields},
    bm1366::frequency_voltage::actual_frequency_mhz,
    bm1370::{
        init::{
            expected_hashrate_ghs, frequency_plan, frequency_ramp_commands, hash_counting_number,
            initialization_plan, job_interval_ms, Bm1370InitConfig, CORE_COUNT,
            DEFAULT_ASIC_TIMEOUT_MS, DEFAULT_DIFFICULTY, DEFAULT_FREQUENCY_MHZ, DEFAULT_VOLTAGE_MV,
            HASH_DOMAINS, RESET_FREQUENCY_QUARTER_MHZ, SMALL_CORE_COUNT,
        },
        protocol::{Bm1370Command, DEFAULT_BAUD, MAX_BAUD},
        work::encode_work_frame,
        Bm1370ProtocolFault, BM1370_CHIP_ID,
    },
    dispatch::{dispatch_catalog_entry, AsicDispatch, DeferredAsicModel},
};

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../fixtures/bm1370/protocol-cases.json"))
        .expect("BM1370 protocol fixture should parse")
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).expect("fixture hex"))
        .collect()
}

fn hex_at(fixture: &Value, pointer: &str) -> Vec<u8> {
    decode_hex(
        fixture
            .pointer(pointer)
            .and_then(Value::as_str)
            .expect(pointer),
    )
}

fn u64_at(fixture: &Value, pointer: &str) -> u64 {
    fixture
        .pointer(pointer)
        .and_then(Value::as_u64)
        .expect(pointer)
}

fn frames(commands: &[Bm1370Command]) -> Vec<Vec<u8>> {
    commands
        .iter()
        .filter_map(|command| command.maybe_frame_bytes().expect("command should encode"))
        .map(|frame| frame.as_slice().to_vec())
        .collect()
}

fn write_payloads(commands: &[Bm1370Command]) -> Vec<Vec<u8>> {
    frames(commands)
        .into_iter()
        .map(|frame| frame[4..frame.len() - 1].to_vec())
        .collect()
}

fn gamma_jobs() -> ValidJobIds {
    let mut jobs = ValidJobIds::empty();
    jobs.insert(JobId::new(24));
    jobs
}

#[test]
fn the_fixture_s_provenance_names_the_pinned_reference() {
    // Act
    let fixture = fixture();
    // Assert
    assert_eq!(
        fixture["schema_version"],
        "bitaxe-bm1370-protocol-fixtures-v1"
    );
    assert_eq!(
        fixture["provenance"]["reference_commit"],
        "c1915b0a63bfabebdb95a515cedfee05146c1d50"
    );
    assert_eq!(
        fixture["provenance"]["license_posture"],
        "reference-derived fixture data; not MIT source expression"
    );
}

#[test]
fn the_profile_matches_the_fixture_and_the_catalog() {
    // Arrange
    let fixture = fixture();
    let catalog = board_catalog()
        .iter()
        .copied()
        .find(|entry| entry.asic().model() == "BM1370")
        .expect("catalog should contain BM1370")
        .asic();
    // Act
    let profile = [
        u64::from(BM1370_CHIP_ID),
        u64::from(DEFAULT_FREQUENCY_MHZ),
        u64::from(DEFAULT_VOLTAGE_MV),
        u64::from(DEFAULT_DIFFICULTY),
        u64::from(CORE_COUNT),
        u64::from(SMALL_CORE_COUNT),
        u64::from(HASH_DOMAINS),
        u64::from(DEFAULT_ASIC_TIMEOUT_MS),
    ];
    // Assert
    let names = [
        "chip_id",
        "default_frequency_mhz",
        "default_voltage_mv",
        "difficulty",
        "core_count",
        "small_core_count",
        "hash_domains",
        "default_asic_timeout_ms",
    ];
    let expected = names.map(|name| u64_at(&fixture, &format!("/profile/{name}")));
    assert_eq!(profile, expected);
    assert_eq!(
        [
            catalog.default_frequency_mhz(),
            catalog.default_voltage_mv(),
            catalog.difficulty(),
            catalog.core_count(),
            catalog.small_core_count(),
        ],
        [
            DEFAULT_FREQUENCY_MHZ,
            DEFAULT_VOLTAGE_MV,
            DEFAULT_DIFFICULTY,
            CORE_COUNT,
            SMALL_CORE_COUNT
        ]
    );
}

#[test]
fn the_init_plan_reproduces_upstream_s_frames_in_order() {
    // Arrange
    let fixture = fixture();
    let golden: Vec<Vec<u8>> = fixture["init"]["frames_hex"]
        .as_array()
        .expect("frames")
        .iter()
        .map(|frame| decode_hex(frame.as_str().expect("frame hex")))
        .collect();
    let split = u64_at(&fixture, "/init/chip_detect_after_frame") as usize;
    // Act
    let plan = initialization_plan(Bm1370InitConfig::gamma_single_chip()).expect("plan");
    // Assert
    assert_eq!(frames(&plan.before_chip_detect), golden[..split]);
    assert_eq!(frames(&plan.after_chip_detect), golden[split..]);
}

#[test]
fn the_max_baud_switch_matches_its_golden_frame() {
    // Act
    let frame = frames(&[Bm1370Command::SetMaxBaud]);
    // Assert
    assert_eq!(frame, [hex_at(&fixture(), "/init/max_baud_frame_hex")]);
}

#[test]
fn every_fixture_frequency_has_upstream_s_pll_payload() {
    // Arrange
    let fixture = fixture();
    let table = fixture["frequency"]["pll"].as_object().expect("pll table");
    // Act / Assert
    for (mhz, case) in table {
        let quarter_mhz = (mhz.parse::<f64>().expect("mhz") * 4.0) as u32;
        let plan = frequency_plan(quarter_mhz).expect("plan");
        let payload = write_payloads(&[Bm1370Command::SetFrequency(plan)]);
        let golden = decode_hex(case["payload_hex"].as_str().expect("payload"));
        assert_eq!(payload, [golden], "{mhz} MHz");
    }
}

#[test]
fn every_fixture_frequency_is_achieved_exactly() {
    // Arrange
    let fixture = fixture();
    let table = fixture["frequency"]["pll"].as_object().expect("pll table");
    // Act / Assert
    for (mhz, case) in table {
        let quarter_mhz = (mhz.parse::<f64>().expect("mhz") * 4.0) as u32;
        let achieved = actual_frequency_mhz(frequency_plan(quarter_mhz).expect("plan"));
        let golden = case["actual_mhz"].as_f64().expect("actual");
        assert_eq!(f64::from(achieved), golden, "{mhz} MHz");
    }
}

#[test]
fn the_baud_rates_match_the_fixture() {
    // Act
    let rates = [DEFAULT_BAUD, MAX_BAUD].map(u64::from);
    // Assert
    assert_eq!(rates, [115_200, u64_at(&fixture(), "/init/max_baud")]);
}

#[test]
fn a_two_chip_chain_addresses_both_chips_and_halves_the_nonce_space() {
    // Arrange
    let fixture = fixture();
    let config = Bm1370InitConfig {
        chip_count: 2,
        ..Bm1370InitConfig::gamma_single_chip()
    };
    // Act
    let plan = initialization_plan(config).expect("plan");
    // Assert
    let addresses: Vec<Vec<u8>> = frames(&plan.after_chip_detect)
        .into_iter()
        .filter(|frame| frame[2] == 0x40)
        .collect();
    let golden: Vec<Vec<u8>> = fixture["init"]["two_chip"]["set_address_frames_hex"]
        .as_array()
        .expect("frames")
        .iter()
        .map(|frame| decode_hex(frame.as_str().expect("hex")))
        .collect();
    assert_eq!(addresses, golden);
    assert_eq!(
        plan.after_chip_detect.last(),
        Some(&Bm1370Command::SetNonceSpace(
            u64_at(&fixture, "/init/two_chip/nonce_space_at_525") as u32
        ))
    );
}

#[test]
fn a_nonce_space_below_the_errata_is_refused() {
    // Act: at an absurd 4 THz clock the space falls below the 268 the errata subtracts.
    let refusal = hash_counting_number(1.0, 4.0e6, 1, CORE_COUNT);
    // Assert
    assert_eq!(refusal, Err(Bm1370ProtocolFault::InvalidNonceSpace));
}

#[test]
fn the_ramp_to_the_default_takes_seventy_six_steps() {
    // Act
    let ramp = frequency_ramp_commands(RESET_FREQUENCY_QUARTER_MHZ, 525 * 4).expect("ramp");
    // Assert
    assert_eq!(
        frames(&ramp).len() as u64,
        u64_at(&fixture(), "/frequency/ramp_50_to_525_writes")
    );
}

#[test]
fn a_target_off_the_grid_ends_the_ramp_on_the_exact_target() {
    // Arrange
    let fixture = fixture();
    // Act
    let ramp = frequency_ramp_commands(RESET_FREQUENCY_QUARTER_MHZ, 530 * 4).expect("ramp");
    // Assert
    let payloads = write_payloads(&ramp);
    assert_eq!(
        payloads.len() as u64,
        u64_at(&fixture, "/frequency/ramp_50_to_530_writes")
    );
    assert_eq!(
        payloads.last().cloned(),
        Some(hex_at(
            &fixture,
            "/frequency/ramp_50_to_530_last_payload_hex"
        ))
    );
}

#[test]
fn a_ramp_down_steps_back_to_the_reset_frequency() {
    // Act
    let ramp = frequency_ramp_commands(525 * 4, RESET_FREQUENCY_QUARTER_MHZ).expect("ramp");
    // Assert
    let payloads = write_payloads(&ramp);
    assert_eq!(payloads.len(), 76);
    assert_eq!(payloads.last().cloned(), Some(decode_hex("000840a80265")));
}

#[test]
fn a_target_under_one_step_away_is_set_directly() {
    // Act
    let near = frequency_ramp_commands(525 * 4, 525 * 4 + 20).expect("ramp");
    // Assert
    assert_eq!(frames(&near).len(), 1);
}

#[test]
fn the_current_frequency_needs_no_ramp() {
    // Act
    let same = frequency_ramp_commands(525 * 4, 525 * 4).expect("ramp");
    // Assert
    assert!(same.is_empty());
}

#[test]
fn a_ramp_from_off_the_grid_starts_at_the_next_step() {
    // Act: from 51 MHz to 62.5 MHz, the grid steps 56.25 and 62.5, and no extra exact write.
    let ramp = frequency_ramp_commands(51 * 4, 250).expect("ramp");
    // Assert
    assert_eq!(
        write_payloads(&ramp),
        [decode_hex("000840bd0265"), decode_hex("000840af0264")]
    );
}

#[test]
fn the_nonce_space_subtracts_the_errata() {
    // Arrange
    let fixture = fixture();
    // Act
    let numbers = [50.0, 525.0, 600.0]
        .map(|mhz| hash_counting_number(1.0, mhz, 1, CORE_COUNT).expect("nonce space"));
    // Assert
    let golden = ["50", "525", "600"].map(|mhz| u64_at(&fixture, &format!("/nonce_space/{mhz}")));
    assert_eq!(numbers.map(u64::from), golden);
}

#[test]
fn an_invalid_nonce_percent_has_no_nonce_space() {
    // Act
    let refusals = [0.0, 1.5, f64::NAN].map(|pct| hash_counting_number(pct, 525.0, 1, CORE_COUNT));
    // Assert
    assert_eq!(refusals, [Err(Bm1370ProtocolFault::InvalidNonceSpace); 3]);
}

#[test]
fn the_job_interval_divides_the_timeout_by_the_chain_rounded_up() {
    // Act
    let intervals = [0, 1, 3].map(job_interval_ms);
    // Assert
    assert_eq!(intervals, [500, 500, 125]);
}

#[test]
fn the_expected_hashrate_at_the_default_matches_the_fixture() {
    // Act
    let hashrate = expected_hashrate_ghs(f64::from(DEFAULT_FREQUENCY_MHZ), 1);
    // Assert
    assert_eq!(
        hashrate,
        u64_at(&fixture(), "/profile/expected_hashrate_ghs_at_default") as f64
    );
}

#[test]
fn a_work_frame_matches_its_golden_bytes() {
    // Arrange
    let fixture = fixture();
    let array4 =
        |pointer: &str| -> [u8; 4] { hex_at(&fixture, pointer).try_into().expect("4 bytes") };
    let fields = WorkFields {
        starting_nonce: array4("/work/starting_nonce_hex"),
        nbits: array4("/work/nbits_hex"),
        ntime: array4("/work/ntime_hex"),
        merkle_root: [u64_at(&fixture, "/work/merkle_root_byte") as u8; 32],
        prev_block_hash: [u64_at(&fixture, "/work/prev_block_hash_byte") as u8; 32],
        version: array4("/work/version_hex"),
    };
    // Act
    let frame = encode_work_frame(JobId::new(24), fields).expect("frame");
    // Assert
    assert_eq!(
        frame.as_slice(),
        hex_at(&fixture, "/work/expected_frame_hex")
    );
}

#[test]
fn job_ids_step_through_sixteen_values_and_repeat() {
    // Arrange
    let golden: Vec<u64> = fixture()["work"]["job_id_sequence"]
        .as_array()
        .expect("sequence")
        .iter()
        .map(|id| id.as_u64().expect("id"))
        .collect();
    let mut job_id = JobId::new(0);
    // Act
    let sequence: Vec<u64> = (0..golden.len())
        .map(|_| {
            job_id = job_id.advance(AsicFamily::Bm1370);
            u64::from(job_id.raw())
        })
        .collect();
    // Assert
    assert_eq!(sequence, golden);
}

#[test]
fn a_chip_count_of_zero_has_no_plan() {
    // Arrange
    let config = Bm1370InitConfig {
        chip_count: 0,
        ..Bm1370InitConfig::gamma_single_chip()
    };
    // Act
    let refusal = initialization_plan(config);
    // Assert
    assert_eq!(
        refusal,
        Err(Bm1370ProtocolFault::InvalidChipCount { chip_count: 0 })
    );
}

#[test]
fn the_gamma_601_stays_deferred_without_hardware_evidence() {
    // Arrange
    let entry = board_catalog()
        .iter()
        .copied()
        .find(|entry| entry.asic().model() == "BM1370")
        .expect("catalog should contain BM1370");
    // Act
    let dispatch = dispatch_catalog_entry(entry);
    // Assert
    let AsicDispatch::Deferred(deferred) = dispatch else {
        panic!("BM1370 must remain deferred without hardware evidence");
    };
    assert_eq!(deferred.model(), DeferredAsicModel::Bm1370);
    assert_eq!(deferred.scope(), VerificationScope::NotHardwareVerified);
}

mod decode;
