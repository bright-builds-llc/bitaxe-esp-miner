//! Typed identity for one immutable operator-visible runtime capture.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Exact byte length of an opaque boot-session identifier.
pub const BOOT_SESSION_BYTES: usize = 16;
/// Exact rendered length of an opaque boot-session identifier.
pub const BOOT_SESSION_HEX_BYTES: usize = BOOT_SESSION_BYTES * 2;

/// Opaque per-boot identity with no ordinal or timestamp semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BootSessionId([u8; BOOT_SESSION_BYTES]);

impl BootSessionId {
    /// Constructs an opaque session from the four hardware-RNG words retained by firmware.
    #[must_use]
    pub fn from_words(words: [u32; 4]) -> Self {
        let mut bytes = [0_u8; BOOT_SESSION_BYTES];
        for (index, word) in words.into_iter().enumerate() {
            let offset = index * size_of::<u32>();
            bytes[offset..offset + size_of::<u32>()].copy_from_slice(&word.to_be_bytes());
        }
        Self(bytes)
    }

    fn parse(value: &str) -> Result<Self, OperatorSnapshotIdentityError> {
        if value.len() != BOOT_SESSION_HEX_BYTES {
            return Err(OperatorSnapshotIdentityError::InvalidBootSessionLength);
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(OperatorSnapshotIdentityError::InvalidBootSessionEncoding);
        }

        let mut bytes = [0_u8; BOOT_SESSION_BYTES];
        for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
            let high = decode_lower_hex(pair[0]);
            let low = decode_lower_hex(pair[1]);
            bytes[index] = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

impl fmt::Display for BootSessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for BootSessionId {
    type Err = OperatorSnapshotIdentityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for BootSessionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for BootSessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

/// Nonzero monotonic operator-snapshot revision within one boot session.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperatorSnapshotRevision(u64);

impl OperatorSnapshotRevision {
    /// Constructs a validated nonzero revision.
    pub const fn new(value: u64) -> Result<Self, OperatorSnapshotIdentityError> {
        if value == 0 {
            return Err(OperatorSnapshotIdentityError::ZeroRevision);
        }
        Ok(Self(value))
    }

    /// Returns the revision's wire integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for OperatorSnapshotRevision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl TryFrom<u64> for OperatorSnapshotRevision {
    type Error = OperatorSnapshotIdentityError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Serialize for OperatorSnapshotRevision {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for OperatorSnapshotRevision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Correlation identity assigned once to one complete operator snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperatorSnapshotIdentity {
    boot_session: BootSessionId,
    revision: OperatorSnapshotRevision,
}

impl OperatorSnapshotIdentity {
    /// Binds a validated revision to its boot session.
    #[must_use]
    pub const fn new(boot_session: BootSessionId, revision: OperatorSnapshotRevision) -> Self {
        Self {
            boot_session,
            revision,
        }
    }

    /// Returns the opaque boot session.
    #[must_use]
    pub const fn boot_session(self) -> BootSessionId {
        self.boot_session
    }

    /// Returns the nonzero within-boot revision.
    #[must_use]
    pub const fn revision(self) -> OperatorSnapshotRevision {
        self.revision
    }

    /// Formats the exact redaction-safe retained correlation marker.
    #[must_use]
    pub fn retained_marker(self) -> String {
        format!(
            "operator_snapshot session={} revision={} redacted=true",
            self.boot_session, self.revision
        )
    }

    /// Deterministic identity reserved exclusively for host fixtures.
    pub(crate) const fn fixture_only() -> Self {
        Self {
            boot_session: BootSessionId([0; BOOT_SESSION_BYTES]),
            revision: OperatorSnapshotRevision(1),
        }
    }
}

/// Checked within-boot revision allocator. Firmware owns one instance per boot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OperatorSnapshotSequence {
    last_revision: u64,
}

impl OperatorSnapshotSequence {
    /// Starts before revision 1.
    #[must_use]
    pub const fn new() -> Self {
        Self { last_revision: 0 }
    }

    /// Reserves the next unique identity or fails rather than wrapping.
    pub fn next_identity(
        &mut self,
        boot_session: BootSessionId,
    ) -> Result<OperatorSnapshotIdentity, OperatorSnapshotSequenceError> {
        let Some(next_revision) = self.last_revision.checked_add(1) else {
            return Err(OperatorSnapshotSequenceError::Exhausted);
        };
        self.last_revision = next_revision;
        let revision = OperatorSnapshotRevision(next_revision);
        Ok(OperatorSnapshotIdentity::new(boot_session, revision))
    }
}

/// Validation failure for an operator-snapshot identity component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatorSnapshotIdentityError {
    InvalidBootSessionLength,
    InvalidBootSessionEncoding,
    ZeroRevision,
}

impl fmt::Display for OperatorSnapshotIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidBootSessionLength => "boot session must contain exactly 32 characters",
            Self::InvalidBootSessionEncoding => {
                "boot session must contain only lowercase hexadecimal characters"
            }
            Self::ZeroRevision => "operator snapshot revision must be nonzero",
        };
        formatter.write_str(message)
    }
}

