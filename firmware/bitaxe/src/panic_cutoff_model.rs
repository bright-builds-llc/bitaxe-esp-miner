//! Pinned Ultra 205 output proof shared by native admission and host regressions.
pub const ENABLE_MASK: u32 = 1 << 10;
pub const RESET_MASK: u32 = 1 << 1;
pub const OUTPUT_MASK: u32 = ENABLE_MASK | RESET_MASK;

/// Safe latches are unconditional; this never changes pin direction or mux.
/// The configured flag is admission evidence, never emergency-write authority.
#[inline(always)]
pub fn cut_latches(mut set: impl FnMut(u32), mut clear: impl FnMut(u32)) {
    set(ENABLE_MASK);
    clear(RESET_MASK);
}

pub const fn outputs_disabled(configured: u32, enabled: u32, output: u32) -> bool {
    configured == OUTPUT_MASK
        && enabled & OUTPUT_MASK == OUTPUT_MASK
        && output & OUTPUT_MASK == ENABLE_MASK
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_both_configured_enabled_outputs() {
        // Arrange / Act / Assert
        for configured in [0, ENABLE_MASK, RESET_MASK] {
            assert!(!outputs_disabled(configured, OUTPUT_MASK, ENABLE_MASK));
        }
        for enabled in [0, ENABLE_MASK, RESET_MASK] {
            assert!(!outputs_disabled(OUTPUT_MASK, enabled, ENABLE_MASK));
        }
    }
    #[test]
    fn rejects_energized_or_released_reset_latches() {
        // Arrange / Act / Assert
        for output in [0, RESET_MASK, OUTPUT_MASK] {
            assert!(!outputs_disabled(OUTPUT_MASK, OUTPUT_MASK, output));
        }
    }
    #[test]
    fn ignores_unrelated_outputs() {
        // Arrange / Act / Assert
        assert!(outputs_disabled(
            OUTPUT_MASK,
            u32::MAX,
            ENABLE_MASK | (1 << 4)
        ));
    }
}

#[cfg(test)]
mod emergency_tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn emergency_latches_are_safe_even_without_configuration_and_preserve_other_pins() {
        // Arrange
        for configured in [0, ENABLE_MASK, RESET_MASK, OUTPUT_MASK, u32::MAX] {
            for original in [0, u32::MAX, RESET_MASK, 0xa5a5a5a5] {
                let latch = Cell::new(original);
                let configuration = Cell::new(configured);
                // Act
                cut_latches(
                    |mask| latch.set(latch.get() | mask),
                    |mask| latch.set(latch.get() & !mask),
                );
                // Assert
                assert_eq!(latch.get() & OUTPUT_MASK, ENABLE_MASK);
                assert_eq!(latch.get() & !OUTPUT_MASK, original & !OUTPUT_MASK);
                assert_eq!(configuration.get(), configured);
            }
        }
    }

    #[test]
    fn power_cut_precedes_reset_assertion() {
        // Arrange
        let calls = std::cell::RefCell::new(Vec::new());
        // Act
        cut_latches(
            |mask| calls.borrow_mut().push((true, mask)),
            |mask| calls.borrow_mut().push((false, mask)),
        );
        // Assert
        assert_eq!(
            calls.into_inner(),
            [(true, ENABLE_MASK), (false, RESET_MASK)]
        );
    }
}
