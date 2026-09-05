//! Fixed-size owner-local trace around the unchanged maintenance state machine.

use super::{
    MaintenanceAction, MaintenanceEvent, MaintenancePhase, UsbMaintenanceState, HANDOFF_WINDOW_MS,
};

/// Maximum records retained without a heap allocation.
pub const MAINTENANCE_TRACE_CAPACITY: usize = 16;

/// Closed transport results; raw errors never enter the trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceTraceOutcome {
    None,
    Ok,
    UnavailableTransport,
    Disconnected,
    PartialWrite,
    Timeout,
    Install,
    Handoff,
}

impl MaintenanceTraceOutcome {
    fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ok => "ok",
            Self::UnavailableTransport => "unavailable_transport",
            Self::Disconnected => "disconnected",
            Self::PartialWrite => "partial_write",
            Self::Timeout => "timeout",
            Self::Install => "install",
            Self::Handoff => "handoff",
        }
    }
}

/// Owner-side observations that do not change protocol behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceTraceEffect {
    QueueLoss,
    ReadyEnqueue,
    CommitEnqueue,
    PhyInvoked,
    PhyReturned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceEvent {
    Coding1200,
    Coding115200,
    CodingOther,
    Dtr0Rts0,
    Dtr0Rts1,
    Dtr1Rts0,
    Dtr1Rts1,
    SafeStopComplete,
    SafeStopFailed,
    Detached,
    Expiry,
    Effect(MaintenanceTraceEffect),
}

impl TraceEvent {
    fn label(self) -> &'static str {
        match self {
            Self::Coding1200 => "coding_1200",
            Self::Coding115200 => "coding_115200",
            Self::CodingOther => "coding_other",
            Self::Dtr0Rts0 => "dtr0_rts0",
            Self::Dtr0Rts1 => "dtr0_rts1",
            Self::Dtr1Rts0 => "dtr1_rts0",
            Self::Dtr1Rts1 => "dtr1_rts1",
            Self::SafeStopComplete => "safe_stop_complete",
            Self::SafeStopFailed => "safe_stop_failed",
            Self::Detached => "detached",
            Self::Expiry => "expiry",
            Self::Effect(MaintenanceTraceEffect::QueueLoss) => "queue_loss",
            Self::Effect(MaintenanceTraceEffect::ReadyEnqueue) => "ready_enqueue",
            Self::Effect(MaintenanceTraceEffect::CommitEnqueue) => "commit_enqueue",
            Self::Effect(MaintenanceTraceEffect::PhyInvoked) => "phy_invoked",
            Self::Effect(MaintenanceTraceEffect::PhyReturned) => "phy_returned",
        }
    }
}

