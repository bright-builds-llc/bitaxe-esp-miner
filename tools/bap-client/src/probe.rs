//! The read-only BAP probe: a fixed sequence that changes no setting, judged step by
//! step, stopping at the first failure so the earliest typed failure is the one kept.
//!
//! Steps: request `systemInfo`, request `shares`, then subscribe to `hashrate`,
//! listen, and unsubscribe. Requests answer only `systemInfo`, `shares`,
//! `block_height`, `found_block` and `show_new_block`; `hashrate` is
//! subscription-only, as in the reference firmware. Each step's outcome reduces to a closed category; any
//! failed step selects `stop_hardware_blocker` with a redacted boundary signature.

use crate::exchange::{run, Link, Operation, Outcome, Report, Timing};
use crate::redaction::printable;
use bitaxe_core::bap::{BapCommand, BapParameter};
use serde_json::{json, Value};
use std::io;

/// The longest a probe subscription may listen.
pub const MAXIMUM_LISTEN_SECONDS: u64 = 30;
const ANSWER_MS: u64 = 3_000;
const QUIET_MS: u64 = 500;
const SUBSCRIPTION_INTERVAL_MS: u32 = 1_000;
/// The board the probe contract names: an Ultra 205, whose upstream family is
/// `Ultra` and whose ASIC is a BM1366.
const DEVICE_MODEL: &str = "Ultra";
const ASIC_MODEL: &str = "BM1366";
const SYSTEM_INFO_TOKENS: [&str; 5] = ["deviceModel", "asicModel", "pool", "poolPort", "poolUser"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    SystemInfo,
    Shares,
    HashrateSubscription,
}

impl Step {
    pub const ALL: [Self; 3] = [Self::SystemInfo, Self::Shares, Self::HashrateSubscription];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SystemInfo => "request_system_info",
            Self::Shares => "request_shares",
            Self::HashrateSubscription => "subscribe_hashrate",
        }
    }

    /// The only answer and acknowledgement tokens this step may receive.
    #[must_use]
    pub const fn expected_tokens(self) -> &'static [&'static str] {
        match self {
            Self::SystemInfo => &SYSTEM_INFO_TOKENS,
            Self::Shares => &["shares"],
            Self::HashrateSubscription => &["hashrate"],
        }
    }

    #[must_use]
    pub const fn operation(self) -> Operation {
        match self {
            Self::SystemInfo => Operation::Request(BapParameter::SystemInfo),
            Self::Shares => Operation::Request(BapParameter::Shares),
            Self::HashrateSubscription => Operation::Subscribe {
                parameter: BapParameter::Hashrate,
                maybe_interval_ms: Some(SUBSCRIPTION_INTERVAL_MS),
            },
        }
    }
}

/// One step's closed result category.
#[must_use]
pub fn judge(step: Step, report: &Report) -> &'static str {
    let has = |command: BapCommand, token: &str| {
        report
            .frames
            .iter()
            .any(|frame| frame.command() == command && frame.parameter_token() == token)
    };
    let category = match &report.outcome {
        _ if report.bytes_received == 0 => "no_bytes_received",
        _ if report.frames.is_empty() => "no_parseable_frames",
        Outcome::Rejected(_) => "device_rejected",
        Outcome::TimedOut => "timed_out",
        Outcome::UnsubscribeUnconfirmed => "unsubscribe_unconfirmed",
        Outcome::Answered if step == Step::SystemInfo => {
            if has(BapCommand::Response, "deviceModel") && has(BapCommand::Response, "asicModel") {
                "passed"
            } else {
                "system_info_incomplete"
            }
        }
        Outcome::Answered if step == Step::Shares && has(BapCommand::Response, "shares") => {
            "passed"
        }
        Outcome::Confirmed
            if step == Step::HashrateSubscription && has(BapCommand::Response, "hashrate") =>
        {
            "passed"
        }
        Outcome::Confirmed if step == Step::HashrateSubscription => "no_subscription_update",
        _ => "unexpected_outcome",
    };
    if category != "passed" {
        return category;
    }
    judge_values(step, report)
}

/// A passing step's frames must carry only its own tokens, in the reference formats.
fn judge_values(step: Step, report: &Report) -> &'static str {
    let answers = report.frames.iter().filter(|frame| {
        matches!(
            frame.command(),
            BapCommand::Response | BapCommand::Acknowledge
        )
    });
    let mut values = Vec::new();
    for frame in answers {
        if !step.expected_tokens().contains(&frame.parameter_token()) {
            return "unexpected_parameter";
        }
        if frame.command() == BapCommand::Response {
            values.push((frame.parameter_token(), frame.value().unwrap_or_default()));
        }
    }
    let valid = values.iter().all(|(token, value)| match *token {
        "deviceModel" => *value == DEVICE_MODEL,
        "asicModel" => *value == ASIC_MODEL,
        "shares" => is_share_counts(value),
        "hashrate" => is_two_decimal_number(value),
        _ => true,
    });
    if valid {
        "passed"
    } else {
        "value_mismatch"
    }
}

/// `accepted/rejected`, as the reference's `%llu/%llu`.
fn is_share_counts(value: &str) -> bool {
    value
        .split_once('/')
        .is_some_and(|(accepted, rejected)| is_digits(accepted) && is_digits(rejected))
}

/// A non-negative number with exactly two decimals, as the reference's `%.2f`.
fn is_two_decimal_number(value: &str) -> bool {
    value.split_once('.').is_some_and(|(whole, fraction)| {
        is_digits(whole) && fraction.len() == 2 && is_digits(fraction)
    })
}

fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// The probe's record: every step run, its frames with sensitive values redacted,
/// and the closed outcome with its boundary signature.
pub struct ProbeRecord {
    pub steps: Vec<Value>,
    pub outcome: &'static str,
    pub maybe_signature: Option<Value>,
}

/// Runs the read-only sequence, stopping at the first step that does not pass.
pub fn probe(link: &mut impl Link, listen_seconds: u64) -> io::Result<ProbeRecord> {
    let timing = Timing {
        answer_ms: ANSWER_MS,
        quiet_ms: QUIET_MS,
        listen_ms: listen_seconds.min(MAXIMUM_LISTEN_SECONDS) * 1_000,
    };
    let mut steps = Vec::new();
    for step in Step::ALL {
        let report = run(&step.operation(), link, timing)?;
        let category = judge(step, &report);
        steps.push(step_record(step, category, &report));
        if category != "passed" {
            let maybe_device_code = match &report.outcome {
                Outcome::Rejected(code) => Some(code.clone()),
                _ => None,
            };
            return Ok(ProbeRecord {
                steps,
                outcome: "stop_hardware_blocker",
                maybe_signature: Some(json!({
                    "step": step.label(),
                    "category": category,
                    "device_error_code": maybe_device_code,
                    "bytes_received": report.bytes_received > 0,
                    "malformed_lines": report.malformed_lines > 0,
                })),
            });
        }
    }
    Ok(ProbeRecord {
        steps,
        outcome: "complete",
        maybe_signature: None,
    })
}

fn step_record(step: Step, category: &str, report: &Report) -> Value {
    let frames: Vec<Value> = report
        .frames
        .iter()
        .map(|frame| {
            let token = frame.parameter_token();
            json!({
                "command": frame.command().token(),
                "parameter": token,
                "value": frame.value().map(|value| printable(token, value)),
            })
        })
        .collect();
    json!({
        "step": step.label(),
        "category": category,
        "bytes_received": report.bytes_received,
        "malformed_lines": report.malformed_lines,
        "frames": frames,
    })
}

#[cfg(test)]
mod tests;
