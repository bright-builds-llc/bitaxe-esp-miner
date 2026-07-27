//! Typed Stratum v1 message parsing and serialization.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/stratum_api.c`
//! - `reference/esp-miner/components/stratum/include/stratum_api.h`
//! - Parity checklist row `STR-001`

mod client;
mod server;

pub use client::StratumV1ClientMessage;
pub use server::{
    parse_server_message, ExtranonceAssignment, MiningNotify, PoolDifficulty, StratumResponse,
    StratumResponseError, StratumV1ServerMessage, VersionMask, MAX_EXTRANONCE_2_LEN,
};

#[cfg(test)]
mod tests;
