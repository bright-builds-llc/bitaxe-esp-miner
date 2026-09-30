use bitaxe_virtual_board::memory::{AllocationError, Capability, CapabilityHeap, FailureRule};

#[test]
fn pressure_profile_preserves_measured_total_and_largest() {
    // Arrange
    let heap = CapabilityHeap::observed_pressure();
    // Act
    let facts = heap.snapshot(Capability::Internal);
    // Assert
    assert_eq!((facts.free_bytes, facts.largest_block), (3451, 2176));
}
#[test]
fn total_free_memory_does_not_override_fragmentation() {
    // Arrange
    let mut heap = CapabilityHeap::observed_pressure();
    // Act
    let result = heap.allocate(3000, 1, Capability::Internal, "status");
    // Assert
    assert_eq!(result, Err(AllocationError::Exhausted));
}
#[test]
fn forced_8192_internal_failure_cannot_fallback_to_psram() {
    // Arrange
    let mut heap = CapabilityHeap::observed_pressure();
    // Act
    let result = heap.allocate(8192, 8, Capability::Internal, "task_stack");
    // Assert
    assert_eq!(result, Err(AllocationError::Exhausted));
    assert_eq!(heap.snapshot(Capability::Psram).free_bytes, 8 * 1024 * 1024);
}
#[test]
fn dma_shares_internal_pool_and_can_use_reserve() {
    // Arrange
    let mut heap = CapabilityHeap::new(&[128], 0, 64);
    // Act
    let ordinary = heap.allocate(80, 1, Capability::Default, "reply");
    let dma = heap.allocate(80, 1, Capability::Dma, "uart");
    // Assert
    assert_eq!(ordinary, Err(AllocationError::Exhausted));
    assert_eq!(dma.expect("DMA reserve").capability, Capability::Dma);
    assert_eq!(heap.snapshot(Capability::Internal).free_bytes, 48);
}
#[test]
fn default_routing_uses_psram_when_internal_is_fragmented() {
    // Arrange
    let mut heap = CapabilityHeap::observed_pressure();
    // Act
    let result = heap.allocate(8192, 8, Capability::Default, "status");
    // Assert
    assert_eq!(
        result.expect("PSRAM fallback").capability,
        Capability::Psram
    );
}
#[test]
fn free_coalesces_only_adjacent_actual_regions() {
    // Arrange
    let mut heap = CapabilityHeap::new(&[64], 0, 0);
    let first = heap
        .allocate(16, 1, Capability::Internal, "first")
        .expect("space");
    let second = heap
        .allocate(16, 1, Capability::Internal, "second")
        .expect("space");
    // Act
    heap.free(first).expect("owned ticket");
    heap.free(second).expect("owned ticket");
    // Assert
    assert_eq!(heap.snapshot(Capability::Internal).largest_block, 64);
}
#[test]
fn freeing_cannot_coalesce_across_fragmentation_barrier() {
    // Arrange
    let mut heap = CapabilityHeap::new(&[16, 16], 0, 0);
    let first = heap
        .allocate(16, 1, Capability::Internal, "first")
        .expect("space");
    // Act
    heap.free(first).expect("owned ticket");
    // Assert
    assert_eq!(heap.snapshot(Capability::Internal).largest_block, 16);
}
#[test]
fn ticket_cannot_be_released_twice() {
    // Arrange
    let mut heap = CapabilityHeap::healthy();
    let ticket = heap
        .allocate(16, 8, Capability::Internal, "reply")
        .expect("space");
    heap.free(ticket).expect("owned ticket");
    // Act
    let result = heap.free(ticket);
    // Assert
    assert_eq!(result, Err(AllocationError::UnknownTicket));
}
#[test]
fn combined_failure_rule_matches_exact_phase_size_class_attempt() {
    // Arrange
    let mut heap = CapabilityHeap::healthy();
    heap.failure_rules.push(FailureRule {
        maybe_attempt: Some(2),
        maybe_bytes: Some(8192),
        maybe_capability: Some(Capability::Internal),
        maybe_phase: Some("start".to_owned()),
    });
    heap.allocate(8192, 8, Capability::Internal, "other")
        .expect("first");
    // Act
    let result = heap.allocate(8192, 8, Capability::Internal, "start");
    // Assert
    assert_eq!(result, Err(AllocationError::Injected));
    assert_eq!(heap.live_allocations(), 1);
}
#[test]
fn invalid_alignment_cannot_mutate_heap() {
    // Arrange
    let mut heap = CapabilityHeap::healthy();
    let before = heap.snapshot(Capability::Internal);
    // Act
    let result = heap.allocate(100, 3, Capability::Internal, "reply");
    // Assert
    assert_eq!(result, Err(AllocationError::Invalid));
    assert_eq!(heap.snapshot(Capability::Internal), before);
}

#[test]
fn forced_internal_can_consume_sdk_default_reserve() {
    // Arrange
    let mut heap = CapabilityHeap::new(&[128], 0, 64);
    // Act
    let result = heap.allocate(80, 1, Capability::Internal, "task_stack");
    // Assert
    assert_eq!(
        result.expect("forced internal reserve").capability,
        Capability::Internal
    );
}

#[test]
fn reboot_reclaims_all_capabilities_to_original_fragmented_profile() {
    // Arrange
    let mut heap = CapabilityHeap::new(&[128, 256], 1024, 64);
    let internal_before = heap.snapshot(Capability::Internal);
    let psram_before = heap.snapshot(Capability::Psram);
    heap.allocate(32, 8, Capability::Internal, "stack")
        .expect("internal");
    heap.allocate(32, 8, Capability::Dma, "uart").expect("DMA");
    heap.allocate(128, 8, Capability::Psram, "snapshot")
        .expect("PSRAM");
    // Act
    heap.reset_boot().expect("new epoch");
    // Assert
    assert_eq!(heap.live_allocations(), 0);
    assert_eq!(heap.snapshot(Capability::Internal), internal_before);
    assert_eq!(heap.snapshot(Capability::Psram), psram_before);
}
#[test]
fn old_boot_ticket_cannot_free_new_allocation_with_reused_id() {
    // Arrange
    let mut heap = CapabilityHeap::new(&[128], 0, 0);
    let old = heap
        .allocate(32, 8, Capability::Internal, "old")
        .expect("old");
    heap.reset_boot().expect("new epoch");
    let new = heap
        .allocate(32, 8, Capability::Internal, "new")
        .expect("new");
    let before = heap.snapshot(Capability::Internal);
    // Act
    let result = heap.free(old);
    // Assert
    assert_eq!(result, Err(AllocationError::UnknownTicket));
    assert_eq!(heap.snapshot(Capability::Internal), before);
    heap.free(new).expect("new token remains valid");
}
