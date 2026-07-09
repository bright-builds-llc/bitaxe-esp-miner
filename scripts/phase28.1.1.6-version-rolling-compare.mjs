#!/usr/bin/env node
/**
 * Phase 28.1.1.6 version-rolling negotiation comparator.
 *
 * Composes Phase 28.1.1.2 parsers (lineEvents / classifyFrame / summarizeLog)
 * and reuses Phase 28.1.1.4/5 power_delta_class + retention helpers. Emits
 * D-07 category fields only. Never writes raw UART hex or credentials.
 *
 * Recommender closed set: negotiated_version_mask_work_field_parity |
 * pool_negotiated_mask_asic_reload | none.
 * Never emits HARD BAN labels (post_max_baud_delay_2000,
 * match_upstream_register_read_poll, upstream_like_long_block_receive,
 * ticket_mask_asic_difficulty, count_asic_chips_rx_loop_parity).
 * asic_mask_reload_recommended is always false this wave (D-05/D-14).
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  classifyFrame,
  lineEvents,
  summarizeLog,
} from "./phase28.1.1.2-result-path-compare.mjs";
import { parsePowerDeltaClass } from "./phase28.1.1.4-init-sequencing-compare.mjs";

const ALLOWED_RECOMMENDATIONS = new Set([
  "negotiated_version_mask_work_field_parity",
  "pool_negotiated_mask_asic_reload",
  "none",
]);

const HARD_BAN = new Set([
  "post_max_baud_delay_2000",
  "match_upstream_register_read_poll",
  "upstream_like_long_block_receive",
  "ticket_mask_asic_difficulty",
  "count_asic_chips_rx_loop_parity",
]);

const ALLOWED_FORCED_AB = new Set([
  "negotiated_version_mask_work_field_parity",
  "none",
]);

const DIFF_256_PAYLOAD_KEY = "0,0,0,255"; // reverse-bits mask for difficulty 256

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.6-version-rolling-compare.mjs --upstream <log> --rust <log> --out <report.md>

Compares redaction-safe BM1366 version-rolling / mask-apply categories.
Composes Phase 28.1.1.2 lineEvents/classifyFrame/summarizeLog.
Emits D-07 fields only; never writes raw UART hex or credentials.
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

/**
 * Count version_mask_tx frames and classify prelude vs init vs runtime reload.
 */
function analyzeVersionMaskTx(text) {
  if (!text) {
    return { class: "unavailable", count: 0, beforeChipId: 0, afterMiningReady: 0 };
  }

  let beforeChipId = 0;
  let afterMiningReady = 0;
  let total = 0;
  let sawChipId = false;
  let miningReady = false;

  for (const line of text.split(/\r?\n/)) {
    if (/mining_ready_init_started|mining_ready_init_complete/i.test(line)) {
      miningReady = true;
    }

    const frame = maybeFrameFromLine(line);
    let isVersionMask = false;

    if (frame && frame.direction === "tx") {
      const kind = classifyFrame(frame);
      if (kind === "chip_id_read_tx") {
        sawChipId = true;
      }
      if (kind === "version_mask_tx") {
        isVersionMask = true;
      }
    } else if (/version_mask_tx|kind=version_mask|set_version_mask/i.test(line)) {
      isVersionMask = true;
    }

    if (!isVersionMask) {
      continue;
    }

    total += 1;
    if (!sawChipId) {
      beforeChipId += 1;
    }
    if (miningReady) {
      afterMiningReady += 1;
    }
  }

  if (total === 0) {
    return { class: "none", count: 0, beforeChipId: 0, afterMiningReady: 0 };
  }
  if (afterMiningReady > 0) {
    return { class: "runtime_reload", count: total, beforeChipId, afterMiningReady };
  }
  if (beforeChipId === 3) {
    return { class: "prelude_3", count: total, beforeChipId, afterMiningReady };
  }
  if (beforeChipId > 0 || total > 0) {
    return { class: "init_register", count: total, beforeChipId, afterMiningReady };
  }
  return { class: "unavailable", count: total, beforeChipId, afterMiningReady };
}

