import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
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
  tcpPayloadSocketErrorsFromMonitor,
  tcpPayloadStagesFromMonitor,
  tcpPayloadTimingsFromMonitor,
} from "./stratum-v2-tcp-payload-markers.js";
import {
  TcpPayloadRecoveryToolingError,
  validateTcpPayloadRecoveryTooling,
} from "./stratum-v2-tcp-recovery-tooling.js";
import {
  projectTcpPayloadConnection,
  tcpPayloadPrivateLocalPortFromMonitor,
} from "./stratum-v2-tcp-connection.js";
import { admitTcpPayloadRestorePreflight } from "./stratum-v2-tcp-restore-preflight.js";
import type { RestoreBundle } from "./stratum-v2-restore-model.js";

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
    "--private-parent", "scratch/str005-tcp-payload/diagnostic-009",
    "--projection", "docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-009.json",
    "--plan", "docs/parity/work-plans/20260829T032813Z-STR-005-CONNECTION-IDENTITY/PLAN.md",
    "--diagnostic-ordinal", "9",
    "--capture-timeout-seconds", "360",
    "--redact-evidence",
  ];
}

async function acceptedProjection(): Promise<Record<string, unknown>> {
  const workspace = evaluatorWorkspace();
  return {
    schema_version: "bitaxe-stratum-v2-tcp-payload-projection-v2",
    status: "accepted",
    board: 205,
    diagnostic_ordinal: 9,
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
    connection: {
      tuple_match: true,
      local_marker_consistent: true,
      exact_peer_connection_count: 1,
      other_exact_peer_connection_count: 0,
      candidate_overflow: false,
      correlated_candidate_found: true,
    },
    send: {
      adapter: "std",
      reported_bytes: 64,
      pre_send_error: "none",
      post_send_error: "none",
      post_shutdown_error: "none",
      category: "accepted",
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
  assert.equal(parsed.diagnosticOrdinal, 9);
  assert.equal(parsed.privateRoot, "scratch/str005-tcp-payload/diagnostic-009");
  assert.equal(parsed.redactEvidence, true);
  assert.throws(() => parseTcpPayloadDiagnosticArgs("start", exactArgs().map(value =>
    value === "9" ? "10" : value)));
});

test("finalize parser reuses only the exact diagnostic-009 evidence contract", () => {
  // Arrange / Act
  const parsed = parseTcpPayloadDiagnosticArgs("finalize", exactArgs());

  // Assert
  assert.equal(parsed.action, "finalize");
  assert.equal(parsed.privateRoot, "scratch/str005-tcp-payload/diagnostic-009");
  assert.equal(parsed.diagnosticOrdinal, 9);
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

test("recovery tooling preflight admits contained esptool and usable NVS Python", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "str005-restore-tools-"));
  const esptool = path.join(
    workspace,
    ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
  );
  const nvsPython = path.join(
    workspace,
    ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/python",
  );
  await mkdir(path.dirname(esptool), { recursive: true });
  await mkdir(path.dirname(nvsPython), { recursive: true });
  await writeFile(esptool, "#!/usr/bin/env python3\n", { mode: 0o755 });
  await writeFile(nvsPython, "#!/usr/bin/env python3\n", { mode: 0o755 });
  await chmod(esptool, 0o755);
  await chmod(nvsPython, 0o755);
  const runProcess = async () => ({ exitCode: 0, stdout: "", stderr: "" });

  try {
    // Act / Assert
    await assert.doesNotReject(validateTcpPayloadRecoveryTooling(workspace, runProcess));
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("recovery tooling preflight rejects an out-of-workspace esptool symlink", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "str005-restore-link-"));
  const external = path.join(await mkdtemp(path.join(tmpdir(), "str005-external-tool-")), "esptool.py");
  const esptool = path.join(
    workspace,
    ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
  );
  const nvsPython = path.join(
    workspace,
    ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/python",
  );
  await mkdir(path.dirname(esptool), { recursive: true });
  await mkdir(path.dirname(nvsPython), { recursive: true });
  await writeFile(external, "tool", { mode: 0o755 });
  await symlink(external, esptool);
  await writeFile(nvsPython, "python", { mode: 0o755 });

  try {
    // Act / Assert
    await assert.rejects(
      validateTcpPayloadRecoveryTooling(
        workspace,
        async () => ({ exitCode: 0, stdout: "", stderr: "" }),
      ),
      (error: unknown) => error instanceof TcpPayloadRecoveryToolingError
        && error.checkpoint === "restore_esptool",
    );
  } finally {
    await rm(workspace, { recursive: true, force: true });
    await rm(path.dirname(external), { recursive: true, force: true });
  }
});

