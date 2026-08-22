//! SV2 responder authority-key decoding.

use super::StratumV2Error;
use crate::v1::payout_address::{decode_base58_check_bytes, encode_base58_check};

#[must_use]
pub fn encode_authority_public_key(key: [u8; 32]) -> String {
    let mut second_version_and_key = Vec::with_capacity(33);
    second_version_and_key.push(0x00);
    second_version_and_key.extend_from_slice(&key);
    encode_base58_check(0x01, &second_version_and_key)
}

pub fn parse_authority_public_key(value: &str) -> Result<Option<[u8; 32]>, StratumV2Error> {
    if value.is_empty() {
        return Ok(None);
    }
    let decoded = decode_base58_check_bytes(value).map_err(|_| StratumV2Error::InvalidField {
        field: "authority_public_key",
        reason: "is not canonical Base58Check",
    })?;
    if decoded.len() != 34 || decoded[..2] != [0x01, 0x00] {
        return Err(StratumV2Error::InvalidField {
            field: "authority_public_key",
            reason: "has the wrong version or key length",
        });
    }
    let mut key = [0; 32];
    key.copy_from_slice(&decoded[2..]);
    Ok(Some(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::payout_address::encode_base58_check;

    #[test]
    fn authority_key_requires_two_byte_version_and_exact_xonly_length() {
        // Arrange
        let mut second_version_and_key = vec![0x00];
        second_version_and_key.extend_from_slice(&[0x22; 32]);
        let encoded = encode_base58_check(0x01, &second_version_and_key);
        let wrong_version = encode_base58_check(0x02, &second_version_and_key);

        // Act
        let parsed = parse_authority_public_key(&encoded);
        let rejected = parse_authority_public_key(&wrong_version);

        // Assert
        assert_eq!(parsed, Ok(Some([0x22; 32])));
        assert!(matches!(rejected, Err(StratumV2Error::InvalidField { .. })));
    }

    #[test]
    fn authority_key_round_trips_canonical_reference_base58check_shape() {
        // Arrange
        let key = [0x33; 32];

        // Act
        let encoded = encode_authority_public_key(key);
        let decoded = parse_authority_public_key(&encoded);

        // Assert
        assert_eq!(decoded, Ok(Some(key)));
        assert!(encoded.len() <= 52);
    }
}
