//! 600-second soak continuity observation and judgement (firmware ADR-0033).
//!
//! The observer polls `/api/system/info` and reads `/api/ws/live` on separate threads and writes a
//! private, metadata-only NDJSON journal. The pure judge credits twenty half-open 30-second windows of
//! device active time, mapped onto host time from Gate status observations.
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use zeroize::Zeroizing;

pub mod judge;
mod observer;
pub mod sample;
#[cfg(test)]
#[path = "soak/tests.rs"]
mod tests;

pub const JOURNAL_SCHEMA: &str = "soak-observer-v1";
const INPUT_SCHEMA: &str = "soak-observer-input-v1";
/// No elapsed deadline: an owner-held stdin bounds the observer (AGENTS.md "Asynchronous Human
/// Checkpoints"). A stop request or stdin EOF, including the owner's exit, ends it.
const OBSERVATION_LIMIT: Duration = Duration::MAX;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Handoff<'a> {
    schema: &'a str,
    ipv4: &'a str,
    port: u16,
    observed_unix_ms: u64,
    expires_unix_ms: u64,
    binding_verified: bool,
}

/// A fresh, possession-bound private IPv4 endpoint from the Gate handoff (AGENTS.md: no discovery).
fn origin(bytes: &[u8], now: u64) -> Result<Zeroizing<String>, &'static str> {
    let handoff: Handoff<'_> = serde_json::from_slice(bytes).map_err(|_| "invalid_input")?;
    let ip: std::net::Ipv4Addr = handoff.ipv4.parse().map_err(|_| "invalid_input")?;
    if handoff.schema != INPUT_SCHEMA
        || !handoff.binding_verified
        || !ip.is_private()
        || handoff.port == 0
        || handoff.observed_unix_ms > now
        || now.saturating_sub(handoff.observed_unix_ms) > 5000
        || handoff.expires_unix_ms <= now
        || handoff
            .expires_unix_ms
            .saturating_sub(handoff.observed_unix_ms)
            > 5000
    {
        return Err("invalid_input");
    }
    Ok(Zeroizing::new(format!("http://{}:{}", ip, handoff.port)))
}

/// Runs the stdin-controlled observer. Stdin carries one handoff line, then `{"op":"stop"}` or EOF.
/// Stdout is the journal; the final line records the closing reason. Raw errors are never printed.
#[cfg(unix)]
pub fn run_observer() -> i32 {
    let mut output = std::io::stdout().lock();
    let result = (|| {
        if std::env::args_os().count() != 1 {
            return Err("invalid_input");
        }
        let mut input = crate::cadence::input::PrivateInput::new(0);
        let handoff = input.initial_line(Instant::now() + Duration::from_secs(5))?;
        let endpoint = Arc::new(origin(&handoff, observer::now_unix_ms())?);
        observer::run_threads(endpoint, &mut output, OBSERVATION_LIMIT, || {
            input.stop_requested()
        })
    })();
    let (event, reason, exit) = match result {
        Ok(reason) => ("stopped", reason, 0),
        Err(reason) => ("error", reason, 1),
    };
    let line = serde_json::json!({"schema": JOURNAL_SCHEMA, "hostUnixMs": observer::now_unix_ms(),
        "transport": null, "event": event, "sample": null, "detail": {"reason": reason}});
    if observer::write_line(&mut output, &line).is_err() {
        return 1;
    }
    exit
}

#[cfg(not(unix))]
pub fn run_observer() -> i32 {
    1
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClockRecord {
    schema: String,
    observations: Vec<judge::ClockObservation>,
}

fn judge_files(arguments: &[String]) -> Result<judge::SoakJudgement, &'static str> {
    let [journal, clock, source, elf] = arguments else {
        return Err("invalid_arguments");
    };
    let base = std::env::var_os("BUILD_WORKING_DIRECTORY")
        .map(std::path::PathBuf::from)
        .unwrap_or_default();
    let text = std::fs::read_to_string(base.join(journal)).map_err(|_| "journal_unreadable")?;
    // A journal cut by a killed observer may end mid-line; only complete lines are judged.
    let complete = text
        .rsplit_once('\n')
        .map_or("", |(complete, _partial)| complete);
    let lines: Vec<judge::JournalLine> = complete
        .lines()
        .map(|line| serde_json::from_str(line).map_err(|_| "journal_invalid"))
        .collect::<Result<_, _>>()?;
    let clock: ClockRecord =
        serde_json::from_slice(&std::fs::read(base.join(clock)).map_err(|_| "clock_unreadable")?)
            .map_err(|_| "clock_invalid")?;
    if clock.schema != "soak-gate-clock-v1" {
        return Err("clock_invalid");
    }
    let expected = judge::ExpectedPackage {
        source_commit: source.clone(),
        app_elf_sha256: elf.clone(),
    };
    Ok(judge::judge(&lines, &clock.observations, &expected))
}

/// `soak-judge <journal> <gate-clock> <expected-source-commit> <expected-app-elf-sha256>`: prints one
/// numeric JSON judgement; exit 0 only when every window and terminal criterion passes.
pub fn run_judge() -> i32 {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match judge_files(&arguments) {
        Ok(judgement) => {
            println!("{}", serde_json::to_string(&judgement).unwrap_or_default());
            i32::from(!judgement.passed)
        }
        Err(reason) => {
            println!(
                "{}",
                serde_json::json!({"schema": "soak-judge-v1", "passed": false, "error": reason})
            );
            2
        }
    }
}
