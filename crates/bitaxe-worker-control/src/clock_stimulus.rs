//! Bounded, boot-local clock-reset stimulus for the BWG-007 `monotonic_reset` scenario.
//!
//! One armed stimulus makes exactly one Worker-control tick observe a clock
//! `OFFSET_MILLISECONDS` below the last observed value, so the production
//! decreasing-clock branch runs. It is RAM-only, arms at most once per boot, and
//! never reaches the native heartbeat, link, revocation or lease deadlines, which
//! keep reading the real uptime clock.

/// How far below the last observed clock the single stimulated sample lands.
pub(crate) const OFFSET_MILLISECONDS: u64 = 1_000;
/// Real-time window after send confirmation during which a tick consumes the stimulus.
pub(crate) const ARMED_FOR_MILLISECONDS: u64 = 2_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StimulusState {
    Idle,
    Armed { expires_real_ms: u64 },
    Consumed,
    Expired,
}

/// Leaving `Idle` is irreversible, which makes the stimulus once per boot.
#[derive(Debug)]
pub(crate) struct ClockStimulus {
    state: StimulusState,
    maybe_pending_token: Option<u64>,
}

impl ClockStimulus {
    pub(crate) const fn new() -> Self {
        Self {
            state: StimulusState::Idle,
            maybe_pending_token: None,
        }
    }

    /// Returns the clock one tick should observe; unchanged unless armed.
    pub(crate) fn sample(&mut self, real_now: u64, maybe_last_observed: Option<u64>) -> u64 {
        let StimulusState::Armed { expires_real_ms } = self.state else {
            return real_now;
        };
        if real_now >= expires_real_ms {
            self.state = StimulusState::Expired;
            return real_now;
        }
        self.state = StimulusState::Consumed;
        maybe_last_observed
            .map_or(real_now, |last| real_now.min(last))
            .saturating_sub(OFFSET_MILLISECONDS)
    }

    /// True only before the boot's single use and with no unconfirmed reply.
    pub(crate) const fn can_prepare(&self) -> bool {
        matches!(self.state, StimulusState::Idle) && self.maybe_pending_token.is_none()
    }

    pub(crate) fn prepare(&mut self, token: u64) {
        self.maybe_pending_token = Some(token);
    }

    pub(crate) fn take_pending(&mut self) -> Option<u64> {
        self.maybe_pending_token.take()
    }

    pub(crate) fn clear_pending(&mut self) {
        self.maybe_pending_token = None;
    }

    /// Arms after true reply delivery; refused unless still idle.
    pub(crate) fn arm(&mut self, real_now: u64) -> bool {
        if !matches!(self.state, StimulusState::Idle) {
            return false;
        }
        self.state = StimulusState::Armed {
            expires_real_ms: real_now.saturating_add(ARMED_FOR_MILLISECONDS),
        };
        true
    }

    /// Any safe stop ends the armed window; the boot's single use stays spent.
    pub(crate) fn cancel(&mut self) {
        self.maybe_pending_token = None;
        if matches!(self.state, StimulusState::Armed { .. }) {
            self.state = StimulusState::Expired;
        }
    }

    pub(crate) const fn state_label(&self) -> &'static str {
        match self.state {
            StimulusState::Idle => "idle",
            StimulusState::Armed { .. } => "armed",
            StimulusState::Consumed => "consumed",
            StimulusState::Expired => "expired",
        }
    }
}

#[cfg(test)]
#[path = "clock_stimulus/tests.rs"]
mod tests;
