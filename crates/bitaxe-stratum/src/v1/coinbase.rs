//! Coinbase, extranonce, and merkle helpers for Stratum v1 mining jobs.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/mining.c`
//! - `reference/esp-miner/components/stratum/utils.c`
//! - Parity checklist rows `STR-003` and `STR-006`

use std::fmt::Write;

use sha2::{Digest, Sha256};

use crate::error::StratumV1Error;
use crate::v1::messages::MAX_EXTRANONCE_2_LEN;

pub fn double_sha256(bytes: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(bytes);
    Sha256::digest(first).into()
}

pub fn double_sha256_hex_parts(parts: &[&str]) -> Result<[u8; 32], StratumV1Error> {
    let mut bytes = Vec::new();
    for part in parts {
        bytes.extend(hex_decode(part, "hex_part")?);
    }

    Ok(double_sha256(&bytes))
}

pub fn merkle_root(coinbase_hash: [u8; 32], branches: &[[u8; 32]]) -> [u8; 32] {
    let mut root = coinbase_hash;
    for branch in branches {
        let mut combined = [0; 64];
        combined[..32].copy_from_slice(&root);
        combined[32..].copy_from_slice(branch);
        root = double_sha256(&combined);
    }

    root
}

pub fn extranonce_2_generate(value: u64, len: usize) -> Result<String, StratumV1Error> {
    if len > usize::from(MAX_EXTRANONCE_2_LEN) {
        return Err(StratumV1Error::InvalidField {
            field: "extranonce2_len",
            reason: "exceeds MAX_EXTRANONCE_2_LEN 32",
        });
    }

    let value_bytes = value.to_le_bytes();
    let copy_len = len.min(value_bytes.len());
    let mut bytes = vec![0; len];
    bytes[..copy_len].copy_from_slice(&value_bytes[..copy_len]);

    Ok(hex_encode(&bytes))
}

pub(crate) fn hex_decode(raw: &str, field: &'static str) -> Result<Vec<u8>, StratumV1Error> {
    if raw.len() % 2 != 0 {
        return Err(StratumV1Error::InvalidField {
            field,
            reason: "expected even-length hexadecimal",
        });
    }

    let mut decoded = Vec::with_capacity(raw.len() / 2);
    for pair in raw.as_bytes().chunks_exact(2) {
        let high = hex_nibble(pair[0], field)?;
        let low = hex_nibble(pair[1], field)?;
        decoded.push((high << 4) | low);
    }

    Ok(decoded)
}

pub(crate) fn hex_32(raw: &str, field: &'static str) -> Result<[u8; 32], StratumV1Error> {
    let bytes = hex_decode(raw, field)?;
    if bytes.len() != 32 {
        return Err(StratumV1Error::InvalidField {
            field,
            reason: "expected 32 decoded bytes",
        });
    }

    let mut fixed = [0; 32];
    fixed.copy_from_slice(&bytes);
    Ok(fixed)
}

fn hex_nibble(byte: u8, field: &'static str) -> Result<u8, StratumV1Error> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(StratumV1Error::InvalidField {
            field,
            reason: "expected hexadecimal character",
        }),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("writing to String should not fail");
    }
    encoded
}

#[cfg(test)]
mod mining_job_tests {
    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn mining_job_extranonce_2_generate_matches_upstream_little_endian_copy() {
        // Arrange
        let value = 1;
        let len = 4;

        // Act
        let extranonce = extranonce_2_generate(value, len);

        // Assert
        assert_eq!(extranonce, Ok("01000000".to_owned()));
    }

    #[test]
    fn mining_job_double_sha256_hex_parts_matches_byte_hashing() {
        // Arrange
        let mut first = Sha256::new();
        first.update([0x00, 0x01]);
        let first_digest = first.finalize();
        let expected = Sha256::digest(first_digest);

        // Act
        let digest = double_sha256_hex_parts(&["00", "01"]);

        // Assert
        assert_eq!(digest, Ok(expected.into()));
    }

    #[test]
    fn mining_job_double_sha256_hex_parts_rejects_malformed_hex() {
        // Arrange
        let parts = ["0"];

        // Act
        let digest = double_sha256_hex_parts(&parts);

        // Assert
        assert!(matches!(
            digest,
            Err(crate::error::StratumV1Error::InvalidField {
                field: "hex_part",
                ..
            })
        ));
    }
}
