//! Pinned shared-I2C transfer policy.
//!
//! Reference: `reference/esp-miner/main/i2c_bitaxe.c`

pub(crate) const I2C_TRANSACTION_TIMEOUT_MS: u64 = 500;
pub(crate) const I2C_RETRY_COUNT: usize = 3;
pub(crate) const I2C_RETRY_DELAY_MS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeI2cBudgetOutcome {
    Ready,
    Recovered,
    DriverFailed,
    BudgetExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeI2cBudget {
    deadline_ms: u64,
    failure_count: usize,
    outcome: RuntimeI2cBudgetOutcome,
}

impl RuntimeI2cBudget {
    pub(crate) const fn new(deadline_ms: u64) -> Self {
        Self {
            deadline_ms,
            failure_count: 0,
            outcome: RuntimeI2cBudgetOutcome::Ready,
        }
    }

    pub(crate) const fn outcome(self) -> RuntimeI2cBudgetOutcome {
        self.outcome
    }

    const fn remaining_ms(self, now_ms: u64) -> u64 {
        self.deadline_ms.saturating_sub(now_ms)
    }

    fn note_failure(&mut self) {
        self.failure_count = self.failure_count.saturating_add(1);
    }

    fn note_success(&mut self) {
        self.outcome = if self.failure_count == 0 {
            RuntimeI2cBudgetOutcome::Ready
        } else {
            RuntimeI2cBudgetOutcome::Recovered
        };
    }

    fn note_driver_failure(&mut self) {
        self.outcome = RuntimeI2cBudgetOutcome::DriverFailed;
    }

    fn note_exhausted(&mut self) {
        self.outcome = RuntimeI2cBudgetOutcome::BudgetExhausted;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RuntimeI2cTransferError<E> {
    Driver(E),
    BudgetExhausted,
}

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

/// Runs a runtime transfer without crossing the sensor producer deadline.
///
/// Startup keeps the complete upstream retry contract. Runtime sensor, display,
/// and actuation work shares this absolute deadline so none of those operations
/// can hide the producer beyond active-safety freshness.
pub(crate) fn retry_runtime_transfer<T, E>(
    budget: &mut RuntimeI2cBudget,
    mut now_ms: impl FnMut() -> u64,
    mut transfer: impl FnMut(u64) -> Result<T, E>,
    mut delay_ms: impl FnMut(u32),
) -> Result<T, RuntimeI2cTransferError<E>> {
    let mut attempt = 0;
    loop {
        let remaining_ms = budget.remaining_ms(now_ms());
        if remaining_ms == 0 {
            budget.note_exhausted();
            return Err(RuntimeI2cTransferError::BudgetExhausted);
        }
        attempt += 1;
        let timeout_ms = remaining_ms.min(I2C_TRANSACTION_TIMEOUT_MS);
        match transfer(timeout_ms) {
            Ok(value) => {
                budget.note_success();
                return Ok(value);
            }
            Err(error) => {
                budget.note_failure();
                if attempt == I2C_RETRY_COUNT {
                    budget.note_driver_failure();
                    return Err(RuntimeI2cTransferError::Driver(error));
                }
                let remaining_ms = budget.remaining_ms(now_ms());
                if remaining_ms <= u64::from(I2C_RETRY_DELAY_MS) {
                    budget.note_exhausted();
                    return Err(RuntimeI2cTransferError::BudgetExhausted);
                }
                delay_ms(I2C_RETRY_DELAY_MS);
            }
        }
    }
}
