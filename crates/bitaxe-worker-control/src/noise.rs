//! Bounded network-only Noise qualification evidence and admission types.
mod fence;
mod record;
pub use fence::{EffectFence, MutationGuard};
mod wire;
pub use record::NoiseRecord;
pub use wire::*;

pub const AUTHORITY_US: u64 = 120_000_000;
pub const OBSERVATION_TAIL_US: u64 = 5_000_000;
pub const PREPARATION_US: u64 = 60_000_000;
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[cfg(test)]
mod tests;
