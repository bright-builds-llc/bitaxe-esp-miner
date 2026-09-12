//! Bounded, nonblocking serial observations. No payloads or operational identifiers enter this store.

use serde::Serialize;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const SERIAL_TRACE_CAPACITY: usize = 64;
const MAXIMUM_SAFE_TIMESTAMP: u64 = 9_007_199_254_740_991;

/// A boot-local epoch and the validated outer serial sequence, never a request or session string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SerialTraceCorrelation {
    pub epoch: u32,
    pub request_sequence: u32,
}

/// Closed observations identify the boundary reached, not delivery or authorization by inference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerialTraceStage {
    Hello,
    Validated,
    ValidationRejected,
    EnqueueStarted,
    Enqueued,
    EnqueueRejected,
    DispatchStarted,
    DispatchRejected,
    ReplyCreated,
    ReplyRejected,
    WriterAccepted,
    WriterRejected,
    WriterStarted,
    WriterQueued,
    WriterCompleted,
    WriterAbandoned,
    EpochRevoked,
}

/// `wireBytes` is the inbound record for validation/enqueue/dispatch; it is zero when
/// reply creation or queue acceptance has not observed outer encoding. From writer_started
/// onward it is the full encoded output length. `queuedBytes` proves native acceptance only.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialTraceEvent {
    pub ordinal: u32,
    pub epoch: u32,
    pub request_sequence: u32,
    pub stage: SerialTraceStage,
    pub at_ms: u64,
    pub wire_bytes: u32,
    pub queued_bytes: u32,
}

impl SerialTraceEvent {
    const EMPTY: Self = Self {
        ordinal: 0,
        epoch: 0,
        request_sequence: 0,
        stage: SerialTraceStage::Hello,
        at_ms: 0,
        wire_bytes: 0,
        queued_bytes: 0,
    };
}

struct TraceBuffer {
    epoch: u32,
    events: [SerialTraceEvent; SERIAL_TRACE_CAPACITY],
    start: usize,
    len: usize,
    next_ordinal: u32,
    overwritten: u32,
}

impl TraceBuffer {
    const fn new() -> Self {
        Self {
            epoch: 0,
            events: [SerialTraceEvent::EMPTY; SERIAL_TRACE_CAPACITY],
            start: 0,
            len: 0,
            next_ordinal: 1,
            overwritten: 0,
        }
    }

    fn reset(&mut self, epoch: u32) {
        self.epoch = epoch;
        self.start = 0;
        self.len = 0;
        self.next_ordinal = 1;
        self.overwritten = 0;
    }

    fn snapshot(&self, mut events: Vec<SerialTraceEvent>) -> SerialTraceWindow {
        for offset in 0..self.len {
            events.push(self.events[(self.start + offset) % SERIAL_TRACE_CAPACITY]);
        }
        SerialTraceWindow {
            epoch: self.epoch,
            first_event_ordinal: events
                .first()
                .map_or(self.next_ordinal, |event| event.ordinal),
            next_event_ordinal: self.next_ordinal,
            overwritten_events: self.overwritten,
            events,
        }
    }
}

struct TraceStorage {
    slots: [TraceBuffer; 2],
    current: usize,
}

struct TraceCell {
    owned: AtomicBool,
    storage: UnsafeCell<TraceStorage>,
}

// SAFETY: storage contains only owned integers/arrays. A single successful acquire CAS
// creates the only guard; all accesses require that guard, whose drop releases ownership.
unsafe impl Sync for TraceCell {}

impl TraceCell {
    const fn new(storage: TraceStorage) -> Self {
        Self {
            owned: AtomicBool::new(false),
            storage: UnsafeCell::new(storage),
        }
    }

    fn try_lock(&self) -> Option<TraceGuard<'_>> {
        self.owned
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| TraceGuard { cell: self })
    }
}

struct TraceGuard<'a> {
    cell: &'a TraceCell,
}

impl Deref for TraceGuard<'_> {
    type Target = TraceStorage;
    fn deref(&self) -> &Self::Target {
        // SAFETY: this unique, non-cloneable guard holds acquired ownership until drop.
        unsafe { &*self.cell.storage.get() }
    }
}

impl DerefMut for TraceGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: exclusive guard ownership and its mutable borrow exclude every other access.
        unsafe { &mut *self.cell.storage.get() }
    }
}

impl Drop for TraceGuard<'_> {
    fn drop(&mut self) {
        self.cell.owned.store(false, Ordering::Release);
    }
}

/// The capture path uses fixed RAM and try-lock only; an explicit snapshot may allocate bounded output.
pub struct SerialTrace {
    buffer: TraceCell,
    dropped: AtomicU32,
}

impl Default for SerialTrace {
    fn default() -> Self {
        Self::new()
    }
}

