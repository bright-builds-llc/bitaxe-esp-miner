//! First receive failure categories; codes are a stable boot-local marker format.

/// Why the receiver closed a link. Codes are pinned; never renumber a variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SerialLinkFailure {
    Read = 1,
    Framing = 2,
    Envelope = 3,
    Sequence = 4,
    HeartbeatPayload = 5,
    HeartbeatTimeout = 6,
    ControlAllocation = 7,
    ControlQueue = 8,
    UnexpectedKind = 9,
    SessionRevoked = 10,
    Integrity = 11,
    ReceiveCounter = 12,
    FrameAllocation = 13,
}

impl SerialLinkFailure {
    const ALL: [Self; 13] = [
        Self::Read,
        Self::Framing,
        Self::Envelope,
        Self::Sequence,
        Self::HeartbeatPayload,
        Self::HeartbeatTimeout,
        Self::ControlAllocation,
        Self::ControlQueue,
        Self::UnexpectedKind,
        Self::SessionRevoked,
        Self::Integrity,
        Self::ReceiveCounter,
        Self::FrameAllocation,
    ];

    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub fn from_code(code: u32) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|failure| u32::from(failure.code()) == code)
    }

    /// The redaction-safe marker label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Framing => "framing",
            Self::Envelope => "envelope",
            Self::Sequence => "sequence",
            Self::HeartbeatPayload => "heartbeat_payload",
            Self::HeartbeatTimeout => "heartbeat_timeout",
            Self::ControlAllocation => "control_allocation",
            Self::ControlQueue => "control_queue",
            Self::UnexpectedKind => "unexpected_kind",
            Self::SessionRevoked => "session_revoked",
            Self::Integrity => "integrity",
            Self::ReceiveCounter => "receive_counter",
            Self::FrameAllocation => "frame_allocation",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SerialLinkFailure;

    #[test]
    fn codes_and_labels_are_pinned() {
        // Arrange
        let expected = [
            (1, "read"),
            (2, "framing"),
            (3, "envelope"),
            (4, "sequence"),
            (5, "heartbeat_payload"),
            (6, "heartbeat_timeout"),
            (7, "control_allocation"),
            (8, "control_queue"),
            (9, "unexpected_kind"),
            (10, "session_revoked"),
            (11, "integrity"),
            (12, "receive_counter"),
            (13, "frame_allocation"),
        ];
        // Act
        let actual: Vec<(u8, &str)> = SerialLinkFailure::ALL
            .into_iter()
            .map(|failure| (failure.code(), failure.label()))
            .collect();
        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn unknown_codes_have_no_failure() {
        assert_eq!(SerialLinkFailure::from_code(0), None);
        assert_eq!(SerialLinkFailure::from_code(14), None);
        assert_eq!(
            SerialLinkFailure::from_code(11),
            Some(SerialLinkFailure::Integrity)
        );
    }
}
