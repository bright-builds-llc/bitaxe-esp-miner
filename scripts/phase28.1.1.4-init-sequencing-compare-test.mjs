#!/usr/bin/env node
/**
 * Phase 28.1.1.4 init-sequencing comparator fixture tests.
 *
 * Covers D-07 fields, ticket-mask mismatch recommender, match→chip-enumerate
 * path, power_delta_class, blocked missing logs, HARD BAN, and CFG-07 redaction.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import {
  HARD_BAN,
  parsePowerDeltaClass,
  recommendInitSequencingInvestigation,
} from "./phase28.1.1.4-init-sequencing-compare.mjs";

const scriptPath = path.join(
  process.cwd(),
  "scripts/phase28.1.1.4-init-sequencing-compare.mjs",
);

// Wire payloads (inline fixtures only — never asserted as committed report content).
const DIFF_16_FRAME = "55 aa 51 09 00 14 00 00 00 f0 00";
const DIFF_256_FRAME = "55 aa 51 09 00 14 00 00 00 ff 00";
const DIFF_1000_FRAME = "55 aa 51 09 00 14 00 00 80 ff 04";
const FREQ_EARLY = "55 aa 51 09 00 08 40 00 00 00 00";
const FREQ_485 = "55 aa 51 09 00 08 50 c2 02 40 1c";
const NONCE_485 = "55 aa 51 09 00 10 00 0d 32 24 10";
const JOB_FRAME = `55 aa 21 56 ${"00 ".repeat(82)}00 00`;

function runComparator({ upstream, rust }) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "phase28.1.1.4-init-"));
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

/**
 * Upstream: early diff_16 init + last-pre-job diff_256, PLL ramp, HCN, then job.
 * Must select last window (not early diff_16).
 */
function mismatchUpstreamLog() {
  return [
    "Starting ASIC initialization (cold boot mode)",
    `tx: [${DIFF_16_FRAME}]`,
    `tx: [${FREQ_EARLY}]`,
    `tx: [${FREQ_485}]`,
    // Second / last mining-ready init before job
    `tx: [${DIFF_256_FRAME}]`,
    `tx: [${FREQ_EARLY}]`,
    `tx: [${FREQ_485}]`,
    `tx: [${NONCE_485}]`,
    `tx: [${JOB_FRAME}]`,
  ].join("\n");
}

/** Rust: pool-1000 ticket mask, enable/reset/voltage, falling power_delta, no correlate. */
function mismatchRustLog() {
  return [
    "asic_reset_status=post_bring_up_pulse",
    "asic_enable_status=active",
    "Set ASIC voltage to 1200 mV",
    "asic_power=enabled",
    `asic_uart_trace=tx len=11 hex=${DIFF_1000_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${FREQ_EARLY}`,
    `asic_uart_trace=tx len=11 hex=${FREQ_485}`,
    `asic_uart_trace=tx len=11 hex=${NONCE_485}`,
    `asic_uart_trace=tx len=88 hex=${JOB_FRAME}`,
    "asic_probe=power_delta baseline_mw=5000 after_mw=4610 delta_mw=-390",
  ].join("\n");
}

/** Both sides last 0x14 = diff_256; PLL/HCN match; still no correlate. */
function matchFramesUpstreamLog() {
  return [
    "Starting ASIC initialization",
    `tx: [${DIFF_256_FRAME}]`,
    `tx: [${FREQ_EARLY}]`,
    `tx: [${FREQ_485}]`,
    `tx: [${NONCE_485}]`,
    `tx: [${JOB_FRAME}]`,
  ].join("\n");
}

function matchFramesRustLog() {
  return [
    "asic_reset_status=post_bring_up_pulse",
    "asic_enable_status=active",
    "Set ASIC voltage to 1200 mV",
    `asic_uart_trace=tx len=11 hex=${DIFF_256_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${FREQ_EARLY}`,
    `asic_uart_trace=tx len=11 hex=${FREQ_485}`,
    `asic_uart_trace=tx len=11 hex=${NONCE_485}`,
    `asic_uart_trace=tx len=88 hex=${JOB_FRAME}`,
    "asic_probe=power_delta baseline_mw=5000 after_mw=4610 delta_mw=-390",
  ].join("\n");
}

function testA_ticketMaskMismatchRecommendsAsicDifficulty() {
  // Arrange
  const upstream = mismatchUpstreamLog();
  const rust = mismatchRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /difficulty_mask_class_upstream: diff_256/);
  assert.match(report, /difficulty_mask_class_rust: diff_1000/);
  assert.match(report, /difficulty_mask_match: false/);
  assert.match(report, /frequency_final_match: true/);
  assert.match(report, /nonce_space_match: true/);
  assert.match(report, /frequency_ramp_present_upstream: true/);
  assert.match(report, /frequency_ramp_present_rust: true/);
  assert.match(report, /asic_enable_active_rust: [1-9]/);
  assert.match(report, /voltage_write_present_rust: true/);
  assert.match(report, /reset_pulse_rust: [1-9]/);
  assert.match(report, /power_delta_class: falling/);
  assert.match(report, /result_correlated: false/);
  assert.match(report, /fake_pool_submit_observed: false/);
  assert.match(report, /recommended_investigation: ticket_mask_asic_difficulty/);
  assert.match(report, /hypothesis_rescope: cores_idle_despite_enable_present/);
  assert.match(report, /raw_bytes_committed: false/);
  assert.match(report, /credential_contents_read: false/);
  assert.match(report, /phase30_promotion_input: pending/);
}

