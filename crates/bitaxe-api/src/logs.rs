//! Pure retained log download and raw `/api/ws` stream contracts.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/log_buffer.c`
//! - `reference/esp-miner/main/log_buffer.h`
//! - `reference/esp-miner/main/http_server/websocket_log.c`

/// Upstream retained log buffer size: 512 KiB.
pub const LOG_RETENTION_BYTES: usize = 512 * 1024;
/// Upstream log read and WebSocket chunk size.
pub const LOG_CHUNK_BYTES: usize = 4096;
/// Log download content type.
pub const DOWNLOAD_CONTENT_TYPE: &str = "text/plain";
/// Log download file name header.
pub const DOWNLOAD_CONTENT_DISPOSITION: &str = "attachment; filename=\"bitaxe-logs.txt\"";

/// Download response headers expected by existing AxeOS clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogDownloadHeaders {
    pub content_type: &'static str,
    pub content_disposition: &'static str,
}

/// Returns the upstream-compatible log download headers.
#[must_use]
pub const fn log_download_headers() -> LogDownloadHeaders {
    LogDownloadHeaders {
        content_type: DOWNLOAD_CONTENT_TYPE,
        content_disposition: DOWNLOAD_CONTENT_DISPOSITION,
    }
}

/// Bounded host-testable retained log buffer model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedLogBuffer {
    buffer: Vec<u8>,
    total_written: u64,
}

impl RetainedLogBuffer {
    /// Creates an empty retained log buffer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: vec![0; LOG_RETENTION_BYTES],
            total_written: 0,
        }
    }

    /// Appends raw log text to the retained buffer.
    pub fn append(&mut self, text: &str) {
        let mut remaining = text.as_bytes();
        while !remaining.is_empty() {
            let write_offset = self.total_written as usize % self.buffer.len();
            let till_end = self.buffer.len() - write_offset;
            let write_len = remaining.len().min(till_end);
            self.buffer[write_offset..write_offset + write_len]
                .copy_from_slice(&remaining[..write_len]);
            self.total_written += write_len as u64;
            remaining = &remaining[write_len..];
        }
    }

    /// Returns total bytes ever written to the absolute log stream.
    #[must_use]
    pub fn total_written(&self) -> u64 {
        self.total_written
    }

    /// Reads retained log text from an absolute cursor.
    pub fn read_absolute_chunk(&self, cursor: &mut u64, max_len: usize) -> String {
        let bytes = self.read_absolute_bytes(cursor, max_len);
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// Returns retained download chunks from the absolute beginning plus an empty terminal chunk.
    #[must_use]
    pub fn download_chunks(&self) -> Vec<String> {
        let mut cursor = 0;
        let mut chunks = Vec::new();

        loop {
            let chunk = self.read_absolute_chunk(&mut cursor, LOG_CHUNK_BYTES);
            let is_terminal = chunk.is_empty();
            chunks.push(chunk);

            if is_terminal {
                return chunks;
            }
        }
    }

    fn read_absolute_bytes(&self, cursor: &mut u64, max_len: usize) -> Vec<u8> {
        if max_len == 0 {
            return Vec::new();
        }

        let total = self.total_written;
        let mut req_pos = (*cursor).min(total);

        if total >= LOG_RETENTION_BYTES as u64 && req_pos < total - LOG_RETENTION_BYTES as u64 {
            req_pos = total - LOG_RETENTION_BYTES as u64;
            req_pos = self.resync_to_next_line(req_pos, total);
        }

        let available = total.saturating_sub(req_pos) as usize;
        let mut to_read = available.min(max_len);

        if to_read == 0 {
            *cursor = req_pos;
            return Vec::new();
        }

        if let Some(newline_idx) = self.first_newline_offset(req_pos, to_read) {
            to_read = newline_idx + 1;
        }

        let bytes = (0..to_read)
            .map(|offset| self.byte_at(req_pos + offset as u64))
            .collect::<Vec<_>>();
        *cursor = req_pos + to_read as u64;
        bytes
    }

    fn resync_to_next_line(&self, req_pos: u64, total: u64) -> u64 {
        let available_scan = total.saturating_sub(req_pos).min(LOG_CHUNK_BYTES as u64) as usize;

        for offset in 0..available_scan {
            if self.byte_at(req_pos + offset as u64) == b'\n' {
                return req_pos + offset as u64 + 1;
            }
        }

        req_pos
    }

    fn first_newline_offset(&self, req_pos: u64, to_read: usize) -> Option<usize> {
        (0..to_read).find(|offset| self.byte_at(req_pos + *offset as u64) == b'\n')
    }

    fn byte_at(&self, abs_pos: u64) -> u8 {
        self.buffer[abs_pos as usize % self.buffer.len()]
    }
}

