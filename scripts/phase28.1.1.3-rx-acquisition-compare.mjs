#!/usr/bin/env node
/**
 * Phase 28.1.1.3 RX-acquisition comparator.
 *
 * Composes Phase 28.1.1.2 result-path parsers (lineEvents / classifyFrame /
 * summarizeLog). Emits first-class D-06 flood-safe RX-acquisition counts.
 * Never writes raw UART hex or credentials.
 *
 * Recommender prefers result_rx_acquisition_model for jobs + result_read +
 * !correlate (no partial_frame≥5 gate). Never recommends falsified
 * match_upstream_register_read_poll or post_max_baud_delay_2000.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  classifyFrame,
  lineEvents,
  summarizeLog,
} from "./phase28.1.1.2-result-path-compare.mjs";

const ALLOWED_RECOMMENDATIONS = new Set([
  "result_rx_acquisition_model",
  "asic_enable_power_sequencing",
  "none",
]);

const SUMMARY_RE =
  /asic_rx_acquisition_summary\s+idle=(\d+)\s+partial=(\d+)\s+clear=(\d+)\s+complete=(\d+)/g;

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.3-rx-acquisition-compare.mjs --upstream <log> --rust <log> --out <report.md>

Compares redaction-safe BM1366 RX-acquisition counts between upstream and Rust logs.
Emits D-06 first-class fields (partial_frame, clear_rx, rx_idle, rx_complete, …).
Composes Phase 28.1.1.2 lineEvents/classifyFrame/summarizeLog; never writes raw UART hex.
result_read_attempt is an RX-poll marker (D-05), not a TX probe.
`);
  process.exit(exitCode);
}

function parseArgs(argv) {
  const args = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      usage(0);
    }

    if (!arg.startsWith("--")) {
      usage(1);
    }

    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      usage(1);
    }

    args.set(arg.slice(2), value);
    index += 1;
  }

  return {
    upstream: args.get("upstream"),
    rust: args.get("rust"),
    out: args.get("out"),
  };
}

function readMaybeFile(filePath) {
  if (!filePath || !fs.existsSync(filePath)) {
    return null;
  }

  return fs.readFileSync(filePath, "utf8");
}

function count(summary, event) {
  return summary.counts.get(event) ?? 0;
}

/**
 * Parse compact flood-safe summary markers and return totals.
 * Shape: asic_rx_acquisition_summary idle=N partial=N clear=N complete=N
 */
function parseAcquisitionSummaries(text) {
  const totals = { idle: 0, partial: 0, clear: 0, complete: 0 };
  if (!text) {
    return totals;
  }

  SUMMARY_RE.lastIndex = 0;
  for (const match of text.matchAll(SUMMARY_RE)) {
    totals.idle += Number.parseInt(match[1], 10);
    totals.partial += Number.parseInt(match[2], 10);
    totals.clear += Number.parseInt(match[3], 10);
    totals.complete += Number.parseInt(match[4], 10);
  }

  return totals;
}

/**
 * Prefer max(event-count, summary-total) so flood-dropped uart_trace does not
 * zero out evidence when compact summary markers are present.
 */
function maxCount(eventCount, summaryTotal) {
  return Math.max(eventCount, summaryTotal);
}

/**
 * Phase 28.1.1.3 recommender (RESEARCH Pitfall 5 / D-04).
 * Never returns post_max_baud_delay_2000 or match_upstream_register_read_poll.
 */
function recommendRxAcquisitionInvestigation({
  blocked,
  jobTxRust,
  resultCorrelated,
  resultReadAttemptRust,
  asicEnableActiveRust,
  resetPulseRust,
}) {
  if (blocked) {
    return "none";
  }

  // Known gap: jobs + RX polls without correlate → RX acquisition model.
  // Do NOT require partial_frame >= 5 (flood makes those markers unreliable).
  if (jobTxRust > 0 && resultReadAttemptRust > 0 && !resultCorrelated) {
    return "result_rx_acquisition_model";
  }

  // Still gapped: only suggest enable/power when those markers are missing.
  if (jobTxRust > 0 && !resultCorrelated) {
    if (asicEnableActiveRust === 0 || resetPulseRust === 0) {
      return "asic_enable_power_sequencing";
    }
    return "none";
  }

  return "none";
}

