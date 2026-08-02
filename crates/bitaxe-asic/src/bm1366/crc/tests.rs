use serde::Deserialize;

use super::{crc16, crc16_false, crc5};

const REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const PROTOCOL_FIXTURE: &str = include_str!("../../../fixtures/bm1366/protocol-cases.json");

#[derive(Deserialize)]
struct ProtocolFixture {
    reference_commit: String,
    cases: Vec<ProtocolCase>,
}

#[derive(Deserialize)]
struct ProtocolCase {
    id: String,
    kind: String,
    #[serde(rename = "input_hex")]
    maybe_input_hex: Option<String>,
    #[serde(rename = "expected_crc_hex")]
    maybe_expected_crc_hex: Option<String>,
}

#[test]
fn reference_crc_vectors_cover_every_upstream_variant() {
    // Arrange
    let fixture: ProtocolFixture =
        serde_json::from_str(PROTOCOL_FIXTURE).expect("protocol fixture must be valid JSON");
    let crc_cases: Vec<_> = fixture
        .cases
        .iter()
        .filter(|case| matches!(case.kind.as_str(), "crc5" | "crc16" | "crc16_false"))
        .collect();

    // Act
    let observed: Vec<_> = crc_cases
        .iter()
        .map(|case| {
            let input = decode_hex(
                case.maybe_input_hex
                    .as_deref()
                    .expect("CRC fixture must contain input_hex"),
            );
            let crc = match case.kind.as_str() {
                "crc5" => u16::from(crc5(&input)),
                "crc16" => crc16(&input),
                "crc16_false" => crc16_false(&input),
                _ => unreachable!("fixture filter admits only CRC variants"),
            };
            (case.id.as_str(), crc)
        })
        .collect();

    // Assert
    assert_eq!(fixture.reference_commit, REFERENCE_COMMIT);
    assert_eq!(crc_cases.len(), 7, "fixture must cover all CRC boundaries");
    for ((id, observed_crc), case) in observed.iter().zip(crc_cases) {
        let expected = u16::from_str_radix(
            case.maybe_expected_crc_hex
                .as_deref()
                .expect("CRC fixture must contain expected_crc_hex"),
            16,
        )
        .expect("expected CRC must be hexadecimal");
        assert_eq!(*observed_crc, expected, "CRC mismatch for {id}");
    }
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .split_ascii_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).expect("fixture input must be hexadecimal"))
        .collect()
}
