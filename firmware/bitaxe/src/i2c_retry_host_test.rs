#[path = "safety_adapter/i2c_retry.rs"]
mod i2c_retry;

use i2c_retry::{
    retry_runtime_transfer, retry_transfer, RuntimeI2cBudget, RuntimeI2cBudgetOutcome,
    RuntimeI2cTransferError, I2C_RETRY_COUNT, I2C_RETRY_DELAY_MS, I2C_TRANSACTION_TIMEOUT_MS,
};

const SENSOR_SWEEP_CADENCE_MS: u64 = 500;
const SENSOR_STALE_AFTER_MS: u64 = 1_000;
const SENSOR_PUBLISH_HEADROOM_MS: u64 = 100;

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

#[test]
fn periodic_sensor_retry_cannot_hide_the_producer_past_active_freshness() {
    // Arrange
    use std::cell::Cell;

    let virtual_now_ms = Cell::new(SENSOR_SWEEP_CADENCE_MS);
    let previous_observation_at_ms = 0;
    let mut budget = RuntimeI2cBudget::new(
        previous_observation_at_ms + SENSOR_STALE_AFTER_MS - SENSOR_PUBLISH_HEADROOM_MS,
    );

    // Act
    let result = retry_runtime_transfer(
        &mut budget,
        || virtual_now_ms.get(),
        |timeout_ms| {
            virtual_now_ms.set(virtual_now_ms.get() + timeout_ms);
            Err::<(), _>("timed_out")
        },
        |delay_ms| virtual_now_ms.set(virtual_now_ms.get() + u64::from(delay_ms)),
    );

    // Assert
    assert_eq!(result, Err(RuntimeI2cTransferError::BudgetExhausted));
    assert_eq!(budget.outcome(), RuntimeI2cBudgetOutcome::BudgetExhausted);
    assert!(
        virtual_now_ms.get() <= previous_observation_at_ms + SENSOR_STALE_AFTER_MS,
        "periodic sensor retry kept the complete-sweep producer blocked until {} ms",
        virtual_now_ms.get()
    );
}

#[test]
fn periodic_sensor_retry_recovers_before_its_deadline() {
    // Arrange
    use std::cell::Cell;

    let virtual_now_ms = Cell::new(500_u64);
    let attempt = Cell::new(0_u8);
    let mut budget = RuntimeI2cBudget::new(900);

    // Act
    let result = retry_runtime_transfer(
        &mut budget,
        || virtual_now_ms.get(),
        |timeout_ms| {
            attempt.set(attempt.get().saturating_add(1));
            if attempt.get() == 1 {
                virtual_now_ms.set(virtual_now_ms.get() + timeout_ms.min(50));
                return Err("transient");
            }
            Ok("ready")
        },
        |delay_ms| virtual_now_ms.set(virtual_now_ms.get() + u64::from(delay_ms)),
    );

    // Assert
    assert_eq!(result, Ok("ready"));
    assert_eq!(attempt.get(), 2);
    assert_eq!(budget.outcome(), RuntimeI2cBudgetOutcome::Recovered);
    assert!(virtual_now_ms.get() < 900);
}

#[test]
fn multiple_runtime_transfers_share_one_absolute_deadline() {
    // Arrange
    use std::cell::Cell;

    let virtual_now_ms = Cell::new(500_u64);
    let mut budget = RuntimeI2cBudget::new(900);

    // Act
    let first = retry_runtime_transfer(
        &mut budget,
        || virtual_now_ms.get(),
        |timeout_ms| {
            virtual_now_ms.set(virtual_now_ms.get() + timeout_ms.min(300));
            Ok::<_, &'static str>(())
        },
        |_| {},
    );
    let second = retry_runtime_transfer(
        &mut budget,
        || virtual_now_ms.get(),
        |timeout_ms| {
            virtual_now_ms.set(virtual_now_ms.get() + timeout_ms);
            Err::<(), _>("timed_out")
        },
        |_| {},
    );

    // Assert
    assert_eq!(first, Ok(()));
    assert_eq!(second, Err(RuntimeI2cTransferError::BudgetExhausted));
    assert_eq!(virtual_now_ms.get(), 900);
    assert_eq!(budget.outcome(), RuntimeI2cBudgetOutcome::BudgetExhausted);
}

#[test]
fn quick_terminal_runtime_failure_preserves_driver_category() {
    // Arrange
    use std::cell::Cell;

    let virtual_now_ms = Cell::new(500_u64);
    let mut budget = RuntimeI2cBudget::new(900);

    // Act
    let result = retry_runtime_transfer(
        &mut budget,
        || virtual_now_ms.get(),
        |_| Err::<(), _>("driver"),
        |delay_ms| virtual_now_ms.set(virtual_now_ms.get() + u64::from(delay_ms)),
    );

    // Assert
    assert_eq!(result, Err(RuntimeI2cTransferError::Driver("driver")));
    assert_eq!(budget.outcome(), RuntimeI2cBudgetOutcome::DriverFailed);
    assert_eq!(virtual_now_ms.get(), 520);
}
