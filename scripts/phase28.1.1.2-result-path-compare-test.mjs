#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const scriptPath = path.join(
  process.cwd(),
  "scripts/phase28.1.1.2-result-path-compare.mjs",
);

const chipIdReadFrame = "55 aa 52 05 00 00 0a";
const jobFrame = `55 aa 21 56 ${"00 ".repeat(82)}00 00`;
const rxFrame = "aa 55 00 00 00 00 00 00 00 00 00";

function runComparator({ upstream, rust }) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "phase28.1.1.2-result-"));
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
  // Many chip_id_read_tx (0x52) lines — upstream hashrate-monitor register poll shape.
  // No submit marker: Test A asserts fake_pool_submit_observed: false for the gap shape.
  const chipReads = Array.from({ length: 40 }, () => `tx: [${chipIdReadFrame}]`).join("\n");
  return [
    "Starting ASIC initialization (cold boot mode)",
    chipReads,
    `tx: [${jobFrame}]`,
  ].join("\n");
}

function gapShapeRustLog() {
  // job_tx + many result_read_attempt, zero result_correlated — known 28.1.1.1 gap.
  const resultReads = Array.from(
    { length: 20 },
    () => "asic_production_trace=result_read_attempt poll_timeout_ms=100",
  ).join("\n");
  return [
    "asic_reset_status=post_bring_up_pulse",
    `asic_uart_trace=tx len=7 hex=${chipIdReadFrame}`,
    `asic_uart_trace=tx len=7 hex=${chipIdReadFrame}`,
    `asic_uart_trace=tx len=7 hex=${chipIdReadFrame}`,
    `asic_uart_trace=tx len=88 hex=${jobFrame}`,
    resultReads,
    "asic_uart_trace=partial_frame",
  ].join("\n");
}

function correlatedRustLog() {
  return [
    "asic_reset_status=post_bring_up_pulse",
    `asic_uart_trace=tx len=88 hex=${jobFrame}`,
    "asic_production_trace=result_read_attempt poll_timeout_ms=100",
    `asic_uart_trace=rx_complete read_count=1 hex=${rxFrame}`,
    "asic_production_trace=register_read_parsed",
    "asic_production_status=result_correlated",
    "share_submission_status=accepted",
  ].join("\n");
}

function testA_recommendsMatchUpstreamRegisterReadPoll() {
  // Arrange
  const upstream = gapShapeUpstreamLog();
  const rust = gapShapeRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /recommended_investigation: match_upstream_register_read_poll/);
  assert.match(report, /result_correlated: false/);
  assert.match(report, /fake_pool_submit_observed: false/);
  assert.match(report, /chip_id_read_tx_upstream: \d+/);
  assert.match(report, /chip_id_read_tx_rust: \d+/);
  assert.match(report, /result_read_attempt_upstream: \d+/);
  assert.match(report, /result_read_attempt_rust: \d+/);
  assert.match(report, /rx_result_frame_upstream: \d+/);
  assert.match(report, /rx_result_frame_rust: \d+/);
  assert.match(report, /register_read_parsed_upstream: \d+/);
  assert.match(report, /register_read_parsed_rust: \d+/);
  assert.match(report, /raw_bytes_committed: false/);
  assert.match(report, /credential_contents_read: false/);
  assert.match(report, /phase30_promotion_input: pending/);
}

function testB_correlatedAndFakePoolSubmit() {
  // Arrange
  const upstream = gapShapeUpstreamLog();
  const rust = correlatedRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /result_correlated: true/);
  assert.match(report, /fake_pool_submit_observed: true/);
}

function testC_missingUpstreamBlocks() {
  // Arrange / Act
  const { report } = runComparator({ upstream: null, rust: gapShapeRustLog() });

  // Assert
  assert.match(report, /comparison_status: blocked_safe_prerequisite/);
  assert.match(report, /recommended_investigation: none/);
}

function testD_reportDoesNotLeakSentinelsOrHexDumps() {
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
  // Full job-frame hex dump must not appear in the report (counts only).
  assert(!combined.includes(jobFrame.trim()));
  assert(!/55 aa 21 56(?: [0-9a-fA-F]{2}){20,}/.test(combined));
}

function testE_neverRecommendsPostMaxBaudDelay() {
  // Arrange
  const upstream = gapShapeUpstreamLog();
  const rust = gapShapeRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.doesNotMatch(report, /recommended_investigation: post_max_baud_delay_2000/);
  assert(!report.includes("post_max_baud_delay_2000"));
}

testA_recommendsMatchUpstreamRegisterReadPoll();
testB_correlatedAndFakePoolSubmit();
testC_missingUpstreamBlocks();
testD_reportDoesNotLeakSentinelsOrHexDumps();
testE_neverRecommendsPostMaxBaudDelay();

console.log("phase28.1.1.2 result-path comparator tests passed");