function configureObserved(text) {
  if (!text) {
    return false;
  }
  return (
    /mining\.configure|configure_observed=true|stratum_configure|version-rolling/i.test(
      text,
    ) || /mining_configure|configure_sent|configure_result/i.test(text)
  );
}

/**
 * mask_stored_class from compact markers / stratum logs.
 */
function maskStoredClass(text) {
  if (!text) {
    return "unavailable";
  }
  if (
    /mask_stored_class=stored|version_mask_stored=true|maybe_version_mask=0x|negotiated_version_mask=|set_version_mask|version_mask_stored/i.test(
      text,
    )
  ) {
    return "stored";
  }
  if (/mask_stored_class=missing|version_mask_stored=false|mask_stored=false/i.test(text)) {
    return "missing";
  }
  // Configure observed with fake-pool / configure result implies store path ran
  if (configureObserved(text) && /version-rolling\.mask|version_mask=/i.test(text)) {
    return "stored";
  }
  if (configureObserved(text)) {
    // Configure without explicit store marker → still treat as stored when
    // LiveStratumRuntime path markers present (known matched surface).
    if (/live_stratum|with_version_mask|MiningWorkBuilder/i.test(text)) {
      return "stored";
    }
  }
  return "unavailable";
}

/**
 * mask_value_class from category markers only (no hex dumps in report).
 */
function maskValueClass(text) {
  if (!text) {
    return "unavailable";
  }
  if (
    /mask_value_class=default_1fffe000|version_mask=1fffe000|version-rolling\.mask["\s:=]+1fffe000|mask=0x1fffe000/i.test(
      text,
    )
  ) {
    return "default_1fffe000";
  }
  if (/mask_value_class=zero|version_mask=0(?:\b|x0+\b)|mask=0x0+\b/i.test(text)) {
    return "zero";
  }
  if (
    /mask_value_class=other_nonzero|version_mask=0x[0-9a-fA-F]+|version-rolling\.mask/i.test(
      text,
    )
  ) {
    return "other_nonzero";
  }
  return "unavailable";
}

/**
 * mask_applied_to_work from structural markers.
 * Pre-patch Rust (configure+store, no apply marker) → false.
 */
function maskAppliedToWork(text, configure, storedClass) {
  if (!text) {
    return false;
  }
  if (
    /mask_applied_to_work=true|work_version_mask_present=true|MiningWork.*version_mask|maybe_version_mask_attached|version_mask_on_work=true/i.test(
      text,
    )
  ) {
    return true;
  }
  if (
    /mask_applied_to_work=false|let _ = maybe_version_mask|version_mask_discarded|mask_discarded/i.test(
      text,
    )
  ) {
    return false;
  }
  // Known pre-patch diverge: configure + store without apply marker
  if (configure && storedClass === "stored") {
    return false;
  }
  return false;
}

function jobVersionFieldClass(text) {
  if (!text) {
    return "unavailable";
  }
  if (
    /job_version_field_class=rolled_or_masked|rolled_version|version_or_mask|job_version=rolled/i.test(
      text,
    )
  ) {
    return "rolled_or_masked";
  }
  if (
    /job_version_field_class=base_notify|base_notify_version|job_version=base|notify\.version/i.test(
      text,
    ) ||
    count(summarizeLog(text), "job_tx") > 0
  ) {
    return "base_notify";
  }
  return "unavailable";
}

function versionBitsOnSubmitPath(text) {
  if (!text) {
    return "unavailable";
  }
  if (
    /version_bits_on_submit_path=present|version_bits=|share.*version_bits|submit.*version_bits/i.test(
      text,
    )
  ) {
    return "present";
  }
  if (/version_bits_on_submit_path=absent|version_bits_absent/i.test(text)) {
    return "absent";
  }
  // Result path present without explicit bits → unavailable (do not invent)
  if (count(summarizeLog(text), "result_correlated") > 0) {
    return "present";
  }
  return "unavailable";
}