test("recovery tooling preflight rejects a failed NVS module import", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "str005-restore-import-"));
  const esptool = path.join(
    workspace,
    ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
  );
  const nvsPython = path.join(
    workspace,
    ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/python",
  );
  await mkdir(path.dirname(esptool), { recursive: true });
  await mkdir(path.dirname(nvsPython), { recursive: true });
  await writeFile(esptool, "tool", { mode: 0o755 });
  await writeFile(nvsPython, "python", { mode: 0o755 });

  try {
    // Act / Assert
    await assert.rejects(
      validateTcpPayloadRecoveryTooling(
        workspace,
        async () => ({ exitCode: 1, stdout: "", stderr: "import failed" }),
      ),
      (error: unknown) => error instanceof TcpPayloadRecoveryToolingError
        && error.checkpoint === "restore_nvs_python",
    );
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("restore admission preflight writes one reusable source-bound receipt", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "str005-restore-admission-"));
  const recoveryPlan = path.join(
    workspace,
    "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md",
  );
  const bundlePath = path.join(workspace, "scratch/recovery/restore-bundle.private.json");
  await mkdir(path.dirname(recoveryPlan), { recursive: true });
  await mkdir(path.dirname(bundlePath), { recursive: true });
  await writeFile(recoveryPlan, "recovery plan\n");
  await writeFile(bundlePath, "bundle\n", { mode: 0o600 });
  const calls: string[][] = [];
  const runProcess = async (
    _workspace: string,
    _program: string,
    args: readonly string[],
  ) => {
    calls.push([...args]);
    return { exitCode: 0, stdout: "restore_admission: ready", stderr: "" };
  };
  const restoreBundle = {
    capture_source_commit: source,
  } as unknown as RestoreBundle;
  const input = {
    workspace,
    flashProgram: "/private/flash",
    port: "/dev/cu.test",
    restoreBundleRelative: "scratch/recovery/restore-bundle.private.json",
    restoreBundlePath: bundlePath,
    restoreBundle,
    planRelative: "docs/private-plan.md",
    planSha256: "f".repeat(64),
    wifiCredentialsRelative: "wifi-credentials.json",
    sourceCommit: source,
    referenceCommit: "b".repeat(40),
    runProcess,
  };

  try {
    // Act
    await admitTcpPayloadRestorePreflight(input);
    await admitTcpPayloadRestorePreflight(input);

    // Assert
    assert.equal(calls.length, 1);
    assert(calls[0]?.includes("--admission-only"));
    assert(calls[0]?.includes("tcp_payload_restore_preflight") === false);
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});

test("recovery parser admits only the fresh recovery root", () => {
  // Arrange
  const values = exactArgs().map(value => value === "scratch/str005-tcp-payload/diagnostic-009"
    ? "scratch/str005-tcp-payload/recovery-003"
    : value);

  // Act
  const parsed = parseTcpPayloadDiagnosticArgs("recover", values);

  // Assert
  assert.equal(parsed.privateRoot, "scratch/str005-tcp-payload/recovery-003");
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
    await validateTcpPayloadDiagnosticProjection(candidate, source, 9, workspace);
    (projection["stages"] as Record<string, unknown>)["payload_sent"] = false;
    await writeFile(candidate, JSON.stringify(projection));
    await assert.rejects(validateTcpPayloadDiagnosticProjection(candidate, source, 9, workspace));
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
    'stratum_v2_tcp_socket_error={"phase":"pre_send","category":"none"}',
    'stratum_v2_tcp_socket_error={"phase":"post_send","category":"none"}',
    'stratum_v2_tcp_socket_error={"phase":"post_shutdown","category":"none"}',
  ].join("\n");

  // Act
  const stages = tcpPayloadStagesFromMonitor(monitor);
  const timings = tcpPayloadTimingsFromMonitor(monitor);
  const socketErrors = tcpPayloadSocketErrorsFromMonitor(monitor);

  // Assert
  assert.equal(stages["monitor_armed"], true);
  assert.equal(stages["tcp_connected"], true);
  assert.equal(timings["write_ms"], 1);
  assert.equal(socketErrors["post_shutdown"], "none");
});

test("private tuple join correlates duplicate firmware markers without publishing ports", () => {
  // Arrange
  const monitor = [
    'stratum_v2_tcp_connection_private={"local_port":49152}',
    'stratum_v2_tcp_connection_private={"local_port":49152}',
  ].join("\n");
  const privateFixture = {
    listener_ready: true,
    connection_accepted: true,
    peer_matched: true,
    unexpected_peer_count: 0,
    exact_peer_connection_count: 2,
    candidate_overflow: false,
    tcp_candidates: [
      { remote_port: 49151, payload_bytes_received: 0, payload_read_category: "timeout" },
      {
        remote_port: 49152,
        payload_bytes_received: 64,
        payload_read_category: "complete",
        payload_digest_match: true,
        extra_bytes_received: 0,
        receipt_ack_sent: true,
      },
    ],
  };

  // Act
  const identity = tcpPayloadPrivateLocalPortFromMonitor(monitor);
  const projected = projectTcpPayloadConnection(monitor, privateFixture);
  const encoded = JSON.stringify(projected);

  // Assert
  assert.deepEqual(identity, { localPort: 49152, markerCount: 2, consistent: true });
  assert.equal(projected.connection["tuple_match"], true);
  assert.equal(projected.connection["other_exact_peer_connection_count"], 1);
  assert.equal(projected.fixture["payload_bytes_received"], 64);
  assert.doesNotMatch(encoded, /4915[12]|remote_port|local_port|tcp_candidates/u);
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
  const args = tcpPayloadDiagnosticValidatorArgs("/private/candidate.json", source, 9);

  // Assert
  assert.deepEqual(args, [
    "run",
    "//tools/automation:stratum_v2_tcp_payload_validator",
    "--",
    "/private/candidate.json",
    source,
    "9",
  ]);
});
