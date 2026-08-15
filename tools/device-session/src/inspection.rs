use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use bitaxe_api::{CommandStatusWire, COMMAND_STATUS_SCHEMA};
use bitaxe_http_transport::StrictHttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::macos::MacOsDeviceAdapter;
use crate::{current_platform, InspectionArtifacts, PlatformCategory, TerminalCategory};

pub const INSPECTION_INTENT_SCHEMA: &str = "bitaxe-device-inspection-intent-v1";
pub const INSPECTION_PROJECTION_SCHEMA: &str = "bitaxe-device-inspection-v1";
const REQUIRED_STABLE_SAMPLES: usize = 3;
const SAMPLE_INTERVAL: Duration = Duration::from_millis(150);

/// Private expected identity for one read-only live inspection.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceInspectionIntent {
    pub schema_version: String,
    pub board_category: String,
    pub trusted_origin: String,
    pub boot_session: String,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
}

impl DeviceInspectionIntent {
    #[must_use]
    pub fn schema_is_valid(&self) -> bool {
        self.schema_version == INSPECTION_INTENT_SCHEMA
            && self.board_category == "205"
            && (self.trusted_origin.starts_with("http://")
                || self.trusted_origin.starts_with("https://"))
            && is_lower_hex(&self.boot_session, 32)
            && is_lower_hex(&self.source_commit, 40)
            && is_lower_hex(&self.reference_commit, 40)
            && is_sha256(&self.app_elf_sha256)
    }
}

/// Closed read-only inspection result safe for public orchestration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceInspectionProjection {
    pub schema_version: &'static str,
    pub terminal_category: &'static str,
    pub platform_category: PlatformCategory,
    pub board_category: &'static str,
    pub stable_physical_device: bool,
    pub stable_enumeration: bool,
    pub receive_accessible: bool,
    pub holder_free: bool,
    pub system_info_ready: bool,
    pub command_status_ready: bool,
    pub build_identity_matches: bool,
    pub boot_session_matches: bool,
    pub cleanup_complete: bool,
    pub sample_count: u8,
    pub http_observation_count: u8,
}

impl DeviceInspectionProjection {
    fn new(platform_category: PlatformCategory) -> Self {
        Self {
            schema_version: INSPECTION_PROJECTION_SCHEMA,
            terminal_category: "hardware_blocked",
            platform_category,
            board_category: "205",
            stable_physical_device: false,
            stable_enumeration: false,
            receive_accessible: false,
            holder_free: false,
            system_info_ready: false,
            command_status_ready: false,
            build_identity_matches: false,
            boot_session_matches: false,
            cleanup_complete: true,
            sample_count: 0,
            http_observation_count: 0,
        }
    }

    fn close(&mut self) {
        let ready = self.platform_category == PlatformCategory::Macos
            && self.stable_physical_device
            && self.stable_enumeration
            && self.receive_accessible
            && self.holder_free
            && self.system_info_ready
            && self.command_status_ready
            && self.build_identity_matches
            && self.boot_session_matches
            && self.cleanup_complete
            && self.sample_count == REQUIRED_STABLE_SAMPLES as u8
            && self.http_observation_count == 2;
        self.terminal_category = if ready { "ready" } else { "hardware_blocked" };
    }
}

