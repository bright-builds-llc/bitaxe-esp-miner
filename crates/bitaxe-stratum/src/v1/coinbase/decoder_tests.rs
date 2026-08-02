use serde::Deserialize;

use super::{
    decode_coinbase_notification, decode_compact_size, CoinbaseScriptKind, StratumV1Error,
    MAX_COINBASE_OUTPUTS,
};
use crate::v1::messages::MiningNotify;

const FIXTURE: &str = include_str!("../../../fixtures/v1/coinbase-decoder-cases.json");
const PINNED_REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";

#[derive(Debug, Deserialize)]
struct CoinbaseDecoderFixture {
    metadata: FixtureMetadata,
    compact_size_cases: Vec<CompactSizeCase>,
    script_cases: Vec<ScriptCase>,
    notification: NotificationCase,
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
struct CompactSizeCase {
    encoded: String,
    expected: u64,
    expected_len: usize,
}

#[derive(Debug, Deserialize)]
struct ScriptCase {
    script_pubkey: String,
    expected_kind: String,
}

#[derive(Debug, Clone, Deserialize)]
struct NotificationCase {
    coinbase_1: String,
    coinbase_2: String,
    extranonce1: String,
    extranonce2_len: usize,
    version: u32,
    nbits: u32,
    expected: ExpectedNotification,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedNotification {
    network_difficulty: f64,
    block_height: u32,
    scriptsig: String,
    total_value_satoshis: u64,
    bip54_signaling: bool,
    bip110_signaling: bool,
    outputs: Vec<ExpectedOutput>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedOutput {
    value_satoshis: u64,
    script_pubkey: String,
    script_kind: String,
}

#[test]
fn coinbase_decoder_fixture_is_pinned_and_owned_by_str_004() {
    // Arrange
    let fixture = fixture();

    // Act
    let metadata = fixture.metadata;

    // Assert
    assert_eq!(metadata.checklist_ids, ["STR-004"]);
    assert_eq!(metadata.reference_commit, PINNED_REFERENCE_COMMIT);
    assert!(metadata
        .source_files
        .iter()
        .any(|path| path.ends_with("components/stratum/coinbase_decoder.c")));
    assert!(metadata
        .source_files
        .iter()
        .any(|path| path.ends_with("components/stratum/test/test_coinbase_decoder.c")));
    assert!(metadata.license_posture.contains("fixture data"));
    assert!(metadata.derivation.contains("independently constructed"));
}

#[test]
fn coinbase_decoder_compact_size_fixture_covers_every_width() {
    // Arrange
    let fixture = fixture();

    for case in fixture.compact_size_cases {
        let bytes = decode_hex(&case.encoded);
        let mut offset = 0;

        // Act
        let value = decode_compact_size(&bytes, &mut offset)
            .expect("pinned CompactSize fixture should decode");

        // Assert
        assert_eq!(value, case.expected);
        assert_eq!(offset, case.expected_len);
    }
}

#[test]
fn coinbase_decoder_compact_size_rejects_each_truncated_width() {
    // Arrange
    let cases = [
        [0xfd].as_slice(),
        &[0xfe, 1, 2, 3],
        &[0xff, 1, 2, 3, 4, 5, 6, 7],
    ];

    for bytes in cases {
        let mut offset = 0;

        // Act
        let result = decode_compact_size(bytes, &mut offset);

        // Assert
        assert!(matches!(
            result,
            Err(StratumV1Error::InvalidField {
                field: "compact_size",
                ..
            })
        ));
    }
}

#[test]
fn coinbase_decoder_fixture_classifies_every_reference_script_shape() {
    // Arrange
    let fixture = fixture();

    for case in fixture.script_cases {
        let notification = single_output_notification(&case.script_pubkey, 1);

        // Act
        let decoded = decode_coinbase_notification(&notification, "aa", 1, true)
            .expect("pinned script fixture should decode");

        // Assert
        assert_eq!(decoded.outputs.len(), 1);
        assert_eq!(
            decoded.outputs[0].script_kind,
            script_kind(&case.expected_kind)
        );
    }
}

#[test]
fn coinbase_decoder_golden_notification_preserves_every_in_scope_field() {
    // Arrange
    let case = fixture().notification;
    let expected = case.expected.clone();

    // Act
    let decoded = decode_coinbase_notification(
        &notify_from(&case),
        &case.extranonce1,
        case.extranonce2_len,
        true,
    )
    .expect("pinned coinbase notification should decode");

    // Assert
    assert!((decoded.network_difficulty - expected.network_difficulty).abs() < 0.1);
    assert_eq!(decoded.block_height, expected.block_height);
    assert_eq!(
        decoded.maybe_scriptsig.as_deref(),
        Some(expected.scriptsig.as_str())
    );
    assert_eq!(decoded.total_value_satoshis, expected.total_value_satoshis);
    assert_eq!(decoded.bip54_signaling, expected.bip54_signaling);
    assert_eq!(decoded.bip110_signaling, expected.bip110_signaling);
    assert_eq!(decoded.outputs.len(), expected.outputs.len());
    for (actual, expected) in decoded.outputs.iter().zip(expected.outputs) {
        assert_eq!(actual.value_satoshis, expected.value_satoshis);
        assert_eq!(encode_hex(&actual.script_pubkey), expected.script_pubkey);
        assert_eq!(actual.script_kind, script_kind(&expected.script_kind));
    }
}

#[test]
fn coinbase_decoder_disabled_projection_keeps_totals_but_suppresses_claims() {
    // Arrange
    let case = fixture().notification;

    // Act
    let decoded = decode_coinbase_notification(
        &notify_from(&case),
        &case.extranonce1,
        case.extranonce2_len,
        false,
    )
    .expect("disabled projection should still parse the transaction");

    // Assert
    assert_eq!(
        decoded.total_value_satoshis,
        case.expected.total_value_satoshis
    );
    assert!(decoded.outputs.is_empty());
    assert!(!decoded.bip54_signaling);
    assert!(!decoded.bip110_signaling);
}

#[test]
fn coinbase_decoder_bip110_expires_at_the_reference_height() {
    // Arrange
    let last_eligible = height_notification(965_663, 0x2000_0010, 0xfeff_ffff);
    let expired = height_notification(965_664, 0x2000_0010, 0xfeff_ffff);

    // Act
    let eligible = decode_coinbase_notification(&last_eligible, "aa", 0, true)
        .expect("last eligible height should decode");
    let expired =
        decode_coinbase_notification(&expired, "aa", 0, true).expect("expiry height should decode");

    // Assert
    assert!(eligible.bip110_signaling);
    assert!(!expired.bip110_signaling);
}

#[test]
fn coinbase_decoder_bip54_requires_nonfinal_sequence() {
    // Arrange
    let case = fixture().notification;
    let mut notification = notify_from(&case);
    notification.coinbase_2 = notification.coinbase_2.replacen("feffffff", "ffffffff", 1);

    // Act
    let decoded =
        decode_coinbase_notification(&notification, &case.extranonce1, case.extranonce2_len, true)
            .expect("final-sequence fixture should decode");

    // Assert
    assert!(!decoded.bip54_signaling);
}

#[test]
fn coinbase_decoder_caps_retained_outputs_without_losing_total_value() {
    // Arrange
    let notification = seven_output_notification();

    // Act
    let decoded = decode_coinbase_notification(&notification, "aa", 1, true)
        .expect("seven-output coinbase should decode");

    // Assert
    assert_eq!(decoded.outputs.len(), MAX_COINBASE_OUTPUTS);
    assert_eq!(decoded.total_value_satoshis, 28);
}

#[test]
fn coinbase_decoder_rejects_truncated_transaction_without_partial_result() {
    // Arrange
    let mut case = fixture().notification;
    case.coinbase_2.truncate(case.coinbase_2.len() - 2);

    // Act
    let result = decode_coinbase_notification(
        &notify_from(&case),
        &case.extranonce1,
        case.extranonce2_len,
        true,
    );

    // Assert
    assert!(matches!(
        result,
        Err(StratumV1Error::InvalidField {
            field: "lock_time",
            ..
        })
    ));
}

#[test]
fn coinbase_decoder_rejects_extranonce_outside_scriptsig() {
    // Arrange
    let mut case = fixture().notification;
    case.coinbase_1.push_str("00000000");

    // Act
    let result = decode_coinbase_notification(
        &notify_from(&case),
        &case.extranonce1,
        case.extranonce2_len,
        true,
    );

    // Assert
    assert!(matches!(
        result,
        Err(StratumV1Error::InvalidField {
            field: "coinbase_split",
            ..
        })
    ));
}

fn fixture() -> CoinbaseDecoderFixture {
    serde_json::from_str(FIXTURE).expect("coinbase decoder fixture should be valid JSON")
}

fn notify_from(case: &NotificationCase) -> MiningNotify {
    MiningNotify {
        job_id: "coinbase-decoder-fixture".to_owned(),
        prev_block_hash: "00".repeat(32),
        coinbase_1: case.coinbase_1.clone(),
        coinbase_2: case.coinbase_2.clone(),
        merkle_branches: Vec::new(),
        version: case.version,
        nbits: case.nbits,
        ntime: 0,
        clean_jobs: true,
    }
}

fn seven_output_notification() -> MiningNotify {
    let mut coinbase_2 = String::from("feffffff07");
    for value in 1_u64..=7 {
        coinbase_2.push_str(&encode_hex(&value.to_le_bytes()));
        coinbase_2.push_str("016a");
    }
    coinbase_2.push_str("00000000");
    MiningNotify {
        job_id: "seven-output-fixture".to_owned(),
        prev_block_hash: "00".repeat(32),
        coinbase_1: format!("0100000001{}ffffffff040101", "00".repeat(32)),
        coinbase_2,
        merkle_branches: Vec::new(),
        version: 0,
        nbits: 0x1701_cdfb,
        ntime: 0,
        clean_jobs: true,
    }
}

fn single_output_notification(script_pubkey: &str, value_satoshis: u64) -> MiningNotify {
    let script_len = script_pubkey.len() / 2;
    let coinbase_2 = format!(
        "feffffff01{}{:02x}{}00000000",
        encode_hex(&value_satoshis.to_le_bytes()),
        script_len,
        script_pubkey
    );
    MiningNotify {
        job_id: "script-classification-fixture".to_owned(),
        prev_block_hash: "00".repeat(32),
        coinbase_1: format!("0100000001{}ffffffff040101", "00".repeat(32)),
        coinbase_2,
        merkle_branches: Vec::new(),
        version: 0,
        nbits: 0x1701_cdfb,
        ntime: 0,
        clean_jobs: true,
    }
}

fn height_notification(block_height: u32, version: u32, sequence: u32) -> MiningNotify {
    let height = block_height.to_le_bytes();
    MiningNotify {
        job_id: "signal-boundary-fixture".to_owned(),
        prev_block_hash: "00".repeat(32),
        coinbase_1: format!(
            "0100000001{}ffffffff0503{}",
            "00".repeat(32),
            encode_hex(&height[..3])
        ),
        coinbase_2: format!("{}0000000000", encode_hex(&sequence.to_le_bytes())),
        merkle_branches: Vec::new(),
        version,
        nbits: 0x1701_cdfb,
        ntime: 0,
        clean_jobs: true,
    }
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("fixture hex should be UTF-8");
            u8::from_str_radix(text, 16).expect("fixture value should be hexadecimal")
        })
        .collect()
}

fn encode_hex(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn script_kind(value: &str) -> CoinbaseScriptKind {
    match value {
        "p2pkh" => CoinbaseScriptKind::P2pkh,
        "p2sh" => CoinbaseScriptKind::P2sh,
        "p2wpkh" => CoinbaseScriptKind::P2wpkh,
        "p2wsh" => CoinbaseScriptKind::P2wsh,
        "p2tr" => CoinbaseScriptKind::P2tr,
        "op_return" => CoinbaseScriptKind::OpReturn,
        "unknown" => CoinbaseScriptKind::Unknown,
        _ => panic!("unsupported fixture script kind"),
    }
}
