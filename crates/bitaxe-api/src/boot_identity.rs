//! Pure boot-identity persistence and replay planning.

/// Magic value identifying an initialized Bitaxe RTC boot record.
pub const BOOT_RECORD_MAGIC: u32 = 0x4258_5254;
/// Schema version for the persisted RTC boot record.
pub const BOOT_RECORD_SCHEMA_VERSION: u32 = 1;
/// Cadence for replaying non-secret boot identity evidence.
pub const BOOT_EVIDENCE_INTERVAL_MS: u64 = 10_000;
/// Duration for replaying a connected device origin after publication.
pub const ORIGIN_REPLAY_WINDOW_MS: u64 = 360_000;

/// Reset-retained boot counter with integrity fields for fail-closed continuity.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcBootRecord {
    /// Record identity marker.
    pub magic: u32,
    /// Record schema version.
    pub schema_version: u32,
    /// Monotonic boot count within one retained-power epoch.
    pub ordinal: u64,
    /// Bitwise complement used to detect torn ordinal writes.
    pub ordinal_complement: u64,
    /// Checksum over all preceding fields.
    pub checksum: u32,
}

impl RtcBootRecord {
    /// Uninitialized RTC memory representation.
    pub const ZERO: Self = Self {
        magic: 0,
        schema_version: 0,
        ordinal: 0,
        ordinal_complement: 0,
        checksum: 0,
    };

    /// Creates a valid record for a nonzero ordinal.
    pub fn new(ordinal: u64) -> Self {
        let mut record = Self {
            magic: BOOT_RECORD_MAGIC,
            schema_version: BOOT_RECORD_SCHEMA_VERSION,
            ordinal,
            ordinal_complement: !ordinal,
            checksum: 0,
        };
        record.checksum = record.expected_checksum();
        record
    }

    /// Returns whether all identity, continuity, and integrity fields agree.
    pub fn is_valid(self) -> bool {
        self.magic == BOOT_RECORD_MAGIC
            && self.schema_version == BOOT_RECORD_SCHEMA_VERSION
            && self.ordinal > 0
            && self.ordinal_complement == !self.ordinal
            && self.checksum == self.expected_checksum()
    }

    fn expected_checksum(self) -> u32 {
        let mut checksum = 0x811c_9dc5_u32;
        for byte in self
            .magic
            .to_le_bytes()
            .into_iter()
            .chain(self.schema_version.to_le_bytes())
            .chain(self.ordinal.to_le_bytes())
            .chain(self.ordinal_complement.to_le_bytes())
        {
            checksum ^= u32::from(byte);
            checksum = checksum.wrapping_mul(0x0100_0193);
        }
        checksum
    }
}

/// Classification of the transition applied to the retained boot record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtcBootTransitionKind {
    /// A cold or zeroed record began a new retained-power epoch.
    Initialized,
    /// A valid warm-reset record advanced exactly once.
    Incremented,
    /// Invalid retained state was discarded rather than trusted.
    ReinitializedCorrupt,
    /// An exhausted counter was discarded rather than wrapped.
    ReinitializedOverflow,
}

/// Result of advancing or safely reinitializing retained boot state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcBootTransition {
    /// Valid record to store for the current boot.
    pub record: RtcBootRecord,
    /// Reason this record value was selected.
    pub kind: RtcBootTransitionKind,
}

/// Advances valid warm-reset state exactly once or reinitializes untrusted state.
pub fn transition_rtc_boot_record(previous: RtcBootRecord, cold_start: bool) -> RtcBootTransition {
    if cold_start || previous == RtcBootRecord::ZERO {
        return RtcBootTransition {
            record: RtcBootRecord::new(1),
            kind: RtcBootTransitionKind::Initialized,
        };
    }
    if !previous.is_valid() {
        return RtcBootTransition {
            record: RtcBootRecord::new(1),
            kind: RtcBootTransitionKind::ReinitializedCorrupt,
        };
    }
    let Some(next_ordinal) = previous.ordinal.checked_add(1) else {
        return RtcBootTransition {
            record: RtcBootRecord::new(1),
            kind: RtcBootTransitionKind::ReinitializedOverflow,
        };
    };
    RtcBootTransition {
        record: RtcBootRecord::new(next_ordinal),
        kind: RtcBootTransitionKind::Incremented,
    }
}

