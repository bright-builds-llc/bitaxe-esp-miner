#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TerminalSettlementInput {
    pub(super) prior_failure: bool,
    pub(super) serial_finished: bool,
    pub(super) terminal_consumed: bool,
    pub(super) terminal_http_valid: bool,
    pub(super) terminal_websocket_valid: bool,
    pub(super) terminal_deadline_expired: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TerminalSettlementDecision {
    Continue,
    RequestSerialClose,
    AcceptAfterSerialClose,
    PreserveFailureAfterSerialClose,
    FailAfterSerialClose,
}

impl TerminalSettlementDecision {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Continue => "pending",
            Self::RequestSerialClose => "close_requested",
            Self::AcceptAfterSerialClose => "accepted_after_serial_close",
            Self::PreserveFailureAfterSerialClose => "prior_failure_after_serial_close",
            Self::FailAfterSerialClose => "failed_after_serial_close",
        }
    }
}

pub(super) const fn terminal_settlement(
    input: TerminalSettlementInput,
) -> TerminalSettlementDecision {
    if input.serial_finished {
        if input.prior_failure {
            return TerminalSettlementDecision::PreserveFailureAfterSerialClose;
        }
        if input.terminal_consumed && input.terminal_http_valid && input.terminal_websocket_valid {
            return TerminalSettlementDecision::AcceptAfterSerialClose;
        }
        return TerminalSettlementDecision::FailAfterSerialClose;
    }
    if input.prior_failure
        || input.terminal_deadline_expired
        || (input.terminal_consumed && input.terminal_http_valid && input.terminal_websocket_valid)
    {
        return TerminalSettlementDecision::RequestSerialClose;
    }
    TerminalSettlementDecision::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> TerminalSettlementInput {
        TerminalSettlementInput {
            prior_failure: false,
            serial_finished: false,
            terminal_consumed: false,
            terminal_http_valid: false,
            terminal_websocket_valid: false,
            terminal_deadline_expired: false,
        }
    }

    #[test]
    fn complete_transport_quorum_requests_close_before_accepting() {
        // Arrange
        let input = TerminalSettlementInput {
            terminal_consumed: true,
            terminal_http_valid: true,
            terminal_websocket_valid: true,
            ..input()
        };

        // Act / Assert
        assert_eq!(
            terminal_settlement(input),
            TerminalSettlementDecision::RequestSerialClose
        );
    }

    #[test]
    fn final_consumed_handoff_accepts_only_after_serial_close() {
        // Arrange
        let input = TerminalSettlementInput {
            serial_finished: true,
            terminal_consumed: true,
            terminal_http_valid: true,
            terminal_websocket_valid: true,
            ..input()
        };

        // Act / Assert
        assert_eq!(
            terminal_settlement(input),
            TerminalSettlementDecision::AcceptAfterSerialClose
        );
    }

    #[test]
    fn deadline_requests_close_then_fails_from_final_non_consumed_state() {
        // Arrange
        let deadline = TerminalSettlementInput {
            terminal_deadline_expired: true,
            ..input()
        };
        let closed = TerminalSettlementInput {
            serial_finished: true,
            terminal_http_valid: true,
            terminal_websocket_valid: true,
            ..input()
        };

        // Act / Assert
        assert_eq!(
            terminal_settlement(deadline),
            TerminalSettlementDecision::RequestSerialClose
        );
        assert_eq!(
            terminal_settlement(closed),
            TerminalSettlementDecision::FailAfterSerialClose
        );
    }

    #[test]
    fn earlier_failure_is_preserved_after_final_handoff() {
        // Arrange
        let input = TerminalSettlementInput {
            prior_failure: true,
            serial_finished: true,
            terminal_consumed: true,
            terminal_http_valid: true,
            terminal_websocket_valid: true,
            ..input()
        };

        // Act / Assert
        assert_eq!(
            terminal_settlement(input),
            TerminalSettlementDecision::PreserveFailureAfterSerialClose
        );
    }
}
