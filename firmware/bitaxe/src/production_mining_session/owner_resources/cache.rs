//! Bounded atomic snapshots; reading never waits for the owner.
use std::sync::atomic::{AtomicU32, Ordering};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum Phase {
    Preparation = 1,
    Active = 2,
    ShutdownComplete = 3,
}
impl Phase {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Preparation => "preparation",
            Self::Active => "active",
            Self::ShutdownComplete => "shutdown_complete",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Snapshot {
    pub generation: u32,
    pub phase: Phase,
    pub observed_at_ms: u64,
    pub heap_free_bytes: u32,
    pub heap_largest_bytes: u32,
    pub stack_free_bytes: u32,
}
pub(crate) struct Cache {
    sequence: AtomicU32,
    words: [AtomicU32; 7],
}
impl Cache {
    pub(crate) const fn new() -> Self {
        Self {
            sequence: AtomicU32::new(0),
            words: [const { AtomicU32::new(0) }; 7],
        }
    }
    /// The production owner is the only publisher.
    pub(crate) fn publish(&self, value: Snapshot) {
        let Some(next) = self.sequence.load(Ordering::SeqCst).checked_add(2) else {
            return;
        };
        self.sequence.store(next - 1, Ordering::SeqCst);
        for (word, value) in self.words.iter().zip([
            value.generation,
            value.phase as u32,
            value.observed_at_ms as u32,
            (value.observed_at_ms >> 32) as u32,
            value.heap_free_bytes,
            value.heap_largest_bytes,
            value.stack_free_bytes,
        ]) {
            word.store(value, Ordering::SeqCst);
        }
        self.sequence.store(next, Ordering::SeqCst);
    }
    pub(crate) fn read(&self, generation: u32, now: u64) -> Option<Snapshot> {
        let sequence = self.sequence.load(Ordering::SeqCst);
        if sequence == 0 || sequence % 2 != 0 {
            return None;
        }
        let words = self
            .words
            .each_ref()
            .map(|word| word.load(Ordering::SeqCst));
        if sequence != self.sequence.load(Ordering::SeqCst)
            || generation == 0
            || words[0] != generation
        {
            return None;
        }
        let observed = u64::from(words[2]) | (u64::from(words[3]) << 32);
        if now < observed || now - observed > 1000 {
            return None;
        }
        let phase = match words[1] {
            1 => Phase::Preparation,
            2 => Phase::Active,
            3 => Phase::ShutdownComplete,
            _ => return None,
        };
        Some(Snapshot {
            generation,
            phase,
            observed_at_ms: observed,
            heap_free_bytes: words[4],
            heap_largest_bytes: words[5],
            stack_free_bytes: words[6],
        })
    }
}
