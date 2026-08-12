use std::sync::mpsc::TrySendError;

use bitaxe_stratum::v1::production_session::{
    ProductionSessionNotificationOutcome, ProductionSessionWakeup,
};

use super::pending_observation::PendingObservationWake;
use super::{OwnerInboxMessage, NOTIFICATIONS};

static OBSERVATIONS_CHANGED_PENDING: PendingObservationWake = PendingObservationWake::new();

pub(super) fn take_pending_observations_changed() -> bool {
    OBSERVATIONS_CHANGED_PENDING.take()
}

/// Non-blockingly wakes the owner with a category-only notification.
#[must_use]
pub fn notify(wakeup: ProductionSessionWakeup) -> ProductionSessionNotificationOutcome {
    let Some(sender) = NOTIFICATIONS.get() else {
        return ProductionSessionNotificationOutcome::OwnerUnavailable;
    };
    if wakeup == ProductionSessionWakeup::ObservationsChanged {
        OBSERVATIONS_CHANGED_PENDING.mark();
    }

    match sender.try_send(OwnerInboxMessage::Wake(wakeup)) {
        Ok(()) => ProductionSessionNotificationOutcome::Queued,
        Err(TrySendError::Full(_)) => ProductionSessionNotificationOutcome::Coalesced,
        Err(TrySendError::Disconnected(_)) => {
            ProductionSessionNotificationOutcome::OwnerUnavailable
        }
    }
}
