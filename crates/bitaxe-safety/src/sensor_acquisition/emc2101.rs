use crate::thermal::{MAX_PLAUSIBLE_TEMP_C, MIN_PLAUSIBLE_TEMP_C};

use super::SensorValidationError;

pub const EMC2101_TACHOMETER_NUMERATOR: u32 = 5_400_000;
pub const EMC2101_TACHOMETER_NO_SPIN_RPM: u32 = 82;
pub const EMC2101_TEMP_FAULT_OPEN_CIRCUIT: u16 = 0x03f8;
pub const EMC2101_TEMP_FAULT_SHORT: u16 = 0x03ff;
pub const ULTRA205_EMC2101_TEMP_OFFSET_C: f64 = 5.0;

pub fn decode_emc2101_external_temperature(bytes: [u8; 2]) -> Result<f64, SensorValidationError> {
    let raw = u16::from_be_bytes(bytes) >> 5;
    if raw == EMC2101_TEMP_FAULT_OPEN_CIRCUIT {
        return Err(SensorValidationError::OpenCircuit);
    }
    if raw == EMC2101_TEMP_FAULT_SHORT {
        return Err(SensorValidationError::ShortCircuit);
    }

    let temperature = f64::from(sign_extend_11_bit(raw)) / 8.0;
    validate_temperature(temperature)
}

pub fn decode_emc2101_internal_temperature(byte: u8) -> Result<f64, SensorValidationError> {
    validate_temperature(f64::from(byte as i8))
}

pub fn apply_ultra205_emc2101_temperature_offset(
    temperature_celsius: f64,
) -> Result<f64, SensorValidationError> {
    validate_temperature(temperature_celsius + ULTRA205_EMC2101_TEMP_OFFSET_C)
}

pub fn decode_emc2101_tachometer(bytes: [u8; 2]) -> Result<u16, SensorValidationError> {
    let raw = u16::from_le_bytes(bytes);
    if raw == 0 {
        return Ok(0);
    }

    let rpm = EMC2101_TACHOMETER_NUMERATOR / u32::from(raw);
    if rpm == EMC2101_TACHOMETER_NO_SPIN_RPM {
        return Ok(0);
    }

    u16::try_from(rpm).map_err(|_| SensorValidationError::TachometerOverflow)
}

fn validate_temperature(temperature: f64) -> Result<f64, SensorValidationError> {
    if !temperature.is_finite()
        || !(MIN_PLAUSIBLE_TEMP_C..=MAX_PLAUSIBLE_TEMP_C).contains(&temperature)
    {
        return Err(SensorValidationError::TemperatureOutOfRange);
    }
    Ok(temperature)
}

fn sign_extend_11_bit(raw: u16) -> i16 {
    if raw & 0x0400 == 0 {
        return raw as i16;
    }
    (raw | 0xf800) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_temperature_decodes_positive_and_negative_values() {
        // Arrange
        let positive = ((60_i16 * 8) as u16) << 5;
        let negative_11_bit = ((-10_i16 * 8) as u16) & 0x07ff;
        let negative = negative_11_bit << 5;

        // Act
        let positive = decode_emc2101_external_temperature(positive.to_be_bytes());
        let negative = decode_emc2101_external_temperature(negative.to_be_bytes());

        // Assert
        assert_eq!(positive, Ok(60.0));
        assert_eq!(negative, Ok(-10.0));
    }

    #[test]
    fn external_temperature_rejects_open_and_short_faults() {
        // Arrange
        let open = (EMC2101_TEMP_FAULT_OPEN_CIRCUIT << 5).to_be_bytes();
        let short = (EMC2101_TEMP_FAULT_SHORT << 5).to_be_bytes();

        // Act / Assert
        assert_eq!(
            decode_emc2101_external_temperature(open),
            Err(SensorValidationError::OpenCircuit)
        );
        assert_eq!(
            decode_emc2101_external_temperature(short),
            Err(SensorValidationError::ShortCircuit)
        );
    }

    #[test]
    fn ultra205_temperature_applies_reference_offset() {
        // Arrange
        let raw_temperature = 45.0;

        // Act
        let adjusted = apply_ultra205_emc2101_temperature_offset(raw_temperature);

        // Assert
        assert_eq!(adjusted, Ok(50.0));
    }

    #[test]
    fn ultra205_temperature_rejects_invalid_adjusted_value() {
        // Arrange
        let raw_temperature = MAX_PLAUSIBLE_TEMP_C;

        // Act
        let adjusted = apply_ultra205_emc2101_temperature_offset(raw_temperature);

        // Assert
        assert_eq!(adjusted, Err(SensorValidationError::TemperatureOutOfRange));
    }

    #[test]
    fn tachometer_handles_zero_sentinel_and_overflow() {
        // Arrange
        let sentinel_raw = u16::MAX;
        let overflow_raw = 1_u16;

        // Act / Assert
        assert_eq!(decode_emc2101_tachometer([0, 0]), Ok(0));
        assert_eq!(decode_emc2101_tachometer(sentinel_raw.to_le_bytes()), Ok(0));
        assert_eq!(
            decode_emc2101_tachometer(overflow_raw.to_le_bytes()),
            Err(SensorValidationError::TachometerOverflow)
        );
    }
}
