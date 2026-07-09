#!/usr/bin/env node
/**
 * Phase 28.1.1.5 chip-enumerate comparator fixture tests.
 *
 * Covers D-08 fields, forced_ab_label, closed recommender, HARD BAN,
 * ReadChipId byte-patch ban, power_delta_class, and CFG-07 redaction.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import {
  HARD_BAN,
  parsePowerDeltaClass,
  recommendChipEnumerateInvestigation,
  forcedAbLabel,
} from "./phase28.1.1.5-chip-enumerate-compare.mjs";

const scriptPath = path.join(
  process.cwd(),
  "scripts/phase28.1.1.5-chip-enumerate-compare.mjs",
);

// Inline fixture frames only — never asserted as committed report content.
const VERSION_MASK_FRAME = "55 aa 51 09 00 a4 00 00 00 1f 00";
const READ_CHIP_ID_FRAME = "55 aa 52 05 00 00 0a";
const CHIP_ID_RX_FRAME = "55 aa 13 68 00 00 00 00 00 00 00";
const SET_CHIP_ADDRESS_FRAME = "55 aa 40 05 00 00 00";
const DIFF_256_FRAME = "55 aa 51 09 00 14 00 00 00 ff 00";
const FREQ_485 = "55 aa 51 09 00 08 50 c2 02 40 1c";
const NONCE_485 = "55 aa 51 09 00 10 00 0d 32 24 10";
const JOB_FRAME = `55 aa 21 56 ${"00 ".repeat(82)}00 00`;

function runComparator({ upstream, rust }) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "phase28.1.1.5-enum-"));
  const upstreamPath = path.join(tempDir, "upstream.log");
  const rustPath = path.join(tempDir, "rust.log");
  const outPath = path.join(tempDir, "report.md");
  if (upstream !== null) fs.writeFileSync(upstreamPath, upstream, "utf8");
  if (rust !== null) fs.writeFileSync(rustPath, rust, "utf8");

  const result = spawnSync(
    process.execPath,
    [
      scriptPath,
      "--upstream",
      upstreamPath,
      "--rust",
      rustPath,
      "--out",
      outPath,
    ],
    { encoding: "utf8" },
  );

  assert.equal(result.status, 0, result.stderr || result.stdout);
  return {
    report: fs.readFileSync(outPath, "utf8"),
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

/** Upstream: 3× version mask → ReadChipId → count drain → address → mining-ready. */
function divergeUpstreamLog() {
  return [
    "Starting ASIC initialization (cold boot mode)",
    `tx: [${VERSION_MASK_FRAME}]`,
    `tx: [${VERSION_MASK_FRAME}]`,
    `tx: [${VERSION_MASK_FRAME}]`,
    `tx: [${READ_CHIP_ID_FRAME}]`,
    "count_asic_chips drain_idle_like",
    `rx: [${CHIP_ID_RX_FRAME}]`,
    "asic_uart_trace=rx_idle",
    "chip_count=1 address_interval=256",
    `tx: [${SET_CHIP_ADDRESS_FRAME}]`,
    "asic_chip_enumerate_summary chip_count_source=counted_rx chip_count=1 address_interval=256 gap=drain_idle_like chip_detected=true",
    `tx: [${DIFF_256_FRAME}]`,
    `tx: [${FREQ_485}]`,
    `tx: [${NONCE_485}]`,
    `tx: [${JOB_FRAME}]`,
  ].join("\n");
}