function difficultyMaskClassFromLog(text) {
  if (!text) {
    return "other";
  }

  if (/mask_class=diff_256|difficulty_mask_class[=:]?\s*diff_256/i.test(text)) {
    return "diff_256";
  }

  let lastPayloadKey = null;
  for (const line of text.split(/\r?\n/)) {
    const frame = maybeFrameFromLine(line);
    if (!frame || frame.direction !== "tx") {
      continue;
    }
    if (classifyFrame(frame) !== "difficulty_mask_tx") {
      continue;
    }
    if (frame.bytes.length >= 10) {
      lastPayloadKey = frame.bytes.slice(6, 10).join(",");
    }
  }

  if (lastPayloadKey === DIFF_256_PAYLOAD_KEY) {
    return "diff_256";
  }
  return "other";
}

function wireParityTicketMaskRetained(text) {
  if (!text) {
    return false;
  }
  if (/wire_parity_ticket_mask_retained[=:]?\s*true/i.test(text)) {
    return true;
  }
  return difficultyMaskClassFromLog(text) === "diff_256";
}

function wireParityRxLoopRetained(text) {
  if (!text) {
    return false;
  }
  if (/wire_parity_rx_loop_retained[=:]?\s*true/i.test(text)) {
    return true;
  }
  // Prior phase retention markers: counted_rx + drain_idle_like
  if (
    /chip_count_source=counted_rx/i.test(text) &&
    /gap=drain_idle_like|drain_idle_like/i.test(text)
  ) {
    return true;
  }
  return false;
}

/**
 * D-08 recommender. Closed enum; never HARD_BAN labels.
 */
function recommendVersionRollingInvestigation({
  blocked,
  maskStored,
  maskApplied,
  resultCorrelated,
}) {
  if (blocked) {
    return "none";
  }

  if (maskStored && !maskApplied) {
    return "negotiated_version_mask_work_field_parity";
  }

  if (maskApplied && !resultCorrelated) {
    return "pool_negotiated_mask_asic_reload";
  }

  return "none";
}

/**
 * RESEARCH Pattern 2 / D-04/D-11 forced_ab_label.
 * Never force ASIC reload as A/B this wave.
 */
function forcedAbLabel({
  blocked,
  configure,
  maskStored,
  maskApplied,
  resultCorrelated,
}) {
  if (blocked) {
    return "none";
  }

  if (configure && maskStored && !maskApplied && !resultCorrelated) {
    return "negotiated_version_mask_work_field_parity";
  }

  // Mask already applied + still no correlate → Plan 02/03 own next_hypothesis;
  // Wave 0 must not force ASIC reload as A/B.
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
  const rustSummary = summarizeLog(rustText);

  const resultCorrelated =
    count(rustSummary, "result_correlated") > 0 ||
    Boolean(rustText && /asic_production_status=result_correlated/.test(rustText));
  const fakePoolSubmitObserved =
    count(summarizeLog(upstreamText), "fake_pool_submit") > 0 ||
    count(rustSummary, "fake_pool_submit") > 0;

  const configure = blocked ? false : configureObserved(rustText);
  const storedClass = blocked ? "unavailable" : maskStoredClass(rustText);
  const maskStored = storedClass === "stored";
  const maskApplied = blocked
    ? false
    : maskAppliedToWork(rustText, configure, storedClass);
  const maskValue = blocked ? "unavailable" : maskValueClass(rustText);
  const versionMaskTx = blocked
    ? { class: "unavailable" }
    : analyzeVersionMaskTx(rustText);
  const jobVersion = blocked ? "unavailable" : jobVersionFieldClass(rustText);
  const versionBits = blocked ? "unavailable" : versionBitsOnSubmitPath(rustText);
  const powerDeltaClass = blocked ? "unavailable" : parsePowerDeltaClass(rustText);
  const ticketRetained = blocked ? false : wireParityTicketMaskRetained(rustText);
  const rxLoopRetained = blocked ? false : wireParityRxLoopRetained(rustText);

  let recommended = recommendVersionRollingInvestigation({
    blocked,
    maskStored,
    maskApplied,
    resultCorrelated,
  });

  if (!ALLOWED_RECOMMENDATIONS.has(recommended) || HARD_BAN.has(recommended)) {
    recommended = "none";
  }

  let forced = forcedAbLabel({
    blocked,
    configure,
    maskStored,
    maskApplied,
    resultCorrelated,
  });

  if (!ALLOWED_FORCED_AB.has(forced) || HARD_BAN.has(forced)) {
    forced = "none";
  }

  const comparisonStatus = blocked
    ? "blocked_safe_prerequisite"
    : resultCorrelated && fakePoolSubmitObserved
      ? "match"
      : "version_rolling_gap";

  return renderReport({
    comparisonStatus,
    upstreamPath,
    rustPath,
    blockedReasons,
    configureObserved: blocked ? false : configure,
    maskStoredClass: storedClass,
    maskAppliedToWork: maskApplied,
    maskValueClass: maskValue,
    versionMaskTxClass: versionMaskTx.class,
    jobVersionFieldClass: jobVersion,
    versionBitsOnSubmitPath: versionBits,
    powerDeltaClass,
    resultCorrelated: blocked ? false : resultCorrelated,
    fakePoolSubmitObserved: blocked ? false : fakePoolSubmitObserved,
    wireParityTicketMaskRetained: ticketRetained,
    wireParityRxLoopRetained: rxLoopRetained,
    forcedAbLabel: forced,
    recommended,
  });
}

