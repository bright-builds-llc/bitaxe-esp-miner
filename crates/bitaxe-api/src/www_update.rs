//! Pure AxeOS static-partition (`/api/system/OTAWWW`) update decisions.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c:POST_WWW_update`
//! - `reference/esp-miner/main/http_server/axe-os/src/app/services/system.service.ts`
//!
//! Upstream erases the whole `www` partition and streams the body over it, so
//! there is no staging copy. One deliberate divergence: upstream rejects only a
//! body larger than the partition, and writes a shorter body at the end of the
//! freshly erased partition, which leaves the UI unmountable. Release `www.bin`
//! images are always exactly partition-sized, so this core admits only an exact
//! size and rejects everything else before any erase.

use crate::route_shell::PublicHttpResponse;

const TEXT_PLAIN: &str = "text/plain";

/// Upstream erase step; erasing in steps lets other tasks and the watchdog run.
pub const WWW_ERASE_STEP_BYTES: usize = 64 * 1024;
/// Upstream receive buffer size.
pub const WWW_RECEIVE_CHUNK_BYTES: usize = 1000;
/// Upstream yields after this many written chunks.
pub const WWW_YIELD_EVERY_CHUNKS: usize = 16;
/// Upstream delay between erase steps and after every yield interval.
pub const WWW_YIELD_MILLIS: u64 = 10;
/// Upstream delay between the success response and the handler's return.
pub const WWW_FINISH_SETTLE_MILLIS: u64 = 1000;

/// Upstream-visible status labels for a static-partition update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WwwUpdateStatus {
    /// Upload accepted; partition lookup and erase are starting.
    Starting,
    /// Upload bytes are being written into the partition.
    Working { percent: u8 },
    /// The HTTP body stream failed before every byte arrived.
    ProtocolError,
    /// ESP-IDF rejected a partition erase or write.
    WriteError,
    /// Every byte was written and the success response was sent.
    Finished,
}

impl WwwUpdateStatus {
    /// Returns the upstream status text.
    #[must_use]
    pub fn status_text(self) -> String {
        match self {
            Self::Starting => "Starting...".to_owned(),
            Self::Working { percent } => format!("Working ({percent}%)"),
            Self::ProtocolError => "Protocol Error".to_owned(),
            Self::WriteError => "Write Error".to_owned(),
            Self::Finished => "Finished...".to_owned(),
        }
    }
}

/// Validated plan for replacing the whole `www` partition with one body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WwwWritePlan {
    size: usize,
}

impl WwwWritePlan {
    /// Partition and body size in bytes; they are equal by construction.
    #[must_use]
    pub const fn size(self) -> usize {
        self.size
    }

    /// Erase ranges as `(offset, length)` covering the whole partition.
    pub fn erase_ranges(self) -> impl Iterator<Item = (usize, usize)> {
        (0..self.size)
            .step_by(WWW_ERASE_STEP_BYTES)
            .map(move |offset| (offset, WWW_ERASE_STEP_BYTES.min(self.size - offset)))
    }

    /// Starts a receive cursor over the admitted body.
    #[must_use]
    pub const fn cursor(self) -> WwwUploadCursor {
        WwwUploadCursor {
            plan: self,
            remaining: self.size,
            chunks: 0,
            maybe_last_percent: None,
        }
    }
}

/// Admits a body for the `www` partition before any erase.
///
/// # Errors
///
/// Returns the public response for a missing partition or a body that is not
/// exactly the partition size.
pub fn admit_www_body(
    content_len: usize,
    maybe_partition_size: Option<usize>,
) -> Result<WwwWritePlan, PublicHttpResponse> {
    let Some(size) = maybe_partition_size else {
        return Err(text_response(500, "WWW partition not found"));
    };
    if content_len > size {
        return Err(text_response(400, "File provided is too large for device"));
    }
    if content_len < size {
        return Err(text_response(400, "File provided is too small for device"));
    }
    Ok(WwwWritePlan { size })
}

/// One `httpd_req_recv` outcome, classified without ESP-IDF types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WwwReceive {
    /// The socket timed out; upstream retries the same read.
    Timeout,
    /// The stream closed or failed with this code.
    Failed(i32),
    /// This many bytes arrived in the buffer.
    Data(usize),
}

/// Next adapter action after a receive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WwwStep {
    /// Retry the read.
    Retry,
    /// Stop with `Protocol Error`; the partition stays partially written.
    ProtocolError { code: i32 },
    /// Write the received bytes at `offset`, then call [`WwwUploadCursor::wrote`].
    Write { offset: usize, len: usize },
}

/// Receive and write position within an admitted body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WwwUploadCursor {
    plan: WwwWritePlan,
    remaining: usize,
    chunks: usize,
    maybe_last_percent: Option<u8>,
}

/// Progress after one written chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WwwChunkWritten {
    /// Status for this chunk.
    pub status: WwwUpdateStatus,
    /// Whether the status differs from the previous chunk's; publishing only
    /// changes keeps a 3 MiB upload to 101 log lines.
    pub status_changed: bool,
    /// Whether the adapter should yield before the next read.
    pub yield_now: bool,
}

impl WwwUploadCursor {
    /// Whether every body byte has been written.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.remaining == 0
    }

    /// Bytes still expected from the body.
    #[must_use]
    pub const fn remaining(self) -> usize {
        self.remaining
    }

    /// Size of the next read request.
    #[must_use]
    pub fn next_read_len(self) -> usize {
        self.remaining.min(WWW_RECEIVE_CHUNK_BYTES)
    }

    /// Classifies a receive into the next adapter action.
    #[must_use]
    pub fn on_receive(self, received: WwwReceive) -> WwwStep {
        match received {
            WwwReceive::Timeout => WwwStep::Retry,
            WwwReceive::Failed(code) => WwwStep::ProtocolError { code },
            WwwReceive::Data(0) => WwwStep::ProtocolError { code: 0 },
            WwwReceive::Data(len) => WwwStep::Write {
                offset: self.plan.size - self.remaining,
                len: len.min(self.remaining),
            },
        }
    }

    /// Records a successful write of `len` bytes from the last receive.
    ///
    /// Upstream computes the percentage before subtracting the chunk, so the
    /// first chunk reports 0% and `Finished...` follows the last chunk.
    pub fn wrote(&mut self, len: usize) -> WwwChunkWritten {
        let percent = 100 - self.remaining.saturating_mul(100) / self.plan.size;
        let percent = u8::try_from(percent).unwrap_or(100);
        self.remaining = self.remaining.saturating_sub(len);
        self.chunks += 1;
        let status_changed = self.maybe_last_percent != Some(percent);
        self.maybe_last_percent = Some(percent);
        WwwChunkWritten {
            status: WwwUpdateStatus::Working { percent },
            status_changed,
            yield_now: self.chunks % WWW_YIELD_EVERY_CHUNKS == 0,
        }
    }
}

/// Public success response after the whole body is written.
#[must_use]
pub const fn www_success_response() -> PublicHttpResponse {
    text_response(200, "WWW update complete\n")
}

/// Public response for a failed body stream.
#[must_use]
pub const fn www_protocol_error_response() -> PublicHttpResponse {
    text_response(500, "Protocol Error")
}

/// Public response for a rejected erase or write.
#[must_use]
pub const fn www_write_error_response() -> PublicHttpResponse {
    text_response(500, "Write Error")
}

const fn text_response(status: u16, body: &'static str) -> PublicHttpResponse {
    PublicHttpResponse {
        status,
        body,
        content_type: Some(TEXT_PLAIN),
    }
}

#[cfg(test)]
mod tests;
