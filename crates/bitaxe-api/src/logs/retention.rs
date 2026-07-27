use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

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

/// A validated marker/runtime-health pair ready for atomic retained storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedPair {
    marker: String,
    runtime_health: String,
    required_bytes: usize,
}

impl RetainedPair {
    /// Validates two complete single-line records and normalizes one trailing newline each.
    pub fn try_new(marker: &str, runtime_health: &str) -> Result<Self, RetainedPairError> {
        let marker = normalize_retained_record(marker)?;
        let runtime_health = normalize_retained_record(runtime_health)?;
        let required_bytes = checked_retained_pair_bytes(marker.len(), runtime_health.len())?;

        Ok(Self {
            marker,
            runtime_health,
            required_bytes,
        })
    }

    /// Returns the normalized marker record.
    #[must_use]
    pub fn marker(&self) -> &str {
        &self.marker
    }

    /// Returns the normalized runtime-health record.
    #[must_use]
    pub fn runtime_health(&self) -> &str {
        &self.runtime_health
    }

    /// Returns the complete number of bytes required to retain both records.
    #[must_use]
    pub const fn required_bytes(&self) -> usize {
        self.required_bytes
    }
}

/// Closed, redaction-safe retained-pair construction and append failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetainedPairError {
    EmptyRecord,
    EmbeddedLineBreak,
    SizeOverflow,
    StorageUnavailable,
    PairExceedsCapacity,
    CounterOverflow,
}

impl fmt::Display for RetainedPairError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category = match self {
            Self::EmptyRecord => "empty_record",
            Self::EmbeddedLineBreak => "embedded_line_break",
            Self::SizeOverflow => "size_overflow",
            Self::StorageUnavailable => "storage_unavailable",
            Self::PairExceedsCapacity => "pair_exceeds_capacity",
            Self::CounterOverflow => "counter_overflow",
        };
        write!(formatter, "retained_pair={category}")
    }
}

impl Error for RetainedPairError {}

fn normalize_retained_record(record: &str) -> Result<String, RetainedPairError> {
    let record = record.trim_end_matches(['\r', '\n']);
    if record.is_empty() {
        return Err(RetainedPairError::EmptyRecord);
    }
    if record.contains(['\r', '\n']) {
        return Err(RetainedPairError::EmbeddedLineBreak);
    }

    let mut normalized = String::new();
    normalized
        .try_reserve_exact(
            record
                .len()
                .checked_add(1)
                .ok_or(RetainedPairError::SizeOverflow)?,
        )
        .map_err(|_| RetainedPairError::StorageUnavailable)?;
    normalized.push_str(record);
    normalized.push('\n');
    Ok(normalized)
}

pub(super) fn checked_retained_pair_bytes(
    marker_bytes: usize,
    runtime_health_bytes: usize,
) -> Result<usize, RetainedPairError> {
    marker_bytes
        .checked_add(runtime_health_bytes)
        .ok_or(RetainedPairError::SizeOverflow)
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

    /// Creates an empty retained log buffer, returning allocation failure instead of aborting.
    pub fn try_new() -> Result<Self, TryReserveError> {
        Self::try_with_capacity(LOG_RETENTION_BYTES)
    }

    /// Creates an empty retained log buffer with a specific capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            total_written: 0,
        }
    }

    /// Creates an empty retained log buffer with a specific capacity.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(capacity)?;
        buffer.resize(capacity, 0);

        Ok(Self {
            buffer,
            total_written: 0,
        })
    }

    /// Creates an unavailable retained log buffer that drops appended bytes.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            buffer: Vec::new(),
            total_written: 0,
        }
    }

    /// Returns the retained byte capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Appends raw log text to the retained buffer.
    pub fn append(&mut self, text: &str) {
        if self.buffer.is_empty() {
            self.total_written = self.total_written.saturating_add(text.len() as u64);
            return;
        }

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

    /// Atomically admits one complete marker/runtime-health pair.
    pub fn try_append_pair(&mut self, pair: &RetainedPair) -> Result<(), RetainedPairError> {
        if self.buffer.is_empty() {
            return Err(RetainedPairError::StorageUnavailable);
        }
        if pair.required_bytes() > self.buffer.len() {
            return Err(RetainedPairError::PairExceedsCapacity);
        }

        let pair_bytes =
            u64::try_from(pair.required_bytes()).map_err(|_| RetainedPairError::CounterOverflow)?;
        self.total_written
            .checked_add(pair_bytes)
            .ok_or(RetainedPairError::CounterOverflow)?;

        self.append(pair.marker());
        self.append(pair.runtime_health());
        Ok(())
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

    /// Selects complete retained lines whose first whitespace-delimited token
    /// exactly matches `token`.
    ///
    /// The returned lines omit only their line terminator so callers can pass
    /// them directly to a logging facade. Partial trailing lines are ignored.
    #[must_use]
    pub fn complete_lines_with_first_token(&self, token: &str) -> Vec<String> {
        let mut cursor = 0;
        let mut lines = Vec::new();
        let mut discarding_partial_line = false;

        loop {
            let chunk = self.read_absolute_chunk(&mut cursor, LOG_CHUNK_BYTES);
            if chunk.is_empty() {
                return lines;
            }
            if !chunk.ends_with('\n') {
                discarding_partial_line = true;
                continue;
            }
            if discarding_partial_line {
                discarding_partial_line = false;
                continue;
            }

            let line_without_newline = chunk.strip_suffix('\n').unwrap_or(&chunk);
            let line = line_without_newline
                .strip_suffix('\r')
                .unwrap_or(line_without_newline);
            if line.split_whitespace().next() == Some(token) {
                lines.push(line.to_owned());
            }
        }
    }

    fn read_absolute_bytes(&self, cursor: &mut u64, max_len: usize) -> Vec<u8> {
        if max_len == 0 {
            return Vec::new();
        }

        let capacity = self.buffer.len();
        if capacity == 0 {
            *cursor = (*cursor).min(self.total_written);
            return Vec::new();
        }

        let total = self.total_written;
        let mut req_pos = (*cursor).min(total);

        if total >= capacity as u64 && req_pos < total - capacity as u64 {
            req_pos = total - capacity as u64;
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
