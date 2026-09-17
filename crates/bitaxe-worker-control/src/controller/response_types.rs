use std::fmt;
use zeroize::Zeroize;

#[derive(Clone, Debug)]
pub(super) enum PreparedEffect {
    V2Observation {
        generation: u64,
        scope: crate::v2::Scope,
        observation: crate::v2::CurrentObservation,
    },
    V2Dispatch {
        generation: u64,
    },
    NoiseObservation {
        generation: u64,
        observation: crate::noise::NoiseObservation,
    },
    NoiseDispatch {
        generation: u64,
    },
    QualificationRestart {
        generation: u64,
        token: u64,
        context: crate::QualificationRestartContext,
        expires_at_ms: u64,
    },
    Admit {
        generation: u64,
        token: u64,
        established_at_monotonic_milliseconds: u64,
        control_session_binding_sha256: String,
    },
    BootRestorationReported {
        generation: u64,
    },
}

/// Bounded response plus a send-confirmation effect; Debug never includes frame bytes.
pub struct PreparedResponse {
    pub(super) frame: Vec<u8>,
    pub(super) maybe_effect: Option<PreparedEffect>,
}

impl PreparedResponse {
    #[must_use]
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
}

impl fmt::Debug for PreparedResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedResponse")
            .field("frame", &"[redacted]")
            .field("has_effect", &self.maybe_effect.is_some())
            .finish()
    }
}

impl Drop for PreparedResponse {
    fn drop(&mut self) {
        self.frame.zeroize();
    }
}
