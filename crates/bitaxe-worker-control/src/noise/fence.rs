use std::sync::atomic::{AtomicU32, Ordering};
const DIAGNOSTIC: u32 = 1 << 31;
/// Shared exclusion for native diagnostic and ordinary configuration/effect owners.
pub struct EffectFence(AtomicU32);
impl Default for EffectFence {
    fn default() -> Self {
        Self::new()
    }
}
impl EffectFence {
    pub const fn new() -> Self {
        Self(AtomicU32::new(0))
    }
    pub fn busy(&self) -> bool {
        self.0.load(Ordering::Acquire) == DIAGNOSTIC
    }
    pub fn claim_diagnostic(&self) -> bool {
        self.0
            .compare_exchange(0, DIAGNOSTIC, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    pub fn release_diagnostic(&self) -> bool {
        self.0
            .compare_exchange(DIAGNOSTIC, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    pub fn maybe_mutation(&self) -> Option<MutationGuard<'_>> {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |state| {
                (state < DIAGNOSTIC - 1).then_some(state + 1)
            })
            .ok()
            .map(|_| MutationGuard(self))
    }
}
pub struct MutationGuard<'a>(&'a EffectFence);
impl Drop for MutationGuard<'_> {
    fn drop(&mut self) {
        self.0 .0.fetch_sub(1, Ordering::AcqRel);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutation_ownership_blocks_diagnostic_until_the_last_deferred_owner_drops() {
        // Arrange
        let fence = EffectFence::new();
        let first = fence.maybe_mutation().expect("first owner");
        let second = fence.maybe_mutation().expect("deferred owner");
        // Act / Assert
        assert!(!fence.claim_diagnostic());
        drop(first);
        assert!(!fence.claim_diagnostic());
        drop(second);
        assert!(fence.claim_diagnostic());
        assert!(fence.maybe_mutation().is_none());
        assert!(fence.release_diagnostic());
        assert!(fence.maybe_mutation().is_some());
    }
}
