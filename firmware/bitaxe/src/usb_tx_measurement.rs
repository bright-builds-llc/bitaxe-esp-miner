//! Size-only boot observations. Access is confined to the sole serial writer task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Category {
    BootstrapDiagnostic,
    Diagnostic,
    DiagnosticReplay,
    Hello,
    Heartbeat,
    ReceiveCredit,
    ControlReply,
    Resynchronization,
}
impl Category {
    fn label(self) -> &'static str {
        match self {
            Self::BootstrapDiagnostic => "bootstrap_diagnostic",
            Self::Diagnostic => "diagnostic",
            Self::DiagnosticReplay => "diagnostic_replay",
            Self::Hello => "hello",
            Self::Heartbeat => "heartbeat",
            Self::ReceiveCredit => "receive_credit",
            Self::ControlReply => "control_reply",
            Self::Resynchronization => "resynchronization",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum RecordKind {
    StartupProgress,
    StatisticsStartup,
    BootIdentity,
    Admission,
    RetainedDiagnostic,
    TxObservation,
    Protocol,
    Resynchronization,
}
impl RecordKind {
    fn label(self) -> &'static str {
        match self {
            Self::StartupProgress => "startup_progress",
            Self::StatisticsStartup => "statistics_startup",
            Self::BootIdentity => "boot_identity",
            Self::Admission => "admission",
            Self::RetainedDiagnostic => "retained_diagnostic",
            Self::TxObservation => "tx_observation",
            Self::Protocol => "protocol",
            Self::Resynchronization => "resynchronization",
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Measurement {
    category: Category,
    kind: RecordKind,
    start: u64,
    end: u64,
    bytes: u32,
    queued: u32,
    queue_calls: u32,
    positive: u32,
    zero: u32,
    last_queue: i32,
    drains: u32,
    timeout: u32,
    success: u32,
    other: u32,
    first_drain: u64,
    last_drain: u64,
    max_drain: u64,
    last_return: i32,
    stage: &'static str,
    flags: u32,
}
impl Measurement {
    pub fn new(category: Category, kind: RecordKind, bytes: usize, start: u64) -> Self {
        Self {
            category,
            kind,
            start,
            end: start,
            bytes: bytes.min(u32::MAX as usize) as u32,
            queued: 0,
            queue_calls: 0,
            positive: 0,
            zero: 0,
            last_queue: 0,
            drains: 0,
            timeout: 0,
            success: 0,
            other: 0,
            first_drain: 0,
            last_drain: 0,
            max_drain: 0,
            last_return: 0,
            stage: "completed",
            flags: if bytes > u32::MAX as usize { 1 } else { 0 },
        }
    }
    pub fn time(&mut self, at: u64) {
        if at < self.end {
            self.flags |= 2;
        }
        self.end = at;
    }
    pub fn queue(&mut self, result: i32, limit: usize, at: u64) {
        self.time(at);
        bump(&mut self.queue_calls, &mut self.flags);
        self.last_queue = result;
        if result > 0 && result as usize > limit {
            self.flags |= 4;
            return;
        }
        if result > 0 {
            bump(&mut self.positive, &mut self.flags);
            match self.queued.checked_add(result as u32) {
                Some(sum) => self.queued = sum,
                None => {
                    self.queued = u32::MAX;
                    self.flags |= 1;
                }
            }
        } else if result == 0 {
            bump(&mut self.zero, &mut self.flags);
        }
    }
    pub fn invalid(&self) -> bool {
        self.flags != 0
    }
    pub fn drain(&mut self, result: i32, start: u64, end: u64) {
        self.time(start);
        self.time(end);
        if self.drains == 0 {
            self.first_drain = start;
        }
        self.last_drain = end;
        self.max_drain = self.max_drain.max(end.saturating_sub(start));
        bump(&mut self.drains, &mut self.flags);
        self.last_return = result;
        if result == 0 {
            bump(&mut self.success, &mut self.flags);
        } else if result == 0x107 {
            bump(&mut self.timeout, &mut self.flags);
        } else {
            bump(&mut self.other, &mut self.flags);
            self.flags |= 4;
        }
    }
    pub fn finish(mut self, retained: &mut Retained, stage: &'static str, at: u64) {
        self.time(at);
        self.stage = stage;
        retained.record(self);
    }
}
fn bump(value: &mut u32, flags: &mut u32) {
    if *value == u32::MAX {
        *flags |= 1;
    } else {
        *value += 1;
    }
}
pub(crate) struct Retained {
    bootstrap: Option<Measurement>,
    failure: Option<Measurement>,
    failures: u32,
    attempts: u32,
    completed: u32,
    flags: u32,
    next: bool,
}
impl Retained {
    pub const fn new() -> Self {
        Self {
            bootstrap: None,
            failure: None,
            failures: 0,
            attempts: 0,
            completed: 0,
            flags: 0,
            next: false,
        }
    }
    pub fn begin(&mut self, kind: RecordKind) {
        if matches!(kind, RecordKind::TxObservation) {
            bump(&mut self.attempts, &mut self.flags);
        }
    }
    fn record(&mut self, m: Measurement) {
        if m.category == Category::BootstrapDiagnostic && self.bootstrap.is_none() {
            self.bootstrap = Some(m);
        }
        if m.stage != "completed" {
            bump(&mut self.failures, &mut self.flags);
            if self.failure.is_none() {
                self.failure = Some(m);
            }
        }
        if matches!(m.kind, RecordKind::TxObservation) {
            if m.stage == "completed" {
                bump(&mut self.completed, &mut self.flags);
            }
        }
    }
    pub fn marker(&mut self) -> Option<String> {
        let (slot, maybe) = if self.next {
            ("first_failure", self.failure)
        } else {
            ("first_bootstrap", self.bootstrap)
        };
        self.next = !self.next;
        let m = maybe?;
        Some(format!("usb_tx_observation schema=v1 slot={} category={} record_kind={} outcome={} stage={} start_ms={} end_ms={} record_bytes={} queued_bytes={} queue_calls={} queue_positive={} queue_zero={} last_queue_return={} drain_calls={} drain_timeout={} drain_success={} drain_other={} first_drain_ms={} last_drain_ms={} max_drain_ms={} last_drain_return={} actual_failures={} replay_attempts={} replay_completed={} flags={} redacted=true",slot,m.category.label(),m.kind.label(),if m.stage=="completed" {"completed"} else {"failed"},m.stage,m.start,m.end,m.bytes,m.queued,m.queue_calls,m.positive,m.zero,m.last_queue,m.drains,m.timeout,m.success,m.other,m.first_drain,m.last_drain,m.max_drain,m.last_return,self.failures,self.attempts,self.completed,m.flags|self.flags))
    }
}
const _: () = assert!(std::mem::size_of::<Measurement>() <= 128);
const _: () = assert!(std::mem::size_of::<Retained>() <= 512);

#[cfg(test)]
mod tests {
    use super::*;
    fn observation(category: Category, kind: RecordKind, stage: &'static str) -> Measurement {
        let mut m = Measurement::new(category, kind, 92, 0);
        m.queue(92, 92, 1);
        m.drain(0x107, 1, 2000);
        m.stage = stage;
        m
    }
    #[test]
    fn first_bootstrap_and_failure_survive_hello_and_successful_replay() {
        // Arrange
        let mut r = Retained::new();
        let original = observation(
            Category::BootstrapDiagnostic,
            RecordKind::StartupProgress,
            "flush_timeout",
        );
        // Act
        r.record(original);
        r.record(observation(
            Category::Hello,
            RecordKind::Protocol,
            "completed",
        ));
        r.begin(RecordKind::TxObservation);
        r.record(observation(
            Category::DiagnosticReplay,
            RecordKind::TxObservation,
            "completed",
        ));
        r.begin(RecordKind::TxObservation);
        r.record(observation(
            Category::DiagnosticReplay,
            RecordKind::TxObservation,
            "flush_timeout",
        ));
        // Assert
        assert_eq!(r.failures, 2);
        assert_eq!(r.attempts, 2);
        assert_eq!(r.completed, 1);
        assert_eq!(
            r.bootstrap.expect("bootstrap").category,
            Category::BootstrapDiagnostic
        );
        assert_eq!(
            r.failure.expect("first failure").category,
            Category::BootstrapDiagnostic
        );
        let first = r.marker().expect("bootstrap marker");
        let second = r.marker().expect("failure marker");
        assert!(first.contains("slot=first_bootstrap"));
        assert!(second.contains("slot=first_failure"));
        assert!(first.len() <= 1024);
        assert!(second.len() <= 1024);
    }
    #[test]
    fn counter_clock_and_invalid_return_flags_are_explicit() {
        // Arrange
        let mut m = Measurement::new(
            Category::BootstrapDiagnostic,
            RecordKind::BootIdentity,
            92,
            10,
        );
        m.queue_calls = u32::MAX;
        // Act
        m.queue(93, 92, 9);
        m.drain(-1, 9, 8);
        // Assert
        assert_eq!(m.flags, 7);
        assert_eq!(m.queue_calls, u32::MAX);
        assert_eq!(m.other, 1);
    }
    #[test]
    fn no_drain_and_zero_uptime_drain_are_distinguished_by_count() {
        // Arrange
        let mut m = Measurement::new(Category::Diagnostic, RecordKind::Admission, 92, 0);
        // Act
        let before = m;
        m.drain(0, 0, 0);
        // Assert
        assert_eq!(before.drains, 0);
        assert_eq!(m.drains, 1);
        assert_eq!(m.first_drain, 0);
        assert_eq!(m.last_drain, 0);
    }
    #[test]
    fn maximum_numeric_marker_fits_bound() {
        // Arrange
        let mut r = Retained::new();
        let mut m = observation(
            Category::BootstrapDiagnostic,
            RecordKind::StatisticsStartup,
            "flush_timeout",
        );
        m.start = u64::MAX;
        m.end = u64::MAX;
        m.first_drain = u64::MAX;
        m.last_drain = u64::MAX;
        m.max_drain = u64::MAX;
        m.bytes = u32::MAX;
        m.queued = u32::MAX;
        m.queue_calls = u32::MAX;
        m.positive = u32::MAX;
        m.zero = u32::MAX;
        m.last_queue = i32::MIN;
        m.drains = u32::MAX;
        m.timeout = u32::MAX;
        m.success = u32::MAX;
        m.other = u32::MAX;
        m.last_return = i32::MIN;
        // Act
        r.record(m);
        r.failures = u32::MAX;
        r.attempts = u32::MAX;
        r.completed = u32::MAX;
        // Assert
        assert!(r.marker().expect("marker").len() <= 1024);
    }
    #[test]
    fn queued_byte_overflow_and_impossible_acceptance_are_not_silent() {
        // Arrange
        let mut m = Measurement::new(Category::Diagnostic, RecordKind::Protocol, 92, 0);
        m.queued = u32::MAX;
        // Act
        m.queue(1, 1, 0);
        let before = m.queued;
        m.queue(93, 92, 0);
        // Assert
        assert_eq!(m.flags, 5);
        assert_eq!(m.queued, before);
        assert_eq!(m.positive, 1);
    }
    #[test]
    fn replay_attempt_is_visible_before_terminal_confirmation() {
        // Arrange
        let mut r = Retained::new();
        // Act
        r.begin(RecordKind::TxObservation);
        // Assert
        assert_eq!(r.attempts, 1);
        assert_eq!(r.completed, 0);
        assert_eq!(r.failures, 0);
    }
}
