//! Single-owner, allocation-free RTC progress writes; no panic hook is required.
use bitaxe_api::panic_receipt::allocation_source_hash;
use bitaxe_api::preparation_receipt::{
    commit_words, read_slot, recover, recover_for_boot, Failure, Outcome, Receipt, Slot, Snapshot,
    VALID, WORDS,
};
use esp_idf_svc::sys;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::OnceLock;

#[cfg_attr(target_os = "espidf", link_section = ".rtc_noinit")]
static RTC_SLOTS: [[AtomicU32; WORDS]; 2] = [const { [const { AtomicU32::new(0) }; WORDS] }; 2];
static PREVIOUS: OnceLock<Snapshot> = OnceLock::new();
static WRITING: AtomicBool = AtomicBool::new(false);
static NEXT_SLOT: AtomicU32 = AtomicU32::new(0);
static SEQUENCE: AtomicU32 = AtomicU32::new(0);
static GENERATION: AtomicU32 = AtomicU32::new(0);
static LAST_COMPLETED: AtomicU32 = AtomicU32::new(0);
const SOURCE_HASH: u64 = allocation_source_hash(env!("BITAXE_FIRMWARE_COMMIT"));

fn slots() -> [Slot; 2] {
    std::array::from_fn(|slot| read_slot(|word| RTC_SLOTS[slot][word].load(Ordering::SeqCst)))
}

/// Called before any runtime owner can write. The immutable snapshot survives reconnects.
pub(crate) fn initialize(boot_ordinal: u64) {
    PREVIOUS.get_or_init(|| {
        let snapshot = recover(&slots(), SOURCE_HASH);
        NEXT_SLOT.store(
            snapshot.maybe_slot.map_or(0, |slot| 1 - slot) as u32,
            Ordering::Release,
        );
        snapshot.for_boot(boot_ordinal.saturating_sub(1))
    });
}
#[derive(Clone, Copy)]
pub(crate) struct ResourceObservation {
    boot_ordinal: u64,
    uptime_ms: u64,
    maybe_heap_free: Option<u32>,
    maybe_heap_largest: Option<u32>,
    maybe_stack_free: Option<u32>,
}
/// Driver/runtime queries are deliberately outside the RTC writer's critical section.
pub(crate) fn current_position() -> ResourceObservation {
    ResourceObservation {
        boot_ordinal: crate::boot_evidence::operator_snapshot_boot_ordinal(),
        uptime_ms: crate::runtime_uptime::millis(),
        maybe_heap_free: None,
        maybe_heap_largest: None,
        maybe_stack_free: None,
    }
}
pub(crate) fn observe_resources(position: ResourceObservation) -> ResourceObservation {
    let caps = sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT;
    let (free, largest, stack) = unsafe {
        (
            sys::heap_caps_get_free_size(caps),
            sys::heap_caps_get_largest_free_block(caps),
            sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()),
        )
    };
    // Pinned ESP-IDF Xtensa port uses uint8_t StackType_t; the watermark is bytes.
    ResourceObservation {
        maybe_heap_free: u32::try_from(free).ok(),
        maybe_heap_largest: u32::try_from(largest).ok(),
        maybe_stack_free: Some(stack),
        ..position
    }
}
struct WriterGuard;
impl Drop for WriterGuard {
    fn drop(&mut self) {
        WRITING.store(false, Ordering::Release);
    }
}
/// Caller is the sole production owner; competing calls fail without waiting or allocating.
pub(crate) fn record(
    generation: u32,
    step: u32,
    outcome: Outcome,
    failure: Failure,
    resources: ResourceObservation,
) {
    let boot_ordinal = resources.boot_ordinal;
    let uptime_ms = resources.uptime_ms;
    if PREVIOUS.get().is_none()
        || WRITING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
    {
        return;
    }
    let _guard = WriterGuard;
    if generation < GENERATION.load(Ordering::Relaxed) {
        return;
    }
    let Some(sequence) = SEQUENCE.load(Ordering::Relaxed).checked_add(1) else {
        return;
    };
    let last = if GENERATION.load(Ordering::Relaxed) == generation {
        LAST_COMPLETED.load(Ordering::Relaxed)
    } else {
        0
    };
    let completed = if outcome == Outcome::Completed {
        step
    } else {
        last
    };
    let receipt = Receipt {
        source_hash: SOURCE_HASH,
        boot_ordinal,
        generation,
        sequence,
        uptime_ms,
        last_completed_step: completed,
        current_step: step,
        outcome,
        failure,
        maybe_heap_free: resources.maybe_heap_free,
        maybe_heap_largest: resources.maybe_heap_largest,
        maybe_stack_free: resources.maybe_stack_free,
    };
    let Some(words) = receipt.encode() else {
        return;
    };
    let slot = NEXT_SLOT.load(Ordering::Relaxed) as usize;
    if slot >= RTC_SLOTS.len() {
        return;
    }
    commit_words(words, |word, value| {
        // Atomic words avoid data races with the diagnostic reader. The final
        // release commit publishes all fields; CRC rejects mixed snapshots.
        let order = if word == 0 && value == VALID {
            Ordering::Release
        } else {
            Ordering::SeqCst
        };
        RTC_SLOTS[slot][word].store(value, order);
    });
    SEQUENCE.store(sequence, Ordering::Relaxed);
    GENERATION.store(generation, Ordering::Relaxed);
    LAST_COMPLETED.store(completed, Ordering::Relaxed);
    NEXT_SLOT.store((1 - slot) as u32, Ordering::Relaxed);
}
/// Marker formatting allocates only on the normal diagnostic task, never in the writer.
pub(crate) fn marker(previous: bool) -> String {
    let snapshot = if previous {
        *PREVIOUS.get().unwrap_or(&Snapshot::unavailable())
    } else if SEQUENCE.load(Ordering::Acquire) == 0 {
        Snapshot::unavailable()
    } else {
        recover_for_boot(
            &slots(),
            SOURCE_HASH,
            crate::boot_evidence::operator_snapshot_boot_ordinal(),
        )
    };
    snapshot.marker(previous)
}

#[cfg(test)]
#[path = "preparation_evidence/tests.rs"]
mod tests;
