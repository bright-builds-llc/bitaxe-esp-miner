#!/usr/bin/env node
/**
 * Phase 28.1.1.3 RX-acquisition comparator fixture tests.
 *
 * Covers D-06 first-class fields, recommender (result_rx_acquisition_model),
 * flood-safe summary markers, and CFG-07 redaction sentinels.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const scriptPath = path.join(
  process.cwd(),
  "scripts/phase28.1.1.3-rx-acquisition-compare.mjs",
);

const chipIdReadFrame = "55 aa 52 05 00 00 0a";
const jobFrame = `55 aa 21 56 ${"00 ".repeat(82)}00 00`;
const rxFrame = "aa 55 00 00 00 00 00 00 00 00 00";

function runComparator({ upstream, rust }) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "phase28.1.1.3-rx-"));
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

function gapShapeUpstreamLog() {
  const chipReads = Array.from({ length: 40 }, () => `tx: [${chipIdReadFrame}]`).join(
    "\n",
  );
  return [
    "Starting ASIC initialization (cold boot mode)",
    chipReads,
    `tx: [${jobFrame}]`,
  ].join("\n");
}

/** Test A: jobs + result_read_attempt, zero correlate, partial_frame may be 0. */
function gapShapeRustLog() {
  const resultReads = Array.from(
    { length: 20 },
    () => "asic_production_trace=result_read_attempt poll_timeout_ms=100",
  ).join("\n");
  return [
    "asic_reset_status=post_bring_up_pulse",
    "asic_enable_status=active",
    `asic_uart_trace=tx len=7 hex=${chipIdReadFrame}`,
    `asic_uart_trace=tx len=88 hex=${jobFrame}`,
    resultReads,
  ].join("\n");
}

function summaryMarkerRustLog() {
  return [
    "asic_reset_status=post_bring_up_pulse",
    "asic_enable_status=active",
    `asic_uart_trace=tx len=88 hex=${jobFrame}`,
    "asic_production_trace=result_read_attempt poll_timeout_ms=100",
    "asic_rx_acquisition_summary idle=10 partial=3 clear=3 complete=1",
  ].join("\n");
}

function correlatedRustLog() {
  return [
    "asic_reset_status=post_bring_up_pulse",
    "asic_enable_status=active",
    `asic_uart_trace=tx len=88 hex=${jobFrame}`,
    "asic_production_trace=result_read_attempt poll_timeout_ms=100",
    `asic_uart_trace=rx_complete read_count=1 hex=${rxFrame}`,
    "asic_production_status=result_correlated",
    "share_submission_status=accepted",
  ].join("\n");
}

function testA_recommendsResultRxAcquisitionModel() {
  // Arrange
  const upstream = gapShapeUpstreamLog();
  const rust = gapShapeRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert — D-06 keys + recommender without requiring partial_frame >= 5
  assert.match(report, /recommended_investigation: result_rx_acquisition_model/);
  assert.match(report, /result_correlated: false/);
  assert.match(report, /fake_pool_submit_observed: false/);
  assert.match(report, /partial_frame_upstream: \d+/);
  assert.match(report, /partial_frame_rust: \d+/);
  assert.match(report, /clear_rx_upstream: \d+/);
  assert.match(report, /clear_rx_rust: \d+/);
  assert.match(report, /rx_idle_upstream: \d+/);
  assert.match(report, /rx_idle_rust: \d+/);
  assert.match(report, /rx_complete_upstream: \d+/);
  assert.match(report, /rx_complete_rust: \d+/);
  assert.match(report, /result_read_attempt_upstream: \d+/);
  assert.match(report, /result_read_attempt_rust: \d+/);
  assert.match(report, /raw_bytes_committed: false/);
  assert.match(report, /credential_contents_read: false/);
  assert.match(report, /phase30_promotion_input: pending/);
}

function testB_summaryMarkersIncrementCounts() {
  // Arrange
  const upstream = gapShapeUpstreamLog();
  const rust = summaryMarkerRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert — flood-safe summary path
  assert.match(report, /partial_frame_rust: ([3-9]|\d{2,})/);
  assert.match(report, /clear_rx_rust: ([3-9]|\d{2,})/);
  assert.match(report, /rx_idle_rust: (1[0-9]|\d{2,})/);
  assert.match(report, /rx_complete_rust: ([1-9]\d*)/);

  const partial = Number(report.match(/partial_frame_rust: (\d+)/)[1]);
  const clear = Number(report.match(/clear_rx_rust: (\d+)/)[1]);
  const idle = Number(report.match(/rx_idle_rust: (\d+)/)[1]);
  const complete = Number(report.match(/rx_complete_rust: (\d+)/)[1]);
  assert.ok(partial >= 3, `partial_frame_rust=${partial}`);
  assert.ok(clear >= 3, `clear_rx_rust=${clear}`);
  assert.ok(idle >= 10, `rx_idle_rust=${idle}`);
  assert.ok(complete >= 1, `rx_complete_rust=${complete}`);
}

function testC_correlatedAndFakePoolSubmit() {
  // Arrange
  const upstream = gapShapeUpstreamLog();
  const rust = correlatedRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /result_correlated: true/);
  assert.match(report, /fake_pool_submit_observed: true/);
}

function testD_missingUpstreamBlocks() {
  // Arrange / Act
  const { report } = runComparator({ upstream: null, rust: gapShapeRustLog() });

  // Assert
  assert.match(report, /comparison_status: blocked_safe_prerequisite/);
  assert.match(report, /recommended_investigation: none/);
}

function testE_reportDoesNotLeakSentinelsOrHexDumps() {
  // Arrange
  const upstream = `${gapShapeUpstreamLog()}\npoolPassword=PHASE28_SENTINEL_PASSWORD\nstratum+tcp://pool.example:3333`;
  const rust = `${gapShapeRustLog()}\nssid=PHASE28_SENTINEL_SSID`;

  // Act
  const { report, stdout, stderr } = runComparator({ upstream, rust });
  const combined = `${report}\n${stdout}\n${stderr}`;

  // Assert
  assert(!combined.includes("PHASE28_SENTINEL_PASSWORD"));
  assert(!combined.includes("PHASE28_SENTINEL_SSID"));
  assert(!combined.includes("poolPassword"));
  assert(!combined.includes("stratum+tcp://"));
  assert(!combined.includes(jobFrame.trim()));
  assert(!/55 aa 21 56(?: [0-9a-fA-F]{2}){20,}/.test(combined));
}

function testF_neverRecommendsFalsifiedLabels() {
  // Arrange
  const upstream = gapShapeUpstreamLog();
  const rust = gapShapeRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert — falsified prior-phase labels must not appear as recommendations
  assert.doesNotMatch(
    report,
    /recommended_investigation: match_upstream_register_read_poll/,
  );
  assert.doesNotMatch(report, /recommended_investigation: post_max_baud_delay_2000/);
  assert(!report.includes("post_max_baud_delay_2000"));
  assert(!report.includes("match_upstream_register_read_poll"));
}

testA_recommendsResultRxAcquisitionModel();
testB_summaryMarkersIncrementCounts();
testC_correlatedAndFakePoolSubmit();
testD_missingUpstreamBlocks();
testE_reportDoesNotLeakSentinelsOrHexDumps();
testF_neverRecommendsFalsifiedLabels();

console.log("phase28.1.1.3 rx-acquisition comparator tests passed");
