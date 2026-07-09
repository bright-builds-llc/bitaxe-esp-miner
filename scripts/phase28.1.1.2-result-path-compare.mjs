#!/usr/bin/env node
/**
 * Phase 28.1.1.2 result-path comparator.
 *
 * Composes the Phase 28.1.1.1 below-job-byte event taxonomy (counts/booleans only).
 * Does not invent a parallel event set. Never writes raw UART hex or credentials.
 *
 * Recommender prefers match_upstream_register_read_poll for the known gap shape
 * and never returns the falsified post-max-baud delay investigation label.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const JOB_FRAME_LEN = 88;
const JOB_HEADER = 0x21;
const JOB_LENGTH_FIELD = 0x56;

const ALLOWED_RECOMMENDATIONS = new Set([
  "match_upstream_register_read_poll",
  "result_rx_acquisition_model",
  "asic_enable_power_sequencing",
  "none",
]);

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.2-result-path-compare.mjs --upstream <log> --rust <log> --out <report.md>

Compares redaction-safe BM1366 result-path counts between upstream and Rust logs.
Emits D-05 count/boolean fields only; never writes raw UART frames or credentials.
Reuses the Phase 28.1.1.1 event taxonomy (chip_id_read_tx, result_read_attempt, …).
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

// --- Parsers composed from phase28.1.1.1-below-job-byte-sequence-compare.mjs ---
// Same event names; do not invent a parallel taxonomy.

function toBytes(text) {
  return [...text.matchAll(/\b[0-9a-fA-F]{2}\b/g)].map((match) =>
    Number.parseInt(match[0], 16),
  );
}

function maybeFrameFromLine(line) {
  const maybeRustTx = line.match(/asic_uart_trace=tx\s+len=\d+\s+hex=([0-9a-fA-F ]+)/);
  if (maybeRustTx) {
    return { direction: "tx", bytes: toBytes(maybeRustTx[1]) };
  }

  const maybeRustRx = line.match(
    /asic_uart_trace=rx_complete\s+read_count=\d+\s+hex=([0-9a-fA-F ]+)/,
  );
  if (maybeRustRx) {
    return { direction: "rx", bytes: toBytes(maybeRustRx[1]) };
  }

  const maybeUpstreamDebug = line.match(/(?:^|\s)(tx|rx):\s*\[([0-9a-fA-F ]+)\]/i);
  if (maybeUpstreamDebug) {
    return {
      direction: maybeUpstreamDebug[1].toLowerCase(),
      bytes: toBytes(maybeUpstreamDebug[2]),
    };
  }

  return null;
}

function classifyFrame(frame) {
  const { direction, bytes } = frame;
  if (direction === "rx") {
    return bytes.length === 11 ? "rx_result_frame" : "rx_other_frame";
  }

  if (
    bytes.length === JOB_FRAME_LEN &&
    bytes[0] === 0x55 &&
    bytes[1] === 0xaa &&
    bytes[2] === JOB_HEADER &&
    bytes[3] === JOB_LENGTH_FIELD
  ) {
    return "job_tx";
  }

  if (bytes.length < 5 || bytes[0] !== 0x55 || bytes[1] !== 0xaa) {
    return "tx_other_frame";
  }

  const header = bytes[2];
  if (header === 0x52) {
    return "chip_id_read_tx";
  }
  if (header === 0x53) {
    return "chain_inactive_tx";
  }
  if (header === 0x40) {
    return "set_chip_address_tx";
  }

  const register = bytes[5];
  switch (register) {
    case 0x08:
      return "frequency_tx";
    case 0x10:
      return "nonce_space_tx";
    case 0x14:
      return "difficulty_mask_tx";
    case 0x18:
      return "misc_control_tx";
    case 0x28:
      return "max_baud_tx";
    case 0x2c:
      return "reg_2c_tx";
    case 0x3c:
      return "reg_3c_tx";
    case 0x54:
      return "reg_54_tx";
    case 0x58:
      return "reg_58_tx";
    case 0xa4:
      return "version_mask_tx";
    case 0xa8:
      return "reg_a8_tx";
    default:
      return "write_register_tx";
  }
}

function lineEvents(line) {
  const events = [];

  if (/phase27_safety_bring_up=started|Starting ASIC initialization/i.test(line)) {
    events.push("bring_up_started");
  }
  if (/asic_enable_status=active|Set ASIC voltage|asic_power=enabled/i.test(line)) {
    events.push("asic_enable_active");
  }
  if (/asic_reset_status=post_bring_up_pulse|ASIC reset|reset_pulse/i.test(line)) {
    events.push("reset_pulse");
  }
  if (/Performing full UART initialization|resetting baud to|kind=use_default_baud/i.test(line)) {
    events.push("use_default_baud");
  }
  if (/kind=wait_tx_done|asic_uart_trace=wait_tx_done/i.test(line)) {
    events.push("wait_tx_done");
  }
  if (/Setting max baud|kind=use_max_baud/i.test(line)) {
    events.push("use_max_baud");
  }
  if (/SERIAL_clear_buffer|kind=clear_rx|asic_uart_trace=clear_rx/i.test(line)) {
    events.push("clear_rx");
  }
  if (/mining_ready_init_started/i.test(line)) {
    events.push("mining_ready_init_started");
  }
  if (/mining_ready_init_complete|ASIC initialized successfully/i.test(line)) {
    events.push("mining_ready_init_complete");
  }
  if (/asic_production_trace=result_read_attempt/i.test(line)) {
    events.push("result_read_attempt");
  }
  if (/asic_uart_trace=rx_idle/i.test(line)) {
    events.push("rx_idle");
  }
  if (/asic_production_trace=register_read_parsed/i.test(line)) {
    events.push("register_read_parsed");
  }
  if (/asic_production_status=result_correlated|production_result_correlated/i.test(line)) {
    events.push("result_correlated");
  }
  if (
    /share_submission_status=(accepted|rejected)|accepted_submit_count=[1-9]|mining\.submit/i.test(
      line,
    )
  ) {
    events.push("fake_pool_submit");
  }
  if (/asic_uart_trace=partial_frame/i.test(line)) {
    events.push("partial_frame");
  }

  const maybeFrame = maybeFrameFromLine(line);
  if (maybeFrame) {
    events.push(classifyFrame(maybeFrame));
  }

  return events;
}

function summarizeLog(text) {
  const counts = new Map();

  if (!text) {
    return { counts };
  }

  for (const line of text.split(/\r?\n/)) {
    for (const event of lineEvents(line)) {
      counts.set(event, (counts.get(event) ?? 0) + 1);
    }
  }

  return { counts };
}

function count(summary, event) {
  return summary.counts.get(event) ?? 0;
}

/**
 * Result-path recommender (D-07/D-08). Closed enum only; never the falsified delay label.
 */
