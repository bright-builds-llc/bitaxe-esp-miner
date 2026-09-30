//! Capability-aware allocation boundary shared by resource adapters.
//!
//! Tokens represent actual adapter reservations, not evidence that a modeled
//! budget matches ESP-IDF. Real allocator observations and model predictions
//! must remain distinct in scenario reports.

/// Closed allocation classes used by current Ultra 205 runtime resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationClass {
    /// Ordinary 8-bit heap allocation with resolved SDK routing preferences.
    Default8Bit,
    /// Forced internal 8-bit allocation, including task stacks.
    Internal8Bit,
    /// Forced internal DMA-capable 8-bit allocation.
    InternalDma8Bit,
    /// Forced external PSRAM 8-bit allocation.
    Psram8Bit,
}

/// One bounded request with a stable caller-owned phase discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationRequest {
    pub bytes: usize,
    pub alignment: usize,
    pub class: AllocationClass,
    pub phase: &'static str,
}

/// Current adapter facts for one capability class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationSnapshot {
    pub free_bytes: usize,
    pub largest_block_bytes: usize,
    pub minimum_free_bytes: usize,
}

/// Adapter-owned capability reservation and release.
///
/// Virtual tokens bind heap segments; physical tokens must own actual memory.
/// A rejected allocation never grants a token or fabricates a successful phase.
pub trait CapabilityAllocation {
    type Token;
    type Error;

    /// Reserves one request or returns the actual rejection.
    fn allocate(&mut self, request: AllocationRequest) -> Result<Self::Token, Self::Error>;
    /// Releases exactly the reservation represented by this token.
    fn release(&mut self, token: Self::Token) -> Result<(), Self::Error>;
    /// Returns current observations without allocating or changing heap state.
    fn snapshot(&self, class: AllocationClass) -> AllocationSnapshot;
}
