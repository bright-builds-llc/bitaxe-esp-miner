import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import {
  parseTcpPayloadDiagnosticArgs,
  runTcpPayloadDiagnosticProcess,
  tcpPayloadDiagnosticWorkspaceRoot,
} from "./stratum-v2-tcp-payload.js";
import { tcpPayloadDiagnosticValidatorArgs } from "./stratum-v2-tcp-payload-process.js";
import { validateTcpPayloadDiagnosticProjection } from "./stratum-v2-tcp-payload-validator.js";
import { tcpPayloadEvaluatorIdentity } from "./stratum-v2-tcp-payload-validator.js";
import {
  tcpPayloadStagesFromMonitor,
  tcpPayloadTimingsFromMonitor,
} from "./stratum-v2-tcp-payload-markers.js";

const source = "a".repeat(40);

function evaluatorWorkspace(): string {
  const maybeTestSrcdir = process.env["TEST_SRCDIR"];
  const maybeWorkspace = process.env["TEST_WORKSPACE"];
  if (maybeTestSrcdir !== undefined && maybeWorkspace !== undefined) {
    return path.join(maybeTestSrcdir, maybeWorkspace);
  }
  return tcpPayloadDiagnosticWorkspaceRoot();
}

function exactArgs(): string[] {
  return [
    "--board", "205",
    "--port", "/dev/cu.test",
    "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
    "--private-parent", "scratch/str005-tcp-payload/diagnostic-001",
    "--projection", "docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-001.json",
    "--plan", "docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md",
    "--diagnostic-ordinal", "1",
    "--capture-timeout-seconds", "360",
    "--redact-evidence",
  ];
}

async function acceptedProjection(): Promise<Record<string, unknown>> {
  const workspace = evaluatorWorkspace();
  return {
    schema_version: "bitaxe-stratum-v2-tcp-payload-projection-v1",
    status: "accepted",
    board: 205,
    diagnostic_ordinal: 1,
    source_commit: source,
    reference_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64),
    payload_sha256: "fdeab9acf3710362bd2658cdc9a29e8f9c757fcf9811603a8c447cd1d9151108",
    evaluator_sha256: await tcpPayloadEvaluatorIdentity(workspace),
    terminal_category: "accepted",
    stages: {
      monitor_armed: true,
      resolved: true,
      tcp_connected: true,
      payload_sent: true,
    },
    timings: {
      connect_ms: 5,
      write_ms: 1,
    },
    fixture: {
      listener_ready: true,
      connection_accepted: true,
      peer_matched: true,
      unexpected_peer_count: 0,
      payload_bytes_received: 64,
      payload_read_category: "complete",
      payload_digest_match: true,
      extra_bytes_received: 0,
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
    redaction_status: "passed",
  };
}

test("diagnostic parser admits only the first exact no-mining contract", () => {
  // Arrange / Act
  const parsed = parseTcpPayloadDiagnosticArgs("start", exactArgs());

  // Assert
  assert.equal(parsed.diagnosticOrdinal, 1);
  assert.equal(parsed.privateRoot, "scratch/str005-tcp-payload/diagnostic-001");
  assert.equal(parsed.redactEvidence, true);
  assert.throws(() => parseTcpPayloadDiagnosticArgs("start", exactArgs().map(value =>
    value === "1" ? "5" : value)));
});

test("projection validator requires the complete authenticated and restored chain", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "str005-noise-validator-"));
  const candidate = path.join(root, "projection.json");
  const projection = await acceptedProjection();
  const workspace = evaluatorWorkspace();
  await writeFile(candidate, JSON.stringify(projection));

  try {
    // Act / Assert
    await validateTcpPayloadDiagnosticProjection(candidate, source, 1, workspace);
    (projection["stages"] as Record<string, unknown>)["payload_sent"] = false;
    await writeFile(candidate, JSON.stringify(projection));
    await assert.rejects(validateTcpPayloadDiagnosticProjection(candidate, source, 1, workspace));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("diagnostic process owner terminates a timed-out real child", async () => {
  // Arrange
  const child = ["-e", "setInterval(() => {}, 1000)"];

  // Act / Assert
  await assert.rejects(
    runTcpPayloadDiagnosticProcess(process.cwd(), process.execPath, child, 50, "real_child"),
    (error: unknown) => error instanceof Error && error.message === "timeout:real_child",
  );
});

test("diagnostic marker parser retains only bounded stages and timings", () => {
  // Arrange
  const monitor = [
    'stratum_v2_tcp_payload={"stage":"monitor_armed"}',
    'stratum_v2_tcp_payload={"stage":"resolved"}',
    'stratum_v2_tcp_payload={"stage":"tcp_connected"}',
    'stratum_v2_tcp_payload={"stage":"payload_sent"}',
    'stratum_v2_tcp_payload_timing={"phase":"connect","duration_ms":5}',
    'stratum_v2_tcp_payload_timing={"phase":"write","duration_ms":1}',
  ].join("\n");

  // Act
  const stages = tcpPayloadStagesFromMonitor(monitor);
  const timings = tcpPayloadTimingsFromMonitor(monitor);

  // Assert
  assert.equal(stages["monitor_armed"], true);
  assert.equal(stages["tcp_connected"], true);
  assert.equal(timings["write_ms"], 1);
});

test("diagnostic projection routes independent validation through Bazel", () => {
  // Arrange / Act
  const args = tcpPayloadDiagnosticValidatorArgs("/private/candidate.json", source, 1);

  // Assert
  assert.deepEqual(args, [
    "run",
    "//tools/automation:stratum_v2_tcp_payload_validator",
    "--",
    "/private/candidate.json",
    source,
    "1",
  ]);
});
