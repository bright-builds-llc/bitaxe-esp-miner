//! Fixed Standard-channel qualification inputs and bounded retained evidence.
mod input;
mod record;
mod release;
pub use release::ShareRelease;
mod wire;
pub use input::*;
pub use record::{ObservedAcknowledgement, V2Record};
pub use wire::*;
pub const AUTHORITY_US: u64 = 120_000_000;
pub const OBSERVATION_US: u64 = 125_000_000;
pub const MAX_EVENTS: usize = 64;
pub const MAX_SHARES: usize = 16;
pub const MAX_FAILURES: usize = 16;

#[cfg(test)]
mod corpus_tests;
#[cfg(test)]
mod encoding_tests;
#[cfg(test)]
mod tests;