/// Performs a read-only admitted inspection with three stable USB samples and two HTTP reads.
pub fn run_admitted_inspection(
    intent: DeviceInspectionIntent,
    admitted_port: String,
    artifacts: InspectionArtifacts,
    timeout: Duration,
) -> Result<TerminalCategory> {
    if !intent.schema_is_valid() {
        anyhow::bail!("device inspection intent schema is invalid");
    }
    let platform = current_platform();
    let mut projection = DeviceInspectionProjection::new(platform);
    if platform != PlatformCategory::Macos {
        artifacts.finish(&projection)?;
        return Ok(TerminalCategory::ObserverUnqualified);
    }

    let deadline = Instant::now() + timeout;
    let mut maybe_physical_identity = None;
    let mut maybe_enumeration = None;
    for sample_index in 0..REQUIRED_STABLE_SAMPLES {
        let Ok(maybe_snapshot) = MacOsDeviceAdapter::maybe_exact_snapshot(&admitted_port) else {
            artifacts.finish(&projection)?;
            return Ok(TerminalCategory::ObserverUnqualified);
        };
        let Some(snapshot) = maybe_snapshot else {
            artifacts.finish(&projection)?;
            return Ok(TerminalCategory::UsbIdentityUnavailable);
        };
        projection.sample_count = projection.sample_count.saturating_add(1);
        projection.receive_accessible |= snapshot.accessible;
        projection.holder_free |= snapshot.holder_count == 0;
        let physical_matches = maybe_physical_identity
            .as_ref()
            .is_none_or(|identity| identity == &snapshot.physical_identity_digest);
        let enumeration_matches = maybe_enumeration
            .as_ref()
            .is_none_or(|enumeration| enumeration == &snapshot.enumeration_token);
        if !snapshot.accessible
            || snapshot.holder_count != 0
            || !physical_matches
            || !enumeration_matches
        {
            artifacts.finish(&projection)?;
            return Ok(TerminalCategory::ObserverUnqualified);
        }
        maybe_physical_identity = Some(snapshot.physical_identity_digest);
        maybe_enumeration = Some(snapshot.enumeration_token);
        if sample_index + 1 < REQUIRED_STABLE_SAMPLES {
            thread::sleep(SAMPLE_INTERVAL);
        }
    }
    projection.stable_physical_device = true;
    projection.stable_enumeration = true;

    let Ok(client) = StrictHttpClient::new(&intent.trusted_origin) else {
        artifacts.finish(&projection)?;
        return Ok(TerminalCategory::ObserverUnqualified);
    };
    let Ok(system) = client.get_system_info(deadline) else {
        artifacts.finish(&projection)?;
        return Ok(TerminalCategory::ServiceRecoveryTimeout);
    };
    projection.http_observation_count = projection.http_observation_count.saturating_add(1);
    let Some(system_response) = system
        .maybe_http_response()
        .filter(|response| matches!(response.status(), 200..=299))
    else {
        artifacts.finish(&projection)?;
        return Ok(TerminalCategory::ServiceRecoveryTimeout);
    };
    artifacts.record_http("system-info.private.json", system_response.body())?;
    let Ok(system_value) = serde_json::from_slice::<Value>(system_response.body()) else {
        artifacts.finish(&projection)?;
        return Ok(TerminalCategory::ObserverUnqualified);
    };
    projection.system_info_ready = true;
    projection.build_identity_matches = required_string(&system_value, "sourceCommit")
        == Some(intent.source_commit.as_str())
        && required_string(&system_value, "referenceCommit")
            == Some(intent.reference_commit.as_str())
        && required_string(&system_value, "appElfSha256") == Some(intent.app_elf_sha256.as_str());

    let Ok(status) = client.get_command_status(deadline) else {
        artifacts.finish(&projection)?;
        return Ok(TerminalCategory::ServiceRecoveryTimeout);
    };
    projection.http_observation_count = projection.http_observation_count.saturating_add(1);
    let Some(status_response) = status
        .maybe_http_response()
        .filter(|response| matches!(response.status(), 200..=299))
    else {
        artifacts.finish(&projection)?;
        return Ok(TerminalCategory::ServiceRecoveryTimeout);
    };
    artifacts.record_http("command-status.private.json", status_response.body())?;
    let Ok(status) = serde_json::from_slice::<CommandStatusWire>(status_response.body()) else {
        artifacts.finish(&projection)?;
        return Ok(TerminalCategory::ObserverUnqualified);
    };
    projection.command_status_ready = status.schema == COMMAND_STATUS_SCHEMA;
    projection.boot_session_matches = status.boot_session.to_string() == intent.boot_session;
    projection.close();
    let ready = projection.terminal_category == "ready";
    artifacts.finish(&projection)?;
    Ok(if ready {
        TerminalCategory::Ready
    } else if !projection.build_identity_matches {
        TerminalCategory::BuildIdentityMismatch
    } else if !projection.boot_session_matches {
        TerminalCategory::BootIdentityInvalid
    } else {
        TerminalCategory::ObserverUnqualified
    })
}

fn required_string<'value>(value: &'value Value, field: &str) -> Option<&'value str> {
    value
        .get(field)?
        .as_str()
        .filter(|candidate| !candidate.is_empty())
}

fn is_sha256(value: &str) -> bool {
    is_lower_hex(value, 64)
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_projection_serialization_contains_only_closed_facts() {
        // Arrange
        let projection = DeviceInspectionProjection::new(PlatformCategory::Macos);

        // Act
        let json = serde_json::to_string(&projection).expect("serialize projection");

        // Assert
        for forbidden in [
            "origin",
            "hostname",
            "port",
            "usb_identity",
            "source_commit",
        ] {
            assert!(!json.contains(forbidden));
        }
    }

    #[test]
    fn inspection_intent_requires_exact_lowercase_boot_and_build_identity() {
        // Arrange
        let valid = DeviceInspectionIntent {
            schema_version: INSPECTION_INTENT_SCHEMA.to_owned(),
            board_category: "205".to_owned(),
            trusted_origin: "http://private-device".to_owned(),
            boot_session: "a".repeat(32),
            source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            app_elf_sha256: "d".repeat(64),
        };
        let uppercase = DeviceInspectionIntent {
            boot_session: "A".repeat(32),
            ..valid.clone()
        };

        // Act and assert
        assert!(valid.schema_is_valid());
        assert!(!uppercase.schema_is_valid());
    }
}