/** Rust Ultra 205 diverge: TX match, config_expected, immediate gap, falling power. */
function divergeRustLog() {
  return [
    "asic_reset_status=post_bring_up_pulse",
    "asic_enable_status=active",
    "Set ASIC voltage to 1200 mV",
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=7 hex=${READ_CHIP_ID_FRAME}`,
    "asic_uart_trace=wait_tx_done",
    `asic_uart_trace=rx_complete read_count=11 hex=${CHIP_ID_RX_FRAME}`,
    "asic_chip_detect=ok",
    "chip_count_source=config_expected chip_count=1 address_interval=256",
    "asic_chip_enumerate_summary chip_count_source=config_expected chip_count=1 address_interval=256 gap=immediate chip_detected=true",
    "mining_ready_init_started",
    `asic_uart_trace=tx len=7 hex=${SET_CHIP_ADDRESS_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${DIFF_256_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${FREQ_485}`,
    `asic_uart_trace=tx len=11 hex=${NONCE_485}`,
    `asic_uart_trace=tx len=88 hex=${JOB_FRAME}`,
    "asic_probe=power_delta baseline_mw=5000 after_mw=4610 delta_mw=-390",
  ].join("\n");
}

/** Both sides drain_idle_like + counted_rx + interval match; still no correlate. */
function matchedEnumerateUpstreamLog() {
  return [
    "Starting ASIC initialization",
    `tx: [${VERSION_MASK_FRAME}]`,
    `tx: [${VERSION_MASK_FRAME}]`,
    `tx: [${VERSION_MASK_FRAME}]`,
    `tx: [${READ_CHIP_ID_FRAME}]`,
    `rx: [${CHIP_ID_RX_FRAME}]`,
    "asic_chip_enumerate_summary chip_count_source=counted_rx chip_count=1 address_interval=256 gap=drain_idle_like chip_detected=true",
    `tx: [${SET_CHIP_ADDRESS_FRAME}]`,
    `tx: [${DIFF_256_FRAME}]`,
    `tx: [${FREQ_485}]`,
    `tx: [${NONCE_485}]`,
    `tx: [${JOB_FRAME}]`,
  ].join("\n");
}

function matchedEnumerateRustLog() {
  return [
    "asic_enable_status=active",
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=7 hex=${READ_CHIP_ID_FRAME}`,
    `asic_uart_trace=rx_complete read_count=11 hex=${CHIP_ID_RX_FRAME}`,
    "asic_uart_trace=rx_idle",
    "asic_chip_enumerate_summary chip_count_source=counted_rx chip_count=1 address_interval=256 gap=drain_idle_like chip_detected=true",
    `asic_uart_trace=tx len=7 hex=${SET_CHIP_ADDRESS_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${DIFF_256_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${FREQ_485}`,
    `asic_uart_trace=tx len=11 hex=${NONCE_485}`,
    `asic_uart_trace=tx len=88 hex=${JOB_FRAME}`,
    "asic_probe=power_delta baseline_mw=5000 after_mw=4610 delta_mw=-390",
  ].join("\n");
}

function testA_ultra205DivergeForcesRxLoopParity() {
  // Arrange
  const upstream = divergeUpstreamLog();
  const rust = divergeRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /read_chip_id_tx_present_upstream: true/);
  assert.match(report, /read_chip_id_tx_present_rust: true/);
  assert.match(report, /read_chip_id_tx_match: true/);
  assert.match(report, /chip_detected_rust: true/);
  assert.match(report, /chip_count_source_class_rust: config_expected/);
  assert.match(report, /chip_count_class_rust: config_expected_1/);
  assert.match(report, /chip_count_class_upstream: counted_1/);
  assert.match(report, /address_interval_class_rust: interval_256/);
  assert.match(report, /address_interval_match: true/);
  assert.match(report, /enumerate_to_mining_ready_gap_class: immediate/);
  assert.match(report, /power_delta_class: falling/);
  assert.match(report, /result_correlated: false/);
  assert.match(report, /fake_pool_submit_observed: false/);
  assert.match(report, /forced_ab_label: count_asic_chips_rx_loop_parity/);
  assert.match(
    report,
    /recommended_investigation: match_upstream_chip_enumerate_before_init/,
  );
  assert.match(report, /read_chip_id_byte_patch_recommended: false/);
  assert.match(report, /raw_bytes_committed: false/);
  assert.match(report, /credential_contents_read: false/);
  assert.match(report, /phase30_promotion_input: pending/);
  assert.match(report, /wire_parity_ticket_mask_retained: true/);
  assert.match(report, /difficulty_mask_class_rust: diff_256/);
}

function testB_enumerateMatchedRecommendsVersionRolling() {
  // Arrange
  const upstream = matchedEnumerateUpstreamLog();
  const rust = matchedEnumerateRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /chip_count_source_class_rust: counted_rx/);
  assert.match(report, /enumerate_to_mining_ready_gap_class: drain_idle_like/);
  assert.match(report, /address_interval_match: true/);
  assert.match(report, /result_correlated: false/);
  assert.match(report, /recommended_investigation: version_rolling_negotiation/);
  assert.match(report, /forced_ab_label: none/);
  assert.match(report, /read_chip_id_byte_patch_recommended: false/);
}

