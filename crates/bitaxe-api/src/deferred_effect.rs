//! Worker-owned response-before-effect orchestration.

use std::sync::mpsc::{self, Receiver, SyncSender};

/// Failure to transfer an effect to the process-lifetime worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeferredEffectQueueUnavailable;

/// Queue handle used by request handlers to transfer effect ownership.
pub struct DeferredEffectQueue<Effect> {
    sender: SyncSender<DeferredEffectRequest<Effect>>,
}

impl<Effect> Clone for DeferredEffectQueue<Effect> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Effect> DeferredEffectQueue<Effect> {
    /// Transfers an effect to the worker and waits until ownership is acknowledged.
    pub fn acquire(
        &self,
        effect: Effect,
    ) -> Result<DeferredEffectLease, DeferredEffectQueueUnavailable> {
        let (ownership_sender, ownership_receiver) = mpsc::sync_channel(0);
        let (release_sender, release_receiver) = mpsc::sync_channel(0);
        self.sender
            .send(DeferredEffectRequest {
                effect,
                ownership_sender,
                release_receiver,
            })
            .map_err(|_| DeferredEffectQueueUnavailable)?;
        ownership_receiver
            .recv()
            .map_err(|_| DeferredEffectQueueUnavailable)?;

        Ok(DeferredEffectLease { release_sender })
    }
}

/// Worker-owned effect waiting for the request handler to schedule its response.
pub struct DeferredEffectLease {
    release_sender: SyncSender<()>,
}

impl DeferredEffectLease {
    /// Releases the owned effect only after the public response has been scheduled.
    pub fn release_after_response(self) -> Result<(), DeferredEffectQueueUnavailable> {
        self.release_sender
            .send(())
            .map_err(|_| DeferredEffectQueueUnavailable)
    }
}

struct DeferredEffectRequest<Effect> {
    effect: Effect,
    ownership_sender: SyncSender<()>,
    release_receiver: Receiver<()>,
}

/// Creates a worker-owned queue through an injectable process-lifetime spawn boundary.
pub fn spawn_deferred_effect_worker<Effect, SpawnError>(
    capacity: usize,
    spawn: impl FnOnce(Box<dyn FnOnce() + Send>) -> Result<(), SpawnError>,
    execute: impl Fn(Effect) + Send + 'static,
) -> Result<DeferredEffectQueue<Effect>, SpawnError>
where
    Effect: Send + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(capacity);
    let worker = Box::new(move || run_deferred_effect_worker(receiver, execute));
    spawn(worker)?;
    Ok(DeferredEffectQueue { sender })
}

fn run_deferred_effect_worker<Effect>(
    receiver: Receiver<DeferredEffectRequest<Effect>>,
    execute: impl Fn(Effect),
) {
    while let Ok(request) = receiver.recv() {
        if request.ownership_sender.send(()).is_err() {
            continue;
        }
        if request.release_receiver.recv().is_err() {
            continue;
        }
        execute(request.effect);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::{spawn_deferred_effect_worker, DeferredEffectQueue};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestEffect {
        Settings,
        Restart,
    }

    fn unavailable_queue() -> DeferredEffectQueue<TestEffect> {
        spawn_deferred_effect_worker(
            1,
            |worker| {
                drop(worker);
                Ok::<(), ()>(())
            },
            |_| {},
        )
        .expect("fake spawn should return a disconnected queue")
    }

    fn live_queue() -> (DeferredEffectQueue<TestEffect>, mpsc::Receiver<TestEffect>) {
        let (executed_sender, executed_receiver) = mpsc::channel();
        let queue = spawn_deferred_effect_worker(
            1,
            |worker| {
                std::thread::Builder::new()
                    .name("deferred-effect-test".to_owned())
                    .spawn(worker)
                    .map(|_| ())
            },
            move |effect| {
                executed_sender
                    .send(effect)
                    .expect("test receiver should remain available");
            },
        )
        .expect("test worker should spawn");
        (queue, executed_receiver)
    }

    #[test]
    fn spawn_failure_returns_before_a_queue_is_available() {
        // Arrange
        let (executed_sender, executed_receiver) = mpsc::channel::<TestEffect>();

        // Act
        let result = spawn_deferred_effect_worker(
            1,
            |_worker| Err("spawn_failed"),
            move |effect| {
                executed_sender
                    .send(effect)
                    .expect("test receiver should remain available");
            },
        );

        // Assert
        assert!(result.is_err());
        assert!(executed_receiver.try_recv().is_err());
    }

    #[test]
    fn unavailable_worker_rejects_settings_ownership_before_response() {
        // Arrange
        let queue = unavailable_queue();

        // Act
        let result = queue.acquire(TestEffect::Settings);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn unavailable_worker_rejects_restart_ownership_before_response() {
        // Arrange
        let queue = unavailable_queue();

        // Act
        let result = queue.acquire(TestEffect::Restart);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn settings_effect_waits_for_response_release() {
        // Arrange
        let (queue, executed_receiver) = live_queue();
        let lease = queue
            .acquire(TestEffect::Settings)
            .expect("worker should own settings effect");

        // Act
        let before_release = executed_receiver.try_recv();
        lease
            .release_after_response()
            .expect("owned settings effect should release");
        let after_release = executed_receiver.recv_timeout(Duration::from_secs(1));

        // Assert
        assert!(before_release.is_err());
        assert_eq!(after_release, Ok(TestEffect::Settings));
    }

    #[test]
    fn restart_effect_waits_for_response_release() {
        // Arrange
        let (queue, executed_receiver) = live_queue();
        let lease = queue
            .acquire(TestEffect::Restart)
            .expect("worker should own restart effect");

        // Act
        let before_release = executed_receiver.try_recv();
        lease
            .release_after_response()
            .expect("owned restart effect should release");
        let after_release = executed_receiver.recv_timeout(Duration::from_secs(1));

        // Assert
        assert!(before_release.is_err());
        assert_eq!(after_release, Ok(TestEffect::Restart));
    }

    #[test]
    fn dropped_lease_discards_owned_effect() {
        // Arrange
        let (queue, executed_receiver) = live_queue();
        let lease = queue
            .acquire(TestEffect::Settings)
            .expect("worker should own settings effect");

        // Act
        drop(lease);
        let result = executed_receiver.recv_timeout(Duration::from_millis(50));

        // Assert
        assert!(result.is_err());
    }
}
