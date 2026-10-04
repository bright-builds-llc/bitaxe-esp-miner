//! Shared production orchestration for physical and functional virtual boards.
//!
//! Authentication, accounting and protocol policy remain in their existing
//! domain crates. This runtime owns identical ordered effects and authority
//! deadlines across adapters, without ESP-IDF or hardware dependencies.

pub mod allocation;
pub mod clock;
pub mod cooling;
pub mod i2c_retry;
pub mod mining_actuation;
pub mod queue;
pub mod reply;
pub mod request_queue;
pub mod revocation;
pub mod shutdown_budget;
