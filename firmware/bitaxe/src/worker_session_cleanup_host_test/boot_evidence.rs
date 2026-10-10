//! Synthetic boot evidence: the reset category the real observer would hold, per test thread.
use bitaxe_api::boot_identity::ResetReasonCategory;
use std::cell::Cell;

thread_local! {
    static RESET_REASON: Cell<Option<ResetReasonCategory>> = const { Cell::new(None) };
}

pub(crate) fn maybe_reset_reason_category() -> Option<ResetReasonCategory> {
    RESET_REASON.with(Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bwg_worker_session::ProductionWorkerSession;
    use bitaxe_worker_control::{BootResetCause, WorkerSession};

    fn reported(maybe_reason: Option<ResetReasonCategory>) -> BootResetCause {
        RESET_REASON.with(|cell| cell.set(maybe_reason));
        ProductionWorkerSession::default().boot_reset_cause()
    }

    #[test]
    fn every_boot_reset_category_reaches_the_boot_review_with_its_label() {
        // Arrange
        let categories = [
            ResetReasonCategory::PowerOn,
            ResetReasonCategory::SoftwareCpu,
            ResetReasonCategory::Watchdog,
            ResetReasonCategory::Panic,
            ResetReasonCategory::Brownout,
            ResetReasonCategory::Other,
        ];

        // Act
        let labels = categories.map(|category| reported(Some(category)).label());

        // Assert
        assert_eq!(labels, categories.map(ResetReasonCategory::label));
    }

    #[test]
    fn an_uninitialized_boot_reset_category_reports_other() {
        // Arrange
        let maybe_reason = None;

        // Act
        let cause = reported(maybe_reason);

        // Assert
        assert_eq!(cause, BootResetCause::Other);
    }
}
