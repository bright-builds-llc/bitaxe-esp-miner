//! Bounded receive-only Worker diagnostics and nonblocking CDC line framing.

const OBSERVER_SETTLE_MS: u64 = 5_100;
const REPORT_WINDOW_MS: u64 = 20_000;
const LINE_INTERVAL_MS: u64 = 100;
/// Maximum diagnostic line length, including its newline.
pub const DIAGNOSTIC_LINE_BYTES: usize = 256;
/// Fixed report slots, including first and last current-boot observations.
pub const DIAGNOSTIC_REPORT_SLOTS: usize = 12;

/// A finite report triggered by ordinary baud/DTR observation, independent of mount time.
#[derive(Debug, Default)]
pub struct WorkerDiagnosticReplay {
    bit_rate: u32,
    dtr: bool,
    maybe_next_ms: Option<u64>,
    expires_ms: u64,
    slot: usize,
}

impl WorkerDiagnosticReplay {
    /// Observes line coding without interpreting CDC payload bytes.
    pub fn line_coding(&mut self, bit_rate: u32, now_ms: u64) {
        let changed = self.bit_rate != bit_rate;
        self.bit_rate = bit_rate;
        if bit_rate != 115_200 {
            self.cancel();
        } else if changed && self.dtr {
            self.begin(now_ms);
        }
    }

    /// Starts only on an observer edge; repeated assertions cannot restart a burst.
    pub fn line_state(&mut self, dtr: bool, now_ms: u64) {
        let rising = dtr && !self.dtr;
        self.dtr = dtr;
        if !dtr {
            self.cancel();
        } else if rising && self.bit_rate == 115_200 {
            self.begin(now_ms);
        }
    }

    /// Returns a due report slot only while the normal Worker ingress remains open.
    pub fn maybe_due_slot(&mut self, now_ms: u64, ingress_open: bool) -> Option<usize> {
        if !ingress_open || now_ms >= self.expires_ms {
            self.cancel();
            return None;
        }
        self.maybe_next_ms
            .filter(|next_ms| now_ms >= *next_ms)
            .map(|_| self.slot)
    }

    /// Advances after a complete accepted line, or a deliberately absent report field.
    pub fn advance(&mut self, now_ms: u64) {
        self.slot += 1;
        self.maybe_next_ms = (self.slot < DIAGNOSTIC_REPORT_SLOTS)
            .then_some(now_ms.saturating_add(LINE_INTERVAL_MS));
    }

    /// Paces retries when the host is not draining the CDC FIFO.
    pub fn retry_later(&mut self, now_ms: u64) {
        self.maybe_next_ms = Some(now_ms.saturating_add(LINE_INTERVAL_MS));
    }

    fn begin(&mut self, now_ms: u64) {
        self.slot = 0;
        self.maybe_next_ms = Some(now_ms.saturating_add(OBSERVER_SETTLE_MS));
        self.expires_ms = now_ms.saturating_add(REPORT_WINDOW_MS);
    }

    fn cancel(&mut self) {
        self.maybe_next_ms = None;
    }
}

/// Minimal CDC transport seam; writes must return the actual accepted byte count.
pub trait CdcEvidenceTransport {
    /// Returns available transmit FIFO bytes without waiting.
    fn available(&self) -> usize;
    /// Enqueues bytes without waiting for a host reader.
    fn write(&mut self, bytes: &[u8]) -> usize;
    /// Requests progress without waiting for transfer completion.
    fn flush(&mut self);
}

/// Preserves line boundaries even when a transport unexpectedly accepts a short write.
#[derive(Debug, Default)]
pub struct CdcEvidenceWriter {
    needs_newline: bool,
}

impl CdcEvidenceWriter {
    /// Creates an empty writer without allocating.
    pub const fn new() -> Self {
        Self {
            needs_newline: false,
        }
    }

    /// Accepts a complete line only when capacity exists; never clears queued receipts.
    pub fn try_emit(&mut self, transport: &mut impl CdcEvidenceTransport, line: &[u8]) -> bool {
        self.try_emit_with_reserve(transport, line, 1)
    }

    /// Leaves space for both maintenance receipts even if the reader stops draining.
    pub fn try_emit_diagnostic(
        &mut self,
        transport: &mut impl CdcEvidenceTransport,
        line: &[u8],
    ) -> bool {
        if line.len() > DIAGNOSTIC_LINE_BYTES {
            return false;
        }
        self.try_emit_with_reserve(transport, line, 128)
    }

    fn try_emit_with_reserve(
        &mut self,
        transport: &mut impl CdcEvidenceTransport,
        line: &[u8],
        reserved: usize,
    ) -> bool {
        if line.is_empty() || line.last() != Some(&b'\n') || line[..line.len() - 1].contains(&b'\n')
        {
            return false;
        }
        if self.needs_newline {
            if transport.available() == 0 || transport.write(b"\n") != 1 {
                transport.flush();
                return false;
            }
            self.needs_newline = false;
        }
        // Reserve a delimiter byte for unexpected short writes. The sole owner
        // serializes all evidence, including maintenance acknowledgments.
        if transport.available().saturating_sub(line.len()) < reserved {
            transport.flush();
            return false;
        }
        let written = transport.write(line);
        if written > 0 && written < line.len() {
            self.needs_newline = transport.write(b"\n") != 1;
        }
        transport.flush();
        written == line.len()
    }
}

/// Selects only exact closed memory/startup fields, never arbitrary retained log text.
#[must_use]
pub fn is_worker_diagnostic_retained_line(line: &str) -> bool {
    let mut fields = line.split(' ');
    match fields.next() {
        Some("usb_memory_checkpoint") => {
            matches!(
                fields.next(),
                Some(
                    "stage=worker_owner_prepare"
                        | "stage=usb_install"
                        | "stage=usb_installed"
                        | "stage=statistics_start"
                        | "stage=statistics_started"
                )
            ) && fields
                .next()
                .is_some_and(|field| decimal_field(field, "free_bytes="))
                && fields
                    .next()
                    .is_some_and(|field| decimal_field(field, "largest_block_bytes="))
                && fields
                    .next()
                    .is_some_and(|field| decimal_field(field, "reserve_bytes="))
                && fields.next() == Some("redacted=true")
                && fields.next().is_none()
        }
        Some("bwg_worker_start_failure") => {
            fields.next() == Some("category=startup_failed")
                && matches!(
                    fields.next(),
                    Some("detail=owner_spawn" | "detail=usb_install" | "detail=control_owner")
                )
                && fields.next() == Some("redacted=true")
                && fields.next().is_none()
        }
        _ => false,
    }
}

fn decimal_field(field: &str, prefix: &str) -> bool {
    field.strip_prefix(prefix).is_some_and(|value| {
        !value.is_empty() && value.len() <= 10 && value.bytes().all(|byte| byte.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests;
