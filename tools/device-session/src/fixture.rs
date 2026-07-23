use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{SessionArtifacts, SessionEvent, SessionRequest, SessionState, TerminalCategory};

pub const FIXTURE_SCHEMA: &str = "esp-device-session-fixture-v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureTranscript {
    pub schema_version: String,
    pub events: Vec<SessionEvent>,
}

pub fn run_fixture_session(
    request: SessionRequest,
    transcript: FixtureTranscript,
    mut artifacts: SessionArtifacts,
) -> Result<TerminalCategory> {
    if transcript.schema_version != FIXTURE_SCHEMA {
        bail!("device-session fixture schema is unsupported");
    }
    if !request.schema_is_valid() {
        bail!("device-session request schema is invalid");
    }
    let mut state = SessionState::new(
        request.baseline,
        request.expected_postcondition,
        request.trusted_origin,
    );
    for event in transcript.events {
        let admitted = artifacts
            .record_event(&event)
            .context("failed to record private device-session event")?;
        if !admitted {
            state.apply(SessionEvent::AdmissionRejected);
            break;
        }
        state.apply(event);
    }
    let terminal_category = state.terminal_category();
    artifacts.finish(&state)?;
    Ok(terminal_category)
}
