#!/usr/bin/env node
/**
 * Phase 28.1.1.4 init-sequencing / dynamic-frame comparator.
 *
 * Composes Phase 28.1.1.2 parsers (lineEvents / classifyFrame / summarizeLog).
 * Compares last mining-ready window before first job_tx for regs 0x14 / 0x08 / 0x10
 * as payload classes only. Never writes raw UART hex or credentials.
 *
 * Recommender closed set: ticket_mask_asic_difficulty |
 * match_upstream_chip_enumerate_before_init | none.
 * Never emits falsified knobs (post_max_baud_delay_2000,
 * match_upstream_register_read_poll, upstream_like_long_block_receive).
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
  "ticket_mask_asic_difficulty",
  "match_upstream_chip_enumerate_before_init",
  "none",
]);

const HARD_BAN = new Set([
  "post_max_baud_delay_2000",
  "match_upstream_register_read_poll",
  "upstream_like_long_block_receive",
]);

const JOB_FRAME_LEN = 88;
const JOB_HEADER = 0x21;
const JOB_LENGTH_FIELD = 0x56;

/** Known FREQUENCY_485 payload (bytes 6–9). */
const FREQUENCY_485_PAYLOAD = [0x50, 0xc2, 0x02, 0x40];
/** Known NONCE_SPACE_485 payload (bytes 6–9). */
const NONCE_SPACE_485_PAYLOAD = [0x00, 0x0d, 0x32, 0x24];

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.4-init-sequencing-compare.mjs --upstream <log> --rust <log> --out <report.md>

Compares redaction-safe BM1366 init-sequencing / dynamic-frame categories.
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

function isJobFrame(bytes) {
  return (
    bytes.length === JOB_FRAME_LEN &&
    bytes[0] === 0x55 &&
    bytes[1] === 0xaa &&
    bytes[2] === JOB_HEADER &&
    bytes[3] === JOB_LENGTH_FIELD
  );
}

function reverseBits(byte) {
  let n = byte & 0xff;
  let reversed = 0;
  for (let i = 0; i < 8; i += 1) {
    reversed = (reversed << 1) | (n & 1);
    n >>= 1;
  }
  return reversed & 0xff;
}

/**
 * Same power-of-two + reverse-bits rule as Rust difficulty_mask_value /
 * upstream get_difficulty_mask. Returns 4-byte payload.
 */
function difficultyMaskPayload(difficulty) {
  let value = Math.ceil(difficulty) >>> 0;
  let power = 0;
  while (value > 1) {
    value >>= 1;
    power += 1;
  }
  const mask = (1 << power) - 1;
  return [
    reverseBits((mask >>> 24) & 0xff),
    reverseBits((mask >>> 16) & 0xff),
    reverseBits((mask >>> 8) & 0xff),
    reverseBits(mask & 0xff),
  ];
}

const KNOWN_MASK_PAYLOADS = new Map([
  [difficultyMaskPayload(16).join(","), { wire: "diff_16", inputs: [16] }],
  [difficultyMaskPayload(256).join(","), { wire: "diff_256", inputs: [256] }],
  [
    difficultyMaskPayload(512).join(","),
    { wire: "mask_eq_pow2_le_512", inputs: [512, 1000] },
  ],
]);

/**
 * Classify a 4-byte difficulty-mask payload.
 * @param {number[]} payload
 * @param {{ side?: "upstream"|"rust", rustSource?: string|null }} [opts]
 */
function classifyDifficultyMaskPayload(payload, opts = {}) {
  if (!payload || payload.length < 4) {
    return "other";
  }

  const key = payload.slice(0, 4).join(",");
  const known = KNOWN_MASK_PAYLOADS.get(key);
  if (!known) {
    return "other";
  }

  if (known.wire === "diff_16") {
    return "diff_16";
  }
  if (known.wire === "diff_256") {
    return "diff_256";
  }

  // 512 and 1000 share identical wire bytes.
  if (opts.side === "rust") {
    if (
      opts.rustSource === "pool_stratumdiff_1000" ||
      opts.rustSource === "pool" ||
      opts.rustSource === "stratumdiff"
    ) {
      return "diff_1000";
    }
    // Wire-only: allow ambiguous label (RESEARCH Pitfall 3).
    return "diff_1000_or_512";
  }

  // Upstream: prefer nearest known; ASIC family is 256 (not this wire).
  // If somehow upstream has 512/1000 wire, report ambiguous.
  return "diff_1000_or_512";
}

function payloadsEqual(a, b) {
  if (!a || !b || a.length < 4 || b.length < 4) {
    return false;
  }
  return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3];
}

function isFrequency485Payload(payload) {
  return payloadsEqual(payload, FREQUENCY_485_PAYLOAD);
}

