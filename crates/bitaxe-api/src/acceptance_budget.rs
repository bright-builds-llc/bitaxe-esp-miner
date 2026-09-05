//! Durable, non-refundable reservations for the approved three-window acceptance run.
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const TOTAL_ACTIVE_MILLISECONDS: u32 = 240_000;
const WINDOWS: [u32; 3] = [180_000, 30_000, 30_000];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptanceBudget {
    campaign_id: String,
    reserved: u8,
    completed: u8,
    charged_ms: u32,
    maybe_running_window: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("acceptance campaign admission rejected")]
pub struct BudgetRejected;

impl AcceptanceBudget {
    pub fn new(id: &str) -> Result<Self, BudgetRejected> {
        if id.len() != 22
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        {
            return Err(BudgetRejected);
        }
        Ok(Self {
            campaign_id: id.to_owned(),
            reserved: 0,
            completed: 0,
            charged_ms: 0,
            maybe_running_window: None,
        })
    }

    pub fn validate(&self) -> Result<(), BudgetRejected> {
        Self::new(&self.campaign_id)?;
        let expected: u32 = WINDOWS
            .iter()
            .enumerate()
            .filter(|(i, _)| self.reserved & (1 << i) != 0)
            .map(|(_, v)| *v)
            .sum();
        if self.reserved & !7 != 0
            || self.completed & !self.reserved != 0
            || self.charged_ms != expected
            || self.charged_ms > TOTAL_ACTIVE_MILLISECONDS
            || self.maybe_running_window.is_some_and(|i| {
                i > 2 || self.reserved & (1 << i) == 0 || self.completed & (1 << i) != 0
            })
        {
            return Err(BudgetRejected);
        }
        Ok(())
    }

    /// Charges the complete window before any hardware work; no refund is possible.
    pub fn reserve(&self, id: &str, window: u8, maximum_ms: u64) -> Result<Self, BudgetRejected> {
        self.validate()?;
        let Some(duration) = WINDOWS.get(usize::from(window)) else {
            return Err(BudgetRejected);
        };
        if id != self.campaign_id
            || maximum_ms != u64::from(*duration)
            || self.maybe_running_window.is_some()
            || self.reserved & (1 << window) != 0
        {
            return Err(BudgetRejected);
        }
        let mut next = self.clone();
        next.reserved |= 1 << window;
        next.charged_ms += duration;
        next.maybe_running_window = Some(window);
        next.validate()?;
        Ok(next)
    }

    /// Called only after qualified safe stop or a qualified boot-safe baseline.
    pub fn finish(&self) -> Result<Self, BudgetRejected> {
        self.validate()?;
        let mut next = self.clone();
        if let Some(window) = next.maybe_running_window.take() {
            next.completed |= 1 << window;
        }
        Ok(next)
    }

    pub fn complete(&self) -> bool {
        self.validate().is_ok() && self.completed == 7
    }
    pub const fn charged_milliseconds(&self) -> u32 {
        self.charged_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: &str = "aaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn reboot_and_retry_cannot_refund_or_repeat_a_window() {
        // Arrange
        let ledger = AcceptanceBudget::new(ID)
            .expect("id")
            .reserve(ID, 0, 180000)
            .expect("reserve");
        let encoded = serde_json::to_vec(&ledger).expect("encode");
        let rebooted: AcceptanceBudget = serde_json::from_slice(&encoded).expect("decode");
        // Act
        let stopped = rebooted.finish().expect("qualified boot baseline");
        // Assert
        assert_eq!(stopped.charged_milliseconds(), 180000);
        assert!(stopped.reserve(ID, 0, 180000).is_err());
        assert!(stopped.reserve("bbbbbbbbbbbbbbbbbbbbbb", 1, 30000).is_err());
    }
    #[test]
    fn exact_three_windows_exhaust_240_seconds_only_after_safe_stop() {
        // Arrange
        let mut ledger = AcceptanceBudget::new(ID).expect("id");
        // Act
        for (window, duration) in WINDOWS.iter().enumerate() {
            ledger = ledger
                .reserve(ID, window as u8, u64::from(*duration))
                .expect("reserve");
            assert!(!ledger.complete());
            ledger = ledger.finish().expect("stop");
        }
        // Assert
        assert!(ledger.complete());
        assert_eq!(ledger.charged_milliseconds(), 240000);
        assert!(ledger.reserve(ID, 2, 30000).is_err());
    }
    #[test]
    fn concurrent_window_and_unrecognized_duration_fail_closed() {
        // Arrange
        let ledger = AcceptanceBudget::new(ID).expect("id");
        // Act / Assert
        assert!(ledger.reserve(ID, 0, 180001).is_err());
        let running = ledger.reserve(ID, 0, 180000).expect("reserve");
        assert!(running.reserve(ID, 1, 30000).is_err());
    }
}
