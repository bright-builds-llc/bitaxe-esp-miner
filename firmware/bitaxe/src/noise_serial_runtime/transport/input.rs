//! Consumes variable-sized control input before the crypto work-item stack is entered.
use bitaxe_worker_control::noise::{canonical_bytes, NoiseStart};
use std::net::{Ipv4Addr, SocketAddrV4};

pub(crate) struct PreparedInput {
    pub endpoint: SocketAddrV4,
    pub authority: [u8; 32],
}

#[inline(never)]
pub(crate) fn maybe_prepare(input: NoiseStart) -> Option<PreparedInput> {
    let ipv4 = input.fixture_ipv4.parse::<Ipv4Addr>().ok()?;
    let authority = canonical_bytes::<32>(&input.authority_public_key)?;
    Some(PreparedInput {
        endpoint: SocketAddrV4::new(ipv4, input.fixture_port),
        authority,
    })
    // NoiseStart drops here, including its zeroizing variable-sized fields.
}
#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use bitaxe_worker_control::noise::StartSchema;
    fn input() -> NoiseStart {
        NoiseStart {
            schema: StartSchema::V2,
            attempt_id: URL_SAFE_NO_PAD.encode([1; 16]),
            expected_boot_ordinal: 2,
            network_observed_at_us: 1_000_000,
            fixture_ipv4: "192.168.1.3".into(),
            fixture_port: 12345,
            authority_public_key: URL_SAFE_NO_PAD.encode([2; 32]),
        }
    }
    #[test]
    fn preparation_preserves_exact_endpoint_and_authority_only() {
        // Arrange / Act
        let prepared = maybe_prepare(input()).expect("prepared input");
        // Assert
        assert_eq!(prepared.endpoint.to_string(), "192.168.1.3:12345");
        assert_eq!(prepared.authority, [2; 32]);
        assert!(std::mem::size_of::<PreparedInput>() <= 48);
    }
    #[test]
    fn malformed_input_cannot_reach_crypto() {
        // Arrange
        let mut input = input();
        input.authority_public_key = "invalid".into();
        // Act / Assert
        assert!(maybe_prepare(input).is_none());
    }
}
