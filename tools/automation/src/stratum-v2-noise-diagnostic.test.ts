import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import {
  parseNoiseDiagnosticArgs,
  runNoiseDiagnosticProcess,
} from "./stratum-v2-noise-diagnostic.js";
import { validateNoiseDiagnosticProjection } from "./stratum-v2-noise-diagnostic-validator.js";

const source = "a".repeat(40);

function exactArgs(): string[] {
  return [
    "--board", "205",
    "--port", "/dev/cu.test",
    "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
    "--private-root", "scratch/str005-noise-diagnostic/diagnostic-001",
    "--projection", "docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection.json",
    "--plan", "docs/parity/work-plans/20260826T210025Z-STR-005-NOISE-DIAGNOSTIC/PLAN.md",
    "--diagnostic-ordinal", "1",
    "--redact-evidence",
  ];
}

function acceptedProjection(): Record<string, unknown> {
  return {
    schema_version: "bitaxe-stratum-v2-noise-diagnostic-projection-v1",
    status: "accepted",
    board: 205,
    diagnostic_ordinal: 1,
    source_commit: source,
    reference_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64),
    terminal_category: "accepted",
    stages: {
      tcp_connected: true,
      act_one_created: true,
      act_one_sent: true,
      act_two_received: true,
      time_sampled: true,
      authenticated: true,
    },
    fixture: {
      listener_ready: true,
      connection_accepted: true,
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
  assert.equal(parsed.diagnosticOrdinal, 1);
  assert.equal(parsed.privateRoot, "scratch/str005-noise-diagnostic/diagnostic-001");
  assert.equal(parsed.redactEvidence, true);
  assert.throws(() => parseNoiseDiagnosticArgs("start", exactArgs().map(value =>
    value === "1" ? "2" : value)));
});

test("projection validator requires the complete authenticated and restored chain", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "str005-noise-validator-"));
  const candidate = path.join(root, "projection.json");
  const projection = acceptedProjection();
  await writeFile(candidate, JSON.stringify(projection));

  try {
    // Act / Assert
    await validateNoiseDiagnosticProjection(candidate, source);
    (projection["stages"] as Record<string, unknown>)["authenticated"] = false;
    await writeFile(candidate, JSON.stringify(projection));
    await assert.rejects(validateNoiseDiagnosticProjection(candidate, source));
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