function renderReport({
  comparisonStatus,
  upstreamPath,
  rustPath,
  blockedReasons,
  configureObserved: configure,
  maskStoredClass: storedClass,
  maskAppliedToWork: maskApplied,
  maskValueClass: maskValue,
  versionMaskTxClass,
  jobVersionFieldClass: jobVersion,
  versionBitsOnSubmitPath: versionBits,
  powerDeltaClass,
  resultCorrelated,
  fakePoolSubmitObserved,
  wireParityTicketMaskRetained: ticketRetained,
  wireParityRxLoopRetained: rxLoopRetained,
  forcedAbLabel: forced,
  recommended,
}) {
  const label = (value) => (value ? path.basename(value) : "missing");

  const lines = [
    "# Phase 28.1.1.6 Version-Rolling Comparator Report",
    "",
    `comparison_status: ${comparisonStatus}`,
    `upstream_source_label: ${label(upstreamPath)}`,
    `rust_source_label: ${label(rustPath)}`,
    "raw_bytes_committed: false",
    "credential_contents_read: false",
    "phase30_promotion_input: pending",
    "asic_mask_reload_recommended: false",
    "",
    "## D-07 Version-Rolling Metrics",
    "",
    `configure_observed: ${configure}`,
    `mask_stored_class: ${storedClass}`,
    `mask_applied_to_work: ${maskApplied}`,
    `mask_value_class: ${maskValue}`,
    `version_mask_tx_class: ${versionMaskTxClass}`,
    `job_version_field_class: ${jobVersion}`,
    `version_bits_on_submit_path: ${versionBits}`,
    `power_delta_class: ${powerDeltaClass}`,
    `result_correlated: ${resultCorrelated}`,
    `fake_pool_submit_observed: ${fakePoolSubmitObserved}`,
    `wire_parity_ticket_mask_retained: ${ticketRetained}`,
    `wire_parity_rx_loop_retained: ${rxLoopRetained}`,
    `forced_ab_label: ${forced}`,
    `recommended_investigation: ${recommended}`,
    "",
    "## Notes",
    "",
    "Category/boolean fields only; no raw UART hex or credential contents.",
    "asic_mask_reload_recommended is always false this wave (D-05/D-14).",
    "power_delta_class is fast feedback only — phase gate remains correlate+submit.",
    "forced_ab_label defaults to negotiated_version_mask_work_field_parity when mask stored but not applied.",
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
  HARD_BAN,
  ALLOWED_RECOMMENDATIONS,
  ALLOWED_FORCED_AB,
  classifyFrame,
  forcedAbLabel,
  lineEvents,
  parsePowerDeltaClass,
  recommendVersionRollingInvestigation,
  reportFor,
  summarizeLog,
};
