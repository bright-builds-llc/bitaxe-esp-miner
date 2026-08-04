//! Stratum v1 protocol core.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/stratum_api.c`
//! - `reference/esp-miner/components/stratum/include/stratum_api.h`
//! - Parity checklist row `STR-001`

pub mod bridge_orchestration;
pub mod coinbase;
mod line_framer;
mod live_runtime;
pub mod messages;
pub mod mining;
pub mod payout_address;
pub mod production_session;
pub mod production_work;
pub mod queue;
mod recovery_policy;
mod share_validation;
pub mod state;
pub mod submit_response;
pub mod telemetry_projection;
