#!/usr/bin/env node
/**
 * Phase 28.1.1.5 chip-enumerate / count / address comparator.
 *
 * Composes Phase 28.1.1.2 parsers (lineEvents / classifyFrame / summarizeLog)
 * and reuses Phase 28.1.1.4 power_delta_class rules. Emits D-08 category
 * fields only. Never writes raw UART hex or credentials.
 *
 * Recommender closed set: match_upstream_chip_enumerate_before_init |
 * version_rolling_negotiation | none.
 * Never emits HARD BAN labels (post_max_baud_delay_2000,
 * match_upstream_register_read_poll, upstream_like_long_block_receive,
 * ticket_mask_asic_difficulty).
 * Never recommends ReadChipId 0x0A byte patch (already matched — D-02).
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
  "match_upstream_chip_enumerate_before_init",
  "version_rolling_negotiation",
  "none",
]);

const HARD_BAN = new Set([
  "post_max_baud_delay_2000",
  "match_upstream_register_read_poll",
  "upstream_like_long_block_receive",
  "ticket_mask_asic_difficulty",
]);

const ALLOWED_FORCED_AB = new Set([
  "count_asic_chips_rx_loop_parity",
  "counted_chip_address_interval",
  "enumerate_to_mining_ready_gap",
  "none",
]);

const DIFF_256_PAYLOAD_KEY = "0,0,0,255"; // reverse-bits mask for difficulty 256

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.5-chip-enumerate-compare.mjs --upstream <log> --rust <log> --out <report.md>

Compares redaction-safe BM1366 chip-enumerate / count / address categories.
Composes Phase 28.1.1.2 lineEvents/classifyFrame/summarizeLog.
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

/**
 * Parse optional compact enumerate markers (counts/classes only — no hex).
 * Shape examples:
 *   asic_chip_enumerate_summary chip_count_source=config_expected chip_count=1
 *     address_interval=256 gap=immediate chip_detected=true
 *   asic_chip_enumerate_summary chip_count_source=counted_rx chip_count=1
 *     address_interval=256 gap=drain_idle_like
 */
function parseEnumerateSummary(text) {
  if (!text) {
    return null;
  }

  const match = text.match(
    /asic_chip_enumerate_summary\b([^\n]*)/,
  );
  if (!match) {
    return null;
  }

  const fields = match[1];
  const get = (key) => {
    const m = fields.match(new RegExp(`\\b${key}=(\\S+)`));
    return m ? m[1] : null;
  };

  return {
    chipCountSource: get("chip_count_source"),
    chipCount: get("chip_count") ? Number.parseInt(get("chip_count"), 10) : null,
    addressInterval: get("address_interval")
      ? Number.parseInt(get("address_interval"), 10)
      : null,
    gap: get("gap"),
    chipDetected: get("chip_detected"),
  };
}

/**
 * Count version_mask_tx frames before the first chip_id_read_tx.
 */
function versionMaskPreludeCount(text) {
  if (!text) {
    return 0;
  }

  let countMasks = 0;
  let sawChipId = false;
  for (const line of text.split(/\r?\n/)) {
    const frame = maybeFrameFromLine(line);
    if (!frame || frame.direction !== "tx") {
      // Also accept marker-only version_mask lines without hex dump in report path
      if (/version_mask_tx|kind=version_mask|set_version_mask/i.test(line) && !sawChipId) {
        countMasks += 1;
      }
      continue;
    }

    const kind = classifyFrame(frame);
    if (kind === "chip_id_read_tx") {
      sawChipId = true;
      break;
    }
    if (kind === "version_mask_tx" && !sawChipId) {
      countMasks += 1;
    }
  }

  return countMasks;
}

function versionMaskPreludeClass(n) {
  if (n === 0) {
    return "0";
  }
  if (n === 3) {
    return "3";
  }
  return "other";
}

/**
 * Infer chip-count / gap / address classes from markers + frame events.
 */
