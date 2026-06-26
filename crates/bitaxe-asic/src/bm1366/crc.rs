//! BM1366 CRC helpers.
//!
//! Reference breadcrumb: `reference/esp-miner/components/asic/crc.c`, parity
//! checklist row `ASIC-006`. The CRC16-FALSE function is computed bitwise to
//! avoid copying the upstream lookup table into MIT source.

const CRC5_INITIAL: u8 = 0x1f;
const CRC16_FALSE_INITIAL: u16 = 0xffff;
const CRC16_POLYNOMIAL: u16 = 0x1021;

pub fn crc5(data: &[u8]) -> u8 {
    let mut crc = CRC5_INITIAL;

    for byte in data {
        let mut shifted_byte = *byte;
        for _ in 0..8 {
            let bit = (shifted_byte >> 7) & 1;
            shifted_byte <<= 1;

            let new_bit = ((crc >> 4) ^ bit) & 1;
            crc = ((crc << 1) | new_bit) ^ (new_bit << 2);
            crc &= CRC5_INITIAL;
        }
    }

    crc
}

pub fn crc16_false(data: &[u8]) -> u16 {
    let mut crc = CRC16_FALSE_INITIAL;

    for byte in data {
        crc ^= u16::from(*byte) << 8;

        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ CRC16_POLYNOMIAL;
            } else {
                crc <<= 1;
            }
        }
    }

    crc
}
