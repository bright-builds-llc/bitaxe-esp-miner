//! One-shot reply slot for per-request replies.
//!
//! `std::sync::mpsc` constructors realign the stack to 64 bytes for their cache-padded
//! state. The Xtensa LLVM backend emits that realignment as a plain stack-pointer add
//! rather than `movsp`, so an interrupt in the prologue makes the caller restore its
//! registers from stale memory. No esp toolchain release fixes it yet; see
//! <https://github.com/espressif/llvm-project/issues/140> and
//! <https://github.com/esp-rs/rust/issues/284>. Requests that create a channel per call
//! use this slot instead: it holds only a mutex, a condition variable and the value.
//! Longer-lived queues use [`crate::queue`].

use std::sync::mpsc::{RecvError, RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

enum State<T> {
    Waiting,
    Ready(T),
    Taken,
    SenderGone,
    ReceiverGone,
}

struct Slot<T> {
    state: Mutex<State<T>>,
    changed: Condvar,
}

impl<T> Slot<T> {
    fn lock(&self) -> MutexGuard<'_, State<T>> {
        // A panicking peer cannot leave the enum half-written, so the state stays valid.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Sending half; delivers at most one value.
pub struct ReplySender<T> {
    slot: Arc<Slot<T>>,
}

/// Receiving half; observes the value or the sender's disappearance.
pub struct ReplyReceiver<T> {
    slot: Arc<Slot<T>>,
}

/// Creates a connected one-shot reply pair.
#[must_use]
pub fn reply<T>() -> (ReplySender<T>, ReplyReceiver<T>) {
    let slot = Arc::new(Slot {
        state: Mutex::new(State::Waiting),
        changed: Condvar::new(),
    });
    (
        ReplySender {
            slot: Arc::clone(&slot),
        },
        ReplyReceiver { slot },
    )
}

impl<T> ReplySender<T> {
    /// Delivers the value, or returns it when the receiver is already gone.
    pub fn send(self, value: T) -> Result<(), T> {
        let mut state = self.slot.lock();
        if !matches!(*state, State::Waiting) {
            return Err(value);
        }
        *state = State::Ready(value);
        drop(state);
        self.slot.changed.notify_all();
        Ok(())
    }
}

impl<T> Drop for ReplySender<T> {
    fn drop(&mut self) {
        let mut state = self.slot.lock();
        if matches!(*state, State::Waiting) {
            *state = State::SenderGone;
            drop(state);
            self.slot.changed.notify_all();
        }
    }
}

impl<T> ReplyReceiver<T> {
    /// Takes the value if it has arrived, without waiting.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        let mut state = self.slot.lock();
        match std::mem::replace(&mut *state, State::Taken) {
            State::Ready(value) => Ok(value),
            State::Waiting => {
                *state = State::Waiting;
                Err(TryRecvError::Empty)
            }
            other => {
                *state = other;
                Err(TryRecvError::Disconnected)
            }
        }
    }

    /// Waits until the value arrives or the sender disappears.
    pub fn recv(&self) -> Result<T, RecvError> {
        let mut state = self.slot.lock();
        while matches!(*state, State::Waiting) {
            state = self
                .slot
                .changed
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        take(&mut state).ok_or(RecvError)
    }

    /// Waits at most `timeout` for the value.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        let deadline = Instant::now().checked_add(timeout);
        let mut state = self.slot.lock();
        while matches!(*state, State::Waiting) {
            let Some(remaining) = deadline.map(|at| at.saturating_duration_since(Instant::now()))
            else {
                state = self
                    .slot
                    .changed
                    .wait(state)
                    .unwrap_or_else(PoisonError::into_inner);
                continue;
            };
            if remaining.is_zero() {
                return Err(RecvTimeoutError::Timeout);
            }
            state = self
                .slot
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        take(&mut state).ok_or(RecvTimeoutError::Disconnected)
    }
}

impl<T> Drop for ReplyReceiver<T> {
    fn drop(&mut self) {
        *self.slot.lock() = State::ReceiverGone;
    }
}

fn take<T>(state: &mut State<T>) -> Option<T> {
    match std::mem::replace(state, State::Taken) {
        State::Ready(value) => Some(value),
        other => {
            *state = other;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn a_sent_value_is_received_once() {
        // Arrange
        let (sender, receiver) = reply();

        // Act
        sender.send(7_u8).expect("receiver is connected");

        // Assert
        assert_eq!(receiver.try_recv(), Ok(7));
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Disconnected));
    }

    #[test]
    fn an_empty_slot_is_empty_until_the_sender_drops() {
        // Arrange
        let (sender, receiver) = reply::<u8>();

        // Act
        let before = receiver.try_recv();
        drop(sender);

        // Assert
        assert_eq!(before, Err(TryRecvError::Empty));
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Disconnected));
        assert_eq!(receiver.recv(), Err(RecvError));
    }

    #[test]
    fn a_dropped_sender_disconnects_a_waiting_receiver() {
        // Arrange
        let (sender, receiver) = reply::<u8>();
        let dropper = thread::spawn(move || drop(sender));

        // Act
        let outcome = receiver.recv_timeout(Duration::from_secs(5));

        // Assert
        dropper.join().expect("dropper thread");
        assert_eq!(outcome, Err(RecvTimeoutError::Disconnected));
    }

    #[test]
    fn a_receiver_times_out_without_a_value() {
        // Arrange
        let (_sender, receiver) = reply::<u8>();

        // Act
        let outcome = receiver.recv_timeout(Duration::from_millis(20));

        // Assert
        assert_eq!(outcome, Err(RecvTimeoutError::Timeout));
    }

    #[test]
    fn a_value_sent_from_another_thread_wakes_the_receiver() {
        // Arrange
        let (sender, receiver) = reply();
        let worker = thread::spawn(move || sender.send(true));

        // Act
        let outcome = receiver.recv_timeout(Duration::from_secs(5));

        // Assert
        assert_eq!(worker.join().expect("sender thread"), Ok(()));
        assert_eq!(outcome, Ok(true));
    }

    #[test]
    fn sending_to_a_dropped_receiver_returns_the_value() {
        // Arrange
        let (sender, receiver) = reply();
        drop(receiver);

        // Act
        let outcome = sender.send(9_u8);

        // Assert
        assert_eq!(outcome, Err(9));
    }

    #[test]
    fn a_late_value_after_a_timeout_is_still_delivered_to_a_later_wait() {
        // Arrange
        let (sender, receiver) = reply();
        let timed_out = receiver.recv_timeout(Duration::from_millis(1));

        // Act
        sender.send(3_u8).expect("receiver is connected");

        // Assert
        assert_eq!(timed_out, Err(RecvTimeoutError::Timeout));
        assert_eq!(receiver.recv(), Ok(3));
    }
}
