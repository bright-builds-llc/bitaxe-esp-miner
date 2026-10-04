//! Multi-producer, single-consumer queues with the `std::sync::mpsc` interface.
//!
//! Firmware must not call the `std::sync::mpsc` constructors. They place
//! cache-padded (`#[repr(align(64))]`) state on the stack. The Xtensa LLVM
//! backend realigns such frames with a plain stack-pointer `add` instead of
//! `movsp`. An interrupt between `entry` and that `add` spills the caller's
//! registers below the old stack pointer, and the return reloads them from
//! below the new one, which corrupts the caller. Every esp toolchain from
//! 1.88.0.0 to 1.99.0.0 is affected:
//! - <https://github.com/espressif/llvm-project/issues/140> (backend fix)
//! - <https://github.com/esp-rs/rust/issues/284> (Rust tracking)
//! - <https://github.com/pRizz/xtensa-movsp-realign-repro> (reproducer)
//!
//! This queue holds only a mutex, two condition variables and a `VecDeque`,
//! so its frames need no realignment. `just audit-stack-realignment` keeps the
//! firmware free of realigning callers. One-shot replies use [`crate::reply`].

use std::collections::VecDeque;
use std::sync::mpsc::{RecvError, RecvTimeoutError, SendError, TryRecvError, TrySendError};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

struct Queue<T> {
    items: VecDeque<T>,
    maybe_capacity: Option<usize>,
    senders: usize,
    receiver_alive: bool,
}

impl<T> Queue<T> {
    fn is_full(&self) -> bool {
        self.maybe_capacity
            .is_some_and(|capacity| self.items.len() >= capacity)
    }
}

struct Shared<T> {
    queue: Mutex<Queue<T>>,
    readable: Condvar,
    writable: Condvar,
}

impl<T> Shared<T> {
    fn lock(&self) -> MutexGuard<'_, Queue<T>> {
        // Every update leaves the queue consistent, so a poisoned lock is still valid.
        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Sending half; clone it for more producers. Bounded senders block while full.
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

/// The bounded sender, named after its `std::sync::mpsc` counterpart.
pub type SyncSender<T> = Sender<T>;

/// Receiving half; yields values in send order, then disconnects once every sender is gone.
pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

/// Creates a queue that holds at most `capacity` values.
///
/// A capacity of zero behaves as one: `std`'s rendezvous mode is not provided.
#[must_use]
pub fn sync_channel<T>(capacity: usize) -> (SyncSender<T>, Receiver<T>) {
    let capacity = capacity.max(1);
    connected(VecDeque::with_capacity(capacity), Some(capacity))
}

/// Creates a queue without a capacity limit; sends never block.
#[must_use]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    connected(VecDeque::new(), None)
}

fn connected<T>(items: VecDeque<T>, maybe_capacity: Option<usize>) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        queue: Mutex::new(Queue {
            items,
            maybe_capacity,
            senders: 1,
            receiver_alive: true,
        }),
        readable: Condvar::new(),
        writable: Condvar::new(),
    });
    (
        Sender {
            shared: Arc::clone(&shared),
        },
        Receiver { shared },
    )
}

impl<T> Sender<T> {
    /// Queues the value, waiting for space when bounded; fails once the receiver is gone.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut queue = self.shared.lock();
        while queue.receiver_alive && queue.is_full() {
            queue = self
                .shared
                .writable
                .wait(queue)
                .unwrap_or_else(PoisonError::into_inner);
        }
        if !queue.receiver_alive {
            return Err(SendError(value));
        }
        queue.items.push_back(value);
        drop(queue);
        self.shared.readable.notify_one();
        Ok(())
    }

    /// Queues the value only if there is space right now.
    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        let mut queue = self.shared.lock();
        if !queue.receiver_alive {
            return Err(TrySendError::Disconnected(value));
        }
        if queue.is_full() {
            return Err(TrySendError::Full(value));
        }
        queue.items.push_back(value);
        drop(queue);
        self.shared.readable.notify_one();
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.shared.lock().senders += 1;
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut queue = self.shared.lock();
        queue.senders -= 1;
        let last = queue.senders == 0;
        drop(queue);
        if last {
            self.shared.readable.notify_all();
        }
    }
}

impl<T> Receiver<T> {
    /// Takes the oldest value without waiting.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        let mut queue = self.shared.lock();
        match self.pop(&mut queue) {
            Some(value) => Ok(value),
            None if queue.senders == 0 => Err(TryRecvError::Disconnected),
            None => Err(TryRecvError::Empty),
        }
    }

    /// Waits for the oldest value; fails once the queue is empty and every sender is gone.
    pub fn recv(&self) -> Result<T, RecvError> {
        let mut queue = self.shared.lock();
        loop {
            if let Some(value) = self.pop(&mut queue) {
                return Ok(value);
            }
            if queue.senders == 0 {
                return Err(RecvError);
            }
            queue = self
                .shared
                .readable
                .wait(queue)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// Waits at most `timeout` for the oldest value.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        let Some(deadline) = Instant::now().checked_add(timeout) else {
            return self
                .recv()
                .map_err(|RecvError| RecvTimeoutError::Disconnected);
        };
        let mut queue = self.shared.lock();
        loop {
            if let Some(value) = self.pop(&mut queue) {
                return Ok(value);
            }
            if queue.senders == 0 {
                return Err(RecvTimeoutError::Disconnected);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(RecvTimeoutError::Timeout);
            }
            queue = self
                .shared
                .readable
                .wait_timeout(queue, remaining)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
    }

    fn pop(&self, queue: &mut Queue<T>) -> Option<T> {
        let value = queue.items.pop_front()?;
        if queue.maybe_capacity.is_some() {
            self.shared.writable.notify_one();
        }
        Some(value)
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut queue = self.shared.lock();
        queue.receiver_alive = false;
        // Drop undelivered values outside the lock: their destructors may block or lock.
        let undelivered = std::mem::take(&mut queue.items);
        drop(queue);
        self.shared.writable.notify_all();
        drop(undelivered);
    }
}

#[cfg(test)]
mod tests;
