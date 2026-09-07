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
struct Slot {
    sequence: AtomicU32,
    words: [AtomicU32; 7],
}
impl Slot {
    const fn new() -> Self {
        Self {
            sequence: AtomicU32::new(0),
            words: [const { AtomicU32::new(0) }; 7],
        }
    }
}
pub(crate) struct Cache {
    active: AtomicU32,
    slots: [Slot; 2],
}
impl Cache {
    pub(crate) const fn new() -> Self {
        Self {
            active: AtomicU32::new(0),
            slots: [const { Slot::new() }; 2],
        }
    }
    /// The sole owner updates the inactive slot, keeping the last committed sample readable.
    pub(crate) fn publish(&self, value: Snapshot) {
        self.publish_with_interleave(value, || {});
    }
    #[cfg(test)]
    pub(crate) fn publish_paused(&self, value: Snapshot, paused: impl FnOnce()) {
        self.publish_with_interleave(value, paused);
    }
    fn publish_with_interleave(&self, value: Snapshot, paused: impl FnOnce()) {
        let inactive = 1 - self.active.load(Ordering::SeqCst);
        let slot = &self.slots[inactive as usize];
        let Some(next) = slot.sequence.load(Ordering::SeqCst).checked_add(2) else {
            return;
        };
        slot.sequence.store(next - 1, Ordering::SeqCst);
        paused();
        for (word, value) in slot.words.iter().zip([
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
        slot.sequence.store(next, Ordering::SeqCst);
        self.active.store(inactive, Ordering::SeqCst);
    }
    pub(crate) fn read_now(&self, generation: u32, now: impl FnOnce() -> u64) -> Option<Snapshot> {
        self.read_with_interleave(generation, now, |_| {})
    }
    #[cfg(test)]
    pub(crate) fn read_contended(
        &self,
        generation: u32,
        now: impl FnOnce() -> u64,
        interleave: impl FnMut(usize),
    ) -> Option<Snapshot> {
        self.read_with_interleave(generation, now, interleave)
    }
    pub(crate) fn read(&self, generation: u32, now: u64) -> Option<Snapshot> {
        self.read_now(generation, || now)
    }
    fn read_with_interleave(
        &self,
        generation: u32,
        now: impl FnOnce() -> u64,
        mut interleave: impl FnMut(usize),
    ) -> Option<Snapshot> {
        if generation == 0 {
            return None;
        }
        let mut coherent = None;
        // Never wait for the owner; bounded retries tolerate a publication crossing this read.
        for attempt in 0..4 {
            let active = self.active.load(Ordering::SeqCst);
            let slot = &self.slots[active as usize];
            let sequence = slot.sequence.load(Ordering::SeqCst);
            interleave(attempt);
            if sequence == 0 || sequence % 2 != 0 {
                continue;
            }
            let words = slot
                .words
                .each_ref()
                .map(|word| word.load(Ordering::SeqCst));
            if sequence == slot.sequence.load(Ordering::SeqCst)
                && active == self.active.load(Ordering::SeqCst)
            {
                coherent = Some(words);
                break;
            }
        }
        let words = coherent?;
        if words[0] != generation {
            return None;
        }
        // Evaluate age only after copying a coherent observation; a request-start timestamp
        // can precede a concurrently published sample and falsely classify it as future data.
        let observed = u64::from(words[2]) | (u64::from(words[3]) << 32);
        let evaluated_at = now();
        if evaluated_at < observed || evaluated_at - observed > 1000 {
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
