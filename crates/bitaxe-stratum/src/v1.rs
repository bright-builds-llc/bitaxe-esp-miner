//! Stratum v1 protocol core.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/stratum_api.c`
//! - `reference/esp-miner/components/stratum/include/stratum_api.h`
//! - Parity checklist row `STR-001`

pub mod bridge_orchestration;
pub mod coinbase;
pub mod controlled_runtime;
pub mod fake_pool;
pub mod live_runtime;
pub mod messages;
pub mod mining;
pub mod mining_loop;
pub mod production_work;
pub mod queue;
pub mod state;
pub mod submit_response;
pub mod telemetry_projection;