impl Default for RetainedLogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

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
            return;
        }

        if self.next_abs > buffer.total_written() {
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

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::logs::{
        log_download_headers, RawLogStreamPlanner, RetainedLogBuffer, DOWNLOAD_CONTENT_DISPOSITION,
        DOWNLOAD_CONTENT_TYPE, LOG_CHUNK_BYTES, LOG_RETENTION_BYTES,
    };

    #[derive(Debug, Deserialize)]
    struct LogFixtureCases {
        download_headers: HeaderFixture,
        raw_stream: RawStreamFixture,
    }

    #[derive(Debug, Deserialize)]
    struct HeaderFixture {
        content_type: String,
        content_disposition: String,
    }

    #[derive(Debug, Deserialize)]
    struct RawStreamFixture {
        payload: String,
        json_enveloped: bool,
    }

    fn fixture_cases() -> LogFixtureCases {
        serde_json::from_str(include_str!("../fixtures/api/log-buffer-cases.json"))
            .expect("log fixture cases should parse")
    }

    #[test]
    fn retained_download_uses_text_headers_chunks_from_beginning_and_empty_terminal_chunk() {
        // Arrange
        let fixture = fixture_cases();
        let mut buffer = RetainedLogBuffer::new();
        buffer.append(&"a".repeat(LOG_CHUNK_BYTES));
        buffer.append("tail");

        // Act
        let headers = log_download_headers();
        let chunks = buffer.download_chunks();

        // Assert
        assert_eq!(headers.content_type, DOWNLOAD_CONTENT_TYPE);
        assert_eq!(headers.content_disposition, DOWNLOAD_CONTENT_DISPOSITION);
        assert_eq!(headers.content_type, fixture.download_headers.content_type);
        assert_eq!(
            headers.content_disposition,
            fixture.download_headers.content_disposition
        );
        assert_eq!(chunks[0].len(), LOG_CHUNK_BYTES);
        assert_eq!(chunks[1], "tail");
        assert_eq!(chunks[2], "");
    }

    #[test]
    fn clamped_readers_resync_to_next_newline_within_bounded_scan() {
        // Arrange
        let mut buffer = RetainedLogBuffer::new();
        let discarded_prefix = "discarded-";
        let retained_prefix = "stale partial line\nkept line\n";
        let tail = "x".repeat(LOG_RETENTION_BYTES - retained_prefix.len());
        buffer.append(discarded_prefix);
        buffer.append(retained_prefix);
        buffer.append(&tail);
        let mut cursor = 0;

        // Act
        let chunk = buffer.read_absolute_chunk(&mut cursor, LOG_CHUNK_BYTES);

        // Assert
        assert!(!chunk.starts_with("stale partial line"));
        assert!(chunk.starts_with("kept line\n"));
    }

    #[test]
    fn raw_ws_client_baseline_starts_at_current_end_not_retained_history() {
        // Arrange
        let mut buffer = RetainedLogBuffer::new();
        buffer.append("retained old line\n");
        let mut stream = RawLogStreamPlanner::new(&buffer);
        stream.set_active_client_count(1, &buffer);
        buffer.append("new live line\n");

        // Act
        let chunks = stream.drain_raw_chunks(&buffer);

        // Assert
        assert_eq!(chunks, vec!["new live line\n"]);
    }

    #[test]
    fn raw_ws_hibernates_without_clients_and_sends_no_backlog_to_later_clients() {
        // Arrange
        let mut buffer = RetainedLogBuffer::new();
        let mut stream = RawLogStreamPlanner::new(&buffer);
        stream.set_active_client_count(0, &buffer);
        buffer.append("not delivered while idle\n");
        let idle_chunks = stream.drain_raw_chunks(&buffer);
        stream.set_active_client_count(1, &buffer);

        // Act
        let reconnect_chunks = stream.drain_raw_chunks(&buffer);

        // Assert
        assert!(idle_chunks.is_empty());
        assert!(reconnect_chunks.is_empty());
    }

    #[test]
    fn raw_ws_chunks_are_text_payloads_without_json_envelope() {
        // Arrange
        let fixture = fixture_cases();
        let mut buffer = RetainedLogBuffer::new();
        let mut stream = RawLogStreamPlanner::new(&buffer);
        stream.set_active_client_count(1, &buffer);
        buffer.append(&fixture.raw_stream.payload);

        // Act
        let chunks = stream.drain_raw_chunks(&buffer);

        // Assert
        assert_eq!(chunks, vec![fixture.raw_stream.payload]);
        assert!(!fixture.raw_stream.json_enveloped);
        assert!(!chunks[0].trim_start().starts_with('{'));
    }
}
