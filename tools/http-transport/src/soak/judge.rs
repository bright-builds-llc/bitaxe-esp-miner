//! Pure soak judgement over an observer journal and Gate active-time observations.
//!
//! Device active time is mapped to host time as `t0 = min(observedUnixMs - activeMs)` over Gate status
//! observations; their spread must stay within `CLOCK_RESIDUAL_LIMIT_MS`. Window `k` holds samples whose
//! active time lies in `[k * 30 000, (k + 1) * 30 000)`.
use serde::{Deserialize, Serialize};

use super::sample::{SoakSample, SoakTransport};
use crate::continuity::advances;

pub const WINDOW_MS: u64 = 30_000;
pub const WINDOWS: usize = 20;
pub const WORK_GATE_MS: u64 = 600_000;
pub const WEBSOCKET_GAP_LIMIT_MS: u64 = 5_000;
pub const CLOCK_RESIDUAL_LIMIT_MS: u64 = 1_000;
/// Mining must be observed stopped on each transport within this bound after the gate closes.
pub const TERMINAL_WINDOW_MS: u64 = 10_000;
/// Samples this close to an active edge, beyond the clock spread, are not credited to a window:
/// transport and Gate record delays bias their mapping by up to the spread.
pub const EDGE_GUARD_MS: u64 = 1_000;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalLine {
    pub schema: String,
    pub host_unix_ms: u64,
    #[serde(rename = "transport")]
    pub maybe_transport: Option<SoakTransport>,
    pub event: String,
    #[serde(default)]
    pub sample: Option<SoakSample>,
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClockObservation {
    pub observed_unix_ms: u64,
    pub active_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedPackage {
    pub source_commit: String,
    pub app_elf_sha256: String,
}

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransportWindow {
    pub samples: u64,
    pub credited: bool,
}

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WindowResult {
    pub index: usize,
    pub http: TransportWindow,
    pub websocket: TransportWindow,
}

/// Numeric and closed-category result only; safe to share after review.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SoakJudgement {
    pub schema: &'static str,
    pub passed: bool,
    pub failures: Vec<&'static str>,
    pub clock_residual_ms: u64,
    pub windows: Vec<WindowResult>,
    pub max_websocket_gap_ms: u64,
    pub max_http_gap_ms: u64,
    pub websocket_reconnects: u64,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub terminal_http_confirmed: bool,
    pub terminal_websocket_confirmed: bool,
    pub final_http_paused: bool,
    pub final_websocket_paused: bool,
    pub pool_settings_retained: bool,
}

struct Timed<'a> {
    active_ms: i128,
    transport: SoakTransport,
    sample: &'a SoakSample,
}

/// `t0` and the spread of `observedUnixMs - activeMs` across active observations.
pub fn clock_origin(observations: &[ClockObservation]) -> Option<(u64, u64)> {
    let offsets: Vec<u64> = observations
        .iter()
        // After the ASIC halts the device freezes active time while the Gate keeps recording, so only
        // observations taken while the work gate is open map device time onto host time.
        .filter(|observation| observation.active_ms > 0 && observation.active_ms < WORK_GATE_MS)
        .map(|observation| {
            observation
                .observed_unix_ms
                .checked_sub(observation.active_ms)
        })
        .collect::<Option<_>>()?;
    let origin = *offsets.iter().min()?;
    Some((origin, offsets.iter().max()? - origin))
}

fn timed<'a>(lines: &'a [JournalLine], origin: u64) -> Vec<Timed<'a>> {
    lines
        .iter()
        .filter(|line| line.event == "sample")
        .filter_map(|line| {
            Some(Timed {
                active_ms: i128::from(line.host_unix_ms) - i128::from(origin),
                transport: line.maybe_transport?,
                sample: line.sample.as_ref()?,
            })
        })
        .collect()
}

fn active_range(sample: &Timed<'_>) -> bool {
    (0..i128::from(WORK_GATE_MS)).contains(&sample.active_ms)
}

