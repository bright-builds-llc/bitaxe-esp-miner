use serde::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
struct Fixture {
    schema_version: String,
    provenance: Vec<String>,
    vectors: Vec<Vector>,
}

#[derive(Debug, Deserialize)]
struct Vector {
    network: String,
    kind: String,
    payload_hex: String,
    address: String,
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!(
        "../../../fixtures/payout-address-vectors.json"
    ))
    .expect("committed payout-address fixture must parse")
}

fn network(value: &str) -> BitcoinNetwork {
    match value {
        "mainnet" => BitcoinNetwork::Mainnet,
        "testnet" => BitcoinNetwork::Testnet,
        "regtest" => BitcoinNetwork::Regtest,
        _ => panic!("fixture network must use the closed vocabulary"),
    }
}

fn kind(value: &str) -> PayoutAddressKind {
    match value {
        "p2pkh" => PayoutAddressKind::P2pkh,
        "p2sh" => PayoutAddressKind::P2sh,
        "p2wpkh" => PayoutAddressKind::P2wpkh,
        "p2wsh" => PayoutAddressKind::P2wsh,
        "p2tr" => PayoutAddressKind::P2tr,
        _ => panic!("fixture kind must use the closed vocabulary"),
    }
}

fn hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0])
                .to_digit(16)
                .expect("fixture must contain hexadecimal payloads");
            let low = char::from(pair[1])
                .to_digit(16)
                .expect("fixture must contain hexadecimal payloads");
            ((high << 4) | low) as u8
        })
        .collect()
}

fn script(kind: PayoutAddressKind, payload: &[u8]) -> Vec<u8> {
    match kind {
        PayoutAddressKind::P2pkh => {
            let mut value = vec![0x76, 0xa9, 0x14];
            value.extend_from_slice(payload);
            value.extend_from_slice(&[0x88, 0xac]);
            value
        }
        PayoutAddressKind::P2sh => {
            let mut value = vec![0xa9, 0x14];
            value.extend_from_slice(payload);
            value.push(0x87);
            value
        }
        PayoutAddressKind::P2wpkh => {
            let mut value = vec![0x00, 0x14];
            value.extend_from_slice(payload);
            value
        }
        PayoutAddressKind::P2wsh => {
            let mut value = vec![0x00, 0x20];
            value.extend_from_slice(payload);
            value
        }
        PayoutAddressKind::P2tr => {
            let mut value = vec![0x51, 0x20];
            value.extend_from_slice(payload);
            value
        }
        PayoutAddressKind::Witness(_) => panic!("fixture scripts must be standard kinds"),
    }
}

#[test]
fn golden_vectors_round_trip_and_validate_every_standard_script() {
    // Arrange
    let fixture = fixture();

    // Act / Assert
    assert_eq!(fixture.schema_version, "bitaxe-payout-address-vectors-v1");
    assert!(fixture.provenance.len() >= 2);
    for vector in fixture.vectors {
        let network = network(&vector.network);
        let kind = kind(&vector.kind);
        let payload = hex(&vector.payload_hex);
        let output_script = script(kind, &payload);

        assert_eq!(
            render_standard_script_address(network, &output_script),
            Ok(vector.address.clone()),
            "{} {}",
            vector.network,
            vector.kind
        );
        assert_eq!(
            decode_payout_address(network, &vector.address),
            Ok(DecodedPayoutAddress {
                network,
                kind,
                payload: payload.clone(),
            })
        );
        assert_eq!(
            payout_address_matches_script(network, &vector.address, &output_script),
            Ok(true)
        );
    }
}

#[test]
fn base58check_rejects_bad_alphabet_checksum_and_network() {
    // Arrange
    let valid = encode_base58_check(0x00, &[7; 20]);
    let mut bad_checksum = valid.clone().into_bytes();
    let last = bad_checksum.len() - 1;
    bad_checksum[last] = if bad_checksum[last] == b'1' {
        b'2'
    } else {
        b'1'
    };
    let bad_checksum = String::from_utf8(bad_checksum).expect("ASCII mutation must remain UTF-8");

    // Act / Assert
    assert_eq!(
        decode_base58_check("10OIl"),
        Err(PayoutAddressError::InvalidBase58)
    );
    assert_eq!(
        decode_base58_check(&bad_checksum),
        Err(PayoutAddressError::InvalidBase58Checksum)
    );
    assert_eq!(
        decode_payout_address(BitcoinNetwork::Testnet, &valid),
        Err(PayoutAddressError::NetworkMismatch)
    );
}