function testB_framesMatchRecommendsChipEnumerate() {
  // Arrange
  const upstream = matchFramesUpstreamLog();
  const rust = matchFramesRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /difficulty_mask_class_upstream: diff_256/);
  assert.match(report, /difficulty_mask_class_rust: diff_256/);
  assert.match(report, /difficulty_mask_match: true/);
  assert.match(report, /frequency_final_match: true/);
  assert.match(report, /nonce_space_match: true/);
  assert.match(report, /result_correlated: false/);
  assert.match(
    report,
    /recommended_investigation: match_upstream_chip_enumerate_before_init/,
  );
}

function testC_missingLogBlocks() {
  // Arrange / Act
  const { report } = runComparator({ upstream: null, rust: mismatchRustLog() });

  // Assert
  assert.match(report, /comparison_status: blocked_safe_prerequisite/);
  assert.match(report, /recommended_investigation: none/);
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
  assert.equal(
    parsePowerDeltaClass(
      "asic_probe=power_delta baseline_mw=1 after_mw=2 delta_mw=100",
    ),
    "flat",
  );

  // CLI path: rising
  const risingRust = mismatchRustLog().replace(
    "delta_mw=-390",
    "delta_mw=2500",
  );
  const { report: risingReport } = runComparator({
    upstream: mismatchUpstreamLog(),
    rust: risingRust,
  });
  assert.match(risingReport, /power_delta_class: rising_hashing/);

  // CLI path: unavailable
  const unavailableRust = mismatchRustLog().replace(
    /asic_probe=power_delta[^\n]*/,
    "asic_probe=power_delta unavailable=true",
  );
  const { report: unavailableReport } = runComparator({
    upstream: mismatchUpstreamLog(),
    rust: unavailableRust,
  });
  assert.match(unavailableReport, /power_delta_class: unavailable/);
}

function testE_neverRecommendsFalsifiedKnobs() {
  // Arrange — gap fixtures that previously tempted falsified levers
  const upstream = mismatchUpstreamLog();
  const rust = mismatchRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });
  const recommended = report.match(/recommended_investigation: (\S+)/)[1];

  // Assert — HARD BAN set must never appear as recommendation
  assert.ok(!HARD_BAN.has(recommended), `banned recommendation: ${recommended}`);
  for (const banned of HARD_BAN) {
    assert.doesNotMatch(report, new RegExp(`recommended_investigation: ${banned}`));
    assert.ok(
      !report.includes(`recommended_investigation: ${banned}`),
      `report must not recommend ${banned}`,
    );
  }

  // Direct recommender unit checks
  const fromRecommender = recommendInitSequencingInvestigation({
    blocked: false,
    difficultyMaskMatch: false,
    difficultyMaskClassUpstream: "diff_256",
    difficultyMaskClassRust: "diff_1000",
    frequencyFinalMatch: true,
    nonceSpaceMatch: true,
    resultCorrelated: false,
  });
  assert.equal(fromRecommender, "ticket_mask_asic_difficulty");
  assert.ok(!HARD_BAN.has(fromRecommender));

  // Ban strings must appear in this test file as assertions (acceptance criterion)
  assert.ok(
    fs
      .readFileSync(new URL(import.meta.url), "utf8")
      .includes("post_max_baud_delay_2000"),
  );
  assert.ok(
    fs
      .readFileSync(new URL(import.meta.url), "utf8")
      .includes("match_upstream_register_read_poll"),
  );
  assert.ok(
    fs
      .readFileSync(new URL(import.meta.url), "utf8")
      .includes("upstream_like_long_block_receive"),
  );
}

function testF_reportDoesNotLeakHexOrCredentials() {
  // Arrange
  const upstream = `${mismatchUpstreamLog()}\npoolPassword=PHASE28_SENTINEL_PASSWORD\npoolURL=stratum+tcp://pool.example:3333`;
  const rust = `${mismatchRustLog()}\nssid=PHASE28_SENTINEL_SSID`;

  // Act
  const { report, stdout, stderr } = runComparator({ upstream, rust });
  const combined = `${report}\n${stdout}\n${stderr}`;

  // Assert — CFG-07 / T-28114-01
  assert(!combined.includes("PHASE28_SENTINEL_PASSWORD"));
  assert(!combined.includes("PHASE28_SENTINEL_SSID"));
  assert(!combined.includes("poolPassword"));
  assert(!combined.includes("poolURL"));
  assert(!combined.includes("stratum+tcp://"));
  // Multi-byte hex dump patterns must not appear in committed report body
  assert(!/55 aa 51(?: [0-9a-fA-F]{2}){4,}/i.test(report));
  assert(!/55 AA 51(?: [0-9a-fA-F]{2}){4,}/.test(report));
  assert(!report.includes(DIFF_1000_FRAME));
  assert(!report.includes(DIFF_256_FRAME));
  assert(!report.includes(JOB_FRAME.trim()));
}

function testG_ignoresEarlyUpstreamDiff16() {
  // Arrange — early diff_16 must not become the sole golden
  const { report } = runComparator({
    upstream: mismatchUpstreamLog(),
    rust: mismatchRustLog(),
  });

  // Assert — last-pre-job class is diff_256, not diff_16
  assert.match(report, /difficulty_mask_class_upstream: diff_256/);
  assert.doesNotMatch(report, /difficulty_mask_class_upstream: diff_16/);
}

testA_ticketMaskMismatchRecommendsAsicDifficulty();
testB_framesMatchRecommendsChipEnumerate();
testC_missingLogBlocks();
testD_powerDeltaClasses();
testE_neverRecommendsFalsifiedKnobs();
testF_reportDoesNotLeakHexOrCredentials();
testG_ignoresEarlyUpstreamDiff16();

console.log("phase28.1.1.4 init-sequencing comparator tests passed");
