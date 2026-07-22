use serde::{Deserialize, Serialize};
use thiserror::Error;

pub(crate) const PHASE35_FLASH_SCHEMA: &str = "phase35-flash-boundary-v1";
const MAX_STAGE_DURATION_MILLIS: u64 = 420_000;
const MAX_PRIVATE_LOG_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FlashStage {
    Probe,
    Factory,
    Nvs,
    Monitor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FlashBoundary {
    VersionMismatch,
    SpawnFailure,
    PreConnectFailure,
    DeviceInfoFailure,
    #[serde(rename = "post_info_pre_transfer_failed")]
    PostInfoPreTransferFailure,
    TransferFailure,
    PostTransferFailure,
    Ready,
}

impl FlashBoundary {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::VersionMismatch => "version_mismatch",
            Self::SpawnFailure => "spawn_failure",
            Self::PreConnectFailure => "pre_connect_failure",
            Self::DeviceInfoFailure => "device_info_failure",
            Self::PostInfoPreTransferFailure => "post_info_pre_transfer_failed",
            Self::TransferFailure => "transfer_failure",
            Self::PostTransferFailure => "post_transfer_failure",
            Self::Ready => "ready",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageMetrics {
    schema_version: String,
    stage: FlashStage,
    tool_version_valid: bool,
    launched: bool,
    connected: bool,
    device_info_complete: bool,
    transfer_started: bool,
    completed: bool,
    duration_millis: u64,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlashBoundaryProjection {
    schema_version: &'static str,
    stage: FlashStage,
    tool_version_valid: bool,
    launched: bool,
    connected: bool,
    device_info_complete: bool,
    transfer_started: bool,
    completed: bool,
    duration_millis: u64,
    pub(crate) terminal_boundary: FlashBoundary,
}

#[derive(Debug, Error)]
pub(crate) enum Phase35FlashError {
    #[error("invalid stage metrics")]
    InvalidMetrics,
    #[error("invalid private child log")]
    InvalidPrivateLog,
}

pub(crate) fn classify_phase35_flash(
    metrics_json: &[u8],
    private_child_log: &[u8],
) -> Result<FlashBoundaryProjection, Phase35FlashError> {
    let metrics: StageMetrics =
        serde_json::from_slice(metrics_json).map_err(|_| Phase35FlashError::InvalidMetrics)?;
    validate_metrics(&metrics)?;
    let private_log = validate_private_log(private_child_log)?;
    let terminal_boundary = classify_boundary(&metrics, private_log);

    Ok(FlashBoundaryProjection {
        schema_version: PHASE35_FLASH_SCHEMA,
        stage: metrics.stage,
        tool_version_valid: metrics.tool_version_valid,
        launched: metrics.launched,
        connected: metrics.connected,
        device_info_complete: metrics.device_info_complete,
        transfer_started: metrics.transfer_started,
        completed: metrics.completed,
        duration_millis: metrics.duration_millis,
        terminal_boundary,
    })
}

fn validate_metrics(metrics: &StageMetrics) -> Result<(), Phase35FlashError> {
    if metrics.schema_version != PHASE35_FLASH_SCHEMA
        || metrics.duration_millis > MAX_STAGE_DURATION_MILLIS
        || (metrics.connected && !metrics.launched)
        || (metrics.device_info_complete && !metrics.connected)
        || (metrics.transfer_started && !metrics.device_info_complete)
        || (metrics.completed && !metrics.transfer_started)
        || (!metrics.launched && metrics.duration_millis != 0)
    {
        return Err(Phase35FlashError::InvalidMetrics);
    }
    Ok(())
}

fn validate_private_log(bytes: &[u8]) -> Result<&str, Phase35FlashError> {
    if bytes.is_empty() || bytes.len() > MAX_PRIVATE_LOG_BYTES || bytes.contains(&0) {
        return Err(Phase35FlashError::InvalidPrivateLog);
    }
    std::str::from_utf8(bytes).map_err(|_| Phase35FlashError::InvalidPrivateLog)
}

fn classify_boundary(metrics: &StageMetrics, private_log: &str) -> FlashBoundary {
    if !metrics.tool_version_valid {
        return FlashBoundary::VersionMismatch;
    }
    if !metrics.launched {
        return FlashBoundary::SpawnFailure;
    }
    if !metrics.connected {
        return FlashBoundary::PreConnectFailure;
    }
    if !metrics.device_info_complete {
        return FlashBoundary::DeviceInfoFailure;
    }
    if !metrics.transfer_started {
        return FlashBoundary::PostInfoPreTransferFailure;
    }
    if !metrics.completed {
        if private_log_has_post_transfer_marker(private_log) {
            return FlashBoundary::PostTransferFailure;
        }
        return FlashBoundary::TransferFailure;
    }
    FlashBoundary::Ready
}

fn private_log_has_post_transfer_marker(private_log: &str) -> bool {
    [
        "Hash of data verified",
        "Checksum verified",
        "checksum-md5",
        "Writing finished",
        "Read finished",
        "OK!",
    ]
    .iter()
    .any(|marker| private_log.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(overrides: serde_json::Value) -> Vec<u8> {
        let mut value = serde_json::json!({
            "schema_version": PHASE35_FLASH_SCHEMA,
            "stage": "factory",
            "tool_version_valid": true,
            "launched": true,
            "connected": true,
            "device_info_complete": true,
            "transfer_started": true,
            "completed": true,
            "duration_millis": 25
        });
        for (key, value_override) in overrides.as_object().expect("object") {
            value[key] = value_override.clone();
        }
        serde_json::to_vec(&value).expect("metrics")
    }

    fn category(overrides: serde_json::Value, log: &str) -> FlashBoundary {
        classify_phase35_flash(&metrics(overrides), log.as_bytes())
            .expect("classification")
            .terminal_boundary
    }

    #[test]
    fn classifies_every_closed_boundary_with_earliest_precedence() {
        // Arrange
        let cases = [
            (
                serde_json::json!({"tool_version_valid":false}),
                FlashBoundary::VersionMismatch,
            ),
            (
                serde_json::json!({"launched":false,"connected":false,"device_info_complete":false,"transfer_started":false,"completed":false,"duration_millis":0}),
                FlashBoundary::SpawnFailure,
            ),
            (
                serde_json::json!({"connected":false,"device_info_complete":false,"transfer_started":false,"completed":false}),
                FlashBoundary::PreConnectFailure,
            ),
            (
                serde_json::json!({"device_info_complete":false,"transfer_started":false,"completed":false}),
                FlashBoundary::DeviceInfoFailure,
            ),
            (
                serde_json::json!({"transfer_started":false,"completed":false}),
                FlashBoundary::PostInfoPreTransferFailure,
            ),
            (
                serde_json::json!({"completed":false}),
                FlashBoundary::TransferFailure,
            ),
        ];

        // Act and Assert
        for (overrides, expected) in cases {
            assert_eq!(category(overrides, "private child output\n"), expected);
        }
        assert_eq!(
            category(
                serde_json::json!({"completed":false}),
                "Hash of data verified\nlate reset failure\n",
            ),
            FlashBoundary::PostTransferFailure
        );
        assert_eq!(
            category(serde_json::json!({}), "Writing finished\n"),
            FlashBoundary::Ready
        );
    }

    #[test]
    fn projection_contains_no_private_child_text() {
        // Arrange
        let canary = "\u{1b}[31msecret-device-path=/dev/private-canary café\u{1b}[0m";

        // Act
        let projection = classify_phase35_flash(&metrics(serde_json::json!({})), canary.as_bytes())
            .expect("classification");
        let rendered = serde_json::to_string(&projection).expect("projection");

        // Assert
        assert!(!rendered.contains(canary));
        assert!(!rendered.contains("/dev/"));
        assert!(!rendered.contains("café"));
    }

    #[test]
    fn projection_serializes_the_canonical_post_info_boundary() {
        // Arrange
        let projection = classify_phase35_flash(
            &metrics(serde_json::json!({
                "transfer_started": false,
                "completed": false
            })),
            b"private child output\n",
        )
        .expect("classification");

        // Act
        let rendered = serde_json::to_value(projection).expect("projection");

        // Assert
        assert_eq!(
            rendered["terminal_boundary"],
            "post_info_pre_transfer_failed"
        );
    }

    #[test]
    fn rejects_unknown_fields_and_non_monotonic_metrics() {
        // Arrange
        let unknown = metrics(serde_json::json!({"raw_path":"/dev/private"}));
        let contradictory = metrics(serde_json::json!({
            "connected": false,
            "device_info_complete": true,
            "transfer_started": true,
            "completed": true
        }));

        // Act
        let unknown_result = classify_phase35_flash(&unknown, b"private\n");
        let contradictory_result = classify_phase35_flash(&contradictory, b"private\n");

        // Assert
        assert!(unknown_result.is_err());
        assert!(contradictory_result.is_err());
    }

    #[test]
    fn accepts_every_stage_and_the_exact_duration_bound() {
        // Arrange
        let stages = ["probe", "factory", "nvs", "monitor"];

        // Act and Assert
        for stage in stages {
            let projection = classify_phase35_flash(
                &metrics(serde_json::json!({
                    "stage": stage,
                    "duration_millis": MAX_STAGE_DURATION_MILLIS
                })),
                b"private child output\n",
            )
            .expect("bounded stage");
            assert_eq!(projection.terminal_boundary, FlashBoundary::Ready);
        }
    }

    #[test]
    fn rejects_over_bound_or_unsafe_private_inputs() {
        // Arrange
        let over_bound = metrics(serde_json::json!({
            "duration_millis": MAX_STAGE_DURATION_MILLIS + 1
        }));
        let oversized = vec![b'a'; MAX_PRIVATE_LOG_BYTES + 1];

        // Act and Assert
        assert!(classify_phase35_flash(&over_bound, b"private\n").is_err());
        assert!(classify_phase35_flash(&metrics(serde_json::json!({})), b"").is_err());
        assert!(classify_phase35_flash(&metrics(serde_json::json!({})), &oversized).is_err());
        assert!(classify_phase35_flash(&metrics(serde_json::json!({})), b"bad\0log").is_err());
        assert!(classify_phase35_flash(&metrics(serde_json::json!({})), &[0xff]).is_err());
    }
}
