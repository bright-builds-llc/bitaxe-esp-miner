use bitaxe_asic::bm1366::work::Bm1366JobId;
use serde::Deserialize;

use super::{MiningWorkBuilder, StratumV1Error};
use crate::v1::coinbase::{double_sha256_hex_parts, extranonce_2_generate, hex_32, merkle_root};
use crate::v1::messages::{ExtranonceAssignment, MiningNotify, PoolDifficulty, VersionMask};

const MINING_JOB_FIXTURE: &str = include_str!("../../../fixtures/v1/mining-job-cases.json");
const PINNED_REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";

#[derive(Debug, Deserialize)]
struct MiningJobFixture {
    metadata: FixtureMetadata,
    extranonce2_cases: Vec<Extranonce2Case>,
    job_case: JobCase,
}

#[derive(Debug, Deserialize)]
struct FixtureMetadata {
    checklist_ids: Vec<String>,
    source_files: Vec<String>,
    reference_commit: String,
    license_posture: String,
    derivation: String,
}

#[derive(Debug, Deserialize)]
struct Extranonce2Case {
    value: u64,
    length: usize,
    expected_hex: String,
}

#[derive(Debug, Clone, Deserialize)]
struct JobCase {
    job_id: String,
    prev_block_hash: String,
    coinbase_1: String,
    coinbase_2: String,
    extranonce1: String,
    extranonce2_value: u64,
    extranonce2_length: u8,
    merkle_branches: Vec<String>,
    version: String,
    nbits: String,
    ntime: String,
    clean_jobs: bool,
    asic_job_id: u8,
    pool_difficulty: f64,
    version_mask: String,
    expected: ExpectedJob,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedJob {
    extranonce2: String,
    coinbase_hash: String,
    merkle_root: String,
    starting_nonce: String,
    nbits_little_endian: String,
    ntime_little_endian: String,
    merkle_root_words_reversed: String,
    prev_block_hash_words_reversed: String,
    version_little_endian: String,
}

#[test]
fn mining_job_fixture_is_pinned_and_owned_by_str_003() {
    // Arrange
    let fixture = fixture();

    // Act
    let metadata = fixture.metadata;

    // Assert
    assert!(metadata.checklist_ids.iter().any(|id| id == "STR-003"));
    assert_eq!(metadata.reference_commit, PINNED_REFERENCE_COMMIT);
    assert!(metadata
        .source_files
        .iter()
        .any(|path| path.ends_with("components/stratum/test/test_mining.c")));
    assert!(metadata.license_posture.contains("fixture data"));
    assert!(metadata.derivation.contains("independently encoded"));
}

#[test]
fn mining_job_fixture_extranonce2_vectors_match_little_endian_copy() {
    // Arrange
    let fixture = fixture();

    for case in fixture.extranonce2_cases {
        // Act
        let encoded = extranonce_2_generate(case.value, case.length)
            .expect("pinned extranonce2 fixture should encode");

        // Assert
        assert_eq!(encoded, case.expected_hex);
    }
}

#[test]
fn mining_job_fixture_hashes_coinbase_and_merkle_path_exactly() {
    // Arrange
    let fixture = fixture();
    let case = fixture.job_case;
    let extranonce2 =
        extranonce_2_generate(case.extranonce2_value, usize::from(case.extranonce2_length))
            .expect("pinned extranonce2 fixture should encode");
    let branches = case
        .merkle_branches
        .iter()
        .map(|branch| hex_32(branch, "merkle_branch"))
        .collect::<Result<Vec<_>, _>>()
        .expect("pinned Merkle branches should decode");

    // Act
    let coinbase_hash = double_sha256_hex_parts(&[
        &case.coinbase_1,
        &case.extranonce1,
        &extranonce2,
        &case.coinbase_2,
    ])
    .expect("pinned coinbase fixture should hash");
    let root = merkle_root(coinbase_hash, &branches);

    // Assert
    assert_eq!(encode_hex(&coinbase_hash), case.expected.coinbase_hash);
    assert_eq!(encode_hex(&root), case.expected.merkle_root);
}

#[test]
fn mining_job_fixture_builds_every_typed_bm1366_field_and_context_value() {
    // Arrange
    let fixture = fixture();
    let case = fixture.job_case;
    let notify = notify_from(&case);
    let pool_difficulty = PoolDifficulty {
        difficulty: case.pool_difficulty,
    };
    let version_mask = VersionMask {
        mask: parse_hex_u32(&case.version_mask),
    };

    // Act
    let work = MiningWorkBuilder::new(
        notify,
        ExtranonceAssignment {
            extranonce1: case.extranonce1.clone(),
            extranonce2_len: case.extranonce2_length,
        },
    )
    .with_extranonce2_value(case.extranonce2_value)
    .with_pool_difficulty(pool_difficulty)
    .with_version_mask(version_mask)
    .build(Bm1366JobId::new(case.asic_job_id))
    .expect("pinned mining-job fixture should build");

    // Assert
    assert_eq!(work.stratum_job_id, case.job_id);
    assert_eq!(work.asic_job_id.raw(), case.asic_job_id);
    assert_eq!(work.extranonce2, case.expected.extranonce2);
    assert_eq!(work.ntime, parse_hex_u32(&case.ntime));
    assert_eq!(work.clean_jobs, case.clean_jobs);
    assert_eq!(work.maybe_pool_difficulty, Some(pool_difficulty));
    assert_eq!(work.maybe_version_mask, Some(version_mask));
    assert_eq!(
        encode_hex(&work.fields.starting_nonce),
        case.expected.starting_nonce
    );
    assert_eq!(
        encode_hex(&work.fields.nbits),
        case.expected.nbits_little_endian
    );
    assert_eq!(
        encode_hex(&work.fields.ntime),
        case.expected.ntime_little_endian
    );
    assert_eq!(
        encode_hex(&work.fields.merkle_root),
        case.expected.merkle_root_words_reversed
    );
    assert_eq!(
        encode_hex(&work.fields.prev_block_hash),
        case.expected.prev_block_hash_words_reversed
    );
    assert_eq!(
        encode_hex(&work.fields.version),
        case.expected.version_little_endian
    );
}

#[test]
fn mining_job_fixture_rejects_malformed_merkle_branch_before_work_creation() {
    // Arrange
    let mut fixture = fixture();
    fixture.job_case.merkle_branches[0] = "not-hex".to_owned();
    let case = fixture.job_case;

    // Act
    let result = MiningWorkBuilder::new(
        notify_from(&case),
        ExtranonceAssignment {
            extranonce1: case.extranonce1,
            extranonce2_len: case.extranonce2_length,
        },
    )
    .build(Bm1366JobId::new(case.asic_job_id));

    // Assert
    assert!(matches!(
        result,
        Err(StratumV1Error::InvalidField {
            field: "merkle_branch",
            ..
        })
    ));
}

fn fixture() -> MiningJobFixture {
    serde_json::from_str(MINING_JOB_FIXTURE).expect("mining-job fixture should be valid JSON")
}

fn notify_from(case: &JobCase) -> MiningNotify {
    MiningNotify {
        job_id: case.job_id.clone(),
        prev_block_hash: case.prev_block_hash.clone(),
        coinbase_1: case.coinbase_1.clone(),
        coinbase_2: case.coinbase_2.clone(),
        merkle_branches: case.merkle_branches.clone(),
        version: parse_hex_u32(&case.version),
        nbits: parse_hex_u32(&case.nbits),
        ntime: parse_hex_u32(&case.ntime),
        clean_jobs: case.clean_jobs,
    }
}

fn parse_hex_u32(value: &str) -> u32 {
    u32::from_str_radix(value, 16).expect("pinned u32 fixture should be hexadecimal")
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
