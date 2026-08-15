use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum CommandFailurePhase {
    Notification,
    Pause,
    Dismiss,
    IdentifyStart,
    IdentifyRendered,
    IdentifyCleared,
    ResumeIntent,
    ResumeActive,
    Terminal,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum CommandFailureCause {
    SerialWitness,
    PhaseDeadline,
    WebsocketWitness,
    HttpSystemInfo,
    HttpCommandStatus,
    HttpSampleValidation,
    CommandRequest,
    CommandStateMachine,
    TerminalDeadline,
    SerialEnded,
    QuorumIncomplete,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::campaign) struct CommandFailureDiagnostic {
    schema: &'static str,
    phase: CommandFailurePhase,
    cause: CommandFailureCause,
}

impl CommandFailureDiagnostic {
    pub(in crate::campaign) const fn new(
        phase: CommandFailurePhase,
        cause: CommandFailureCause,
    ) -> Self {
        Self {
            schema: "mining-command-failure-diagnostic-v1",
            phase,
            cause,
        }
    }
}