/// Redaction-safe reset-reason vocabulary used by serial boot evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetReasonCategory {
    /// Power-on reset.
    PowerOn,
    /// Software CPU reset, including the approved HTTP restart effect.
    SoftwareCpu,
    /// Interrupt, task, or generic watchdog reset.
    Watchdog,
    /// Panic reset.
    Panic,
    /// Brownout reset.
    Brownout,
    /// Any reset reason not admitted above.
    Other,
}

impl ResetReasonCategory {
    /// Returns the stable serial label for this reset category.
    pub const fn label(self) -> &'static str {
        match self {
            Self::PowerOn => "power_on",
            Self::SoftwareCpu => "software_cpu",
            Self::Watchdog => "watchdog",
            Self::Panic => "panic",
            Self::Brownout => "brownout",
            Self::Other => "other",
        }
    }
}

/// Formats one session-bound boot identity marker.
pub fn boot_identity_marker(
    session: [u32; 4],
    ordinal: u64,
    reset_reason: ResetReasonCategory,
    uptime_ms: u64,
) -> String {
    let [a, b, c, d] = session;
    format!(
        "runtime_boot_identity session={a:08x}{b:08x}{c:08x}{d:08x} boot_ordinal={ordinal} reset_reason={} uptime_ms={uptime_ms} redacted=true",
        reset_reason.label()
    )
}

/// Formats one session-bound, origin-only replay marker.
pub fn runtime_origin_marker(session: [u32; 4], ordinal: u64, device_url: &str) -> String {
    let [a, b, c, d] = session;
    format!(
        "runtime_origin session={a:08x}{b:08x}{c:08x}{d:08x} boot_ordinal={ordinal} device_url={device_url} redacted=true"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_record_increments_once() {
        // Arrange
        let previous = RtcBootRecord::new(41);

        // Act
        let transition = transition_rtc_boot_record(previous, false);

        // Assert
        assert_eq!(transition.kind, RtcBootTransitionKind::Incremented);
        assert_eq!(transition.record.ordinal, 42);
        assert!(transition.record.is_valid());
    }

    #[test]
    fn cold_start_initializes_even_when_record_is_valid() {
        // Arrange
        let previous = RtcBootRecord::new(41);

        // Act
        let transition = transition_rtc_boot_record(previous, true);

        // Assert
        assert_eq!(transition.kind, RtcBootTransitionKind::Initialized);
        assert_eq!(transition.record.ordinal, 1);
        assert!(transition.record.is_valid());
    }

    #[test]
    fn checksum_corruption_fails_closed() {
        // Arrange
        let mut previous = RtcBootRecord::new(41);
        previous.checksum ^= 1;

        // Act
        let transition = transition_rtc_boot_record(previous, false);

        // Assert
        assert_eq!(transition.kind, RtcBootTransitionKind::ReinitializedCorrupt);
        assert_eq!(transition.record.ordinal, 1);
    }

    #[test]
    fn torn_record_fails_closed() {
        // Arrange
        let mut previous = RtcBootRecord::new(41);
        previous.ordinal_complement ^= 1;

        // Act
        let transition = transition_rtc_boot_record(previous, false);

        // Assert
        assert_eq!(transition.kind, RtcBootTransitionKind::ReinitializedCorrupt);
        assert_eq!(transition.record.ordinal, 1);
    }

    #[test]
    fn overflow_fails_closed() {
        // Arrange
        let previous = RtcBootRecord::new(u64::MAX);

        // Act
        let transition = transition_rtc_boot_record(previous, false);

        // Assert
        assert_eq!(
            transition.kind,
            RtcBootTransitionKind::ReinitializedOverflow
        );
        assert_eq!(transition.record.ordinal, 1);
    }
}