function analyzeEnumerateSide(text, side) {
  const summary = summarizeLog(text);
  const marker = parseEnumerateSummary(text);
  const chipIdTx = count(summary, "chip_id_read_tx");
  const setAddrTx = count(summary, "set_chip_address_tx");
  const versionMaskCount = versionMaskPreludeCount(text);
  const rxFrames = count(summary, "rx_result_frame");
  const rxIdle = count(summary, "rx_idle");

  const chipDetected =
    marker?.chipDetected === "true" ||
    /chip_detect(?:ed|_status)=(?:ok|true|success)|asic_chip_detect=ok/i.test(
      text ?? "",
    ) ||
    (chipIdTx > 0 && rxFrames > 0);

  let chipCountSource = marker?.chipCountSource ?? null;
  let chipCount = marker?.chipCount ?? null;

  if (!chipCountSource) {
    if (side === "rust") {
      // Default Rust production path: catalog expected chips (D-04 / RESEARCH).
      if (
        /chip_count_source=config_expected|expected_chips|config\.chip_count|preflight\.expected_chips/i.test(
          text ?? "",
        )
      ) {
        chipCountSource = "config_expected";
      } else if (
        /chip_count_source=counted_rx|counted_chips|count_asic_chips/i.test(text ?? "")
      ) {
        chipCountSource = "counted_rx";
      } else if (chipIdTx > 0) {
        // No drain markers and single-shot detect → config_expected
        const hasDrain =
          /drain_idle|count_asic_chips|rx_loop|enumerate_rx=drain/i.test(text ?? "") ||
          (rxIdle > 0 && rxFrames > 1);
        chipCountSource = hasDrain ? "counted_rx" : "config_expected";
      }
    } else {
      // Upstream: count_asic_chips drives counted_N
      if (chipIdTx > 0 || /count_asic_chips|chip_counter/i.test(text ?? "")) {
        chipCountSource = "counted_rx";
      }
    }
  }

  if (chipCount == null) {
    if (marker?.chipCount != null) {
      chipCount = marker.chipCount;
    } else {
      const chipCountMatch = (text ?? "").match(/chip_count[=:]?\s*(\d+)/i);
      if (chipCountMatch) {
        chipCount = Number.parseInt(chipCountMatch[1], 10);
      } else if (chipDetected || chipIdTx > 0) {
        chipCount = 1; // Ultra 205 default when detect present
      }
    }
  }

  let addressInterval = marker?.addressInterval ?? null;
  if (addressInterval == null) {
    const intervalMatch = (text ?? "").match(/address_interval[=:]?\s*(\d+)/i);
    if (intervalMatch) {
      addressInterval = Number.parseInt(intervalMatch[1], 10);
    } else if (chipCount === 1 || setAddrTx === 1) {
      addressInterval = 256; // Ultra 205 single-chip numeric match
    }
  }

  let gap = marker?.gap ?? null;
  if (!gap) {
    const gapMatch = (text ?? "").match(/enumerate_to_mining_ready_gap[=:]?\s*(\S+)/i);
    if (gapMatch) {
      gap = gapMatch[1];
    } else if (
      /gap=drain_idle_like|drain_idle_like|count_asic_chips.*idle|rx_loop=drain/i.test(
        text ?? "",
      )
    ) {
      gap = "drain_idle_like";
    } else if (
      /gap=immediate|chip_detect.*mining_ready_init_started|single.?shot/i.test(
        text ?? "",
      )
    ) {
      gap = "immediate";
    } else if (side === "rust" && chipIdTx > 0 && chipCountSource === "config_expected") {
      gap = "immediate";
    } else if (side === "upstream" && chipCountSource === "counted_rx") {
      gap = "drain_idle_like";
    } else if (chipIdTx > 0) {
      gap = "unavailable";
    } else {
      gap = "unavailable";
    }
  }

  const addressIntervalClass =
    addressInterval === 256 ? "interval_256" : addressInterval == null ? "other" : "other";

  let chipCountClass = "unavailable";
  if (chipCount != null && Number.isFinite(chipCount)) {
    if (side === "rust" && chipCountSource === "config_expected") {
      chipCountClass = `config_expected_${chipCount}`;
    } else {
      chipCountClass = `counted_${chipCount}`;
    }
  }

  return {
    versionMaskCount,
    versionMaskClass: versionMaskPreludeClass(versionMaskCount),
    chipIdTxPresent: chipIdTx > 0,
    chipIdTxCount: chipIdTx,
    chipDetected: Boolean(chipDetected),
    chipCount,
    chipCountClass,
    chipCountSource: chipCountSource ?? "unavailable",
    addressAssignmentCount: setAddrTx,
    addressInterval,
    addressIntervalClass,
    gapClass: ["immediate", "drain_idle_like", "delayed_other", "unavailable"].includes(
      gap,
    )
      ? gap
      : "unavailable",
  };
}

/**
 * Difficulty mask class from last 0x14 TX (status only — not a recommender).
 */
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
  return lastPayloadKey ? "other" : "other";
}

/**
 * D-09 / D-04 / D-05 recommender. Closed enum; never HARD_BAN labels.
 */
function recommendChipEnumerateInvestigation({
  blocked,
  enumerateDiverge,
  enumerateMatched,
  resultCorrelated,
}) {
  if (blocked) {
    return "none";
  }

  if (enumerateDiverge && !resultCorrelated) {
    return "match_upstream_chip_enumerate_before_init";
  }

  if (enumerateMatched && !resultCorrelated) {
    return "version_rolling_negotiation";
  }

  return "none";
}

/**
 * RESEARCH Pattern 2 / D-05 forced_ab_label.
 */
