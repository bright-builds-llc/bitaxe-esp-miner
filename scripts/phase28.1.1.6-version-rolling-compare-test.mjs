#!/usr/bin/env node
/**
 * Phase 28.1.1.6 version-rolling comparator fixture tests.
 *
 * Covers D-07 fields, forced_ab_label, closed recommender, HARD BAN,
 * asic_mask_reload_recommended ban, power_delta_class, and CFG-07 redaction.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import {
  HARD_BAN,
  parsePowerDeltaClass,
  recommendVersionRollingInvestigation,
  forcedAbLabel,
} from "./phase28.1.1.6-version-rolling-compare.mjs";

const scriptPath = path.join(
  process.cwd(),
  "scripts/phase28.1.1.6-version-rolling-compare.mjs",
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
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "phase28.1.1.6-vr-"));
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

/** Upstream: configure + mask + mining-ready with version-rolling. */
function baselineUpstreamLog() {
  return [
    "Starting ASIC initialization (cold boot mode)",
    "mining.configure version-rolling",
    "version-rolling.mask=1fffe000",
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

/**
 * Expected Ultra 205 diverge: configure + mask stored, mask not applied,
 * falling power, no correlate.
 */
function divergeRustLog() {
  return [
    "asic_reset_status=post_bring_up_pulse",
    "asic_enable_status=active",
    "Set ASIC voltage to 1200 mV",
    "live_stratum mining.configure version-rolling",
    "version-rolling.mask=1fffe000",
    "version_mask_stored=true",
    "mask_stored_class=stored",
    "mask_value_class=default_1fffe000",
    "with_version_mask MiningWorkBuilder",
    "mask_applied_to_work=false",
    "let _ = maybe_version_mask",
    "job_version_field_class=base_notify",
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${VERSION_MASK_FRAME}`,
    `asic_uart_trace=tx len=7 hex=${READ_CHIP_ID_FRAME}`,
    `asic_uart_trace=rx_complete read_count=11 hex=${CHIP_ID_RX_FRAME}`,
    "asic_chip_enumerate_summary chip_count_source=counted_rx chip_count=1 address_interval=256 gap=drain_idle_like chip_detected=true",
    "wire_parity_ticket_mask_retained=true",
    "wire_parity_rx_loop_retained=true",
    "mining_ready_init_started",
    `asic_uart_trace=tx len=7 hex=${SET_CHIP_ADDRESS_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${DIFF_256_FRAME}`,
    `asic_uart_trace=tx len=11 hex=${FREQ_485}`,
    `asic_uart_trace=tx len=11 hex=${NONCE_485}`,
    `asic_uart_trace=tx len=88 hex=${JOB_FRAME}`,
    "asic_probe=power_delta baseline_mw=5000 after_mw=4610 delta_mw=-390",
  ].join("\n");
}

/** Mask applied to work, still no correlate → recommend ASIC reload (not forced A/B). */
function maskAppliedRustLog() {
  return divergeRustLog()
    .replace("mask_applied_to_work=false", "mask_applied_to_work=true")
    .replace("let _ = maybe_version_mask", "maybe_version_mask_attached version_mask_on_work=true");
}

function testA_ultra205DivergeForcesWorkFieldParity() {
  // Arrange
  const upstream = baselineUpstreamLog();
  const rust = divergeRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /configure_observed: true/);
  assert.match(report, /mask_stored_class: stored/);
  assert.match(report, /mask_applied_to_work: false/);
  assert.match(report, /mask_value_class: default_1fffe000/);
  assert.match(report, /power_delta_class: falling/);
  assert.match(report, /result_correlated: false/);
  assert.match(report, /fake_pool_submit_observed: false/);
  assert.match(
    report,
    /forced_ab_label: negotiated_version_mask_work_field_parity/,
  );
  assert.match(
    report,
    /recommended_investigation: negotiated_version_mask_work_field_parity/,
  );
  assert.match(report, /asic_mask_reload_recommended: false/);
  assert.match(report, /raw_bytes_committed: false/);
  assert.match(report, /credential_contents_read: false/);
  assert.match(report, /phase30_promotion_input: pending/);
  assert.match(report, /wire_parity_ticket_mask_retained: true/);
  assert.match(report, /wire_parity_rx_loop_retained: true/);
  assert.match(report, /job_version_field_class: base_notify/);
}

function testB_maskAppliedRecommendsAsicReloadNotForcedAb() {
  // Arrange
  const upstream = baselineUpstreamLog();
  const rust = maskAppliedRustLog();

  // Act
  const { report } = runComparator({ upstream, rust });

  // Assert
  assert.match(report, /mask_applied_to_work: true/);
  assert.match(report, /result_correlated: false/);
  assert.match(
    report,
    /recommended_investigation: pool_negotiated_mask_asic_reload/,
  );
  assert.match(report, /forced_ab_label: none/);
  assert.match(report, /asic_mask_reload_recommended: false/);
  // Never force ASIC reload as A/B this wave
  assert.doesNotMatch(
    report,
    /forced_ab_label: pool_negotiated_mask_asic_reload/,
  );
}

function testC_missingLogBlocks() {
  // Arrange / Act
  const { report } = runComparator({ upstream: null, rust: divergeRustLog() });

  // Assert
  assert.match(report, /comparison_status: blocked_safe_prerequisite/);
  assert.match(report, /recommended_investigation: none/);
  assert.match(report, /forced_ab_label: none/);
  assert.match(report, /asic_mask_reload_recommended: false/);
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
    upstream: baselineUpstreamLog(),
    rust: risingRust,
  });
  assert.match(risingReport, /power_delta_class: rising_hashing/);

  // CLI path: unavailable
  const unavailableRust = divergeRustLog().replace(
    /asic_probe=power_delta[^\n]*/,
    "asic_probe=power_delta unavailable=true",
  );
  const { report: unavailableReport } = runComparator({
    upstream: baselineUpstreamLog(),
    rust: unavailableRust,
  });
  assert.match(unavailableReport, /power_delta_class: unavailable/);
}

function testE_neverRecommendsFalsifiedKnobs() {
  // Arrange — gap fixtures that previously tempted falsified levers
  const upstream = baselineUpstreamLog();
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
  assert.ok(selfSource.includes("count_asic_chips_rx_loop_parity"));

  // Direct recommender unit checks
  assert.equal(
    recommendVersionRollingInvestigation({
      blocked: false,
      maskStored: true,
      maskApplied: false,
      resultCorrelated: false,
    }),
    "negotiated_version_mask_work_field_parity",
  );
  assert.equal(
    recommendVersionRollingInvestigation({
      blocked: false,
      maskStored: true,
      maskApplied: true,
      resultCorrelated: false,
    }),
    "pool_negotiated_mask_asic_reload",
  );
  assert.equal(
    recommendVersionRollingInvestigation({
      blocked: true,
      maskStored: true,
      maskApplied: false,
      resultCorrelated: false,
    }),
    "none",
  );
  assert.ok(
    !HARD_BAN.has(
      recommendVersionRollingInvestigation({
        blocked: false,
        maskStored: true,
        maskApplied: false,
        resultCorrelated: false,
      }),
    ),
  );

  assert.equal(
    forcedAbLabel({
      blocked: false,
      configure: true,
      maskStored: true,
      maskApplied: false,
      resultCorrelated: false,
    }),
    "negotiated_version_mask_work_field_parity",
  );
  assert.equal(
    forcedAbLabel({
      blocked: false,
      configure: true,
      maskStored: true,
      maskApplied: true,
      resultCorrelated: false,
    }),
    "none",
  );
}

function testF_reportDoesNotLeakHexOrCredentials() {
  // Arrange
  const upstream = `${baselineUpstreamLog()}\npoolPassword=PHASE28_SENTINEL_PASSWORD\npoolURL=stratum+tcp://pool.example:3333`;
  const rust = `${divergeRustLog()}\nssid=PHASE28_SENTINEL_SSID`;

  // Act
  const { report, stdout, stderr } = runComparator({ upstream, rust });
  const combined = `${report}\n${stdout}\n${stderr}`;

  // Assert — CFG-07 / T-28116-01
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

function testG_asicMaskReloadRecommendedAlwaysFalse() {
  // Arrange / Act
  const diverge = runComparator({
    upstream: baselineUpstreamLog(),
    rust: divergeRustLog(),
  });
  const applied = runComparator({
    upstream: baselineUpstreamLog(),
    rust: maskAppliedRustLog(),
  });

  // Assert — D-05/D-14
  assert.match(diverge.report, /asic_mask_reload_recommended: false/);
  assert.match(applied.report, /asic_mask_reload_recommended: false/);
  assert.doesNotMatch(diverge.report, /asic_mask_reload_recommended: true/);
  assert.doesNotMatch(applied.report, /asic_mask_reload_recommended: true/);
}

testA_ultra205DivergeForcesWorkFieldParity();
testB_maskAppliedRecommendsAsicReloadNotForcedAb();
testC_missingLogBlocks();
testD_powerDeltaClasses();
testE_neverRecommendsFalsifiedKnobs();
testF_reportDoesNotLeakHexOrCredentials();
testG_asicMaskReloadRecommendedAlwaysFalse();

console.log("phase28.1.1.6 version-rolling comparator tests passed");
