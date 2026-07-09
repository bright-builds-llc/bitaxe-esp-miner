#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const JOB_FRAME_LEN = 88;
const JOB_HEADER = 0x21;
const JOB_LENGTH_FIELD = 0x56;

const REQUIRED_EVENTS = [
  "reset_pulse",
  "version_mask_tx",
  "chip_id_read_tx",
  "rx_result_frame",
  "chain_inactive_tx",
  "set_chip_address_tx",
  "frequency_tx",
  "nonce_space_tx",
  "max_baud_tx",
  "clear_rx",
  "job_tx",
  "result_read_attempt",
];

const ORDER_EVENTS = [
  "reset_pulse",
  "use_default_baud",
  "version_mask_tx",
  "chip_id_read_tx",
  "rx_result_frame",
  "chain_inactive_tx",
  "set_chip_address_tx",
  "frequency_tx",
  "nonce_space_tx",
  "max_baud_tx",
  "clear_rx",
  "job_tx",
  "result_read_attempt",
  "rx_idle",
  "result_correlated",
  "fake_pool_submit",
];

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.1-below-job-byte-sequence-compare.mjs --upstream <log> --rust <log> --out <report.md>

Compares redaction-safe BM1366 below-job-byte semantic events from upstream and Rust logs.
The report records event names, counts, and verdicts only; it never writes raw frames.
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

  const maybeRustRx = line.match(/asic_uart_trace=rx_complete\s+read_count=\d+\s+hex=([0-9a-fA-F ]+)/);
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
  if (/share_submission_status=(accepted|rejected)|accepted_submit_count=[1-9]|mining\.submit/i.test(line)) {
    events.push("fake_pool_submit");
  }

  const maybeFrame = maybeFrameFromLine(line);
  if (maybeFrame) {
    events.push(classifyFrame(maybeFrame));
  }

  return events;
}

function summarizeLog(text) {
  const counts = new Map();
  const orderedEvents = [];

  if (!text) {
    return { counts, orderedEvents };
  }

  for (const line of text.split(/\r?\n/)) {
    for (const event of lineEvents(line)) {
      counts.set(event, (counts.get(event) ?? 0) + 1);
      if (ORDER_EVENTS.includes(event)) {
        orderedEvents.push(event);
      }
    }
  }

  return { counts, orderedEvents };
}

function count(summary, event) {
  return summary.counts.get(event) ?? 0;
}

function statusForEvent(upstreamSummary, rustSummary, event) {
  const upstreamCount = count(upstreamSummary, event);
  const rustCount = count(rustSummary, event);
  if (upstreamCount === 0 && rustCount === 0) {
    return "not_observed";
  }
  if (rustCount === 0) {
    return "missing_in_rust";
  }
  return "observed";
}

function requiredRows(upstreamSummary, rustSummary) {
  return REQUIRED_EVENTS.map((event) => ({
    event,
    upstreamCount: count(upstreamSummary, event),
    rustCount: count(rustSummary, event),
    verdict: statusForEvent(upstreamSummary, rustSummary, event),
  }));
}

function firstOrderedProjection(summary) {
  const seen = new Set();
  const projection = [];
  for (const event of summary.orderedEvents) {
    if (seen.has(event)) {
      continue;
    }
    seen.add(event);
    projection.push(event);
  }
  return projection;
}

function sequenceHasRequiredOrder(summary) {
  const projection = firstOrderedProjection(summary);
  let cursor = -1;
  for (const event of ["chip_id_read_tx", "frequency_tx", "nonce_space_tx", "max_baud_tx", "job_tx"]) {
    const index = projection.indexOf(event);
    if (index === -1 || index < cursor) {
      return false;
    }
    cursor = index;
  }
  return true;
}

