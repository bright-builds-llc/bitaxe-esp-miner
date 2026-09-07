use super::*;
#[test]
fn only_bound_owner_captures_and_retired_generation_refreshes_with_fresh_time() {
    // Arrange
    bind_owner_thread();
    // Act
    capture(2, Phase::Preparation);
    let preparation = observation(2).expect("fresh preparation");
    capture(2, Phase::Active);
    let active = observation(2).expect("fresh active");
    capture(2, Phase::ShutdownComplete);
    crate::runtime_uptime::NOW.store(1600, Ordering::SeqCst);
    refresh_retired();
    let terminal = observation(2).expect("retired generation refreshed");
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
    assert!(observation(3).is_none());
    crate::runtime_uptime::NOW.store(2601, Ordering::SeqCst);
    assert!(observation(2).is_none());
    crate::runtime_uptime::NOW.store(1599, Ordering::SeqCst);
    assert!(observation(2).is_none());
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

#[test]
fn evaluation_clock_is_sampled_after_the_coherent_snapshot() {
    // Arrange
    let cache = Cache::new();
    let sample = Snapshot {
        generation: 1,
        phase: Phase::Active,
        observed_at_ms: 1000,
        heap_free_bytes: 10,
        heap_largest_bytes: 8,
        stack_free_bytes: 8192,
    };
    cache.publish(sample);
    // Act: a publication races with the timestamp read, after its value was sampled.
    let observed = cache.read_now(1, || {
        cache.publish(Snapshot {
            observed_at_ms: 1001,
            ..sample
        });
        1000
    });
    // Assert
    assert_eq!(observed, Some(sample));
}
#[test]
fn bounded_retry_recovers_a_publication_interleaved_with_the_first_read() {
    // Arrange
    let cache = Cache::new();
    let sample = Snapshot {
        generation: 1,
        phase: Phase::Active,
        observed_at_ms: 1000,
        heap_free_bytes: 10,
        heap_largest_bytes: 8,
        stack_free_bytes: 8192,
    };
    cache.publish(sample);
    let next = Snapshot {
        observed_at_ms: 1001,
        ..sample
    };
    // Act
    let observed = cache.read_contended(
        1,
        || 1001,
        |attempt| {
            if attempt == 0 {
                cache.publish(next);
            }
        },
    );
    // Assert
    assert_eq!(observed, Some(next));
}

#[test]
fn continuous_publication_contention_is_bounded_and_returns_no_unqualified_sample() {
    // Arrange
    let cache = Cache::new();
    let sample = Snapshot {
        generation: 1,
        phase: Phase::Active,
        observed_at_ms: 1000,
        heap_free_bytes: 10,
        heap_largest_bytes: 8,
        stack_free_bytes: 8192,
    };
    cache.publish(sample);
    let calls = std::cell::Cell::new(0);
    // Act
    let observed = cache.read_contended(
        1,
        || panic!("no clock query without a coherent snapshot"),
        |_| {
            calls.set(calls.get() + 1);
            cache.publish(sample);
        },
    );
    // Assert
    assert!(observed.is_none());
    assert_eq!(calls.get(), 4);
}

#[test]
fn paused_publisher_keeps_the_preceding_fresh_coherent_snapshot_readable() {
    // Arrange
    let cache = Cache::new();
    let prior = Snapshot {
        generation: 1,
        phase: Phase::Active,
        observed_at_ms: 1000,
        heap_free_bytes: 10,
        heap_largest_bytes: 8,
        stack_free_bytes: 8192,
    };
    cache.publish(prior);
    let next = Snapshot {
        observed_at_ms: 1250,
        ..prior
    };
    // Act / Assert: the reader runs while its sole publisher is paused mid-publication.
    cache.publish_paused(next, || {
        assert_eq!(cache.read(1, 1250), Some(prior));
        assert!(cache.read(2, 1250).is_none());
        assert!(cache.read(1, 2001).is_none());
    });
    assert_eq!(cache.read(1, 1250), Some(next));
}