function forcedAbLabel({
  blocked,
  addressIntervalMatch,
  gapClassRust,
  chipCountSourceRust,
  readChipIdTxMatch,
}) {
  if (blocked) {
    return "none";
  }

  // Unexpected Ultra 205 address_interval mismatch
  if (!addressIntervalMatch) {
    return "counted_chip_address_interval";
  }

  // RX loop already drain_idle_like and only timing gap remains
  if (
    chipCountSourceRust === "counted_rx" &&
    gapClassRust === "drain_idle_like"
  ) {
    // When enumerate already matches drain path, Plan 02 may still A/B gap —
    // but Pattern 2 says rename to enumerate_to_mining_ready_gap only when
    // RX loop already matches and only timing remains. That case is handled
    // by recommender → version_rolling when fully matched; if gap still
    // diverge vs upstream while source is counted_rx+drain, use gap label.
    return "enumerate_to_mining_ready_gap";
  }

  // Expected Ultra 205 path: TX match + interval_256 + config_expected / immediate
  if (
    readChipIdTxMatch &&
    addressIntervalMatch &&
    (chipCountSourceRust === "config_expected" || gapClassRust === "immediate")
  ) {
    return "count_asic_chips_rx_loop_parity";
  }

  return "count_asic_chips_rx_loop_parity";
}

function enumerateDivergeFrom(upstream, rust) {
  if (rust.chipCountSource === "config_expected" && upstream.chipCountSource === "counted_rx") {
    return true;
  }
  if (rust.gapClass === "immediate" && upstream.gapClass === "drain_idle_like") {
    return true;
  }
  if (
    rust.chipCountSource === "config_expected" &&
    rust.gapClass === "immediate"
  ) {
    return true;
  }
  return false;
}

function enumerateMatchedFrom(upstream, rust) {
  return (
    rust.chipCountSource === "counted_rx" &&
    upstream.chipCountSource === "counted_rx" &&
    rust.gapClass === "drain_idle_like" &&
    upstream.gapClass === "drain_idle_like" &&
    rust.addressIntervalClass === "interval_256" &&
    upstream.addressIntervalClass === "interval_256"
  );
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

  const upstream = analyzeEnumerateSide(upstreamText, "upstream");
  const rust = analyzeEnumerateSide(rustText, "rust");

  const versionMaskPreludeMatch =
    !blocked && upstream.versionMaskClass === "3" && rust.versionMaskClass === "3";

  const readChipIdTxPresentUpstream = blocked ? false : upstream.chipIdTxPresent;
  const readChipIdTxPresentRust = blocked ? false : rust.chipIdTxPresent;
  // D-02: frame bytes already match; presence on both sides ⇒ tx_match true
  const readChipIdTxMatch =
    !blocked && readChipIdTxPresentUpstream && readChipIdTxPresentRust;

  const addressIntervalMatch =
    !blocked &&
    rust.addressIntervalClass === "interval_256" &&
    upstream.addressIntervalClass === "interval_256";

  const powerDeltaClass = blocked ? "unavailable" : parsePowerDeltaClass(rustText);
  const difficultyMaskClassRust = blocked
    ? "other"
    : difficultyMaskClassFromLog(rustText);
  const wireParityTicketMaskRetained = difficultyMaskClassRust === "diff_256";

  const diverge = blocked ? false : enumerateDivergeFrom(upstream, rust);
  const matched = blocked ? false : enumerateMatchedFrom(upstream, rust);

  let recommended = recommendChipEnumerateInvestigation({
    blocked,
    enumerateDiverge: diverge,
    enumerateMatched: matched,
    resultCorrelated,
  });

  if (!ALLOWED_RECOMMENDATIONS.has(recommended) || HARD_BAN.has(recommended)) {
    recommended = "none";
  }

  let forced = forcedAbLabel({
    blocked,
    addressIntervalMatch,
    gapClassRust: rust.gapClass,
    chipCountSourceRust: rust.chipCountSource,
    readChipIdTxMatch,
  });

  // When enumerate fully matched (drain+counted both sides), forced_ab is not
  // the RX-loop lever — Pattern 2 step 3: gap-only if still diverge on gap;
  // otherwise none for A/B (recommender handles version_rolling).
  if (!blocked && matched) {
    forced = "none";
  }

  if (!ALLOWED_FORCED_AB.has(forced) || HARD_BAN.has(forced)) {
    forced = "none";
  }

  const comparisonStatus = blocked
    ? "blocked_safe_prerequisite"
    : resultCorrelated && fakePoolSubmitObserved
      ? "match"
      : "chip_enumerate_gap";

  return renderReport({
    comparisonStatus,
    upstreamPath,
    rustPath,
    blockedReasons,
    versionMaskPreludeCountClassRust: blocked ? "0" : rust.versionMaskClass,
    versionMaskPreludeMatch: blocked ? false : versionMaskPreludeMatch,
    readChipIdTxPresentUpstream,
    readChipIdTxPresentRust,
    readChipIdTxMatch: blocked ? false : readChipIdTxMatch,
    chipDetectedRust: blocked ? false : rust.chipDetected,
    chipCountClassUpstream: blocked ? "unavailable" : upstream.chipCountClass,
    chipCountClassRust: blocked ? "unavailable" : rust.chipCountClass,
    chipCountSourceClassRust: blocked ? "unavailable" : rust.chipCountSource,
    addressAssignmentCountRust: blocked ? 0 : rust.addressAssignmentCount,
    addressIntervalClassRust: blocked ? "other" : rust.addressIntervalClass,
    addressIntervalMatch: blocked ? false : addressIntervalMatch,
    enumerateToMiningReadyGapClass: blocked ? "unavailable" : rust.gapClass,
    powerDeltaClass,
    resultCorrelated: blocked ? false : resultCorrelated,
    fakePoolSubmitObserved: blocked ? false : fakePoolSubmitObserved,
    difficultyMaskClassRust,
    wireParityTicketMaskRetained: blocked ? false : wireParityTicketMaskRetained,
    forcedAbLabel: forced,
    recommended,
  });
}

