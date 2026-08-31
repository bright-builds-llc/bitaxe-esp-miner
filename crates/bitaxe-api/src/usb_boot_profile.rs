use serde::{Deserialize, Serialize};

pub const USB_BOOT_PROFILE_MARKER: &str = "usb_boot_profile=";
pub const USB_BOOT_PROFILE_SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbBootTransport {
    WorkerRuntime,
    SerialJtagRuntime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbBootProfileReason {
    WorkerStarted,
    DiagnosticOwner,
    BootBaselineUnconfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbBootBaseline {
    Confirmed,
    Diagnostic,
    Unconfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsbBootProfileMarker {
    schema_version: u8,
    transport: UsbBootTransport,
    reason: UsbBootProfileReason,
    baseline: UsbBootBaseline,
    firmware_commit: String,
    app_elf_sha256: String,
    boot_ordinal: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbBootProfileReplay {
    marker: UsbBootProfileMarker,
    interval_ms: u64,
    next_deadline_ms: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum UsbBootProfileError {
    #[error("USB boot profile marker identity is invalid")]
    InvalidIdentity,
    #[error("USB boot profile marker JSON is invalid")]
    InvalidJson(#[from] serde_json::Error),
    #[error("USB boot profile marker prefix is missing")]
    MissingPrefix,
}

impl UsbBootProfileMarker {
    pub fn new(
        transport: UsbBootTransport,
        reason: UsbBootProfileReason,
        baseline: UsbBootBaseline,
        firmware_commit: String,
        app_elf_sha256: String,
        boot_ordinal: u64,
    ) -> Result<Self, UsbBootProfileError> {
        let marker = Self {
            schema_version: USB_BOOT_PROFILE_SCHEMA_VERSION,
            transport,
            reason,
            baseline,
            firmware_commit,
            app_elf_sha256,
            boot_ordinal,
        };
        marker.validate()?;
        Ok(marker)
    }

    pub fn parse(line: &str) -> Result<Self, UsbBootProfileError> {
        let json = line
            .strip_prefix(USB_BOOT_PROFILE_MARKER)
            .ok_or(UsbBootProfileError::MissingPrefix)?;
        let marker: Self = serde_json::from_str(json)?;
        marker.validate()?;
        Ok(marker)
    }

    pub fn render(&self) -> String {
        format!(
            "{USB_BOOT_PROFILE_MARKER}{}",
            serde_json::to_string(self).expect("validated marker serializes")
        )
    }

    pub const fn transport(&self) -> UsbBootTransport {
        self.transport
    }

    pub const fn boot_ordinal(&self) -> u64 {
        self.boot_ordinal
    }

    pub fn firmware_commit(&self) -> &str {
        &self.firmware_commit
    }

    pub fn app_elf_sha256(&self) -> &str {
        &self.app_elf_sha256
    }

    fn validate(&self) -> Result<(), UsbBootProfileError> {
        if self.schema_version != USB_BOOT_PROFILE_SCHEMA_VERSION
            || !lower_hex(&self.firmware_commit, 40)
            || !lower_hex(&self.app_elf_sha256, 64)
            || self.boot_ordinal == 0
        {
            return Err(UsbBootProfileError::InvalidIdentity);
        }
        Ok(())
    }
}

impl UsbBootProfileReplay {
    pub fn new(marker: UsbBootProfileMarker, now_ms: u64, interval_ms: u64) -> Self {
        let interval_ms = interval_ms.max(1);
        Self {
            marker,
            interval_ms,
            next_deadline_ms: now_ms.saturating_add(interval_ms),
        }
    }

    pub fn immediate(&self) -> String {
        self.marker.render()
    }

    pub fn maybe_take_due(&mut self, now_ms: u64) -> Option<String> {
        if now_ms < self.next_deadline_ms {
            return None;
        }
        self.next_deadline_ms = now_ms.saturating_add(self.interval_ms);
        Some(self.marker.render())
    }

    pub const fn next_deadline_ms(&self) -> u64 {
        self.next_deadline_ms
    }
}

fn lower_hex(value: &str, bytes: usize) -> bool {
    value.len() == bytes
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{
        UsbBootBaseline, UsbBootProfileMarker, UsbBootProfileReason, UsbBootProfileReplay,
        UsbBootTransport, USB_BOOT_PROFILE_MARKER,
    };

    #[test]
    fn boot_profile_marker_round_trips_the_closed_application_identity() {
        // Arrange
        let marker = UsbBootProfileMarker::new(
            UsbBootTransport::SerialJtagRuntime,
            UsbBootProfileReason::BootBaselineUnconfirmed,
            UsbBootBaseline::Unconfirmed,
            "1".repeat(40),
            "2".repeat(64),
            7,
        )
        .expect("valid marker");

        // Act
        let rendered = marker.render();
        let parsed = UsbBootProfileMarker::parse(&rendered).expect("round-trip marker");

        // Assert
        assert_eq!(parsed, marker);
        assert!(rendered.starts_with("usb_boot_profile={"));
        let object = serde_json::from_str::<serde_json::Value>(
            rendered
                .strip_prefix(USB_BOOT_PROFILE_MARKER)
                .expect("marker prefix"),
        )
        .expect("marker JSON");
        for forbidden in ["port", "mac", "device"] {
            assert!(object.get(forbidden).is_none());
        }
    }

    #[test]
    fn boot_profile_replay_is_immediate_then_periodic() {
        // Arrange
        let marker = UsbBootProfileMarker::new(
            UsbBootTransport::WorkerRuntime,
            UsbBootProfileReason::WorkerStarted,
            UsbBootBaseline::Confirmed,
            "1".repeat(40),
            "2".repeat(64),
            3,
        )
        .expect("valid marker");
        let mut replay = UsbBootProfileReplay::new(marker.clone(), 1_000, 5_000);

        // Act / Assert
        assert_eq!(replay.immediate(), marker.render());
        assert_eq!(replay.maybe_take_due(5_999), None);
        assert_eq!(replay.maybe_take_due(6_000), Some(marker.render()));
        assert_eq!(replay.maybe_take_due(6_001), None);
    }
}