function reportFor({ upstreamPath, rustPath, upstreamText, rustText }) {
  const blockedReasons = [];
  if (!upstreamText) {
    blockedReasons.push("upstream_log_missing");
  }
  if (!rustText) {
    blockedReasons.push("rust_log_missing");
  }

  const blocked = blockedReasons.length > 0;
  const upstreamSummary = summarizeLog(upstreamText);
  const rustSummary = summarizeLog(rustText);
  const upstreamAcq = parseAcquisitionSummaries(upstreamText);
  const rustAcq = parseAcquisitionSummaries(rustText);

  const partialFrameUpstream = maxCount(
    count(upstreamSummary, "partial_frame"),
    upstreamAcq.partial,
  );
  const partialFrameRust = maxCount(
    count(rustSummary, "partial_frame"),
    rustAcq.partial,
  );
  const clearRxUpstream = maxCount(count(upstreamSummary, "clear_rx"), upstreamAcq.clear);
  const clearRxRust = maxCount(count(rustSummary, "clear_rx"), rustAcq.clear);
  const rxIdleUpstream = maxCount(count(upstreamSummary, "rx_idle"), upstreamAcq.idle);
  const rxIdleRust = maxCount(count(rustSummary, "rx_idle"), rustAcq.idle);
  // rx_complete maps from rx_result_frame and/or summary complete=
  const rxCompleteUpstream = maxCount(
    count(upstreamSummary, "rx_result_frame"),
    upstreamAcq.complete,
  );
  const rxCompleteRust = maxCount(count(rustSummary, "rx_result_frame"), rustAcq.complete);

  const resultReadUpstream = count(upstreamSummary, "result_read_attempt");
  const resultReadRust = count(rustSummary, "result_read_attempt");
  const jobTxRust = count(rustSummary, "job_tx");
  const resultCorrelated =
    count(rustSummary, "result_correlated") > 0 ||
    Boolean(rustText && /asic_production_status=result_correlated/.test(rustText));
  const fakePoolSubmitObserved =
    count(upstreamSummary, "fake_pool_submit") > 0 ||
    count(rustSummary, "fake_pool_submit") > 0;
  const asicEnableActiveRust = count(rustSummary, "asic_enable_active");
  const resetPulseRust = count(rustSummary, "reset_pulse");

  let recommended = recommendRxAcquisitionInvestigation({
    blocked,
    jobTxRust,
    resultCorrelated,
    resultReadAttemptRust: resultReadRust,
    asicEnableActiveRust,
    resetPulseRust,
  });

  if (!ALLOWED_RECOMMENDATIONS.has(recommended)) {
    recommended = "none";
  }

  // Hard guard: never emit falsified prior-phase labels.
  if (
    recommended === "match_upstream_register_read_poll" ||
    recommended === "post_max_baud_delay_2000"
  ) {
    recommended = "none";
  }

  const comparisonStatus = blocked
    ? "blocked_safe_prerequisite"
    : resultCorrelated && fakePoolSubmitObserved
      ? "match"
      : "result_rx_acquisition_gap";

  return renderReport({
    comparisonStatus,
    upstreamPath,
    rustPath,
    blockedReasons,
    partialFrameUpstream,
    partialFrameRust,
    clearRxUpstream,
    clearRxRust,
    rxIdleUpstream,
    rxIdleRust,
    rxCompleteUpstream,
    rxCompleteRust,
    resultReadUpstream,
    resultReadRust,
    resultCorrelated,
    fakePoolSubmitObserved,
    recommended,
  });
}

function renderReport({
  comparisonStatus,
  upstreamPath,
  rustPath,
  blockedReasons,
  partialFrameUpstream,
  partialFrameRust,
  clearRxUpstream,
  clearRxRust,
  rxIdleUpstream,
  rxIdleRust,
  rxCompleteUpstream,
  rxCompleteRust,
  resultReadUpstream,
  resultReadRust,
  resultCorrelated,
  fakePoolSubmitObserved,
  recommended,
}) {
  const label = (value) => (value ? path.basename(value) : "missing");
  const resultReadUpstreamLabel =
    resultReadUpstream === 0
      ? `${resultReadUpstream} (expected_no_marker)`
      : String(resultReadUpstream);

  const lines = [
    "# Phase 28.1.1.3 RX-Acquisition Comparator Report",
    "",
    `comparison_status: ${comparisonStatus}`,
    `upstream_source_label: ${label(upstreamPath)}`,
    `rust_source_label: ${label(rustPath)}`,
    "raw_bytes_committed: false",
    "credential_contents_read: false",
    "phase30_promotion_input: pending",
    "",
    "## D-06 RX-Acquisition Metrics",
    "",
    `partial_frame_upstream: ${partialFrameUpstream}`,
    `partial_frame_rust: ${partialFrameRust}`,
    `clear_rx_upstream: ${clearRxUpstream}`,
    `clear_rx_rust: ${clearRxRust}`,
    `rx_idle_upstream: ${rxIdleUpstream}`,
    `rx_idle_rust: ${rxIdleRust}`,
    `rx_complete_upstream: ${rxCompleteUpstream}`,
    `rx_complete_rust: ${rxCompleteRust}`,
    `result_read_attempt_upstream: ${resultReadUpstreamLabel}`,
    `result_read_attempt_rust: ${resultReadRust}`,
    `result_correlated: ${resultCorrelated}`,
    `fake_pool_submit_observed: ${fakePoolSubmitObserved}`,
    `recommended_investigation: ${recommended}`,
    "",
    "## Notes",
    "",
    "`result_read_attempt` is an RX-poll marker (D-05), not a TX probe.",
    "Counts prefer max(event, asic_rx_acquisition_summary) for flood-safe evidence.",
    "Counts and booleans only; no raw UART hex or credential contents.",
  ];

  if (blockedReasons.length > 0) {
    lines.push("", `blocked_reasons: ${blockedReasons.join(",")}`);
  }

  return `${lines.join("\n")}\n`;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (!args.upstream || !args.rust || !args.out) {
    usage(1);
  }

  const report = reportFor({
    upstreamPath: args.upstream,
    rustPath: args.rust,
    upstreamText: readMaybeFile(args.upstream),
    rustText: readMaybeFile(args.rust),
  });

  fs.mkdirSync(path.dirname(args.out), { recursive: true });
  fs.writeFileSync(args.out, report, "utf8");
}

const isMain =
  process.argv[1] &&
  path.resolve(fileURLToPath(import.meta.url)) === path.resolve(process.argv[1]);

if (isMain) {
  main();
}

export {
  classifyFrame,
  lineEvents,
  parseAcquisitionSummaries,
  recommendRxAcquisitionInvestigation,
  reportFor,
  summarizeLog,
};
