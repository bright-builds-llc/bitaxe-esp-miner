//! The read-only BAP probe: a fixed sequence that changes no setting, judged step by
//! step, stopping at the first failure so the earliest typed failure is the one kept.
//!
//! Steps: request `systemInfo`, `shares`, `block_height`, `found_block` and
//! `show_new_block`, then subscribe to each numeric
//! parameter (`hashrate`, `temperature`, `power`, `voltage`, `current`,
//! `fan_speed`, `best_difficulty`, `block_height`, `found_block`), listen, and
//! unsubscribe. Requests answer only `systemInfo`, `shares`, `block_height`,
//! `found_block` and `show_new_block`; `hashrate` is subscription-only, as in the
//! reference firmware. Each step's outcome reduces to a closed category; any
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
    /// Another answered request: `block_height`, `found_block` or `show_new_block`.
    Request(BapParameter),
    /// Subscribe, listen, then unsubscribe.
    Subscription(BapParameter),
}

impl Step {
    /// Every read-only step: each answered request, then each numeric subscription.
    /// `wifi` is left out because it carries the network name and address.
    pub const ALL: [Self; 14] = [
        Self::SystemInfo,
        Self::Shares,
        Self::Request(BapParameter::BlockHeight),
        Self::Request(BapParameter::FoundBlock),
        Self::Request(BapParameter::ShowNewBlock),
        Self::Subscription(BapParameter::Hashrate),
        Self::Subscription(BapParameter::Temperature),
        Self::Subscription(BapParameter::Power),
        Self::Subscription(BapParameter::Voltage),
        Self::Subscription(BapParameter::Current),
        Self::Subscription(BapParameter::FanSpeed),
        Self::Subscription(BapParameter::BestDifficulty),
        Self::Subscription(BapParameter::BlockHeight),
        Self::Subscription(BapParameter::FoundBlock),
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SystemInfo => "request_system_info",
            Self::Shares => "request_shares",
            Self::Request(BapParameter::BlockHeight) => "request_block_height",
            Self::Request(BapParameter::FoundBlock) => "request_found_block",
            Self::Request(BapParameter::ShowNewBlock) => "request_show_new_block",
            Self::Request(_) => "request_other",
            Self::Subscription(BapParameter::Hashrate) => "subscribe_hashrate",
            Self::Subscription(BapParameter::Temperature) => "subscribe_temperature",
            Self::Subscription(BapParameter::Power) => "subscribe_power",
            Self::Subscription(BapParameter::Voltage) => "subscribe_voltage",
            Self::Subscription(BapParameter::Current) => "subscribe_current",
            Self::Subscription(BapParameter::FanSpeed) => "subscribe_fan_speed",
            Self::Subscription(BapParameter::BestDifficulty) => "subscribe_best_difficulty",
            Self::Subscription(BapParameter::BlockHeight) => "subscribe_block_height",
            Self::Subscription(BapParameter::FoundBlock) => "subscribe_found_block",
            Self::Subscription(_) => "subscribe_other",
        }
    }

    /// The `RES` tokens this step's answer or update carries, as in the reference
    /// `bap_handlers.c` and `bap_subscription.c`.
    #[must_use]
    pub const fn update_tokens(self) -> &'static [&'static str] {
        match self {
            Self::SystemInfo => &SYSTEM_INFO_TOKENS,
            Self::Shares => &["shares"],
            Self::Request(BapParameter::BlockHeight) => &["block_height"],
            // The reference answers this request with `block_found`, not `found_block`.
            Self::Request(BapParameter::FoundBlock) => &["block_found"],
            Self::Request(BapParameter::ShowNewBlock) => &["show_new_block"],
            Self::Request(_) => &[],
            Self::Subscription(BapParameter::Temperature) => &["chipTemp", "vrTemp"],
            Self::Subscription(BapParameter::Hashrate) => &["hashrate"],
            Self::Subscription(BapParameter::Power) => &["power"],
            Self::Subscription(BapParameter::Voltage) => &["voltage"],
            Self::Subscription(BapParameter::Current) => &["current"],
            Self::Subscription(BapParameter::FanSpeed) => &["fan_speed"],
            Self::Subscription(BapParameter::BestDifficulty) => &["best_difficulty"],
            Self::Subscription(BapParameter::BlockHeight) => &["block_height"],
            Self::Subscription(BapParameter::FoundBlock) => &["found_block"],
            Self::Subscription(_) => &[],
        }
    }

    /// Whether `token` may arrive in this step: one of its answer or update
    /// tokens, or a subscription's own acknowledgement token.
    #[must_use]
    pub fn expects(self, token: &str) -> bool {
        let acknowledged = match self {
            Self::Subscription(parameter) => parameter.token() == token,
            _ => false,
        };
        acknowledged || self.update_tokens().contains(&token)
    }

    #[must_use]
    pub const fn operation(self) -> Operation {
        match self {
            Self::SystemInfo => Operation::Request(BapParameter::SystemInfo),
            Self::Shares => Operation::Request(BapParameter::Shares),
            Self::Request(parameter) => Operation::Request(parameter),
            Self::Subscription(parameter) => Operation::Subscribe {
                parameter,
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
        Outcome::Answered
            if matches!(step, Step::Shares | Step::Request(_))
                && step
                    .update_tokens()
                    .iter()
                    .any(|token| has(BapCommand::Response, token)) =>
        {
            "passed"
        }
        Outcome::Confirmed
            if matches!(step, Step::Subscription(_))
                && step
                    .update_tokens()
                    .iter()
                    .any(|token| has(BapCommand::Response, token)) =>
        {
            "passed"
        }
        Outcome::Confirmed if matches!(step, Step::Subscription(_)) => "no_subscription_update",
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
        if !step.expects(frame.parameter_token()) {
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
        // `%.2f`
        "hashrate" | "power" | "voltage" | "current" => is_decimal(value, 2),
        // `%f`; a temperature may be negative.
        "chipTemp" | "vrTemp" => is_decimal(value.strip_prefix('-').unwrap_or(value), 6),
        // `%d`
        "fan_speed" | "block_height" | "found_block" | "block_found" => is_digits(value),
        "show_new_block" => *value == "0" || *value == "1",
        "best_difficulty" => is_suffixed_difficulty(value),
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

/// A non-negative number with exactly `places` decimals, as the reference's `%.Nf`.
fn is_decimal(value: &str, places: usize) -> bool {
    value.split_once('.').is_some_and(|(whole, fraction)| {
        is_digits(whole) && fraction.len() == places && is_digits(fraction)
    })
}

/// The reference `suffixString(value, ..., 0)`: a bare integer below 1,000, else
/// two decimals and one SI suffix.
fn is_suffixed_difficulty(value: &str) -> bool {
    if is_digits(value) {
        return true;
    }
    value
        .strip_suffix(['k', 'M', 'G', 'T', 'P', 'E'])
        .is_some_and(|number| is_decimal(number, 2))
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

/// Runs every read-only step, stopping at the first that does not pass.
pub fn probe(link: &mut impl Link, listen_seconds: u64) -> io::Result<ProbeRecord> {
    probe_steps(link, listen_seconds, &Step::ALL)
}

/// Runs `steps` in order, stopping at the first that does not pass.
pub fn probe_steps(
    link: &mut impl Link,
    listen_seconds: u64,
    sequence: &[Step],
) -> io::Result<ProbeRecord> {
    let timing = Timing {
        answer_ms: ANSWER_MS,
        quiet_ms: QUIET_MS,
        listen_ms: listen_seconds.min(MAXIMUM_LISTEN_SECONDS) * 1_000,
    };
    let mut steps = Vec::new();
    for &step in sequence {
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