function recommendResultPathInvestigation({
  blocked,
  chipIdUpstream,
  chipIdRust,
  jobTxRust,
  resultCorrelated,
  resultReadAttemptRust,
  partialFrameRust,
  asicEnableActiveRust,
  resetPulseRust,
}) {
  if (blocked) {
    return "none";
  }

  // Known gap: upstream register-read poll flood vs Rust low chip_id_read_tx.
  if (
    chipIdUpstream >= 10 * Math.max(chipIdRust, 1) &&
    jobTxRust > 0 &&
    !resultCorrelated
  ) {
    return "match_upstream_register_read_poll";
  }

  // High partial_frame with jobs + result reads but no correlate → RX acquisition.
  if (
    jobTxRust > 0 &&
    resultReadAttemptRust > 0 &&
    !resultCorrelated &&
    partialFrameRust >= 5
  ) {
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

  const chipIdUpstream = count(upstreamSummary, "chip_id_read_tx");
  const chipIdRust = count(rustSummary, "chip_id_read_tx");
  const resultReadUpstream = count(upstreamSummary, "result_read_attempt");
  const resultReadRust = count(rustSummary, "result_read_attempt");
  const rxUpstream = count(upstreamSummary, "rx_result_frame");
  const rxRust = count(rustSummary, "rx_result_frame");
  const registerReadUpstream = count(upstreamSummary, "register_read_parsed");
  const registerReadRust = count(rustSummary, "register_read_parsed");
  const jobTxRust = count(rustSummary, "job_tx");
  const resultCorrelated =
    count(rustSummary, "result_correlated") > 0 ||
    Boolean(rustText && /asic_production_status=result_correlated/.test(rustText));
  const fakePoolSubmitObserved =
    count(upstreamSummary, "fake_pool_submit") > 0 ||
    count(rustSummary, "fake_pool_submit") > 0;
  const partialFrameRust = count(rustSummary, "partial_frame");
  const asicEnableActiveRust = count(rustSummary, "asic_enable_active");
  const resetPulseRust = count(rustSummary, "reset_pulse");

  let recommended = recommendResultPathInvestigation({
    blocked,
    chipIdUpstream,
    chipIdRust,
    jobTxRust,
    resultCorrelated,
    resultReadAttemptRust: resultReadRust,
    partialFrameRust,
    asicEnableActiveRust,
    resetPulseRust,
  });

  if (!ALLOWED_RECOMMENDATIONS.has(recommended)) {
    recommended = "none";
  }

  const comparisonStatus = blocked
    ? "blocked_safe_prerequisite"
    : resultCorrelated && fakePoolSubmitObserved
      ? "match"
      : "result_path_gap";

  return renderReport({
    comparisonStatus,
    upstreamPath,
    rustPath,
    blockedReasons,
    chipIdUpstream,
    chipIdRust,
    resultReadUpstream,
    resultReadRust,
    rxUpstream,
    rxRust,
    registerReadUpstream,
    registerReadRust,
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
  chipIdUpstream,
  chipIdRust,
  resultReadUpstream,
  resultReadRust,
  rxUpstream,
  rxRust,
  registerReadUpstream,
  registerReadRust,
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
    "# Phase 28.1.1.2 Result-Path Comparator Report",
    "",
    `comparison_status: ${comparisonStatus}`,
    `upstream_source_label: ${label(upstreamPath)}`,
    `rust_source_label: ${label(rustPath)}`,
    "raw_bytes_committed: false",
    "credential_contents_read: false",
    "phase30_promotion_input: pending",
    "",
    "## D-05 Result-Path Metrics",
    "",
    `chip_id_read_tx_upstream: ${chipIdUpstream}`,
    `chip_id_read_tx_rust: ${chipIdRust}`,
    `result_read_attempt_upstream: ${resultReadUpstreamLabel}`,
    `result_read_attempt_rust: ${resultReadRust}`,
    `rx_result_frame_upstream: ${rxUpstream}`,
    `rx_result_frame_rust: ${rxRust}`,
    `register_read_parsed_upstream: ${registerReadUpstream}`,
    `register_read_parsed_rust: ${registerReadRust}`,
    `result_correlated: ${resultCorrelated}`,
    `fake_pool_submit_observed: ${fakePoolSubmitObserved}`,
    `recommended_investigation: ${recommended}`,
    "",
    "## Notes",
    "",
    "Upstream `result_read_attempt=0` is labeled `expected_no_marker` (passive SERIAL_rx; no equivalent log line).",
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
  recommendResultPathInvestigation,
  reportFor,
  summarizeLog,
};