impl Error for OperatorSnapshotIdentityError {}

/// Checked sequence failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatorSnapshotSequenceError {
    Exhausted,
}

impl fmt::Display for OperatorSnapshotSequenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted => formatter.write_str("operator snapshot revision sequence exhausted"),
        }
    }
}

impl Error for OperatorSnapshotSequenceError {}

fn decode_lower_hex(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!("boot session encoding is validated before decoding"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = "0123456789abcdef0011223344556677";

    #[test]
    fn boot_session_round_trips_through_json() {
        // Arrange
        let session = SESSION
            .parse::<BootSessionId>()
            .expect("lowercase session fixture should parse");

        // Act
        let json = serde_json::to_string(&session).expect("boot session should serialize");
        let decoded: BootSessionId =
            serde_json::from_str(&json).expect("boot session should deserialize");

        // Assert
        assert_eq!(json, format!("\"{SESSION}\""));
        assert_eq!(decoded, session);
    }

    #[test]
    fn boot_session_rejects_malformed_encodings() {
        // Arrange
        let malformed = [
            "0123456789ABCDEF0011223344556677",
            "g123456789abcdef0011223344556677",
            "0123456789abcdef001122334455667",
            "0123456789abcdef00112233445566770",
        ];

        // Act
        let results = malformed.map(str::parse::<BootSessionId>);

        // Assert
        assert!(results.into_iter().all(|result| result.is_err()));
    }

    #[test]
    fn revision_rejects_zero() {
        // Arrange
        let raw_revision = 0;

        // Act
        let result = OperatorSnapshotRevision::new(raw_revision);

        // Assert
        assert_eq!(result, Err(OperatorSnapshotIdentityError::ZeroRevision));
    }

    #[test]
    fn sequence_starts_at_one_and_strictly_increases() {
        // Arrange
        let session = BootSessionId::from_words([1, 2, 3, 4]);
        let mut sequence = OperatorSnapshotSequence::new();

        // Act
        let first = sequence
            .next_identity(session)
            .expect("first revision should be available");
        let second = sequence
            .next_identity(session)
            .expect("second revision should be available");

        // Assert
        assert_eq!(first.revision().get(), 1);
        assert_eq!(second.revision().get(), 2);
        assert!(second.revision() > first.revision());
    }

    #[test]
    fn exhausted_sequence_fails_without_wraparound() {
        // Arrange
        let session = BootSessionId::from_words([1, 2, 3, 4]);
        let mut sequence = OperatorSnapshotSequence {
            last_revision: u64::MAX,
        };

        // Act
        let result = sequence.next_identity(session);

        // Assert
        assert_eq!(result, Err(OperatorSnapshotSequenceError::Exhausted));
        assert_eq!(sequence.last_revision, u64::MAX);
    }

    #[test]
    fn retained_marker_uses_exact_redacted_contract() {
        // Arrange
        let session = SESSION
            .parse::<BootSessionId>()
            .expect("lowercase session fixture should parse");
        let revision =
            OperatorSnapshotRevision::new(42).expect("nonzero revision fixture should parse");

        // Act
        let marker = OperatorSnapshotIdentity::new(session, revision).retained_marker();

        // Assert
        assert_eq!(
            marker,
            "operator_snapshot session=0123456789abcdef0011223344556677 revision=42 redacted=true"
        );
    }
}
