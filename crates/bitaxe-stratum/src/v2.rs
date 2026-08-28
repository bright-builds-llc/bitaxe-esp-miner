//! Bounded Stratum V2 protocol core for the pinned ESP-Miner subset.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum_v2/sv2_protocol.c`
//! - `reference/esp-miner/components/stratum_v2/sv2_noise.c`
//! - `reference/esp-miner/main/tasks/stratum_v2_task.c`

pub mod authority;
pub mod connection_order;
pub mod frame;
pub mod messages;
pub mod noise;
pub mod session;
pub mod work;

mod error;

pub use error::StratumV2Error;

pub const PROTOCOL_VERSION: u16 = 2;
pub const MINING_PROTOCOL: u8 = 0;
pub const CHANNEL_MESSAGE_FLAG: u16 = 0x8000;
pub const MAX_FRAME_PAYLOAD: usize = 2_048;
pub const MAX_MERKLE_BRANCHES: usize = 20;
pub const PENDING_JOB_CAPACITY: usize = 8;
