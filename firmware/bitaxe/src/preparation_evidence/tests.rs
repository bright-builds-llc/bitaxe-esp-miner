use super::*;
#[test]
fn actual_writer_allocates_nothing_calls_no_drivers_and_keeps_previous_snapshot() {
    // Arrange
    let old = Receipt {
        source_hash: SOURCE_HASH,
        boot_ordinal: 6,
        generation: 1,
        sequence: 1,
        uptime_ms: 500,
        last_completed_step: 0,
        current_step: 1,
        outcome: Outcome::Started,
        failure: Failure::None,
        maybe_heap_free: None,
        maybe_heap_largest: None,
        maybe_stack_free: None,
    };
    let words = old.encode().expect("old receipt");
    for (index, word) in words.into_iter().enumerate() {
        RTC_SLOTS[0][index].store(word, Ordering::SeqCst);
    }
    initialize(7);
    let previous = marker(true);
    let resources = observe_resources(current_position());
    let query_count = crate::sys::QUERIES.load(Ordering::SeqCst);
    crate::ALLOCATIONS.with(|count| count.set(0));
    crate::TRACK.with(|track| track.set(true));
    // Act
    record(2, 1, Outcome::Started, Failure::None, resources);
    record(2, 1, Outcome::Completed, Failure::None, resources);
    record(1, 2, Outcome::Started, Failure::None, resources); // A stale generation cannot overwrite progress.
    crate::TRACK.with(|track| track.set(false));
    // Assert
    assert_eq!(crate::ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(crate::sys::QUERIES.load(Ordering::SeqCst), query_count);
    assert_eq!(marker(true), previous);
    let current = recover(&slots(), SOURCE_HASH)
        .for_boot(7)
        .maybe_receipt
        .expect("current receipt");
    assert_eq!(
        (
            current.generation,
            current.sequence,
            current.current_step,
            current.outcome
        ),
        (2, 2, 1, Outcome::Completed)
    );
    assert_eq!(
        marker(true),
        previous,
        "reconnection/replay cannot consume previous receipt"
    );
}
use std::cell::Cell;
