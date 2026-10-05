//! Periodic internal-heap sample written only while no Worker session owns the serial link.
//!
//! It exists to diagnose the STR-005 heap loss after a heartbeat-loss shutdown: a passive,
//! receive-only reader can then tell idle loss (allocated blocks rising) from fragmentation
//! (free blocks rising at stable free bytes) without opening the Worker connection under test.

/// One `heap_caps_get_info(MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT)` observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InternalHeapSample {
    pub uptime_ms: u64,
    pub free_bytes: u64,
    pub allocated_bytes: u64,
    pub largest_block_bytes: u64,
    pub minimum_free_bytes: u64,
    pub allocated_blocks: u64,
    pub free_blocks: u64,
    pub revoked: bool,
}

/// Minimum spacing between samples; the line is cheap but the serial link is shared.
pub const INTERNAL_HEAP_SAMPLE_INTERVAL_MS: u64 = 60_000;

/// Renders closed numeric fields only, in the repository's `name schema=v1 … redacted=true` form.
#[must_use]
pub fn internal_heap_sample_marker(sample: &InternalHeapSample) -> String {
    format!(
        "internal_heap_sample schema=v1 uptime_ms={} free_bytes={} allocated_bytes={} \
         largest_block_bytes={} minimum_free_bytes={} allocated_blocks={} free_blocks={} \
         revoked={} redacted=true",
        sample.uptime_ms,
        sample.free_bytes,
        sample.allocated_bytes,
        sample.largest_block_bytes,
        sample.minimum_free_bytes,
        sample.allocated_blocks,
        sample.free_blocks,
        sample.revoked,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_lists_every_field_in_order() {
        // Arrange
        let sample = InternalHeapSample {
            uptime_ms: 665_299,
            free_bytes: 9_687,
            allocated_bytes: 250_000,
            largest_block_bytes: 1_920,
            minimum_free_bytes: 607,
            allocated_blocks: 1_234,
            free_blocks: 56,
            revoked: false,
        };

        // Act
        let marker = internal_heap_sample_marker(&sample);

        // Assert
        assert_eq!(
            marker,
            "internal_heap_sample schema=v1 uptime_ms=665299 free_bytes=9687 \
             allocated_bytes=250000 largest_block_bytes=1920 minimum_free_bytes=607 \
             allocated_blocks=1234 free_blocks=56 revoked=false redacted=true"
        );
    }

    #[test]
    fn marker_is_not_a_retained_worker_diagnostic() {
        // Arrange
        let sample = InternalHeapSample {
            uptime_ms: 1,
            free_bytes: 2,
            allocated_bytes: 3,
            largest_block_bytes: 4,
            minimum_free_bytes: 5,
            allocated_blocks: 6,
            free_blocks: 7,
            revoked: true,
        };

        // Act
        let marker = internal_heap_sample_marker(&sample);

        // Assert
        assert!(!super::super::is_worker_diagnostic_retained_line(&marker));
    }
}
