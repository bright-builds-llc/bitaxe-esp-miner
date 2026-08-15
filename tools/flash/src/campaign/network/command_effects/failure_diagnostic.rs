use super::super::super::CampaignTerminalCategory;
use super::super::model::{CommandFailureCause, CommandFailureDiagnostic, CommandFailurePhase};
use super::CommandPhase;

impl CommandPhase {
    pub(super) const fn diagnostic_phase(self) -> CommandFailurePhase {
        match self {
            Self::Notification => CommandFailurePhase::Notification,
            #[cfg(test)]
            Self::Pause(_) => CommandFailurePhase::Pause,
            #[cfg(test)]
            Self::ResumeIntent => CommandFailurePhase::ResumeIntent,
            #[cfg(test)]
            Self::ResumeActive => CommandFailurePhase::ResumeActive,
            #[cfg(test)]
            Self::IdentifyReady => CommandFailurePhase::IdentifyStart,
            #[cfg(test)]
            Self::IdentifyRendered { .. }
            | Self::IdentifyReplayPending { .. }
            | Self::IdentifyReplayed { .. }
            | Self::IdentifyObserved { .. } => CommandFailurePhase::IdentifyRendered,
            #[cfg(test)]
            Self::IdentifyCleared => CommandFailurePhase::IdentifyCleared,
            #[cfg(test)]
            Self::PausedDismiss => CommandFailurePhase::Dismiss,
            Self::ProgrammaticPause(_) => CommandFailurePhase::Pause,
            Self::ProgrammaticDismiss => CommandFailurePhase::Dismiss,
            Self::ProgrammaticIdentifyStart => CommandFailurePhase::IdentifyStart,
            Self::ProgrammaticIdentifyRendered => CommandFailurePhase::IdentifyRendered,
            Self::ProgrammaticIdentifyCleared => CommandFailurePhase::IdentifyCleared,
            Self::ProgrammaticResumeIntent => CommandFailurePhase::ResumeIntent,
            Self::ProgrammaticResumeActive => CommandFailurePhase::ResumeActive,
            Self::Terminal => CommandFailurePhase::Terminal,
        }
    }
}

pub(super) fn record_command_failure(
    maybe_failure: &mut Option<CampaignTerminalCategory>,
    maybe_diagnostic: &mut Option<CommandFailureDiagnostic>,
    phase: CommandPhase,
    cause: CommandFailureCause,
    category: CampaignTerminalCategory,
) {
    if maybe_failure.is_some() {
        return;
    }
    *maybe_failure = Some(category);
    *maybe_diagnostic = Some(CommandFailureDiagnostic::new(
        phase.diagnostic_phase(),
        cause,
    ));
}
