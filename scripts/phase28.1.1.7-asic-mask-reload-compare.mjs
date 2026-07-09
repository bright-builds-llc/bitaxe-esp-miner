#!/usr/bin/env node
/**
 * Phase 28.1.1.7 ASIC mask-reload comparator.
 *
 * Composes/extends Phase 28.1.1.6 version-rolling taxonomy and Phase
 * 28.1.1.2 parsers (lineEvents / classifyFrame / summarizeLog). Emits
 * D-08 category fields only. Never writes raw UART hex or credentials.
 *
 * Recommender closed set: pool_negotiated_mask_asic_reload |
 * remaining_nonce_production_blocker_narrowing | none.
 * Never emits HARD BAN labels (post_max_baud_delay_2000,
 * match_upstream_register_read_poll, upstream_like_long_block_receive,
 * ticket_mask_asic_difficulty, count_asic_chips_rx_loop_parity,
 * negotiated_version_mask_work_field_parity).
 *
 * mask_reload_tx_observed is true only for post_configure_runtime after
 * configure — never from prelude_3 / init_register alone (D-05/D-11).
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
// Compose/extend phase28.1.1.6-version-rolling-compare.mjs taxonomy (D-10):
// same configure/mask_stored/mask_applied/power_delta/correlate/submit fields,
// plus mask_reload_tx_observed + init vs post_configure_runtime class rules.

const ALLOWED_RECOMMENDATIONS = new Set([
  "pool_negotiated_mask_asic_reload",
  "remaining_nonce_production_blocker_narrowing",
  "none",
]);

const HARD_BAN = new Set([
  "post_max_baud_delay_2000",
  "match_upstream_register_read_poll",
  "upstream_like_long_block_receive",
  "ticket_mask_asic_difficulty",
  "count_asic_chips_rx_loop_parity",
  "negotiated_version_mask_work_field_parity",
]);

const ALLOWED_FORCED_AB = new Set([
  "pool_negotiated_mask_asic_reload",
  "none",
]);

const DIFF_256_PAYLOAD_KEY = "0,0,0,255"; // reverse-bits mask for difficulty 256

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.7-asic-mask-reload-compare.mjs --upstream <log> --rust <log> --out <report.md>

Compares redaction-safe BM1366 pool-negotiated ASIC mask-reload categories.
Composes Phase 28.1.1.6 / 28.1.1.2 parsers (lineEvents/classifyFrame/summarizeLog).
Emits D-08 fields only; never writes raw UART hex or credentials.
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
  if (configureObserved(text) && /version-rolling\.mask|version_mask=/i.test(text)) {
    return "stored";
  }
  if (configureObserved(text)) {
    if (/live_stratum|with_version_mask|MiningWorkBuilder/i.test(text)) {
      return "stored";
    }
  }
  return "unavailable";
}

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

function maskAppliedToWork(text) {
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
  if (
    /chip_count_source=counted_rx/i.test(text) &&
    /gap=drain_idle_like|drain_idle_like/i.test(text)
  ) {
    return true;
  }
  return false;
}

function wireParityMaskOnWorkRetained(text, maskApplied) {
  if (!text) {
    return false;
  }
  if (/wire_parity_mask_on_work_retained[=:]?\s*true/i.test(text)) {
    return true;
  }
  return maskApplied === true;
}

/**
 * Locate first configure / mask_stored evidence line index (0-based).
 * Returns -1 when neither is present.
 */
function configureEvidenceLineIndex(lines) {
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (
      /mining\.configure|configure_observed=true|stratum_configure|configure_sent|configure_result|mask_stored_class=stored|version_mask_stored=true/i.test(
        line,
      )
    ) {
      return index;
    }
  }
  return -1;
}

/**
 * Analyze version_mask TX class and mask_reload_tx_observed.
 *
 * mask_reload_tx_observed is true ONLY when an explicit post_configure_runtime
 * marker (or equivalent) appears AFTER configure/mask_stored evidence.
 * Prelude / init_register frames alone never count as reload (Pitfall 2).
 */
