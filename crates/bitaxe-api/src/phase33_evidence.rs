//! Strict, typed classification of Phase 33 serial evidence.

use std::collections::BTreeSet;

use serde::Serialize;
use thiserror::Error;

/// Admissible boot identity and current-session origin for a Phase 33 proof stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Phase33BootEvidence {
    /// Opaque boot-session identifier.
    pub session: String,
    /// Reset-retained boot ordinal.
    pub boot_ordinal: u64,
    /// Origin-only URL emitted by the matching boot session.
    pub device_url: String,
}

/// Stable fail-closed category returned by Phase 33 evidence classification.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{category}")]
pub struct Phase33EvidenceError {
    /// Redaction-safe failure category.
    pub category: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Identity {
    session: String,
    ordinal: u64,
    reset_reason: String,
    uptime_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Origin {
    session: String,
    ordinal: u64,
    device_url: String,
}

/// Classifies setup capture evidence into one baseline session, ordinal, and origin.
pub fn classify_phase33_baseline(text: &str) -> Result<Phase33BootEvidence, Phase33EvidenceError> {
    let identities = parse_identities(text)?;
    let identity = unique_identity(&identities, "baseline_identity_missing")?;
    validate_monotonic_uptime(&identities, &identity.session, identity.ordinal)?;
    let device_url = unique_bound_origin(text, &identity.session, identity.ordinal)?;
    Ok(Phase33BootEvidence {
        session: identity.session.clone(),
        boot_ordinal: identity.ordinal,
        device_url,
    })
}

/// Proves that the passive reader delivered bytes from the expected baseline boot.
pub fn classify_phase33_delivery(
    text: &str,
    expected_session: &str,
    expected_ordinal: u64,
) -> Result<(), Phase33EvidenceError> {
    let identities = parse_identities(text)?;
    if identities.iter().any(|identity| {
        identity.session == expected_session && identity.ordinal == expected_ordinal
    }) {
        return Ok(());
    }
    Err(error("passive_byte_delivery_unproved"))
}

/// Classifies post-restart evidence and requires exactly one software-reset transition.
pub fn classify_phase33_post_restart(
    text: &str,
    baseline_session: &str,
    baseline_ordinal: u64,
) -> Result<Phase33BootEvidence, Phase33EvidenceError> {
    let identities = parse_identities(text)?;
    if identities.is_empty() {
        return Err(error("post_restart_identity_missing"));
    }
    let sessions: BTreeSet<_> = identities
        .iter()
        .map(|identity| &identity.session)
        .collect();
    if sessions.len() > 1 {
        return Err(error("post_restart_multiple_sessions"));
    }
    let identity = &identities[0];
    if identity.session == baseline_session {
        return Err(error("post_restart_session_unchanged"));
    }
    let Some(expected_ordinal) = baseline_ordinal.checked_add(1) else {
        return Err(error("baseline_ordinal_overflow"));
    };
    if identities
        .iter()
        .any(|candidate| candidate.ordinal != expected_ordinal)
    {
        return Err(error("post_restart_ordinal_nonmonotonic"));
    }
    if identities
        .iter()
        .any(|candidate| candidate.reset_reason != "software_cpu")
    {
        return Err(error("post_restart_reset_reason_wrong"));
    }
    validate_monotonic_uptime(&identities, &identity.session, expected_ordinal)?;
    let device_url = unique_bound_origin(text, &identity.session, expected_ordinal)?;
    Ok(Phase33BootEvidence {
        session: identity.session.clone(),
        boot_ordinal: expected_ordinal,
        device_url,
    })
}

fn parse_identities(text: &str) -> Result<Vec<Identity>, Phase33EvidenceError> {
    text.lines()
        .filter_map(|line| {
            line.split_once("runtime_boot_identity ")
                .map(|(_, tail)| tail)
        })
        .map(parse_identity)
        .collect()
}

fn parse_identity(tail: &str) -> Result<Identity, Phase33EvidenceError> {
    let fields: Vec<_> = tail.split_whitespace().take(6).collect();
    if fields.len() != 5 || fields[4] != "redacted=true" {
        return Err(error("boot_identity_corrupt"));
    }
    let session = field(fields[0], "session=")?;
    if session.len() != 32 || !session.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(error("boot_identity_corrupt"));
    }
    Ok(Identity {
        session: session.to_ascii_lowercase(),
        ordinal: parse_u64_field(fields[1], "boot_ordinal=")?,
        reset_reason: field(fields[2], "reset_reason=")?.to_owned(),
        uptime_ms: parse_u64_field(fields[3], "uptime_ms=")?,
    })
}

fn parse_origins(text: &str) -> Result<Vec<Origin>, Phase33EvidenceError> {
    text.lines()
        .filter_map(|line| line.split_once("runtime_origin ").map(|(_, tail)| tail))
        .map(|tail| {
            let fields: Vec<_> = tail.split_whitespace().take(5).collect();
            if fields.len() != 4 || fields[3] != "redacted=true" {
                return Err(error("runtime_origin_corrupt"));
            }
            let session = field(fields[0], "session=")?;
            if session.len() != 32 || !session.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(error("runtime_origin_corrupt"));
            }
            let device_url = field(fields[2], "device_url=")?;
            if !is_origin_only_url(device_url) {
                return Err(error("runtime_origin_corrupt"));
            }
            Ok(Origin {
                session: session.to_ascii_lowercase(),
                ordinal: parse_u64_field(fields[1], "boot_ordinal=")?,
                device_url: device_url.to_owned(),
            })
        })
        .collect()
}

