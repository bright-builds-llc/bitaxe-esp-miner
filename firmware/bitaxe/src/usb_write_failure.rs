//! Closed transfer evidence: sizes/timing only, never bytes, session IDs, or errors from callees.
#[derive(Clone, Copy, Debug)]
pub(crate) enum WriteStage {
    Write,
    WriteTimeout,
    FlushTimeout,
}
impl WriteStage {
    fn label(self) -> &'static str {
        match self {
            Self::Write => "write",
            Self::WriteTimeout => "write_timeout",
            Self::FlushTimeout => "flush_timeout",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct WriteFailure {
    pub stage: WriteStage,
    pub elapsed_ms: u64,
    pub queued_bytes: usize,
    pub record_bytes: usize,
}
impl WriteFailure {
    pub fn marker(self) -> String {
        format!("usb_tx_failure schema=v1 stage={} elapsed_ms={} queued_bytes={} record_bytes={} redacted=true",
            self.stage.label(), self.elapsed_ms.min(u64::from(u32::MAX)), self.queued_bytes.min(66560), self.record_bytes.min(66560))
    }
}
impl std::fmt::Display for WriteFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.stage.label())
    }
}
impl std::error::Error for WriteFailure {}
