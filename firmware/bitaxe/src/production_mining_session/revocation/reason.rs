#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum RevocationReason {
    NotRevoked = 0,
    HeartbeatTimeout = 1,
    LeaseOrBudgetExpired = 2,
    RestorationRequested = 3,
    UnsafeObservation = 4,
    LinkClosed = 5,
    ControlFailed = 6,
}
impl RevocationReason {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::NotRevoked => "none",
            Self::HeartbeatTimeout => "heartbeat_timeout",
            Self::LeaseOrBudgetExpired => "lease_or_budget_expired",
            Self::RestorationRequested => "restoration_requested",
            Self::UnsafeObservation => "unsafe_observation",
            Self::LinkClosed => "link_closed",
            Self::ControlFailed => "control_failed",
        }
    }
    pub(super) const fn from_code(code: u32) -> Self {
        match code {
            0 => Self::NotRevoked,
            1 => Self::HeartbeatTimeout,
            2 => Self::LeaseOrBudgetExpired,
            3 => Self::RestorationRequested,
            4 => Self::UnsafeObservation,
            5 => Self::LinkClosed,
            _ => Self::ControlFailed,
        }
    }
}
