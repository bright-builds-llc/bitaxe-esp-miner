use super::*;
#[test]
fn only_bound_owner_captures_and_retired_generation_refreshes_with_fresh_time() {
    // Arrange
    bind_owner_thread();
    // Act
    capture(2, Phase::Preparation);
    let preparation = observation(2, 1000).expect("fresh preparation");
    capture(2, Phase::Active);
    let active = observation(2, 1000).expect("fresh active");
    capture(2, Phase::ShutdownComplete);
    crate::runtime_uptime::NOW.store(1600, Ordering::SeqCst);
    refresh_retired();
    let terminal = observation(2, 1600).expect("retired generation refreshed");
    // Assert
    assert_eq!(preparation["phase"], "preparation");
    assert_eq!(active["phase"], "active");
    assert_eq!(
        terminal,
        serde_json::json!({"schema":"worker-owner-resources-v1","generation":2,"phase":"shutdown_complete","observed_at_ms":"1600","heap_free_bytes":15807,"heap_largest_bytes":8192,"stack_free_bytes":8220})
    );
    let queries = crate::sys::QUERIES.load(Ordering::SeqCst);
    crate::sys::TASK.store(9, Ordering::SeqCst);
    capture(3, Phase::Active);
    assert_eq!(
        crate::sys::QUERIES.load(Ordering::SeqCst),
        queries,
        "control reader must not measure its own stack"
    );
    assert!(observation(3, 1600).is_none());
    assert!(observation(2, 2601).is_none());
    assert!(observation(2, 1599).is_none());
}
#[test]
fn coherent_cache_rejects_cross_generation_stale_and_partial_reads() {
    // Arrange
    let cache = Cache::new();
    let sample = Snapshot {
        generation: 7,
        phase: Phase::Active,
        observed_at_ms: (1u64 << 32) + 5,
        heap_free_bytes: 1,
        heap_largest_bytes: 2,
        stack_free_bytes: 3,
    };
    // Act
    cache.publish(sample);
    // Assert
    assert_eq!(cache.read(7, sample.observed_at_ms + 1000), Some(sample));
    assert!(cache.read(8, sample.observed_at_ms).is_none());
    assert!(cache.read(7, sample.observed_at_ms + 1001).is_none());
}
#[test]
fn concurrent_cache_reads_never_mix_generation_and_resource_words() {
    // Arrange
    let cache = std::sync::Arc::new(Cache::new());
    let writer = cache.clone();
    // Act
    let task = std::thread::spawn(move || {
        for value in 1..=5000 {
            writer.publish(Snapshot {
                generation: 1,
                phase: Phase::Active,
                observed_at_ms: 1000,
                heap_free_bytes: value,
                heap_largest_bytes: value,
                stack_free_bytes: value,
            });
        }
    });
    for _ in 0..5000 {
        if let Some(value) = cache.read(1, 1000) {
            assert_eq!(value.heap_free_bytes, value.heap_largest_bytes);
            assert_eq!(value.heap_free_bytes, value.stack_free_bytes);
        }
    }
    task.join().expect("writer completed");
    // Assert
    assert_eq!(
        cache.read(1, 1000).expect("final sample").stack_free_bytes,
        5000
    );
}