#[test]
fn base58_preserves_the_exact_canonical_leading_zero_count() {
    // Arrange
    let bytes = [0, 0];

    // Act
    let encoded = encode_base58(&bytes);
    let decoded = decode_base58(&encoded);

    // Assert
    assert_eq!(encoded, "11");
    assert_eq!(decoded, Ok(bytes.to_vec()));
}

#[test]
fn segwit_rejects_mixed_case_wrong_checksum_network_variant_and_program_bounds() {
    // Arrange
    let valid = encode_segwit_address(BitcoinNetwork::Mainnet, 0, &[3; 20])
        .expect("valid witness program must encode");
    let mixed_case = format!("B{}", &valid[1..]);
    let mut bad_checksum = valid.clone().into_bytes();
    let last = bad_checksum.len() - 1;
    bad_checksum[last] = if bad_checksum[last] == b'q' {
        b'p'
    } else {
        b'q'
    };
    let bad_checksum = String::from_utf8(bad_checksum).expect("ASCII mutation must remain UTF-8");
    let mut wrong_variant_data = vec![1];
    wrong_variant_data
        .extend(convert_bits(&[4; 32], 8, 5, true).expect("valid program conversion must succeed"));
    let wrong_variant = encode_bech32("bc", &wrong_variant_data, Bech32Encoding::Bech32)
        .expect("private wrong-variant fixture must encode");

    // Act / Assert
    assert_eq!(
        decode_payout_address(BitcoinNetwork::Mainnet, &mixed_case),
        Err(PayoutAddressError::InvalidBech32)
    );
    assert_eq!(
        decode_payout_address(BitcoinNetwork::Mainnet, &bad_checksum),
        Err(PayoutAddressError::InvalidBech32)
    );
    assert_eq!(
        decode_payout_address(BitcoinNetwork::Testnet, &valid),
        Err(PayoutAddressError::NetworkMismatch)
    );
    assert_eq!(
        decode_payout_address(BitcoinNetwork::Mainnet, &wrong_variant),
        Err(PayoutAddressError::InvalidWitnessProgram)
    );
    assert_eq!(
        encode_segwit_address(BitcoinNetwork::Mainnet, 17, &[0; 20]),
        Err(PayoutAddressError::InvalidWitnessProgram)
    );
    assert_eq!(
        encode_segwit_address(BitcoinNetwork::Mainnet, 0, &[0; 25]),
        Err(PayoutAddressError::InvalidWitnessProgram)
    );
    assert_eq!(
        convert_bits(&[1], 5, 8, false),
        Err(PayoutAddressError::InvalidWitnessProgram)
    );
}

#[test]
fn uppercase_segwit_and_future_witness_versions_round_trip() {
    // Arrange
    let encoded = encode_segwit_address(BitcoinNetwork::Regtest, 2, &[9; 16])
        .expect("valid future witness program must encode");

    // Act
    let decoded = decode_payout_address(BitcoinNetwork::Regtest, &encoded.to_ascii_uppercase());

    // Assert
    assert_eq!(
        decoded,
        Ok(DecodedPayoutAddress {
            network: BitcoinNetwork::Regtest,
            kind: PayoutAddressKind::Witness(2),
            payload: vec![9; 16],
        })
    );
}

#[test]
fn payout_validation_rejects_script_mismatch_and_unknown_scripts() {
    // Arrange
    let payload = [5; 20];
    let expected_script = script(PayoutAddressKind::P2pkh, &payload);
    let other_script = script(PayoutAddressKind::P2sh, &payload);
    let address = render_standard_script_address(BitcoinNetwork::Mainnet, &expected_script)
        .expect("standard script must render");

    // Act / Assert
    assert_eq!(
        payout_address_matches_script(BitcoinNetwork::Mainnet, &address, &other_script),
        Ok(false)
    );
    assert_eq!(
        render_standard_script_address(BitcoinNetwork::Mainnet, &[0x6a, 0x00]),
        Err(PayoutAddressError::UnsupportedScript)
    );
}
