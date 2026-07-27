use super::retention::{RetainedLogBuffer, LOG_CHUNK_BYTES};

/// Planner for raw `/api/ws` log text broadcasts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLogStreamPlanner {
    next_abs: u64,
    active_clients: usize,
}

impl RawLogStreamPlanner {
    /// Starts the stream cursor at the current log-buffer end.
    #[must_use]
    pub fn new(buffer: &RetainedLogBuffer) -> Self {
        Self {
            next_abs: buffer.total_written(),
            active_clients: 0,
        }
    }

    /// Updates active client count and resets baseline when no clients are present.
    pub fn set_active_client_count(&mut self, active_clients: usize, buffer: &RetainedLogBuffer) {
        let was_inactive = self.active_clients == 0;
        self.active_clients = active_clients;

        if active_clients == 0 || was_inactive {
            self.next_abs = buffer.total_written();
        }
    }

    /// Drains raw text chunks for connected log clients.
    #[must_use]
    pub fn drain_raw_chunks(&mut self, buffer: &RetainedLogBuffer) -> Vec<String> {
        if self.active_clients == 0 {
            self.next_abs = buffer.total_written();
            return Vec::new();
        }

        let mut chunks = Vec::new();
        loop {
            let chunk = buffer.read_absolute_chunk(&mut self.next_abs, LOG_CHUNK_BYTES);
            if chunk.is_empty() {
                return chunks;
            }
            chunks.push(chunk);
        }
    }
}