impl From<MaintenanceEvent> for TraceEvent {
    fn from(event: MaintenanceEvent) -> Self {
        match event {
            MaintenanceEvent::LineCoding { bit_rate: 1_200 } => Self::Coding1200,
            MaintenanceEvent::LineCoding { bit_rate: 115_200 } => Self::Coding115200,
            MaintenanceEvent::LineCoding { .. } => Self::CodingOther,
            MaintenanceEvent::LineState {
                dtr: false,
                rts: false,
            } => Self::Dtr0Rts0,
            MaintenanceEvent::LineState {
                dtr: false,
                rts: true,
            } => Self::Dtr0Rts1,
            MaintenanceEvent::LineState {
                dtr: true,
                rts: false,
            } => Self::Dtr1Rts0,
            MaintenanceEvent::LineState {
                dtr: true,
                rts: true,
            } => Self::Dtr1Rts1,
            MaintenanceEvent::SafeStopComplete => Self::SafeStopComplete,
            MaintenanceEvent::SafeStopFailed => Self::SafeStopFailed,
            MaintenanceEvent::Detached => Self::Detached,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TraceRecord {
    sequence: u32,
    event: TraceEvent,
    before: MaintenancePhase,
    after: MaintenancePhase,
    action: MaintenanceAction,
    expired: bool,
    remaining_ms: u16,
    outcome: MaintenanceTraceOutcome,
}

impl TraceRecord {
    fn marker(self) -> String {
        format!("usb_maintenance_trace schema=v1 seq={} event={} before={} after={} action={} expired={} remaining_ms={} outcome={} redacted=true",
            self.sequence, self.event.label(), phase_label(self.before), phase_label(self.after), action_label(self.action), self.expired, self.remaining_ms, self.outcome.label())
    }
}

/// Adds bounded observation only; all transitions still use the original state machine.
#[derive(Debug)]
pub struct TracedUsbMaintenanceState {
    state: UsbMaintenanceState,
    records: [Option<TraceRecord>; MAINTENANCE_TRACE_CAPACITY],
    next: usize,
    count: usize,
    sequence: u32,
    effect_failed: bool,
}

impl Default for TracedUsbMaintenanceState {
    fn default() -> Self {
        Self {
            state: UsbMaintenanceState::default(),
            records: [None; MAINTENANCE_TRACE_CAPACITY],
            next: 0,
            count: 0,
            sequence: 0,
            effect_failed: false,
        }
    }
}

impl TracedUsbMaintenanceState {
    /// Records the exact event and its before/after transition, including deadline expiry.
    pub fn observe(&mut self, event: MaintenanceEvent, now_ms: u64) -> MaintenanceAction {
        let before = self.state.phase;
        let remaining_ms = self.remaining_ms(now_ms);
        let expired = self.expired(now_ms);
        let action = self.state.observe(event, now_ms);
        if matches!(
            action,
            MaintenanceAction::RequestSafeStop | MaintenanceAction::CommitRestart
        ) {
            self.effect_failed = false;
        }
        self.append(TraceRecord {
            sequence: 0,
            event: event.into(),
            before,
            after: self.state.phase,
            action,
            expired,
            remaining_ms,
            outcome: MaintenanceTraceOutcome::None,
        });
        action
    }

    /// Records timer expiry once, without filling the ring on ordinary owner ticks.
    pub fn expire(&mut self, now_ms: u64) {
        let before = self.state.phase;
        let expired = self.expired(now_ms);
        self.state.expire(now_ms);
        if expired {
            self.append(TraceRecord {
                sequence: 0,
                event: TraceEvent::Expiry,
                before,
                after: self.state.phase,
                action: MaintenanceAction::None,
                expired: true,
                remaining_ms: 0,
                outcome: MaintenanceTraceOutcome::None,
            });
        }
    }

    /// Records enqueue/PHY results without changing the maintenance protocol or ingress.
    pub fn record_effect(
        &mut self,
        effect: MaintenanceTraceEffect,
        outcome: MaintenanceTraceOutcome,
        now_ms: u64,
    ) {
        if matches!(
            effect,
            MaintenanceTraceEffect::ReadyEnqueue
                | MaintenanceTraceEffect::CommitEnqueue
                | MaintenanceTraceEffect::PhyReturned
        ) && !matches!(
            outcome,
            MaintenanceTraceOutcome::None | MaintenanceTraceOutcome::Ok
        ) {
            self.effect_failed = true;
        }
        self.append(TraceRecord {
            sequence: 0,
            event: TraceEvent::Effect(effect),
            before: self.state.phase,
            after: self.state.phase,
            action: MaintenanceAction::None,
            expired: self.expired(now_ms),
            remaining_ms: self.remaining_ms(now_ms),
            outcome,
        });
    }

    /// Allows a fresh finite diagnostic observer after disarm/failure, without reopening ingress.
    pub fn diagnostics_allowed(&self) -> bool {
        self.effect_failed
            || matches!(
                self.state.phase,
                MaintenancePhase::Idle | MaintenancePhase::DtrAsserted
            )
    }

    /// Builds a single oldest-first retained marker only when replay is permitted.
    pub fn maybe_trace_marker(&self, index: usize) -> Option<String> {
        if !self.diagnostics_allowed() || index >= self.count {
            return None;
        }
        let oldest =
            (self.next + MAINTENANCE_TRACE_CAPACITY - self.count) % MAINTENANCE_TRACE_CAPACITY;
        self.records[(oldest + index) % MAINTENANCE_TRACE_CAPACITY].map(TraceRecord::marker)
    }

    fn append(&mut self, mut record: TraceRecord) {
        self.sequence = self.sequence.saturating_add(1);
        record.sequence = self.sequence;
        self.records[self.next] = Some(record);
        self.next = (self.next + 1) % MAINTENANCE_TRACE_CAPACITY;
        self.count = (self.count + 1).min(MAINTENANCE_TRACE_CAPACITY);
    }

    fn expired(&self, now_ms: u64) -> bool {
        self.state
            .deadline_ms
            .is_some_and(|deadline| now_ms >= deadline)
    }

    fn remaining_ms(&self, now_ms: u64) -> u16 {
        self.state.deadline_ms.map_or(0, |deadline| {
            deadline.saturating_sub(now_ms).min(HANDOFF_WINDOW_MS) as u16
        })
    }
}

fn phase_label(phase: MaintenancePhase) -> &'static str {
    match phase {
        MaintenancePhase::Idle => "idle",
        MaintenancePhase::DtrAsserted => "dtr_asserted",
        MaintenancePhase::SafeStopPending => "safe_stop_pending",
        MaintenancePhase::Ready => "ready",
        MaintenancePhase::Committed => "committed",
    }
}

fn action_label(action: MaintenanceAction) -> &'static str {
    match action {
        MaintenanceAction::None => "none",
        MaintenanceAction::RequestSafeStop => "request_safe_stop",
        MaintenanceAction::EmitReady => "emit_ready",
        MaintenanceAction::CommitRestart => "commit_restart",
    }
}

#[cfg(test)]
mod tests;