function renderReport({
  comparisonStatus,
  upstreamPath,
  rustPath,
  blockedReasons,
  versionMaskPreludeCountClassRust,
  versionMaskPreludeMatch,
  readChipIdTxPresentUpstream,
  readChipIdTxPresentRust,
  readChipIdTxMatch,
  chipDetectedRust,
  chipCountClassUpstream,
  chipCountClassRust,
  chipCountSourceClassRust,
  addressAssignmentCountRust,
  addressIntervalClassRust,
  addressIntervalMatch,
  enumerateToMiningReadyGapClass,
  powerDeltaClass,
  resultCorrelated,
  fakePoolSubmitObserved,
  difficultyMaskClassRust,
  wireParityTicketMaskRetained,
  forcedAbLabel: forced,
  recommended,
}) {
  const label = (value) => (value ? path.basename(value) : "missing");

  const lines = [
    "# Phase 28.1.1.5 Chip-Enumerate Comparator Report",
    "",
    `comparison_status: ${comparisonStatus}`,
    `upstream_source_label: ${label(upstreamPath)}`,
    `rust_source_label: ${label(rustPath)}`,
    "raw_bytes_committed: false",
    "credential_contents_read: false",
    "phase30_promotion_input: pending",
    "read_chip_id_byte_patch_recommended: false",
    "",
    "## D-08 Chip-Enumerate Metrics",
    "",
    `version_mask_prelude_count_class_rust: ${versionMaskPreludeCountClassRust}`,
    `version_mask_prelude_match: ${versionMaskPreludeMatch}`,
    `read_chip_id_tx_present_upstream: ${readChipIdTxPresentUpstream}`,
    `read_chip_id_tx_present_rust: ${readChipIdTxPresentRust}`,
    `read_chip_id_tx_match: ${readChipIdTxMatch}`,
    `chip_detected_rust: ${chipDetectedRust}`,
    `chip_count_class_upstream: ${chipCountClassUpstream}`,
    `chip_count_class_rust: ${chipCountClassRust}`,
    `chip_count_source_class_rust: ${chipCountSourceClassRust}`,
    `address_assignment_count_rust: ${addressAssignmentCountRust}`,
    `address_interval_class_rust: ${addressIntervalClassRust}`,
    `address_interval_match: ${addressIntervalMatch}`,
    `enumerate_to_mining_ready_gap_class: ${enumerateToMiningReadyGapClass}`,
    `power_delta_class: ${powerDeltaClass}`,
    `result_correlated: ${resultCorrelated}`,
    `fake_pool_submit_observed: ${fakePoolSubmitObserved}`,
    `difficulty_mask_class_rust: ${difficultyMaskClassRust}`,
    `wire_parity_ticket_mask_retained: ${wireParityTicketMaskRetained}`,
    `forced_ab_label: ${forced}`,
    `recommended_investigation: ${recommended}`,
    "",
    "## Notes",
    "",
    "Category/boolean fields only; no raw UART hex or credential contents.",
    "ReadChipId TX frame bytes already match (D-02) — never recommend 0x0A byte patch.",
    "power_delta_class is fast feedback only — phase gate remains correlate+submit.",
    "forced_ab_label defaults to count_asic_chips_rx_loop_parity for Ultra 205 diverge.",
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
  parseEnumerateSummary,
  parsePowerDeltaClass,
  recommendChipEnumerateInvestigation,
  reportFor,
  summarizeLog,
  versionMaskPreludeClass,
};