function recommendInvestigation(rows, rustSummary) {
  const missingEvents = new Set(rows.filter((row) => row.verdict === "missing_in_rust").map((row) => row.event));

  if (missingEvents.has("max_baud_tx") || missingEvents.has("clear_rx")) {
    return "post_max_baud_delay_2000";
  }
  if (count(rustSummary, "clear_rx") === 0 && count(rustSummary, "job_tx") > 0) {
    return "clear_rx_before_production_work";
  }
  if (count(rustSummary, "result_read_attempt") === 0 && count(rustSummary, "job_tx") > 0) {
    return "single_dispatch_bounded_read";
  }
  if (
    count(rustSummary, "job_tx") > 0 &&
    count(rustSummary, "result_read_attempt") > 0 &&
    count(rustSummary, "result_correlated") === 0
  ) {
    return "post_max_baud_delay_2000";
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

  const upstreamSummary = summarizeLog(upstreamText);
  const rustSummary = summarizeLog(rustText);
  const rows = requiredRows(upstreamSummary, rustSummary);
  const missingRows = rows.filter((row) => row.verdict === "missing_in_rust");
  const rustOrderOk = sequenceHasRequiredOrder(rustSummary);
  const upstreamOrderOk = sequenceHasRequiredOrder(upstreamSummary);

  if (count(upstreamSummary, "job_tx") === 0) {
    blockedReasons.push("upstream_job_tx_missing");
  }
  if (count(rustSummary, "job_tx") === 0) {
    blockedReasons.push("rust_job_tx_missing");
  }

  const recommendedInvestigation =
    blockedReasons.length > 0 ? "none" : recommendInvestigation(rows, rustSummary);
  const comparisonStatus =
    blockedReasons.length > 0
      ? "blocked_safe_prerequisite"
      : missingRows.length > 0 || !rustOrderOk
        ? "mismatch"
        : count(rustSummary, "result_correlated") > 0
          ? "match"
          : "match_with_downstream_gap";

  return renderReport({
    comparisonStatus,
    upstreamPath,
    rustPath,
    blockedReasons,
    upstreamSummary,
    rustSummary,
    rows,
    rustOrderOk,
    upstreamOrderOk,
    recommendedInvestigation,
  });
}

function renderReport({
  comparisonStatus,
  upstreamPath,
  rustPath,
  blockedReasons,
  upstreamSummary,
  rustSummary,
  rows,
  rustOrderOk,
  upstreamOrderOk,
  recommendedInvestigation,
}) {
  const label = (value) => (value ? path.basename(value) : "missing");
  const missingEvents = rows
    .filter((row) => row.verdict === "missing_in_rust")
    .map((row) => row.event);
  const matchedCount = rows.length - missingEvents.length;

  const lines = [
    "# Phase 28.1.1.1 Below-Job-Byte Sequence Comparator Report",
    "",
    `comparison_status: ${comparisonStatus}`,
    `upstream_source_label: ${label(upstreamPath)}`,
    `rust_source_label: ${label(rustPath)}`,
    "raw_bytes_committed: false",
    "credential_contents_read: false",
    `upstream_required_order: ${upstreamOrderOk ? "observed" : "not_observed"}`,
    `rust_required_order: ${rustOrderOk ? "observed" : "not_observed"}`,
    `matched_required_event_count: ${matchedCount}/${rows.length}`,
    `mismatched_events: ${missingEvents.length === 0 ? "none" : missingEvents.join(",")}`,
    `recommended_investigation: ${recommendedInvestigation}`,
    `rust_result_correlated_observed: ${count(rustSummary, "result_correlated") > 0}`,
    `rust_fake_pool_submit_observed: ${count(rustSummary, "fake_pool_submit") > 0}`,
    "",
    "## Required Events",
    "",
    "| Event | Upstream Count | Rust Count | Verdict |",
    "| --- | --- | --- | --- |",
  ];

  for (const row of rows) {
    lines.push(`| ${row.event} | ${row.upstreamCount} | ${row.rustCount} | ${row.verdict} |`);
  }

  lines.push("", "## Verdict", "");
  if (blockedReasons.length > 0) {
    lines.push(`blocked_reasons: ${blockedReasons.join(",")}`, "");
    lines.push("The comparator could not produce a below-job-byte sequence verdict.");
  } else if (comparisonStatus === "match") {
    lines.push("The required below-job-byte semantic events and Rust result correlation are present.");
  } else if (comparisonStatus === "match_with_downstream_gap") {
    lines.push("The required below-job-byte semantic events are present, but Rust still lacks result-correlation evidence.");
  } else {
    lines.push("One or more below-job-byte semantic events expected from upstream are missing in Rust.");
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

main();
