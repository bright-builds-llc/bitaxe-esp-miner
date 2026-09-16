//! Characterizes the current production API's missing in-call cancellation seam.
//! The held synthetic RNG is not a claim about ESP hardware RNG or crypto speed.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use rand::{CryptoRng, RngCore};

use super::{NoiseInitiator, NoisePreparationStage};

struct HeldRng {
    entered: mpsc::SyncSender<()>,
    maybe_release: Option<mpsc::Receiver<()>>,
}

impl RngCore for HeldRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, destination: &mut [u8]) {
        if let Some(release) = self.maybe_release.take() {
            self.entered.send(()).expect("test observer remains alive");
            release
                .recv_timeout(Duration::from_secs(15))
                .expect("test releases the synthetic dependency within its bound");
        }
        destination.fill(1);
    }

    fn try_fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(destination);
        Ok(())
    }
}

impl CryptoRng for HeldRng {}

#[test]
fn preparation_observer_cannot_acknowledge_cancellation_inside_a_held_dependency() {
    // Arrange: call the production preparation function and pinned Noise library.
    let (entered_sender, entered_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let (completed_sender, completed_receiver) = mpsc::sync_channel(1);
    let cancelled = Arc::new(AtomicBool::new(false));
    let observed_cancel = Arc::new(AtomicBool::new(false));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let worker_cancelled = Arc::clone(&cancelled);
    let worker_observed = Arc::clone(&observed_cancel);
    let worker_callbacks = Arc::clone(&callbacks);
    let worker = std::thread::spawn(move || {
        let mut rng = HeldRng {
            entered: entered_sender,
            maybe_release: Some(release_receiver),
        };
        let result = NoiseInitiator::prepare_with_observer(None, &mut rng, |_| {
            worker_callbacks.fetch_add(1, Ordering::SeqCst);
            if worker_cancelled.load(Ordering::SeqCst) {
                worker_observed.store(true, Ordering::SeqCst);
            }
        });
        let succeeded = result.is_ok();
        drop(result);
        completed_sender
            .send(succeeded)
            .expect("test observes completion");
    });

    // Act: an external supervisor can revoke authority, but cannot exit this call.
    let entered = entered_receiver.recv_timeout(Duration::from_secs(2));
    cancelled.store(true, Ordering::SeqCst);
    let completion_before_release = completed_receiver.recv_timeout(Duration::from_millis(5_050));
    let callbacks_before_release = callbacks.load(Ordering::SeqCst);
    let observed_before_release = observed_cancel.load(Ordering::SeqCst);
    // Always release and join before asserting, including a failed characterization.
    let release = release_sender.send(());
    let joined = worker.join();
    let completion_after_release = completed_receiver.recv_timeout(Duration::from_secs(1));

    // Assert: the current hooks supply neither in-call cancellation nor quiescence.
    assert!(entered.is_ok());
    assert!(release.is_ok());
    assert!(joined.is_ok());
    assert_eq!(
        completion_before_release,
        Err(mpsc::RecvTimeoutError::Timeout)
    );
    assert_eq!(callbacks_before_release, 0);
    assert!(!observed_before_release);
    assert_eq!(completion_after_release, Ok(true));
    assert!(observed_cancel.load(Ordering::SeqCst));
    assert_eq!(callbacks.load(Ordering::SeqCst), 2);
}

#[test]
fn observing_cancellation_at_keypair_ready_does_not_prevent_act_one_construction() {
    // Arrange
    let (entered, _receiver) = mpsc::sync_channel(1);
    let mut rng = HeldRng {
        entered,
        maybe_release: None,
    };
    let mut stages = Vec::new();
    let mut cancellation_observed = false;

    // Act
    let result = NoiseInitiator::prepare_with_observer(None, &mut rng, |stage| {
        stages.push(stage);
        if stage == NoisePreparationStage::KeypairReady {
            cancellation_observed = true;
        }
    });

    // Assert: the observer's unit return cannot stop the next synchronous operation.
    assert!(result.is_ok());
    assert!(cancellation_observed);
    assert_eq!(
        stages,
        [
            NoisePreparationStage::KeypairReady,
            NoisePreparationStage::ActOneReady
        ]
    );
}
