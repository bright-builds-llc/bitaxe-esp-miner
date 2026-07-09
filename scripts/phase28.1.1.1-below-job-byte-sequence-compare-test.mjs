#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const scriptPath = path.join(
  process.cwd(),
  "scripts/phase28.1.1.1-below-job-byte-sequence-compare.mjs",
);

const versionMaskFrame = "55 aa 51 09 00 a4 90 00 ff ff 1c";
const chipIdReadFrame = "55 aa 52 05 00 00 0a";
const rxFrame = "aa 55 00 00 00 00 00 00 00 00 00";
const chainInactiveFrame = "55 aa 53 05 00 00 03";
const setAddressFrame = "55 aa 40 05 00 00 1c";
const frequencyFrame = "55 aa 51 09 00 08 40 c2 01 11 00";
const nonceSpaceFrame = "55 aa 51 09 00 10 00 95 02 f9 00";
const maxBaudFrame = "55 aa 51 09 00 28 11 30 02 00 03";
const jobFrame = `55 aa 21 56 ${"00 ".repeat(82)}00 00`;

function runComparator({ upstream, rust }) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "phase28-sequence-"));
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

  assert.equal(result.status, 0, result.stderr);
  return {
    report: fs.readFileSync(outPath, "utf8"),
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

function baseUpstreamLog() {
  return [
    "Starting ASIC initialization (cold boot mode)",
    "Performing full UART initialization",
    `tx: [${versionMaskFrame}]`,
    `tx: [${versionMaskFrame}]`,
    `tx: [${versionMaskFrame}]`,
    `tx: [${chipIdReadFrame}]`,
    `rx: [${rxFrame}]`,
    `tx: [${chainInactiveFrame}]`,
    `tx: [${setAddressFrame}]`,
    `tx: [${frequencyFrame}]`,
    `tx: [${nonceSpaceFrame}]`,
    "Setting max baud of 1000000",
    `tx: [${maxBaudFrame}]`,
    "SERIAL_clear_buffer",
    `tx: [${jobFrame}]`,
    "asic_production_trace=result_read_attempt",
    "asic_production_status=result_correlated",
    "mining.submit",
  ].join("\n");
}

function baseRustLog() {
  return [
    "asic_reset_status=post_bring_up_pulse",
    "asic_work_result_trace=init_action kind=use_default_baud baud=115200",
    `asic_uart_trace=tx len=11 hex=${versionMaskFrame}`,
    `asic_uart_trace=tx len=11 hex=${versionMaskFrame}`,
    `asic_uart_trace=tx len=11 hex=${versionMaskFrame}`,
    `asic_uart_trace=tx len=7 hex=${chipIdReadFrame}`,
    `asic_uart_trace=rx_complete read_count=1 hex=${rxFrame}`,
    `asic_uart_trace=tx len=7 hex=${chainInactiveFrame}`,
    `asic_uart_trace=tx len=7 hex=${setAddressFrame}`,
    `asic_uart_trace=tx len=11 hex=${frequencyFrame}`,
    `asic_uart_trace=tx len=11 hex=${nonceSpaceFrame}`,
    `asic_uart_trace=tx len=11 hex=${maxBaudFrame}`,
    "asic_work_result_trace=init_action kind=use_max_baud baud=1000000",
    "asic_work_result_trace=init_action kind=clear_rx",
    `asic_uart_trace=tx len=88 hex=${jobFrame}`,
    "asic_production_trace=result_read_attempt poll_timeout_ms=1000",
    "asic_production_status=result_correlated",
    "share_submission_status=accepted",
  ].join("\n");
}

function matchingSequenceReportsMatch() {
  // Arrange
  const upstream = baseUpstreamLog();
  const rust = baseRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /comparison_status: match/);
  assert.match(report, /mismatched_events: none/);
  assert.match(report, /raw_bytes_committed: false/);
}

function missingMaxBaudClearReportsMismatch() {
  // Arrange
  const upstream = baseUpstreamLog();
  const rust = baseRustLog()
    .replace(`asic_uart_trace=tx len=11 hex=${maxBaudFrame}\n`, "")
    .replace("asic_work_result_trace=init_action kind=clear_rx\n", "");

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /comparison_status: mismatch/);
  assert.match(report, /mismatched_events: max_baud_tx,clear_rx/);
  assert.match(report, /recommended_investigation: post_max_baud_delay_2000/);
}

function missingResultReadReportsSingleDispatchKnob() {
  // Arrange
  const upstream = baseUpstreamLog();
  const rust = baseRustLog().replace("asic_production_trace=result_read_attempt poll_timeout_ms=1000\n", "");

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /comparison_status: mismatch/);
  assert.match(report, /mismatched_events: result_read_attempt/);
  assert.match(report, /recommended_investigation: single_dispatch_bounded_read/);
}

function missingLogBlocks() {
  // Arrange
  const upstream = null;
  const rust = baseRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /comparison_status: blocked_safe_prerequisite/);
  assert.match(report, /blocked_reasons: upstream_log_missing,upstream_job_tx_missing/);
}

function outputDoesNotLeakSentinels() {
  // Arrange
  const upstream = `${baseUpstreamLog()}\npoolPassword=PHASE28_SENTINEL_PASSWORD`;
  const rust = `${baseRustLog()}\nssid=PHASE28_SENTINEL_SSID`;

  // Act
  const { report, stdout, stderr } = runComparator({ upstream, rust });
  const combined = `${report}\n${stdout}\n${stderr}`;

  // Assert
  assert(!combined.includes("PHASE28_SENTINEL_PASSWORD"));
  assert(!combined.includes("PHASE28_SENTINEL_SSID"));
}

matchingSequenceReportsMatch();
missingMaxBaudClearReportsMismatch();
missingResultReadReportsSingleDispatchKnob();
missingLogBlocks();
outputDoesNotLeakSentinels();

console.log("phase28.1.1.1 below-job-byte sequence comparator tests passed");