fn window_credited(samples: &[&Timed<'_>]) -> bool {
    let first =
        |pick: fn(&SoakSample) -> Option<u64>| samples.iter().find_map(|value| pick(value.sample));
    let last = |pick: fn(&SoakSample) -> Option<u64>| {
        samples.iter().rev().find_map(|value| pick(value.sample))
    };
    samples.len() >= 2
        && samples
            .iter()
            .all(|value| value.sample.mining_active && value.sample.safety_valid)
        && advances(
            first(|sample| sample.maybe_watchdog_feed_sequence),
            last(|sample| sample.maybe_watchdog_feed_sequence),
        )
        && advances(
            first(|sample| sample.maybe_checkpoint_sequence),
            last(|sample| sample.maybe_checkpoint_sequence),
        )
}

fn windows(samples: &[Timed<'_>], margin: u64) -> Vec<WindowResult> {
    let guarded = i128::from(margin)..i128::from(WORK_GATE_MS.saturating_sub(margin));
    (0..WINDOWS)
        .map(|index| {
            let window_range =
                i128::from(index as u64 * WINDOW_MS)..i128::from((index as u64 + 1) * WINDOW_MS);
            let range = window_range.start.max(guarded.start)..window_range.end.min(guarded.end);
            let window = |transport| {
                let selected: Vec<&Timed<'_>> = samples
                    .iter()
                    .filter(|value| {
                        value.transport == transport && range.contains(&value.active_ms)
                    })
                    .collect();
                TransportWindow {
                    samples: selected.len() as u64,
                    credited: window_credited(&selected),
                }
            };
            WindowResult {
                index,
                http: window(SoakTransport::Http),
                websocket: window(SoakTransport::Websocket),
            }
        })
        .collect()
}

/// Largest gap across the active range, including the leading and trailing edges.
fn max_gap(samples: &[Timed<'_>], transport: SoakTransport) -> u64 {
    let mut edges = vec![0_i128];
    edges.extend(
        samples
            .iter()
            .filter(|value| value.transport == transport && active_range(value))
            .map(|value| value.active_ms),
    );
    edges.push(i128::from(WORK_GATE_MS));
    edges
        .windows(2)
        .map(|pair| u64::try_from(pair[1] - pair[0]).unwrap_or(u64::MAX))
        .max()
        .unwrap_or(u64::MAX)
}

fn counters_regress(samples: &[Timed<'_>], transport: SoakTransport) -> bool {
    let values: Vec<_> = samples
        .iter()
        .filter(|value| value.transport == transport)
        .map(|value| {
            (
                value.sample.shares_accepted,
                value.sample.shares_rejected,
                value.sample.revision,
            )
        })
        .collect();
    values
        .windows(2)
        .any(|pair| pair[1].0 < pair[0].0 || pair[1].1 < pair[0].1 || pair[1].2 < pair[0].2)
}

/// Mining observed stopped within `TERMINAL_WINDOW_MS` of the gate closing.
fn terminal_confirmed(samples: &[Timed<'_>], transport: SoakTransport) -> bool {
    let end = i128::from(WORK_GATE_MS);
    samples.iter().any(|value| {
        value.transport == transport
            && (end..end + i128::from(TERMINAL_WINDOW_MS)).contains(&value.active_ms)
            && !value.sample.mining_active
            && !value.sample.start_mining_on_boot
    })
}

/// The last sample after restoration shows the paused baseline with `mineonboot` off.
fn final_paused(samples: &[Timed<'_>], transport: SoakTransport) -> bool {
    samples
        .iter()
        .rev()
        .find(|value| value.transport == transport)
        .is_some_and(|value| {
            value.active_ms >= i128::from(WORK_GATE_MS)
                && value.sample.mining_paused
                && !value.sample.mining_active
                && !value.sample.start_mining_on_boot
        })
}

fn identity_consistent(samples: &[Timed<'_>], expected: &ExpectedPackage) -> bool {
    let Some(first) = samples.first() else {
        return false;
    };
    samples
        .iter()
        .all(|value| value.sample.same_identity(first.sample))
        && first.sample.source_commit == expected.source_commit
        && first.sample.app_elf_sha256 == expected.app_elf_sha256
}

fn share_deltas(samples: &[Timed<'_>]) -> (u64, u64) {
    let http: Vec<&SoakSample> = samples
        .iter()
        .filter(|value| value.transport == SoakTransport::Http)
        .map(|value| value.sample)
        .collect();
    match (http.first(), http.last()) {
        (Some(first), Some(last)) => (
            last.shares_accepted.saturating_sub(first.shares_accepted),
            last.shares_rejected.saturating_sub(first.shares_rejected),
        ),
        _ => (0, 0),
    }
}

fn reconnects(lines: &[JournalLine]) -> u64 {
    lines
        .iter()
        .filter(|line| {
            line.event == "connected"
                && line
                    .detail
                    .as_ref()
                    .and_then(|detail| detail["reconnect"].as_bool())
                    == Some(true)
        })
        .count() as u64
}

/// Judge one soak. Every failure is a closed category; the earliest is not privileged here because
/// the supervisor preserves the earliest typed failure separately.
#[must_use]
pub fn judge(
    lines: &[JournalLine],
    observations: &[ClockObservation],
    expected: &ExpectedPackage,
) -> SoakJudgement {
    let mut failures = Vec::new();
    if lines
        .iter()
        .any(|line| line.schema != super::JOURNAL_SCHEMA)
    {
        failures.push("journal_schema");
    }
    let (origin, residual) = clock_origin(observations).unwrap_or((0, u64::MAX));
    if residual > CLOCK_RESIDUAL_LIMIT_MS {
        failures.push("clock_correlation_failed");
    }
    let samples = timed(lines, origin);
    let margin = residual.saturating_add(EDGE_GUARD_MS);
    let windows = windows(&samples, margin);
    let (accepted, rejected) = share_deltas(&samples);
    let http: Vec<&Timed<'_>> = samples
        .iter()
        .filter(|value| value.transport == SoakTransport::Http)
        .collect();
    // Every HTTP sample, not just the last, so a change reverted before the end still fails.
    let pool_retained = !http.is_empty()
        && http
            .iter()
            .all(|value| value.sample.maybe_pool_matches_initial == Some(true));
    let judgement = SoakJudgement {
        schema: "soak-judge-v1",
        passed: false,
        failures: Vec::new(),
        clock_residual_ms: residual,
        max_websocket_gap_ms: max_gap(&samples, SoakTransport::Websocket),
        max_http_gap_ms: max_gap(&samples, SoakTransport::Http),
        websocket_reconnects: reconnects(lines),
        accepted_shares: accepted,
        rejected_shares: rejected,
        terminal_http_confirmed: terminal_confirmed(&samples, SoakTransport::Http),
        terminal_websocket_confirmed: terminal_confirmed(&samples, SoakTransport::Websocket),
        final_http_paused: final_paused(&samples, SoakTransport::Http),
        final_websocket_paused: final_paused(&samples, SoakTransport::Websocket),
        pool_settings_retained: pool_retained,
        windows,
    };
    for (failed, category) in [
        (!identity_consistent(&samples, expected), "identity_changed"),
        (
            judgement.windows.iter().any(|window| !window.http.credited),
            "http_window_uncredited",
        ),
        (
            judgement
                .windows
                .iter()
                .any(|window| !window.websocket.credited),
            "websocket_window_uncredited",
        ),
        (
            judgement.max_websocket_gap_ms > WEBSOCKET_GAP_LIMIT_MS,
            "websocket_gap_exceeded",
        ),
        (
            counters_regress(&samples, SoakTransport::Http)
                || counters_regress(&samples, SoakTransport::Websocket),
            "counter_regressed",
        ),
        (accepted == 0, "no_accepted_share"),
        (
            !judgement.terminal_http_confirmed,
            "terminal_http_unconfirmed",
        ),
        (
            !judgement.terminal_websocket_confirmed,
            "terminal_websocket_unconfirmed",
        ),
        (!pool_retained, "pool_settings_changed"),
        (
            !judgement.final_http_paused || !judgement.final_websocket_paused,
            "final_state_not_paused",
        ),
    ] {
        if failed {
            failures.push(category);
        }
    }
    SoakJudgement {
        passed: failures.is_empty(),
        failures,
        ..judgement
    }
}