impl SerialTrace {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: TraceCell::new(TraceStorage {
                slots: [TraceBuffer::new(), TraceBuffer::new()],
                current: 0,
            }),
            dropped: AtomicU32::new(0),
        }
    }

    /// Observes without waiting for another recorder or exporter. Loss is explicit in every snapshot.
    pub fn record(
        &self,
        correlation: SerialTraceCorrelation,
        stage: SerialTraceStage,
        at_ms: u64,
        wire_bytes: usize,
        queued_bytes: usize,
    ) {
        if at_ms > MAXIMUM_SAFE_TIMESTAMP
            || wire_bytes > super::MAXIMUM_WIRE_FRAME_BYTES
            || queued_bytes > wire_bytes
        {
            self.note_dropped();
            return;
        }
        let Some(mut storage) = self.buffer.try_lock() else {
            self.note_dropped();
            return;
        };
        let current = storage.current;
        if correlation.epoch > storage.slots[current].epoch {
            storage.current = 1 - current;
            let next = storage.current;
            storage.slots[next].reset(correlation.epoch);
        }
        let current = storage.current;
        let index = if storage.slots[current].epoch == correlation.epoch {
            current
        } else if storage.slots[1 - current].epoch == correlation.epoch {
            1 - current
        } else {
            self.note_dropped();
            return;
        };
        let buffer = &mut storage.slots[index];
        if buffer.next_ordinal == u32::MAX {
            self.note_dropped();
            return;
        }
        let event = SerialTraceEvent {
            ordinal: buffer.next_ordinal,
            epoch: correlation.epoch,
            request_sequence: correlation.request_sequence,
            stage,
            at_ms,
            wire_bytes: wire_bytes as u32,
            queued_bytes: queued_bytes as u32,
        };
        buffer.next_ordinal += 1;
        let index = (buffer.start + buffer.len) % SERIAL_TRACE_CAPACITY;
        buffer.events[index] = event;
        if buffer.len == SERIAL_TRACE_CAPACITY {
            buffer.start = (buffer.start + 1) % SERIAL_TRACE_CAPACITY;
            buffer.overwritten += 1;
        } else {
            buffer.len += 1;
        }
    }

    fn note_dropped(&self) {
        let _ = self
            .dropped
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                Some(count.saturating_add(1))
            });
    }

    /// Applies the production owner-epoch guard and records whether queued control reaches dispatch.
    pub fn admit_dispatch(
        &self,
        correlation: SerialTraceCorrelation,
        owner_epoch: u32,
        current_epoch: u32,
        at_ms: u64,
        wire_bytes: usize,
    ) -> bool {
        let admitted = correlation.epoch == owner_epoch && correlation.epoch == current_epoch;
        self.record(
            correlation,
            if admitted {
                SerialTraceStage::DispatchStarted
            } else {
                SerialTraceStage::DispatchRejected
            },
            at_ms,
            wire_bytes,
            0,
        );
        admitted
    }

    /// Exports the retained window without clearing it or waiting for a busy recorder.
    #[must_use]
    pub fn snapshot(&self) -> SerialTraceSnapshot {
        let mut current_events = Vec::new();
        let mut previous_events = Vec::new();
        if current_events
            .try_reserve_exact(SERIAL_TRACE_CAPACITY)
            .is_err()
            || previous_events
                .try_reserve_exact(SERIAL_TRACE_CAPACITY)
                .is_err()
        {
            return self.unavailable();
        }
        let Some(storage) = self.buffer.try_lock() else {
            return self.unavailable();
        };
        let previous = &storage.slots[1 - storage.current];
        SerialTraceSnapshot {
            schema: "worker-serial-trace-v1",
            capacity: SERIAL_TRACE_CAPACITY,
            dropped_events: self.dropped.load(Ordering::Relaxed),
            snapshot_available: true,
            current: storage.slots[storage.current].snapshot(current_events),
            previous: (previous.epoch > 0).then(|| previous.snapshot(previous_events)),
        }
    }

    fn unavailable(&self) -> SerialTraceSnapshot {
        SerialTraceSnapshot {
            schema: "worker-serial-trace-v1",
            capacity: SERIAL_TRACE_CAPACITY,
            dropped_events: self.dropped.load(Ordering::Relaxed),
            snapshot_available: false,
            current: SerialTraceWindow {
                epoch: 0,
                first_event_ordinal: 0,
                next_event_ordinal: 0,
                overwritten_events: 0,
                events: Vec::new(),
            },
            previous: None,
        }
    }
}

/// Explicit authenticated export. Overwrite and capture loss are evidence limits, never hidden resets.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialTraceSnapshot {
    pub schema: &'static str,
    pub capacity: usize,
    pub dropped_events: u32,
    pub snapshot_available: bool,
    pub current: SerialTraceWindow,
    pub previous: Option<SerialTraceWindow>,
}

/// One epoch's retained tail. Successor traffic never consumes the previous epoch's capacity.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialTraceWindow {
    pub epoch: u32,
    pub first_event_ordinal: u32,
    pub next_event_ordinal: u32,
    pub overwritten_events: u32,
    pub events: Vec<SerialTraceEvent>,
}

#[cfg(test)]
mod tests;
