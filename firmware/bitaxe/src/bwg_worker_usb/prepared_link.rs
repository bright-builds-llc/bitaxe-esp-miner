//! Reserves the sole RX thread; no USB read occurs before the original install boundary.
use crate::prepared_thread::{self, Prepared};
use std::io;

pub(super) fn prepare() -> io::Result<Prepared> {
    prepare_with(|| {
        prepared_thread::spawn("bwg-serial-link", 8192, Box::new(|| super::link::run()))
    })
}

fn prepare_with(spawn: impl FnOnce() -> io::Result<Prepared>) -> io::Result<Prepared> {
    spawn()
}

#[cfg(test)]
#[path = "prepared_link_tests.rs"]
mod tests;
