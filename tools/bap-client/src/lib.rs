//! A host client for the Bitaxe Accessory Port (BAP) over a USB-to-UART adapter.
//!
//! The exchange is pure and reuses the firmware's own BAP framing
//! (`bitaxe_core::bap`); the serial adapter is a thin shell around it. Sensitive
//! values are never printed and never accepted on the command line.

pub mod binding;
pub mod evidence;
pub mod exchange;
pub mod policy;
pub mod port;
pub mod probe;
pub mod redaction;
pub mod secret;
#[cfg(test)]
pub(crate) mod test_link;