function isNonceSpace485Payload(payload) {
  return payloadsEqual(payload, NONCE_SPACE_485_PAYLOAD);
}

/**
 * Walk log lines and collect typed TX frame events with payloads until first job_tx.
 * Returns the last complete mining-ready window (events before first job).
 */
function collectPreJobWindow(text) {
  const frames = [];
  if (!text) {
    return { frames, jobSeen: false };
  }

  for (const line of text.split(/\r?\n/)) {
    const maybeFrame = maybeFrameFromLine(line);
    if (!maybeFrame || maybeFrame.direction !== "tx") {
      continue;
    }

    const { bytes } = maybeFrame;
    if (isJobFrame(bytes)) {
      return { frames, jobSeen: true };
    }

    if (bytes.length < 6 || bytes[0] !== 0x55 || bytes[1] !== 0xaa) {
      continue;
    }

    const kind = classifyFrame(maybeFrame);
    if (
      kind !== "difficulty_mask_tx" &&
      kind !== "frequency_tx" &&
      kind !== "nonce_space_tx"
    ) {
      continue;
    }

    const payload = bytes.length >= 10 ? bytes.slice(6, 10) : [];
    frames.push({ kind, payload, register: bytes[5] });
  }

  return { frames, jobSeen: false };
}

/**
 * Infer rust difficulty source from log markers when present.
 */
function inferRustDifficultySource(text) {
  if (!text) {
    return null;
  }
  if (/pool_stratumdiff_1000|stratumdiff[=:]?\s*1000|difficulty[=:]\s*1000/i.test(text)) {
    return "pool_stratumdiff_1000";
  }
  if (/asic_init_sequencing_summary\s+mask_class=diff_1000/.test(text)) {
    return "pool_stratumdiff_1000";
  }
  if (/asic_init_sequencing_summary\s+mask_class=diff_256/.test(text)) {
    return "asic_family_256";
  }
  return null;
}

function analyzeWindow(frames, { side, rustSource }) {
  const difficultyFrames = frames.filter((f) => f.kind === "difficulty_mask_tx");
  const frequencyFrames = frames.filter((f) => f.kind === "frequency_tx");
  const nonceFrames = frames.filter((f) => f.kind === "nonce_space_tx");

  const lastDifficulty = difficultyFrames.at(-1) ?? null;
  const lastFrequency = frequencyFrames.at(-1) ?? null;
  const lastNonce = nonceFrames.at(-1) ?? null;

  const difficultyClass = lastDifficulty
    ? classifyDifficultyMaskPayload(lastDifficulty.payload, { side, rustSource })
    : "other";

  const frequencyFinalMatch = lastFrequency
    ? isFrequency485Payload(lastFrequency.payload)
    : false;
  const nonceSpaceMatch = lastNonce ? isNonceSpace485Payload(lastNonce.payload) : false;
  const frequencyRampPresent = frequencyFrames.length >= 2;

  return {
    difficultyClass,
    frequencyFinalMatch,
    nonceSpaceMatch,
    frequencyRampPresent,
    difficultyCount: difficultyFrames.length,
    frequencyCount: frequencyFrames.length,
    nonceCount: nonceFrames.length,
  };
}

/**
 * Parse asic_probe=power_delta into power_delta_class (D-06).
 */
function parsePowerDeltaClass(text) {
  if (!text) {
    return "unavailable";
  }

  if (/asic_probe=power_delta[^\n]*unavailable=true/.test(text)) {
    return "unavailable";
  }

  const match = text.match(/asic_probe=power_delta[^\n]*\bdelta_mw=(-?\d+)/);
  if (!match) {
    return "unavailable";
  }

  const deltaMw = Number.parseInt(match[1], 10);
  if (deltaMw >= 2000) {
    return "rising_hashing";
  }
  if (deltaMw <= -200) {
    return "falling";
  }
  return "flat";
}

function voltageWritePresent(text) {
  if (!text) {
    return false;
  }
  return /Set ASIC voltage|asic_power=enabled|asic_power=voltage/i.test(text);
}

/**
 * D-07 / D-04 / D-05 recommender. Closed enum; never HARD_BAN labels.
 */
