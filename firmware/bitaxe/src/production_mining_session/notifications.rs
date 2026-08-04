use std::sync::mpsc::TrySendError;

use bitaxe_stratum::v1::production_session::{
    ProductionSessionNotificationOutcome, ProductionSessionWakeup,
};

use super::{OwnerInboxMessage, NOTIFICATIONS};

/// Non-blockingly wakes the owner with a category-only notification.
#[must_use]
pub fn notify(wakeup: ProductionSessionWakeup) -> ProductionSessionNotificationOutcome {
    let Some(sender) = NOTIFICATIONS.get() else {
        return ProductionSessionNotificationOutcome::OwnerUnavailable;
    };

    match sender.try_send(OwnerInboxMessage::Wake(wakeup)) {
        Ok(()) => ProductionSessionNotificationOutcome::Queued,
        Err(TrySendError::Full(_)) => ProductionSessionNotificationOutcome::Coalesced,
        Err(TrySendError::Disconnected(_)) => {
            ProductionSessionNotificationOutcome::OwnerUnavailable
        }
    }
}
