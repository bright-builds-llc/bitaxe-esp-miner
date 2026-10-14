//! A scripted board for tests: each arrival becomes readable at its time, and time
//! moves only as the client waits.

use crate::exchange::Link;
use bitaxe_core::bap::{BapCommand, BapFrame};
use std::io;

/// A scripted board: each arrival is (time it becomes readable, bytes); time moves
/// only as the client waits.
pub(crate) struct Scripted {
    pub(crate) now: u64,
    arrivals: Vec<(u64, Vec<u8>)>,
    pub(crate) sent: Vec<BapFrame>,
}

impl Scripted {
    pub(crate) fn new(arrivals: &[(u64, &str)]) -> Self {
        Self {
            now: 0,
            arrivals: arrivals
                .iter()
                .map(|(at, text)| (*at, text.as_bytes().to_vec()))
                .collect(),
            sent: Vec::new(),
        }
    }
}

impl Link for Scripted {
    fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.sent
            .push(BapFrame::parse(bytes).expect("canonical outgoing frame"));
        Ok(())
    }

    fn receive(&mut self, wait_ms: u64) -> io::Result<Vec<u8>> {
        let until = self.now + wait_ms;
        let Some(index) = self.arrivals.iter().position(|(at, _)| *at <= until) else {
            self.now = until;
            return Ok(Vec::new());
        };
        let (at, bytes) = self.arrivals.remove(index);
        self.now = self.now.max(at);
        Ok(bytes)
    }

    fn now_ms(&self) -> u64 {
        self.now
    }
}

pub(crate) fn frame(command: BapCommand, token: &str, value: &str) -> String {
    BapFrame::new_token(command, token, Some(value.to_owned()))
        .expect("frame")
        .encode()
        .expect("encode")
}