function analyzeMaskReload(text) {
  if (!text) {
    return {
      versionMaskTxClass: "unavailable",
      maskReloadTxObserved: false,
    };
  }

  const lines = text.split(/\r?\n/);
  const configureIndex = configureEvidenceLineIndex(lines);

  let beforeChipId = 0;
  let totalVersionMask = 0;
  let sawChipId = false;
  let sawInitRegisterMarker = false;
  let postConfigureRuntimeAfterConfigure = false;
  let anyPostConfigureMarker = false;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];

    if (
      /version_mask_tx_class=post_configure_runtime|mask_reload_tx_observed=true/i.test(
        line,
      )
    ) {
      anyPostConfigureMarker = true;
      if (configureIndex >= 0 && index > configureIndex) {
        postConfigureRuntimeAfterConfigure = true;
      }
    }

    if (
      /version_mask_tx_class=init_register|stage=mining_ready/i.test(line)
    ) {
      sawInitRegisterMarker = true;
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
    } else if (
      /version_mask_tx|kind=version_mask|set_version_mask/i.test(line) &&
      !/version_mask_tx_class=post_configure_runtime|mask_reload_tx_observed=true/i.test(
        line,
      )
    ) {
      isVersionMask = true;
    }

    if (!isVersionMask) {
      continue;
    }

    totalVersionMask += 1;
    if (!sawChipId) {
      beforeChipId += 1;
    }
  }

  const maskReloadTxObserved = postConfigureRuntimeAfterConfigure;

  let versionMaskTxClass;
  if (maskReloadTxObserved || (anyPostConfigureMarker && configureIndex >= 0)) {
    // Prefer post_configure_runtime only when after configure evidence.
    versionMaskTxClass = maskReloadTxObserved
      ? "post_configure_runtime"
      : totalVersionMask === 0
        ? "none"
        : beforeChipId === 3
          ? "prelude_3"
          : "init_register";
  } else if (totalVersionMask === 0 && !sawInitRegisterMarker) {
    versionMaskTxClass = "none";
  } else if (beforeChipId === 3 && !sawInitRegisterMarker) {
    versionMaskTxClass = "prelude_3";
  } else if (totalVersionMask > 0 || sawInitRegisterMarker) {
    // Init-only: prelude and/or init register — never post_configure_runtime.
    versionMaskTxClass =
      beforeChipId === 3 && !sawInitRegisterMarker ? "prelude_3" : "init_register";
    // When both prelude and later init exist, prefer init_register if mining-ready
    // markers or extra frames after chip_id are present.
    if (sawInitRegisterMarker || (sawChipId && totalVersionMask > beforeChipId)) {
      versionMaskTxClass = "init_register";
    } else if (beforeChipId === 3) {
      versionMaskTxClass = "prelude_3";
    } else {
      versionMaskTxClass = "init_register";
    }
  } else {
    versionMaskTxClass = "unavailable";
  }

  // Explicit post_configure wins over init classification.
  if (maskReloadTxObserved) {
    versionMaskTxClass = "post_configure_runtime";
  }

  return {
    versionMaskTxClass,
    maskReloadTxObserved,
  };
}

/**
 * D-09 recommender. Closed enum; never HARD_BAN labels.
 */
function recommendMaskReloadInvestigation({
  blocked,
  maskApplied,
  maskReloadTxObserved,
  resultCorrelated,
}) {
  if (blocked) {
    return "none";
  }

  if (maskApplied && !maskReloadTxObserved && !resultCorrelated) {
    return "pool_negotiated_mask_asic_reload";
  }

  if (maskReloadTxObserved && !resultCorrelated) {
    return "remaining_nonce_production_blocker_narrowing";
  }

  return "none";
}

/**
 * RESEARCH Pattern 2 / D-04/D-12 forced_ab_label.
 * Default Ultra 205 pre-patch: pool_negotiated_mask_asic_reload.
 */
