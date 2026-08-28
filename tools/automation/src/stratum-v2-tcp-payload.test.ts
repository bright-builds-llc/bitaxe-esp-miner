import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import {
  parseTcpPayloadDiagnosticArgs,
  runTcpPayloadDiagnosticProcess,
  shouldWaitForTcpFixture,
  tcpPayloadDiagnosticWorkspaceRoot,
} from "./stratum-v2-tcp-payload.js";
import { tcpPayloadFixtureArgs } from "./stratum-v2-tcp-fixture.js";
import {
  tcpPayloadDiagnosticAccepted,
  tcpPayloadDiagnosticValidatorArgs,
} from "./stratum-v2-tcp-payload-process.js";
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
    "--private-parent", "scratch/str005-tcp-payload/diagnostic-007",
    "--projection", "docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-007.json",
    "--plan", "docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md",
    "--diagnostic-ordinal", "7",
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
    diagnostic_ordinal: 7,
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
      write_half_closed: true,
      receipt_acknowledged: true,
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
      receipt_ack_sent: true,
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
  assert.equal(parsed.diagnosticOrdinal, 7);
  assert.equal(parsed.privateRoot, "scratch/str005-tcp-payload/diagnostic-007");
  assert.equal(parsed.redactEvidence, true);
  assert.throws(() => parseTcpPayloadDiagnosticArgs("start", exactArgs().map(value =>
    value === "7" ? "8" : value)));
});

test("fixture owner uses an admitted session timeout below the capture timeout", () => {
  // Arrange / Act
  const args = tcpPayloadFixtureArgs("/private/fixture", "192.0.2.1", "192.0.2.2");
  const sessionIndex = args.indexOf("--session-timeout-seconds");

  // Assert
  assert.notEqual(sessionIndex, -1);
  assert.equal(args[sessionIndex + 1], "120");
  assert(Number(args[sessionIndex + 1]) <= 300);
});

test("recovery parser admits only the fresh recovery root", () => {
  // Arrange
  const values = exactArgs().map(value => value === "scratch/str005-tcp-payload/diagnostic-007"
    ? "scratch/str005-tcp-payload/recovery-002"
    : value);

  // Act
  const parsed = parseTcpPayloadDiagnosticArgs("recover", values);

  // Assert
  assert.equal(parsed.privateRoot, "scratch/str005-tcp-payload/recovery-002");
  assert.throws(() => parseTcpPayloadDiagnosticArgs("recover", exactArgs()));
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
    await validateTcpPayloadDiagnosticProjection(candidate, source, 7, workspace);
    (projection["stages"] as Record<string, unknown>)["payload_sent"] = false;
    await writeFile(candidate, JSON.stringify(projection));
    await assert.rejects(validateTcpPayloadDiagnosticProjection(candidate, source, 7, workspace));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("diagnostic process owner terminates a timed-out real child", async () => {
  // Arrange
  const child = [
    "-e",
    "process.stdout.write('partial-stage'); process.stderr.write('partial-error'); setInterval(() => {}, 1000)",
  ];

  // Act / Assert
  await assert.rejects(
    runTcpPayloadDiagnosticProcess(process.cwd(), process.execPath, child, 500, "real_child"),
    (error: unknown) => error instanceof Error
      && error.message === "timeout:real_child"
      && (error as { stdout?: string }).stdout === "partial-stage"
      && (error as { stderr?: string }).stderr === "partial-error",
  );
});

test("diagnostic marker parser retains only bounded stages and timings", () => {
  // Arrange
  const monitor = [
    'stratum_v2_tcp_payload={"stage":"monitor_armed"}',
    'stratum_v2_tcp_payload={"stage":"resolved"}',
    'stratum_v2_tcp_payload={"stage":"tcp_connected"}',
    'stratum_v2_tcp_payload={"stage":"payload_sent"}',
    'stratum_v2_tcp_payload={"stage":"write_half_closed"}',
    'stratum_v2_tcp_payload={"stage":"receipt_acknowledged"}',
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

test("pre-monitor child failure stops before the fixture accept deadline", () => {
  // Arrange / Act / Assert
  assert.equal(shouldWaitForTcpFixture(1, { category: "terminal_missing" }), false);
  assert.equal(shouldWaitForTcpFixture(0, { category: "terminal_missing" }), true);
  assert.equal(shouldWaitForTcpFixture(1, { category: "connect" }), true);
});

test("accepted fixture and firmware evidence survives the bounded monitor timeout", () => {
  // Arrange
  const terminal = { accepted: true };
  const fixture = { status: "accepted", terminal_category: "accepted" };

  // Act / Assert
  assert.equal(tcpPayloadDiagnosticAccepted(1, true, terminal, fixture), true);
  assert.equal(tcpPayloadDiagnosticAccepted(1, false, terminal, fixture), false);
  assert.equal(tcpPayloadDiagnosticAccepted(1, true, { accepted: false }, fixture), false);
});

test("diagnostic projection routes independent validation through Bazel", () => {
  // Arrange / Act
  const args = tcpPayloadDiagnosticValidatorArgs("/private/candidate.json", source, 7);

  // Assert
  assert.deepEqual(args, [
    "run",
    "//tools/automation:stratum_v2_tcp_payload_validator",
    "--",
    "/private/candidate.json",
    source,
    "7",
  ]);
});