function recommendInitSequencingInvestigation({
  blocked,
  difficultyMaskMatch,
  difficultyMaskClassUpstream,
  difficultyMaskClassRust,
  frequencyFinalMatch,
  nonceSpaceMatch,
  resultCorrelated,
}) {
  if (blocked) {
    return "none";
  }

  const rustIsPool1000Family =
    difficultyMaskClassRust === "diff_1000" ||
    difficultyMaskClassRust === "diff_1000_or_512";

  if (
    !difficultyMaskMatch &&
    difficultyMaskClassUpstream === "diff_256" &&
    rustIsPool1000Family
  ) {
    return "ticket_mask_asic_difficulty";
  }

  if (
    difficultyMaskMatch &&
    frequencyFinalMatch &&
    nonceSpaceMatch &&
    !resultCorrelated
  ) {
    return "match_upstream_chip_enumerate_before_init";
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

  const asicEnableActiveRust = count(rustSummary, "asic_enable_active");
  const resetPulseRust = count(rustSummary, "reset_pulse");
  const voltagePresent = voltageWritePresent(rustText);

  const resultCorrelated =
    count(rustSummary, "result_correlated") > 0 ||
    Boolean(rustText && /asic_production_status=result_correlated/.test(rustText));
  const fakePoolSubmitObserved =
    count(summarizeLog(upstreamText), "fake_pool_submit") > 0 ||
    count(rustSummary, "fake_pool_submit") > 0;

  const rustSource = inferRustDifficultySource(rustText);
  // When wire is 512/1000 collision and no explicit source, prefer pool-1000 for Rust
  // (confirmed RESEARCH mismatch: Rust uses stratumdiff 1000).
  const effectiveRustSource = rustSource ?? "pool_stratumdiff_1000";

  const upstreamWindow = collectPreJobWindow(upstreamText);
  const rustWindow = collectPreJobWindow(rustText);

  const upstreamAnalysis = analyzeWindow(upstreamWindow.frames, {
    side: "upstream",
    rustSource: null,
  });
  const rustAnalysis = analyzeWindow(rustWindow.frames, {
    side: "rust",
    rustSource: effectiveRustSource,
  });

  // Upstream last-pre-job 0x14: if wire is 256, label diff_256 (not ambiguous).
  let difficultyMaskClassUpstream = upstreamAnalysis.difficultyClass;
  if (
    upstreamAnalysis.difficultyClass === "diff_1000_or_512" &&
    !upstreamWindow.frames.some((f) => f.kind === "difficulty_mask_tx")
  ) {
    difficultyMaskClassUpstream = "other";
  }
  // Re-classify upstream with side-aware: 256 wire → diff_256 already handled.
  // For upstream, if last payload is 256 mask, class is diff_256.
  const upstreamLastDiff = upstreamWindow.frames
    .filter((f) => f.kind === "difficulty_mask_tx")
    .at(-1);
  if (upstreamLastDiff) {
    difficultyMaskClassUpstream = classifyDifficultyMaskPayload(
      upstreamLastDiff.payload,
      { side: "upstream" },
    );
    // Upstream ASIC family uses 256; if payload matches 256, force diff_256.
    if (
      payloadsEqual(upstreamLastDiff.payload, difficultyMaskPayload(256))
    ) {
      difficultyMaskClassUpstream = "diff_256";
    }
  }

  let difficultyMaskClassRust = rustAnalysis.difficultyClass;
  const rustLastDiff = rustWindow.frames
    .filter((f) => f.kind === "difficulty_mask_tx")
    .at(-1);
  if (rustLastDiff) {
    difficultyMaskClassRust = classifyDifficultyMaskPayload(rustLastDiff.payload, {
      side: "rust",
      rustSource: effectiveRustSource,
    });
  }

  const difficultyMaskMatch =
    !blocked &&
    difficultyMaskClassUpstream !== "other" &&
    difficultyMaskClassRust !== "other" &&
    difficultyMaskClassUpstream === difficultyMaskClassRust;

  // frequency_final_match / nonce_space_match: compare last payloads across sides
  // when both present; also accept each side matching known 485 fixtures.
  const upstreamLastFreq = upstreamWindow.frames
    .filter((f) => f.kind === "frequency_tx")
    .at(-1);
  const rustLastFreq = rustWindow.frames
    .filter((f) => f.kind === "frequency_tx")
    .at(-1);
  const upstreamLastNonce = upstreamWindow.frames
    .filter((f) => f.kind === "nonce_space_tx")
    .at(-1);
  const rustLastNonce = rustWindow.frames
    .filter((f) => f.kind === "nonce_space_tx")
    .at(-1);

  let frequencyFinalMatch = false;
  if (upstreamLastFreq && rustLastFreq) {
    frequencyFinalMatch =
      payloadsEqual(upstreamLastFreq.payload, rustLastFreq.payload) ||
      (isFrequency485Payload(upstreamLastFreq.payload) &&
        isFrequency485Payload(rustLastFreq.payload));
  } else if (!blocked) {
    frequencyFinalMatch =
      upstreamAnalysis.frequencyFinalMatch && rustAnalysis.frequencyFinalMatch;
  }

  let nonceSpaceMatch = false;
  if (upstreamLastNonce && rustLastNonce) {
    nonceSpaceMatch =
      payloadsEqual(upstreamLastNonce.payload, rustLastNonce.payload) ||
      (isNonceSpace485Payload(upstreamLastNonce.payload) &&
        isNonceSpace485Payload(rustLastNonce.payload));
  } else if (!blocked) {
    nonceSpaceMatch =
      upstreamAnalysis.nonceSpaceMatch && rustAnalysis.nonceSpaceMatch;
  }

  const powerDeltaClass = parsePowerDeltaClass(rustText);

  let recommended = recommendInitSequencingInvestigation({
    blocked,
    difficultyMaskMatch,
    difficultyMaskClassUpstream,
    difficultyMaskClassRust,
    frequencyFinalMatch,
    nonceSpaceMatch,
    resultCorrelated,
  });

  if (!ALLOWED_RECOMMENDATIONS.has(recommended) || HARD_BAN.has(recommended)) {
    recommended = "none";
  }

  const comparisonStatus = blocked
    ? "blocked_safe_prerequisite"
    : resultCorrelated && fakePoolSubmitObserved
      ? "match"
      : "init_sequencing_gap";

  return renderReport({
    comparisonStatus,
    upstreamPath,
    rustPath,
    blockedReasons,
    asicEnableActiveRust,
    voltageWritePresent: voltagePresent,
    resetPulseRust,
    difficultyMaskClassUpstream: blocked ? "other" : difficultyMaskClassUpstream,
    difficultyMaskClassRust: blocked ? "other" : difficultyMaskClassRust,
    difficultyMaskMatch: blocked ? false : difficultyMaskMatch,
    frequencyFinalMatch: blocked ? false : frequencyFinalMatch,
    frequencyRampPresentUpstream: blocked
      ? false
      : upstreamAnalysis.frequencyRampPresent,
    frequencyRampPresentRust: blocked ? false : rustAnalysis.frequencyRampPresent,
    nonceSpaceMatch: blocked ? false : nonceSpaceMatch,
    powerDeltaClass: blocked ? "unavailable" : powerDeltaClass,
    resultCorrelated: blocked ? false : resultCorrelated,
    fakePoolSubmitObserved: blocked ? false : fakePoolSubmitObserved,
    recommended,
  });
}

function renderReport({
  comparisonStatus,
  upstreamPath,
  rustPath,
  blockedReasons,
  asicEnableActiveRust,
  voltageWritePresent: voltagePresent,
  resetPulseRust,
  difficultyMaskClassUpstream,
  difficultyMaskClassRust,
  difficultyMaskMatch,
  frequencyFinalMatch,
  frequencyRampPresentUpstream,
  frequencyRampPresentRust,
  nonceSpaceMatch,
  powerDeltaClass,
  resultCorrelated,
  fakePoolSubmitObserved,
  recommended,
}) {
  const label = (value) => (value ? path.basename(value) : "missing");

  const lines = [
    "# Phase 28.1.1.4 Init-Sequencing Comparator Report",
    "",
    `comparison_status: ${comparisonStatus}`,
    `upstream_source_label: ${label(upstreamPath)}`,
    `rust_source_label: ${label(rustPath)}`,
    "raw_bytes_committed: false",
    "credential_contents_read: false",
    "phase30_promotion_input: pending",
    "hypothesis_rescope: cores_idle_despite_enable_present",
    "",
    "## D-07 Init-Sequencing Metrics",
    "",
    `asic_enable_active_rust: ${asicEnableActiveRust}`,
    `voltage_write_present_rust: ${voltagePresent}`,
    `reset_pulse_rust: ${resetPulseRust}`,
    `difficulty_mask_class_upstream: ${difficultyMaskClassUpstream}`,
    `difficulty_mask_class_rust: ${difficultyMaskClassRust}`,
    `difficulty_mask_match: ${difficultyMaskMatch}`,
    `frequency_final_match: ${frequencyFinalMatch}`,
    `frequency_ramp_present_upstream: ${frequencyRampPresentUpstream}`,
    `frequency_ramp_present_rust: ${frequencyRampPresentRust}`,
    `nonce_space_match: ${nonceSpaceMatch}`,
    `power_delta_class: ${powerDeltaClass}`,
    `result_correlated: ${resultCorrelated}`,
    `fake_pool_submit_observed: ${fakePoolSubmitObserved}`,
    `recommended_investigation: ${recommended}`,
    "",
    "## Notes",
    "",
    "Compares last mining-ready window before first job_tx (Pattern 2).",
    "Payload classes only; no raw UART hex or credential contents.",
    "power_delta_class is fast feedback only — phase gate remains correlate+submit.",
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
  classifyDifficultyMaskPayload,
  classifyFrame,
  collectPreJobWindow,
  difficultyMaskPayload,
  HARD_BAN,
  lineEvents,
  parsePowerDeltaClass,
  recommendInitSequencingInvestigation,
  reportFor,
  summarizeLog,
};
