import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import {
  parseNoiseDiagnosticArgs,
  runNoiseDiagnosticProcess,
} from "./stratum-v2-noise-diagnostic.js";
import { noiseDiagnosticValidatorArgs } from "./stratum-v2-noise-diagnostic-process.js";
import { validateNoiseDiagnosticProjection } from "./stratum-v2-noise-diagnostic-validator.js";
import {
  noiseStagesFromMonitor,
  noiseTimingsFromMonitor,
} from "./stratum-v2-noise-diagnostic-markers.js";

const source = "a".repeat(40);

function exactArgs(): string[] {
  return [
    "--board", "205",
    "--port", "/dev/cu.test",
    "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
    "--private-root", "scratch/str005-noise-diagnostic/diagnostic-004",
    "--projection", "docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-004.json",
    "--plan", "docs/parity/work-plans/20260828T030951Z-STR-005-PRECONNECT-NOISE-VERIFY/PLAN.md",
    "--diagnostic-ordinal", "4",
    "--redact-evidence",
  ];
}

function acceptedProjection(): Record<string, unknown> {
  return {
    schema_version: "bitaxe-stratum-v2-noise-diagnostic-projection-v1",
    status: "accepted",
    board: 205,
    diagnostic_ordinal: 4,
    source_commit: source,
    reference_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64),
    terminal_category: "accepted",
    stages: {
      noise_prepared: true,
      tcp_connected: true,
      act_one_created: true,
      act_one_sent: true,
      act_two_received: true,
      time_sampled: true,
      authenticated: true,
    },
    timings: {
      keypair_preparation_ms: 120,
      act_one_construction_ms: 30,
      connect_ms: 5,
      act_one_write_ms: 1,
      act_two_read_ms: 4,
    },
    fixture: {
      listener_ready: true,
      connection_accepted: true,
      peer_matched: true,
      unexpected_peer_count: 0,
      act_one_bytes_received: 64,
      act_one_read_category: "complete",
      accept_to_first_byte_millis: 2,
      act_one_read_millis: 3,
      act_one_received: true,
      responder_created: true,
      act_two_created: true,
      act_two_sent: true,
      client_authenticated: true,
    },
    campaign_started: false,
    mining_started: false,
    asic_touched: false,
    fan_touched: false,
    voltage_touched: false,
    restoration: {
      identity_exact: true,
      settings_exact: true,
      mineonboot_disabled: true,
      mining_inactive: true,
      zero_work: true,
      usb_cleanup_complete: true,
      owned_processes_remaining: 0,
    },
    redaction_complete: true,
  };
}

test("diagnostic parser admits only the first exact no-mining contract", () => {
  // Arrange / Act
  const parsed = parseNoiseDiagnosticArgs("start", exactArgs());

  // Assert
  assert.equal(parsed.diagnosticOrdinal, 4);
  assert.equal(parsed.privateRoot, "scratch/str005-noise-diagnostic/diagnostic-004");
  assert.equal(parsed.redactEvidence, true);
  assert.throws(() => parseNoiseDiagnosticArgs("start", exactArgs().map(value =>
    value === "4" ? "5" : value)));
});

test("projection validator requires the complete authenticated and restored chain", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "str005-noise-validator-"));
  const candidate = path.join(root, "projection.json");
  const projection = acceptedProjection();
  await writeFile(candidate, JSON.stringify(projection));

  try {
    // Act / Assert
    await validateNoiseDiagnosticProjection(candidate, source, 4);
    (projection["stages"] as Record<string, unknown>)["authenticated"] = false;
    await writeFile(candidate, JSON.stringify(projection));
    await assert.rejects(validateNoiseDiagnosticProjection(candidate, source, 4));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("diagnostic process owner terminates a timed-out real child", async () => {
  // Arrange
  const child = ["-e", "setInterval(() => {}, 1000)"];

  // Act / Assert
  await assert.rejects(
    runNoiseDiagnosticProcess(process.cwd(), process.execPath, child, 50, "real_child"),
    (error: unknown) => error instanceof Error && error.message === "timeout:real_child",
  );
});

test("diagnostic marker parser retains only bounded stages and timings", () => {
  // Arrange
  const monitor = [
    'stratum_v2_noise_diagnostic={"stage":"noise_prepared"}',
    'stratum_v2_noise_diagnostic={"stage":"tcp_connected"}',
    'stratum_v2_noise_timing={"phase":"keypair_preparation","duration_ms":120}',
    'stratum_v2_noise_timing={"phase":"act_one_construction","duration_ms":30}',
    'stratum_v2_noise_timing={"phase":"connect","duration_ms":5}',
    'stratum_v2_noise_timing={"phase":"act_one_write","duration_ms":1}',
    'stratum_v2_noise_timing={"phase":"act_two_read","duration_ms":4}',
  ].join("\n");

  // Act
  const stages = noiseStagesFromMonitor(monitor);
  const timings = noiseTimingsFromMonitor(monitor);

  // Assert
  assert.equal(stages["noise_prepared"], true);
  assert.equal(stages["tcp_connected"], true);
  assert.equal(timings["keypair_preparation_ms"], 120);
  assert.equal(timings["act_one_construction_ms"], 30);
  assert.equal(timings["act_two_read_ms"], 4);
});

test("diagnostic projection routes independent validation through Bazel", () => {
  // Arrange / Act
  const args = noiseDiagnosticValidatorArgs("/private/candidate.json", source, 4);

  // Assert
  assert.deepEqual(args, [
    "run",
    "//tools/automation:stratum_v2_noise_diagnostic_validator",
    "--",
    "/private/candidate.json",
    source,
    "4",
  ]);
});
