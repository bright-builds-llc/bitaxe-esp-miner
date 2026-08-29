use crate::LeaseAuthorizationError;

/// Closed, non-secret durable journal for one potentially active mining effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistedWorkerEffectState {
    Clear,
    EffectPending,
    RebootBaselineConfirmed,
}

impl PersistedWorkerEffectState {
    pub fn parse(maybe_value: Option<u8>) -> Result<Self, LeaseAuthorizationError> {
        match maybe_value {
            None => Ok(Self::Clear),
            Some(1) => Ok(Self::EffectPending),
            Some(2) => Ok(Self::RebootBaselineConfirmed),
            Some(_) => Err(LeaseAuthorizationError::Persistence),
        }
    }

    #[must_use]
    pub const fn after_boot_baseline(self) -> Self {
        match self {
            Self::Clear => Self::Clear,
            Self::EffectPending | Self::RebootBaselineConfirmed => Self::RebootBaselineConfirmed,
        }
    }

    #[must_use]
    pub const fn stored_value(self) -> Option<u8> {
        match self {
            Self::Clear => None,
            Self::EffectPending => Some(1),
            Self::RebootBaselineConfirmed => Some(2),
        }
    }

    #[must_use]
    pub const fn requires_reboot_report(self) -> bool {
        matches!(self, Self::RebootBaselineConfirmed)
    }
}
