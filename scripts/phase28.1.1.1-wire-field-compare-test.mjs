#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const comparator = path.join(repoRoot, "scripts/phase28.1.1.1-wire-field-compare.mjs");

function buildFrame({ ntimeByte = 0x31 } = {}) {
  const payload = [
    0x28,
    0x01,
    0x00,
    0x00,
    0x00,
    0x00,
    0x11,
    0x12,
    0x13,
    0x14,
    ntimeByte,
    0x32,
    0x33,
    0x34,
    ...Array(32).fill(0x45),
    ...Array(32).fill(0x67),
    0x21,
    0x22,
    0x23,
    0x24,
  ];
  assert.equal(payload.length, 82);

  const frame = [0x55, 0xaa, 0x21, 0x56, ...payload, 0x00, 0x00];
  assert.equal(frame.length, 88);
  return frame.map((byte) => byte.toString(16).padStart(2, "0")).join(" ");
}

function writeSourceWorkReport(caseDir, label, fingerprint = "a".repeat(64)) {
  const reportPath = path.join(caseDir, `${label}-source-work.json`);
  fs.writeFileSync(
    reportPath,
    `${JSON.stringify({
      status: "stopped",
      source_work_fingerprint: fingerprint,
      raw_messages_committed: false,
      credential_contents_read: false,
    })}\n`,
    "utf8",
  );
  return reportPath;
}

function runComparator(tempDir, upstreamText, rustText, sourceWork = null) {
  const caseDir = fs.mkdtempSync(path.join(tempDir, "case-"));
  const upstreamPath = path.join(caseDir, "upstream.log");
  const rustPath = path.join(caseDir, "rust.log");
  const outPath = path.join(caseDir, "report.md");

  if (upstreamText !== null) fs.writeFileSync(upstreamPath, upstreamText, "utf8");
  if (rustText !== null) fs.writeFileSync(rustPath, rustText, "utf8");

  const args = [comparator, "--upstream", upstreamPath, "--rust", rustPath, "--out", outPath];
  if (sourceWork) {
    args.push(
      "--upstream-source-work",
      writeSourceWorkReport(caseDir, "upstream", sourceWork.upstreamFingerprint),
      "--rust-source-work",
      writeSourceWorkReport(caseDir, "rust", sourceWork.rustFingerprint),
    );
  }

  const result = spawnSync(process.execPath, args, { cwd: repoRoot, encoding: "utf8" });

  assert.equal(result.status, 0, result.stderr);
  return fs.readFileSync(outPath, "utf8");
}

const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "phase28-wire-field-"));

try {
  const matchingFrame = buildFrame();
  const matchReport = runComparator(
    tempDir,
    `tx: [${matchingFrame.toUpperCase()}]\n`,
    `asic_uart_trace=tx len=88 hex=${matchingFrame}\n`,
  );
  assert.match(matchReport, /comparison_status: match/);
  assert.match(matchReport, /mismatched_fields: none/);
  assert.match(matchReport, /comparison_pair: best_field_match/);
  assert.match(matchReport, /source_work_alignment_status: not_proven/);

  const mismatchReport = runComparator(
    tempDir,
    `tx: [${matchingFrame.toUpperCase()}]\n`,
    `asic_uart_trace=tx len=88 hex=${buildFrame({ ntimeByte: 0x99 })}\n`,
  );
  assert.match(mismatchReport, /comparison_status: mismatch_unconfirmed_dynamic_work_fields/);
  assert.match(mismatchReport, /mismatched_fields: ntime/);

  const alignedMismatchReport = runComparator(
    tempDir,
    `tx: [${matchingFrame.toUpperCase()}]\n`,
    `asic_uart_trace=tx len=88 hex=${buildFrame({ ntimeByte: 0x99 })}\n`,
    { upstreamFingerprint: "b".repeat(64), rustFingerprint: "b".repeat(64) },
  );
  assert.match(alignedMismatchReport, /comparison_status: mismatch/);
  assert.match(alignedMismatchReport, /source_work_alignment_status: matched/);
  assert.match(alignedMismatchReport, /mismatched_fields: ntime/);

  const sourceMismatchReport = runComparator(
    tempDir,
    `tx: [${matchingFrame.toUpperCase()}]\n`,
    `asic_uart_trace=tx len=88 hex=${matchingFrame}\n`,
    { upstreamFingerprint: "c".repeat(64), rustFingerprint: "d".repeat(64) },
  );
  assert.match(sourceMismatchReport, /comparison_status: blocked_safe_prerequisite/);
  assert.match(sourceMismatchReport, /source_work_alignment_status: mismatch/);
  assert.match(sourceMismatchReport, /blocked_reasons: source_work_fingerprint_mismatch/);

  const distractorFrame = buildFrame({ ntimeByte: 0x88 });
  const bestPairReport = runComparator(
    tempDir,
    `tx: [${distractorFrame.toUpperCase()}]\ntx: [${matchingFrame.toUpperCase()}]\n`,
    `asic_uart_trace=tx len=88 hex=${matchingFrame}\n`,
  );
  assert.match(bestPairReport, /comparison_status: match/);
  assert.match(bestPairReport, /upstream_frame_index: 1/);
  assert.match(bestPairReport, /rust_frame_index: 0/);

  const blockedReport = runComparator(
    tempDir,
    null,
    `asic_uart_trace=tx len=88 hex=${matchingFrame}\n`,
  );
  assert.match(blockedReport, /comparison_status: blocked_safe_prerequisite/);
  assert.match(blockedReport, /blocked_reasons: upstream_log_missing/);

  console.log("phase28.1.1.1 wire field comparator tests passed");
} finally {
  fs.rmSync(tempDir, { recursive: true, force: true });
}
