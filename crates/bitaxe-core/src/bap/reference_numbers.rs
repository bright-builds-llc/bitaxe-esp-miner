//! The reference parses BAP setting values with newlib `atoi` and `atof`, then
//! casts some to `uint16_t` (`bap_handlers.c:BAP_handle_settings`). These
//! functions reproduce that on the 32-bit ESP32-S3, where `long` and `int` are
//! 32 bits: leading C whitespace is skipped, the longest numeric prefix is
//! read, garbage reads as 0, and `atoi` overflow saturates as `strtol` does.
//!
//! Divergence: hexadecimal floats (`0x1p3`) read as 0 here, where `strtod`
//! would accept them.

/// newlib `atoi`: `(int)strtol(value, NULL, 10)` with 32-bit `long`.
#[must_use]
pub fn reference_atoi(value: &str) -> i32 {
    let bytes = trim_c_space(value).as_bytes();
    let (negative, digits) = match bytes.first() {
        Some(b'-') => (true, &bytes[1..]),
        Some(b'+') => (false, &bytes[1..]),
        _ => (false, bytes),
    };
    let mut magnitude: i64 = 0;
    for digit in digits.iter().take_while(|byte| byte.is_ascii_digit()) {
        magnitude = (magnitude * 10 + i64::from(digit - b'0')).min(i64::from(i32::MAX) + 1);
    }
    let signed = if negative { -magnitude } else { magnitude };
    i32::try_from(signed.clamp(i64::from(i32::MIN), i64::from(i32::MAX))).unwrap_or_default()
}

/// The reference's `(uint16_t)atoi(value)`: the `int` result modulo 2^16.
#[must_use]
pub fn reference_atoi_u16(value: &str) -> u16 {
    reference_atoi(value) as u16
}

/// newlib `atof` (`strtod`): the longest decimal, `inf` or `nan` prefix, else 0.
#[must_use]
pub fn reference_atof(value: &str) -> f64 {
    let text = trim_c_space(value);
    let bytes = text.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let lower = text[end..].to_ascii_lowercase();
    if lower.starts_with("inf") || lower.starts_with("nan") {
        let magnitude = if lower.starts_with("inf") {
            f64::INFINITY
        } else {
            f64::NAN
        };
        return if bytes.first() == Some(&b'-') {
            -magnitude
        } else {
            magnitude
        };
    }
    let integer = count_digits(&bytes[end..]);
    end += integer;
    let mut fraction = 0;
    if bytes.get(end) == Some(&b'.') {
        fraction = count_digits(&bytes[end + 1..]);
        end += 1 + fraction;
    }
    if integer + fraction == 0 {
        return 0.0;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let sign = usize::from(matches!(bytes.get(end + 1), Some(b'+' | b'-')));
        let exponent = count_digits(&bytes[end + 1 + sign..]);
        if exponent > 0 {
            end += 1 + sign + exponent;
        }
    }
    text[..end].parse::<f64>().unwrap_or(0.0)
}

fn trim_c_space(value: &str) -> &str {
    value.trim_start_matches([' ', '\t', '\n', '\u{0b}', '\u{0c}', '\r'])
}

fn count_digits(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atoi_reads_the_leading_number_and_stops_at_garbage() {
        assert_eq!(
            [" 42", "+7x", "-12abc", "abc", "", "1e3"].map(reference_atoi),
            [42, 7, -12, 0, 0, 1]
        );
    }

    #[test]
    fn atoi_boundaries_match_newlib() {
        assert_eq!(
            ["2147483647", "2147483648", "\u{0b}\u{0c}9", "+-5", "-0"].map(reference_atoi),
            [i32::MAX, i32::MAX, 9, 0, 0]
        );
    }

    #[test]
    fn atof_reads_a_fraction_without_digits_and_an_exponent() {
        assert_eq!(
            ["5.e3", "5.", "-.5e1"].map(reference_atof),
            [5000.0, 5.0, -5.0]
        );
    }

    #[test]
    fn atoi_saturates_like_32_bit_strtol() {
        assert_eq!(
            ["99999999999", "-99999999999"].map(reference_atoi),
            [i32::MAX, i32::MIN]
        );
    }

    #[test]
    fn a_uint16_cast_wraps_modulo_65536() {
        // Arrange: 66236 - 65536 = 700, an in-range reference voltage.
        let values = ["66236", "-1", "1200"];
        // Act
        let parsed = values.map(reference_atoi_u16);
        // Assert
        assert_eq!(parsed, [700, 65_535, 1_200]);
    }

    #[test]
    fn atof_reads_the_longest_decimal_prefix() {
        assert_eq!(
            [
                "485",
                " 485.5MHz",
                "5e2",
                "5e",
                ".5",
                "-1.25",
                "x1",
                "1.2.3"
            ]
            .map(reference_atof),
            [485.0, 485.5, 500.0, 5.0, 0.5, -1.25, 0.0, 1.2]
        );
    }

    #[test]
    fn atof_reads_infinity_and_nan_prefixes() {
        // Arrange / Act
        let infinite = reference_atof("-Infinity");
        let not_a_number = reference_atof("nan");
        // Assert
        assert!(infinite.is_infinite() && infinite.is_sign_negative());
        assert!(not_a_number.is_nan());
    }
}