function testC_missingLogBlocks() {
  // Arrange / Act
  const { report } = runComparator({ upstream: null, rust: divergeRustLog() });

  // Assert
  assert.match(report, /comparison_status: blocked_safe_prerequisite/);
  assert.match(report, /recommended_investigation: none/);
  assert.match(report, /forced_ab_label: none/);
  assert.match(report, /read_chip_id_byte_patch_recommended: false/);
}

function testD_powerDeltaClasses() {
  // Arrange / Act / Assert — unit helpers
  assert.equal(
    parsePowerDeltaClass(
      "asic_probe=power_delta baseline_mw=1 after_mw=2 delta_mw=-390",
    ),
    "falling",
  );
  assert.equal(
    parsePowerDeltaClass(
      "asic_probe=power_delta baseline_mw=1 after_mw=5000 delta_mw=2500",
    ),
    "rising_hashing",
  );
  assert.equal(
    parsePowerDeltaClass("asic_probe=power_delta unavailable=true"),
    "unavailable",
  );
  assert.equal(parsePowerDeltaClass(""), "unavailable");

  // CLI path: rising
  const risingRust = divergeRustLog().replace("delta_mw=-390", "delta_mw=2500");
  const { report: risingReport } = runComparator({
    upstream: divergeUpstreamLog(),
    rust: risingRust,
  });
  assert.match(risingReport, /power_delta_class: rising_hashing/);

  // CLI path: unavailable
  const unavailableRust = divergeRustLog().replace(
    /asic_probe=power_delta[^\n]*/,
    "asic_probe=power_delta unavailable=true",
  );
  const { report: unavailableReport } = runComparator({
    upstream: divergeUpstreamLog(),
    rust: unavailableRust,
  });
  assert.match(unavailableReport, /power_delta_class: unavailable/);
}

