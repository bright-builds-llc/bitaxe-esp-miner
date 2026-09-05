//! Logical channel liveness is independent of mining permission and cooling.

use super::HEARTBEAT_TIMEOUT_MILLISECONDS;

/// A fresh possession authenticates heartbeat ownership until transport closure.
#[derive(Debug)]
pub struct SerialLinkLiveness {
    last_observation: u64,
    last_heartbeat: u64,
    authenticated: bool,
    closed: bool,
}

impl SerialLinkLiveness {
    #[must_use]
    pub const fn new(now: u64) -> Self {
        Self {
            last_observation: now,
            last_heartbeat: now,
            authenticated: false,
            closed: false,
        }
    }

    /// Possession confirmation authenticates the channel, never reopens a closed channel.
    pub fn authenticate(&mut self) {
        if !self.closed {
            self.authenticated = true;
        }
    }

    /// Checks the independent transport deadline, including clock continuity.
    pub fn poll(&mut self, now: u64) -> bool {
        if self.closed
            || now < self.last_observation
            || now.saturating_sub(self.last_heartbeat) >= HEARTBEAT_TIMEOUT_MILLISECONDS
        {
            self.closed = true;
            self.authenticated = false;
            return false;
        }
        self.last_observation = now;
        true
    }

    /// Only the caller's validated, advancing heartbeat may refresh this clock.
    pub fn heartbeat(&mut self, now: u64) -> bool {
        if !self.poll(now) || !self.authenticated {
            return false;
        }
        self.last_heartbeat = now;
        true
    }

    #[must_use]
    pub const fn is_authenticated(&self) -> bool {
        self.authenticated && !self.closed
    }
}