function forcedAbLabel({
  blocked,
  configure,
  maskStored,
  maskApplied,
  maskReloadTxObserved,
  resultCorrelated,
}) {
  if (blocked) {
    return "none";
  }

  if (
    configure &&
    maskStored &&
    maskApplied &&
    !maskReloadTxObserved &&
    !resultCorrelated
  ) {
    return "pool_negotiated_mask_asic_reload";
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
  const maskApplied = blocked ? false : maskAppliedToWork(rustText);
  const maskValue = blocked ? "unavailable" : maskValueClass(rustText);
  const reloadAnalysis = blocked
    ? { versionMaskTxClass: "unavailable", maskReloadTxObserved: false }
    : analyzeMaskReload(rustText);
  const jobVersion = blocked ? "unavailable" : jobVersionFieldClass(rustText);
  const powerDeltaClass = blocked ? "unavailable" : parsePowerDeltaClass(rustText);
  const ticketRetained = blocked ? false : wireParityTicketMaskRetained(rustText);
  const rxLoopRetained = blocked ? false : wireParityRxLoopRetained(rustText);
  const maskOnWorkRetained = blocked
    ? false
    : wireParityMaskOnWorkRetained(rustText, maskApplied);

  let recommended = recommendMaskReloadInvestigation({
    blocked,
    maskApplied,
    maskReloadTxObserved: reloadAnalysis.maskReloadTxObserved,
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
    maskReloadTxObserved: reloadAnalysis.maskReloadTxObserved,
    resultCorrelated,
  });

  if (!ALLOWED_FORCED_AB.has(forced) || HARD_BAN.has(forced)) {
    forced = "none";
  }

  const comparisonStatus = blocked
    ? "blocked_safe_prerequisite"
    : resultCorrelated && fakePoolSubmitObserved
      ? "match"
      : "mask_reload_gap";

  return renderReport({
    comparisonStatus,
    upstreamPath,
    rustPath,
    blockedReasons,
    configureObserved: blocked ? false : configure,
    maskStoredClass: storedClass,
    maskAppliedToWork: maskApplied,
    maskReloadTxObserved: blocked ? false : reloadAnalysis.maskReloadTxObserved,
    maskValueClass: maskValue,
    versionMaskTxClass: reloadAnalysis.versionMaskTxClass,
    jobVersionFieldClass: jobVersion,
    powerDeltaClass,
    resultCorrelated: blocked ? false : resultCorrelated,
    fakePoolSubmitObserved: blocked ? false : fakePoolSubmitObserved,
    wireParityMaskOnWorkRetained: maskOnWorkRetained,
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
  maskReloadTxObserved,
  maskValueClass: maskValue,
  versionMaskTxClass,
  jobVersionFieldClass: jobVersion,
  powerDeltaClass,
  resultCorrelated,
  fakePoolSubmitObserved,
  wireParityMaskOnWorkRetained: maskOnWorkRetained,
  wireParityTicketMaskRetained: ticketRetained,
  wireParityRxLoopRetained: rxLoopRetained,
  forcedAbLabel: forced,
  recommended,
}) {
  const label = (value) => (value ? path.basename(value) : "missing");

  const lines = [
    "# Phase 28.1.1.7 ASIC Mask-Reload Comparator Report",
    "",
    `comparison_status: ${comparisonStatus}`,
    `upstream_source_label: ${label(upstreamPath)}`,
    `rust_source_label: ${label(rustPath)}`,
    "raw_bytes_committed: false",
    "credential_contents_read: false",
    "phase30_promotion_input: pending",
    "",
    "## D-08 ASIC Mask-Reload Metrics",
    "",
    `configure_observed: ${configure}`,
    `mask_stored_class: ${storedClass}`,
    `mask_applied_to_work: ${maskApplied}`,
    `mask_reload_tx_observed: ${maskReloadTxObserved}`,
    `mask_value_class: ${maskValue}`,
    `version_mask_tx_class: ${versionMaskTxClass}`,
    `job_version_field_class: ${jobVersion}`,
    `power_delta_class: ${powerDeltaClass}`,
    `result_correlated: ${resultCorrelated}`,
    `fake_pool_submit_observed: ${fakePoolSubmitObserved}`,
    `wire_parity_mask_on_work_retained: ${maskOnWorkRetained}`,
    `wire_parity_ticket_mask_retained: ${ticketRetained}`,
    `wire_parity_rx_loop_retained: ${rxLoopRetained}`,
    `forced_ab_label: ${forced}`,
    `recommended_investigation: ${recommended}`,
    "",
    "## Notes",
    "",
    "Category/boolean fields only; no raw UART hex or credential contents.",
    "mask_reload_tx_observed requires post_configure_runtime after configure — init frames do not count.",
    "power_delta_class is fast feedback only — phase gate remains correlate+submit.",
    "forced_ab_label defaults to pool_negotiated_mask_asic_reload when mask applied but reload TX absent.",
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
  analyzeMaskReload,
  classifyFrame,
  forcedAbLabel,
  lineEvents,
  parsePowerDeltaClass,
  recommendMaskReloadInvestigation,
  reportFor,
  summarizeLog,
};
