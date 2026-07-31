//! Bounded request-queue mechanics shared by synchronous and observed effects.

use std::sync::mpsc::{SyncSender, TrySendError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnqueueOutcome {
    Queued,
    Full,
    Disconnected,
}

pub(super) struct ActuationEnvelope<C, R> {
    command: C,
    reply_sender: SyncSender<R>,
}

impl<C, R> ActuationEnvelope<C, R> {
    pub(super) fn into_parts(self) -> (C, SyncSender<R>) {
        (self.command, self.reply_sender)
    }
}

pub(super) fn enqueue<C, R>(
    sender: &SyncSender<ActuationEnvelope<C, R>>,
    command: C,
    reply_sender: SyncSender<R>,
) -> EnqueueOutcome {
    let envelope = ActuationEnvelope {
        command,
        reply_sender,
    };
    match sender.try_send(envelope) {
        Ok(()) => EnqueueOutcome::Queued,
        Err(TrySendError::Full(_)) => EnqueueOutcome::Full,
        Err(TrySendError::Disconnected(_)) => EnqueueOutcome::Disconnected,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn deferred_effect_enqueue_does_not_wait_for_its_reply() {
        // Arrange
        let (sender, receiver) = mpsc::sync_channel(1);
        let (reply_sender, reply_receiver) = mpsc::sync_channel(1);

        // Act
        let outcome = enqueue(&sender, 7_u8, reply_sender);
        let (command, queued_reply_sender) = receiver
            .try_recv()
            .expect("queued deferred effect must be available")
            .into_parts();
        queued_reply_sender
            .try_send(11_u8)
            .expect("deferred reply channel must remain connected");

        // Assert
        assert_eq!(outcome, EnqueueOutcome::Queued);
        assert_eq!(command, 7);
        assert_eq!(reply_receiver.try_recv(), Ok(11));
    }

    #[test]
    fn synchronous_effect_enqueue_preserves_its_reply_channel() {
        // Arrange
        let (sender, receiver) = mpsc::sync_channel(1);
        let (reply_sender, reply_receiver) = mpsc::sync_channel(1);

        // Act
        let outcome = enqueue(&sender, 7_u8, reply_sender);
        let (_, queued_reply_sender) = receiver
            .try_recv()
            .expect("queued synchronous effect must be available")
            .into_parts();
        queued_reply_sender
            .try_send(11_u8)
            .expect("reply channel must remain connected");

        // Assert
        assert_eq!(outcome, EnqueueOutcome::Queued);
        assert_eq!(reply_receiver.try_recv(), Ok(11));
    }

    #[test]
    fn bounded_queue_reports_backpressure_without_blocking() {
        // Arrange
        let (sender, _receiver) = mpsc::sync_channel(1);
        let (first_reply_sender, _first_reply_receiver) = mpsc::sync_channel::<u8>(1);
        let (second_reply_sender, _second_reply_receiver) = mpsc::sync_channel::<u8>(1);
        let first = enqueue(&sender, 7_u8, first_reply_sender);

        // Act
        let second = enqueue(&sender, 8_u8, second_reply_sender);

        // Assert
        assert_eq!(first, EnqueueOutcome::Queued);
        assert_eq!(second, EnqueueOutcome::Full);
    }
}
