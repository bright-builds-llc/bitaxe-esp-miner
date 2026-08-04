#[path = "safety_adapter/i2c_retry.rs"]
mod i2c_retry;

use i2c_retry::{
    retry_transfer, I2C_RETRY_COUNT, I2C_RETRY_DELAY_MS, I2C_TRANSACTION_TIMEOUT_MS,
};

#[test]
fn constants_match_the_pinned_reference_contract() {
    // Arrange / Act / Assert
    assert_eq!(I2C_TRANSACTION_TIMEOUT_MS, 500);
    assert_eq!(I2C_RETRY_COUNT, 3);
    assert_eq!(I2C_RETRY_DELAY_MS, 10);
}

#[test]
fn first_attempt_success_returns_without_delay() {
    // Arrange
    let mut attempts = 0;
    let mut delays = Vec::new();

    // Act
    let result = retry_transfer(
        || {
            attempts += 1;
            Ok::<_, &'static str>("ready")
        },
        |delay| delays.push(delay),
    );

    // Assert
    assert_eq!(result, Ok("ready"));
    assert_eq!(attempts, 1);
    assert!(delays.is_empty());
}

#[test]
fn eventual_success_delays_once_per_preceding_failure() {
    // Arrange
    let mut attempts = 0;
    let mut delays = Vec::new();

    // Act
    let result = retry_transfer(
        || {
            attempts += 1;
            if attempts < I2C_RETRY_COUNT {
                return Err("transient");
            }
            Ok("ready")
        },
        |delay| delays.push(delay),
    );

    // Assert
    assert_eq!(result, Ok("ready"));
    assert_eq!(attempts, I2C_RETRY_COUNT);
    assert_eq!(delays, vec![I2C_RETRY_DELAY_MS; I2C_RETRY_COUNT - 1]);
}

#[test]
fn terminal_failure_preserves_the_final_error_and_terminal_delay() {
    // Arrange
    let errors = ["first", "second", "terminal"];
    let mut attempts = 0;
    let mut delays = Vec::new();

    // Act
    let result = retry_transfer(
        || {
            let error = errors[attempts];
            attempts += 1;
            Err::<(), _>(error)
        },
        |delay| delays.push(delay),
    );

    // Assert
    assert_eq!(result, Err("terminal"));
    assert_eq!(attempts, I2C_RETRY_COUNT);
    assert_eq!(delays, vec![I2C_RETRY_DELAY_MS; I2C_RETRY_COUNT]);
}
