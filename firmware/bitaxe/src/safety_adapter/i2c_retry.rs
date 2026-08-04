//! Pinned shared-I2C transfer policy.
//!
//! Reference: `reference/esp-miner/main/i2c_bitaxe.c`

pub(crate) const I2C_TRANSACTION_TIMEOUT_MS: u64 = 500;
pub(crate) const I2C_RETRY_COUNT: usize = 3;
pub(crate) const I2C_RETRY_DELAY_MS: u32 = 10;

/// Runs one transfer with the exact upstream attempt and delay contract.
///
/// Upstream delays after every failed attempt, including the terminal failure,
/// before it returns the final driver error.
pub(crate) fn retry_transfer<T, E>(
    mut transfer: impl FnMut() -> Result<T, E>,
    mut delay_ms: impl FnMut(u32),
) -> Result<T, E> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        match transfer() {
            Ok(value) => return Ok(value),
            Err(error) => {
                delay_ms(I2C_RETRY_DELAY_MS);
                if attempt == I2C_RETRY_COUNT {
                    return Err(error);
                }
            }
        }
    }
}
