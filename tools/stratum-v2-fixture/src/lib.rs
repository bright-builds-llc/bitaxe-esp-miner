//! Pure fixture oracle reusable by host and emulated firmware scenarios.
//! This library has no sockets, files, credentials or physical device access.
pub mod functional;
#[path = "v2_serial/oracle.rs"]
pub mod oracle;
