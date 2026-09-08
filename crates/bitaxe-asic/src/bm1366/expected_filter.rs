//! Software expectation for the configured BM1366 ticket mask, not silicon readback proof.

/// Eight ticket-mask bits plus the software model's implicit 32-bit baseline.
pub const PROFILE: &str = "bm1366-ticket-256-leading-zero-40-v1";

/// Tests SHA256d output bytes as a little-endian integer: H < 2^216.
/// This is not the stricter Bitcoin-difficulty-256 target comparison.
#[must_use]
pub fn matches(sha256d_bytes: &[u8; 32]) -> bool {
    sha256d_bytes[27..].iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn exact_boundary_accepts_last_integer_below_two_to_216() {
        // Arrange
        let mut last = [0xff; 32];
        last[27..].fill(0);
        let mut boundary = [0; 32];
        boundary[27] = 1;
        // Act / Assert
        assert!(super::matches(&last));
        assert!(!super::matches(&boundary));
    }

    #[test]
    fn every_high_filter_bit_must_be_zero() {
        for bit in 216..256 {
            // Arrange
            let mut hash = [0; 32];
            hash[bit / 8] = 1 << (bit % 8);
            // Act / Assert
            assert!(!super::matches(&hash));
        }
    }

    #[test]
    fn expected_mask_accepts_values_above_nominal_bitcoin_difficulty_256_target() {
        // Arrange: D1/256 = 65535*2^200. Adding one fails that nominal target,
        // but remains below 2^216 and passes the software ticket-filter model.
        let mut just_above_nominal = [0; 32];
        just_above_nominal[25] = 0xff;
        just_above_nominal[26] = 0xff;
        just_above_nominal[0] = 1;
        // Act / Assert
        assert!(super::matches(&just_above_nominal));
        assert_eq!(
            super::super::mining_ready::difficulty_mask_value(256.0),
            [0, 0, 0, 0xff]
        );
    }
}
