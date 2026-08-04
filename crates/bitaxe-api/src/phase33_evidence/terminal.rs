use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocatedIdentity {
    identity: Identity,
    line_start: usize,
}

/// Classifies the final complete boot epoch from a sequential flash-monitor transcript.
///
/// Exact-package flashing can reset the same device more than once before its final
/// receive-only monitor is attached. Earlier queued epochs are admissible only as a
/// well-ordered prefix; the terminal epoch must independently satisfy the strict
/// baseline and passive safe-state contracts.
pub fn classify_phase33_terminal_baseline(
    text: &str,
) -> Result<Phase33BootEvidence, Phase33EvidenceError> {
    let identities = parse_located_identities(text)?;
    validate_known_origins(text, &identities)?;
    let terminal_start = terminal_epoch_start(&identities)?;
    let terminal = &text[terminal_start..];
    if !has_passive_safe_state(terminal) {
        return Err(error("terminal_safe_state_missing"));
    }
    classify_phase33_baseline(terminal)
}

fn parse_located_identities(text: &str) -> Result<Vec<LocatedIdentity>, Phase33EvidenceError> {
    let mut line_start = 0;
    let mut identities = Vec::new();
    for line in text.split_inclusive('\n') {
        if let Some((_, tail)) = line.split_once("runtime_boot_identity ") {
            identities.push(LocatedIdentity {
                identity: parse_identity(tail)?,
                line_start,
            });
        }
        line_start += line.len();
    }
    Ok(identities)
}

fn validate_known_origins(
    text: &str,
    identities: &[LocatedIdentity],
) -> Result<(), Phase33EvidenceError> {
    for origin in parse_origins(text)? {
        if !identities.iter().any(|located| {
            located.identity.session == origin.session && located.identity.ordinal == origin.ordinal
        }) {
            return Err(error("terminal_epoch_ambiguous"));
        }
    }
    Ok(())
}

fn terminal_epoch_start(identities: &[LocatedIdentity]) -> Result<usize, Phase33EvidenceError> {
    let Some(first) = identities.first() else {
        return Err(error("baseline_identity_missing"));
    };
    let mut seen_sessions = BTreeSet::from([first.identity.session.as_str()]);
    let mut previous = &first.identity;
    let mut terminal_start = first.line_start;

    for located in &identities[1..] {
        let current = &located.identity;
        if current.session == previous.session {
            if current.ordinal != previous.ordinal || current.reset_reason != previous.reset_reason
            {
                return Err(error("terminal_epoch_ambiguous"));
            }
        } else {
            let Some(expected_ordinal) = previous.ordinal.checked_add(1) else {
                return Err(error("baseline_ordinal_overflow"));
            };
            if current.ordinal != expected_ordinal {
                return Err(error("terminal_epoch_nonsequential"));
            }
            if !seen_sessions.insert(current.session.as_str()) {
                return Err(error("terminal_epoch_ambiguous"));
            }
            terminal_start = located.line_start;
        }
        previous = current;
    }
    Ok(terminal_start)
}

fn has_passive_safe_state(text: &str) -> bool {
    text.lines().any(|line| {
        let boot_safe_state = line.contains("safe_state:")
            && line.contains("mining=disabled")
            && line.contains("asic_work_submission=disabled")
            && line.contains("hardware_control=disabled");
        let runtime_attestation = line.contains("runtime_boot_attestation ")
            && line.contains("mining=disabled")
            && line.contains("work_submission=disabled")
            && line.contains("hardware_control=disabled")
            && line.contains("redacted=true");
        boot_safe_state || runtime_attestation
    })
}