function testE_neverRecommendsFalsifiedKnobsOrTicketMask() {
  // Arrange — gap fixtures that previously tempted falsified levers
  const upstream = divergeUpstreamLog();
  const rust = divergeRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });
  const recommended = report.match(/recommended_investigation: (\S+)/)[1];
  const forced = report.match(/forced_ab_label: (\S+)/)[1];

  // Assert — HARD BAN set must never appear as recommendation or forced_ab
  assert.ok(!HARD_BAN.has(recommended), `banned recommendation: ${recommended}`);
  assert.ok(!HARD_BAN.has(forced), `banned forced_ab_label: ${forced}`);
  for (const banned of HARD_BAN) {
    assert.doesNotMatch(
      report,
      new RegExp(`recommended_investigation: ${banned}`),
    );
    assert.doesNotMatch(report, new RegExp(`forced_ab_label: ${banned}`));
  }

  // Explicit ban strings present in this test file (acceptance criterion)
  const selfSource = fs.readFileSync(new URL(import.meta.url), "utf8");
  assert.ok(selfSource.includes("post_max_baud_delay_2000"));
  assert.ok(selfSource.includes("match_upstream_register_read_poll"));
  assert.ok(selfSource.includes("upstream_like_long_block_receive"));
  assert.ok(selfSource.includes("ticket_mask_asic_difficulty"));

  // Direct recommender unit checks
  assert.equal(
    recommendChipEnumerateInvestigation({
      blocked: false,
      enumerateDiverge: true,
      enumerateMatched: false,
      resultCorrelated: false,
    }),
    "match_upstream_chip_enumerate_before_init",
  );
  assert.equal(
    recommendChipEnumerateInvestigation({
      blocked: false,
      enumerateDiverge: false,
      enumerateMatched: true,
      resultCorrelated: false,
    }),
    "version_rolling_negotiation",
  );
  assert.equal(
    recommendChipEnumerateInvestigation({
      blocked: true,
      enumerateDiverge: true,
      enumerateMatched: false,
      resultCorrelated: false,
    }),
    "none",
  );
  assert.ok(
    !HARD_BAN.has(
      recommendChipEnumerateInvestigation({
        blocked: false,
        enumerateDiverge: true,
        enumerateMatched: false,
        resultCorrelated: false,
      }),
    ),
  );

  assert.equal(
    forcedAbLabel({
      blocked: false,
      addressIntervalMatch: true,
      gapClassRust: "immediate",
      chipCountSourceRust: "config_expected",
      readChipIdTxMatch: true,
    }),
    "count_asic_chips_rx_loop_parity",
  );
}

function testF_reportDoesNotLeakHexOrCredentials() {
  // Arrange
  const upstream = `${divergeUpstreamLog()}\npoolPassword=PHASE28_SENTINEL_PASSWORD\npoolURL=stratum+tcp://pool.example:3333`;
  const rust = `${divergeRustLog()}\nssid=PHASE28_SENTINEL_SSID`;

  // Act
  const { report, stdout, stderr } = runComparator({ upstream, rust });
  const combined = `${report}\n${stdout}\n${stderr}`;

  // Assert — CFG-07 / T-28115-01
  assert(!combined.includes("PHASE28_SENTINEL_PASSWORD"));
  assert(!combined.includes("PHASE28_SENTINEL_SSID"));
  assert(!combined.includes("poolPassword"));
  assert(!combined.includes("poolURL"));
  assert(!combined.includes("stratum+tcp://"));
  // Multi-byte hex dump patterns must not appear in committed report body
  assert(!/55 aa 52(?: [0-9a-fA-F]{2}){4,}/i.test(report));
  assert(!/55 AA 52(?: [0-9a-fA-F]{2}){4,}/.test(report));
  assert(!/55 aa 51(?: [0-9a-fA-F]{2}){4,}/i.test(report));
  assert(!/55 AA 51(?: [0-9a-fA-F]{2}){4,}/.test(report));
  assert(!report.includes(READ_CHIP_ID_FRAME));
  assert(!report.includes(DIFF_256_FRAME));
  assert(!report.includes(JOB_FRAME.trim()));
}

function testG_neverRecommendsReadChipIdBytePatch() {
  // Arrange / Act
  const { report } = runComparator({
    upstream: divergeUpstreamLog(),
    rust: divergeRustLog(),
  });

  // Assert — D-02
  assert.match(report, /read_chip_id_byte_patch_recommended: false/);
  assert.match(report, /read_chip_id_tx_match: true/);
  assert.doesNotMatch(report, /read_chip_id_byte_patch_recommended: true/);
}

testA_ultra205DivergeForcesRxLoopParity();
testB_enumerateMatchedRecommendsVersionRolling();
testC_missingLogBlocks();
testD_powerDeltaClasses();
testE_neverRecommendsFalsifiedKnobsOrTicketMask();
testF_reportDoesNotLeakHexOrCredentials();
testG_neverRecommendsReadChipIdBytePatch();

console.log("phase28.1.1.5 chip-enumerate comparator tests passed");