fn is_origin_only_url(value: &str) -> bool {
    let maybe_authority = value
        .strip_prefix("http://")
        .or_else(|| value.strip_prefix("https://"));
    let Some(authority) = maybe_authority else {
        return false;
    };
    !authority.is_empty()
        && !authority.chars().any(|character| {
            matches!(character, '/' | '@' | '?' | '#') || character.is_whitespace()
        })
}

fn unique_identity<'a>(
    identities: &'a [Identity],
    missing: &'static str,
) -> Result<&'a Identity, Phase33EvidenceError> {
    let Some(first) = identities.first() else {
        return Err(error(missing));
    };
    if identities.iter().any(|identity| {
        identity.session != first.session
            || identity.ordinal != first.ordinal
            || identity.reset_reason != first.reset_reason
    }) {
        return Err(error("baseline_multiple_sessions"));
    }
    Ok(first)
}

fn validate_monotonic_uptime(
    identities: &[Identity],
    session: &str,
    ordinal: u64,
) -> Result<(), Phase33EvidenceError> {
    let mut maybe_previous = None;
    for identity in identities
        .iter()
        .filter(|item| item.session == session && item.ordinal == ordinal)
    {
        if maybe_previous.is_some_and(|previous| identity.uptime_ms < previous) {
            return Err(error("boot_identity_uptime_nonmonotonic"));
        }
        maybe_previous = Some(identity.uptime_ms);
    }
    Ok(())
}

fn unique_bound_origin(
    text: &str,
    session: &str,
    ordinal: u64,
) -> Result<String, Phase33EvidenceError> {
    let origins = parse_origins(text)?;
    if origins.is_empty() {
        return Err(error("runtime_origin_missing"));
    }
    if origins
        .iter()
        .any(|origin| origin.session != session || origin.ordinal != ordinal)
    {
        return Err(error("runtime_origin_wrong_session"));
    }
    let urls: BTreeSet<_> = origins.iter().map(|origin| &origin.device_url).collect();
    if urls.len() != 1 {
        return Err(error("runtime_origin_multiple"));
    }
    Ok(origins[0].device_url.clone())
}

fn field<'a>(value: &'a str, prefix: &str) -> Result<&'a str, Phase33EvidenceError> {
    value
        .strip_prefix(prefix)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error("phase33_marker_corrupt"))
}

fn parse_u64_field(value: &str, prefix: &str) -> Result<u64, Phase33EvidenceError> {
    field(value, prefix)?
        .parse()
        .map_err(|_| error("phase33_marker_corrupt"))
}

