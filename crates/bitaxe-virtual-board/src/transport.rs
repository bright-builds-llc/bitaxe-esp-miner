//! Bounded byte transport; authentication and encryption remain production code.
use crate::ModelError;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct VirtualTransport {
    pub connected: bool,
    pub reject_connect: bool,
    pub reject_close: bool,
    pub latency_ms: u64,
    capacity_bytes: usize,
    queued_bytes: usize,
    records: VecDeque<(u64, Vec<u8>)>,
}
impl VirtualTransport {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            connected: false,
            reject_connect: false,
            reject_close: false,
            latency_ms: 0,
            capacity_bytes,
            queued_bytes: 0,
            records: VecDeque::new(),
        }
    }
    pub fn connect(&mut self) -> Result<(), ModelError> {
        if self.reject_connect {
            return Err(ModelError::Unavailable("network connect"));
        }
        self.connected = true;
        Ok(())
    }
    pub fn send(&mut self, now_ms: u64, bytes: &[u8]) -> Result<(), ModelError> {
        if !self.connected {
            return Err(ModelError::Unavailable("transport disconnected"));
        }
        if bytes.len() > self.capacity_bytes.saturating_sub(self.queued_bytes) {
            return Err(ModelError::Unavailable("transport queue full"));
        }
        let ready = now_ms
            .checked_add(self.latency_ms)
            .ok_or(ModelError::Invalid("transport time overflow"))?;
        self.records.push_back((ready, bytes.to_vec()));
        self.queued_bytes += bytes.len();
        Ok(())
    }
    pub fn receive(&mut self, now_ms: u64) -> Result<Option<Vec<u8>>, ModelError> {
        if !self.connected {
            return Err(ModelError::Unavailable("transport disconnected"));
        }
        if self
            .records
            .front()
            .is_none_or(|(ready, _)| *ready > now_ms)
        {
            return Ok(None);
        }
        let Some((_, bytes)) = self.records.pop_front() else {
            return Ok(None);
        };
        self.queued_bytes -= bytes.len();
        Ok(Some(bytes))
    }
    pub fn close(&mut self) -> Result<(), ModelError> {
        if self.reject_close {
            return Err(ModelError::Unavailable("transport close"));
        }
        self.connected = false;
        self.records.clear();
        self.queued_bytes = 0;
        Ok(())
    }
    /// Reboot destroys the old endpoint even when an ordinary Close would reject.
    pub fn reset_boot(&mut self) {
        self.connected = false;
        self.records.clear();
        self.queued_bytes = 0;
    }
    pub fn queued_bytes(&self) -> usize {
        self.queued_bytes
    }
}
