//! Pure `/api/system/scoreboard` response mapping.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c:GET_scoreboard`
//! - `reference/esp-miner/main/tasks/scoreboard.h`
//! - `reference/esp-miner/main/http_server/axe-os/src/models/ISystemScoreboard.ts`

use serde::{Deserialize, Serialize};

/// Typed scoreboard entry held by firmware/runtime adapters.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreboardEntry {
    pub difficulty: f64,
    pub job_id: String,
    pub extranonce2: String,
    pub ntime: u32,
    pub nonce: u32,
    pub version_bits: u32,
}

impl ScoreboardEntry {
    /// Creates a typed scoreboard entry from upstream-owned fields.
    #[must_use]
    pub fn new(
        difficulty: f64,
        job_id: impl Into<String>,
        extranonce2: impl Into<String>,
        ntime: u32,
        nonce: u32,
        version_bits: u32,
    ) -> Self {
        Self {
            difficulty,
            job_id: job_id.into(),
            extranonce2: extranonce2.into(),
            ntime,
            nonce,
            version_bits,
        }
    }
}

/// Upstream scoreboard wire entry. Client-only rank/since are intentionally absent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreboardEntryWire {
    pub difficulty: f64,
    pub job_id: String,
    pub extranonce2: String,
    pub ntime: u32,
    pub nonce: String,
    pub version_bits: String,
}

/// Maps typed scoreboard entries into the upstream server-owned array shape.
#[must_use]
pub fn scoreboard_response(entries: &[ScoreboardEntry]) -> Vec<ScoreboardEntryWire> {
    entries
        .iter()
        .map(|entry| ScoreboardEntryWire {
            difficulty: entry.difficulty,
            job_id: entry.job_id.clone(),
            extranonce2: entry.extranonce2.clone(),
            ntime: entry.ntime,
            nonce: uppercase_hex_u32(entry.nonce),
            version_bits: uppercase_hex_u32(entry.version_bits),
        })
        .collect()
}

fn uppercase_hex_u32(value: u32) -> String {
    format!("{value:08X}")
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use crate::scoreboard::{scoreboard_response, ScoreboardEntry};

    #[test]
    fn empty_scoreboard_serializes_as_array_fixture() {
        // Arrange
        let fixture = include_str!("../fixtures/api/scoreboard-empty.json");
        let expected: Value =
            serde_json::from_str(fixture).expect("scoreboard fixture should be valid JSON");

        // Act
        let response = scoreboard_response(&[]);
        let actual = serde_json::to_value(response).expect("scoreboard should serialize");

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual, json!([]));
    }

    #[test]
    fn populated_scoreboard_entry_uses_only_upstream_server_owned_fields() {
        // Arrange
        let entries = [ScoreboardEntry::new(
            42.5,
            "job-1",
            "abcdef",
            1_719_820_000,
            0x1234_abcd,
            0x2000_0000,
        )];

        // Act
        let response = scoreboard_response(&entries);
        let actual = serde_json::to_value(response).expect("scoreboard should serialize");

        // Assert
        let entry = actual[0]
            .as_object()
            .expect("scoreboard entry should be object");
        assert_eq!(entry.get("difficulty"), Some(&json!(42.5)));
        assert_eq!(entry.get("job_id"), Some(&json!("job-1")));
        assert_eq!(entry.get("extranonce2"), Some(&json!("abcdef")));
        assert_eq!(entry.get("ntime"), Some(&json!(1_719_820_000_u32)));
        assert_eq!(entry.get("nonce"), Some(&json!("1234ABCD")));
        assert_eq!(entry.get("version_bits"), Some(&json!("20000000")));
        assert!(entry.get("rank").is_none());
        assert!(entry.get("since").is_none());
        assert_eq!(entry.len(), 6);
    }
}
