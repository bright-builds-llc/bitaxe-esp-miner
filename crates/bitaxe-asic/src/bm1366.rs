//! BM1366 protocol facade.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/bm1366.c`
//! - `reference/esp-miner/components/asic/crc.c`
//! - parity checklist rows `ASIC-001`, `ASIC-002`, and `ASIC-006`

pub mod command;
pub mod crc;
pub mod observation;
pub mod packet;
pub mod registers;
pub mod result;
pub mod work;

pub const BM1366_CHIP_ID: u16 = 0x1366;
pub const BM1366_RESULT_FRAME_LEN: usize = 11;