const fn error(category: &'static str) -> Phase33EvidenceError {
    Phase33EvidenceError { category }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn identity(session: &str, ordinal: u64, reset: &str, uptime: u64) -> String {
        format!("runtime_boot_identity session={session} boot_ordinal={ordinal} reset_reason={reset} uptime_ms={uptime} redacted=true")
    }

    fn origin(session: &str, ordinal: u64, url: &str) -> String {
        format!("runtime_origin session={session} boot_ordinal={ordinal} device_url={url} redacted=true")
    }

    #[test]
    fn late_attach_accepts_duplicate_replay_without_one_shot_markers() {
        // Arrange
        let text = format!(
            "runtime_heartbeat session={B} sequence=119 uptime_ms=130181 cadence_ms=10000 listener_armed=true redacted=true\n{}\n{}\n{}",
            identity(B, 8, "software_cpu", 130_000),
            origin(B, 8, "http://device"),
            origin(B, 8, "http://device")
        );

        // Act
        let result = classify_phase33_post_restart(&text, A, 7);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn post_restart_rejects_n_plus_two() {
        // Arrange
        let text = format!(
            "{}\n{}",
            identity(B, 9, "software_cpu", 1),
            origin(B, 9, "http://device")
        );

        // Act
        let error = classify_phase33_post_restart(&text, A, 7).expect_err("N+2 must fail");

        // Assert
        assert_eq!(error.category, "post_restart_ordinal_nonmonotonic");
    }

    #[test]
    fn post_restart_rejects_unchanged_or_multiple_sessions() {
        // Arrange
        let unchanged = identity(A, 8, "software_cpu", 1);
        let multiple = format!(
            "{}\n{}",
            identity(B, 8, "software_cpu", 1),
            identity(A, 8, "software_cpu", 2)
        );

        // Act
        let unchanged_error =
            classify_phase33_post_restart(&unchanged, A, 7).expect_err("unchanged must fail");
        let multiple_error =
            classify_phase33_post_restart(&multiple, A, 7).expect_err("multiple must fail");

        // Assert
        assert_eq!(unchanged_error.category, "post_restart_session_unchanged");
        assert_eq!(multiple_error.category, "post_restart_multiple_sessions");
    }

    #[test]
    fn post_restart_rejects_wrong_reset_and_wrong_origin_binding() {
        // Arrange
        let wrong_reset = format!(
            "{}\n{}",
            identity(B, 8, "power_on", 1),
            origin(B, 8, "http://device")
        );
        let wrong_origin = format!(
            "{}\n{}",
            identity(B, 8, "software_cpu", 1),
            origin(A, 8, "http://device")
        );

        // Act
        let reset_error =
            classify_phase33_post_restart(&wrong_reset, A, 7).expect_err("reset must fail");
        let origin_error =
            classify_phase33_post_restart(&wrong_origin, A, 7).expect_err("binding must fail");

        // Assert
        assert_eq!(reset_error.category, "post_restart_reset_reason_wrong");
        assert_eq!(origin_error.category, "runtime_origin_wrong_session");
    }

    #[test]
    fn post_restart_rejects_zero_or_multiple_origins() {
        // Arrange
        let no_origin = identity(B, 8, "software_cpu", 1);
        let multiple = format!(
            "{}\n{}\n{}",
            identity(B, 8, "software_cpu", 1),
            origin(B, 8, "http://one"),
            origin(B, 8, "http://two")
        );

        // Act
        let missing_error =
            classify_phase33_post_restart(&no_origin, A, 7).expect_err("missing must fail");
        let multiple_error =
            classify_phase33_post_restart(&multiple, A, 7).expect_err("multiple must fail");

        // Assert
        assert_eq!(missing_error.category, "runtime_origin_missing");
        assert_eq!(multiple_error.category, "runtime_origin_multiple");
    }

    #[test]
    fn post_restart_rejects_corrupt_origin_and_nonmonotonic_uptime() {
        // Arrange
        let corrupt_origin = format!(
            "{}\n{}",
            identity(B, 8, "software_cpu", 1),
            origin(B, 8, "http://device/path")
        );
        let nonmonotonic = format!(
            "{}\n{}\n{}",
            identity(B, 8, "software_cpu", 20),
            identity(B, 8, "software_cpu", 10),
            origin(B, 8, "http://device")
        );

        // Act
        let corrupt_error = classify_phase33_post_restart(&corrupt_origin, A, 7)
            .expect_err("corrupt origin must fail");
        let uptime_error = classify_phase33_post_restart(&nonmonotonic, A, 7)
            .expect_err("nonmonotonic uptime must fail");

        // Assert
        assert_eq!(corrupt_error.category, "runtime_origin_corrupt");
        assert_eq!(uptime_error.category, "boot_identity_uptime_nonmonotonic");
    }

    #[test]
    fn baseline_rejects_inconsistent_reset_reason_replay() {
        // Arrange
        let text = format!(
            "{}\n{}\n{}",
            identity(A, 7, "power_on", 10),
            identity(A, 7, "software_cpu", 20),
            origin(A, 7, "http://device")
        );

        // Act
        let error = classify_phase33_baseline(&text).expect_err("inconsistent replay must fail");

        // Assert
        assert_eq!(error.category, "baseline_multiple_sessions");
    }
}
