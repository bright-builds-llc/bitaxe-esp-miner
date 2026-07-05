//! Pure UART read accumulation for BM1366 fixed-length frames.
//!
//! Host-testable mirror of firmware `AsicUart::read_accumulate` loop semantics.

use crate::Bm1366ProtocolFault;

use super::result::BM1366_RESULT_FRAME_LEN;

/// Accumulate byte counts from successive partial reads until `target_len` or deadline exhaustion.
///
/// Returns `Ok(())` when `read_sizes` sum to exactly `target_len`.
/// Returns `Err` with total bytes received when sum is less than `target_len` after all chunks.
pub fn validate_accumulated_read_sizes(
    read_sizes: &[usize],
    target_len: usize,
) -> Result<(), Bm1366ProtocolFault> {
    let total: usize = read_sizes.iter().sum();
    if total == target_len {
        return Ok(());
    }

    Err(Bm1366ProtocolFault::InvalidLength {
        expected: target_len,
        actual: total,
    })
}

#[must_use]
pub const fn chip_detect_frame_len() -> usize {
    BM1366_RESULT_FRAME_LEN
}

#[cfg(test)]
mod tests {
    use super::{chip_detect_frame_len, validate_accumulated_read_sizes};
    use crate::Bm1366ProtocolFault;

    #[test]
    fn split_nine_plus_two_read_accumulates_to_eleven_byte_chip_detect_frame() {
        // Arrange
        let read_sizes = [9, 2];
        let target = chip_detect_frame_len();

        // Act
        let outcome = validate_accumulated_read_sizes(&read_sizes, target);

        // Assert
        assert!(outcome.is_ok());
    }

    #[test]
    fn single_nine_byte_read_fails_accumulation_for_eleven_byte_target() {
        // Arrange
        let read_sizes = [9];
        let target = chip_detect_frame_len();

        // Act
        let outcome = validate_accumulated_read_sizes(&read_sizes, target);

        // Assert
        assert_eq!(
            outcome,
            Err(Bm1366ProtocolFault::InvalidLength {
                expected: 11,
                actual: 9,
            })
        );
    }

    #[test]
    fn three_partial_reads_can_complete_eleven_byte_frame() {
        // Arrange
        let read_sizes = [4, 3, 4];
        let target = chip_detect_frame_len();

        // Act
        let outcome = validate_accumulated_read_sizes(&read_sizes, target);

        // Assert
        assert!(outcome.is_ok());
    }
}
