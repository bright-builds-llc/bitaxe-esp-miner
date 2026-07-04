//! Stratum v1 protocol core.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/stratum_api.c`
//! - `reference/esp-miner/components/stratum/include/stratum_api.h`
//! - Parity checklist row `STR-001`

pub mod coinbase;
pub mod controlled_runtime;
pub mod fake_pool;
pub mod messages;
pub mod mining;
pub mod mining_loop;
pub mod queue;
pub mod state;
