import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureOperatorSnapshotEvidence,
  OperatorSnapshotEvidenceError,
  type OperatorSnapshotEvidenceOptions,
} from "./operator-snapshot-evidence.js";
import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";
import type { WebSocketClient, WebSocketFactory } from "./websocket.js";

const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });
const failed = (): ProcessOutcome => ({ exitCode: 1, stdout: "", stderr: "", timedOut: false });
const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const baselineSession = "1".repeat(32);
const postSession = "2".repeat(32);
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

const readyProjection = {
  schema_version: "esp-device-session-v1", terminal_category: "ready", platform_category: "macos",
  board_category: "205", same_physical_device: true, stable_enumeration: true, reenumerated: false,
  reader_armed: true, pre_restart_serial_delivery: true, post_restart_serial_delivery: true,
  serial_delivery: "correlated", request_outcome: "response_received", request_attempt_count: 1,
  service_loss_observed: true, trusted_origin_preserved: true, application_recovered: true,
  build_identity_matches: true, boot_session_changed: true, boot_ordinal_advanced_by_one: true,
  software_reset_observed: true, postcondition_matches: true, cleanup_complete: true,
  usb_disappearance_count: 0, enumeration_change_count: 0, serial_byte_count: 128,
  http_observation_count: 3, duration_millis: 1000,
} as const;

type Fixture = {
  readonly root: string;
  readonly manifest: string;
  readonly credentials: string;
  readonly projection: string;
  readonly options: OperatorSnapshotEvidenceOptions;
};

async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-snapshot-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  const projection = path.join(root, "docs", "snapshot.json");
  await writeFile(manifest, JSON.stringify({ source_commit: sourceCommit, reference_commit: referenceCommit, app_elf_sha256: appElfSha256 }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return {
    root, manifest, credentials, projection,
    options: {
      privateRoot: "scratch/attempt-001", packageManifest: manifest, wifiCredentials: credentials,
      port: "/dev/private-ultra205", projection, captureTimeoutSeconds: 360,
    },
  };
}

function snapshot(session: string, ordinal: number, revision: number) {
  return {
    bootSession: session, bootOrdinal: ordinal, operatorSnapshotRevision: revision,
    ASICModel: "BM1366", boardVersion: "205", version: "test-dev", semanticVersion: "0.1.0",
    sourceCommit, referenceCommit, appElfSha256, buildChannel: "dev", sourceDirty: false,
    releaseTag: null, axeOSVersion: "test-dev", idfVersion: "v5.5.4", miningPaused: false,
    miningActivity: "safe_blocked", startMiningOnBoot: false, hostname: "private-hostname",
  };
}

function installHttp(maybeMutate?: (value: Record<string, unknown>, index: number) => void) {
  const original = globalThis.fetch;
  let apiIndex = 0;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname === "/api/system/logs") {
      const session = apiIndex < 2 ? baselineSession : postSession;
      return new Response(`operator_snapshot session=${session} revision=7 redacted=true\noperator_snapshot session=${session} revision=8 redacted=true\n`, { status: 200 });
    }
    const epoch = apiIndex++;
    const value = epoch === 0 ? snapshot(baselineSession, 4, 7) : snapshot(postSession, 5, 7);
    maybeMutate?.(value, epoch);
    return new Response(JSON.stringify(value), { status: 200, headers: { "content-type": "application/json" } });
  };
  return () => { globalThis.fetch = original; };
}

function websocketFactory(maybeMutate?: (value: Record<string, unknown>, index: number) => void): WebSocketFactory {
  let index = 0;
  return () => {
    const listeners = new Map<string, (event: { readonly data: unknown }) => void>();
    const client: WebSocketClient = {
      addEventListener(type, listener): void { listeners.set(type, listener); },
      close(): void {},
    };
    queueMicrotask(() => {
      const epoch = index++;
      const value = epoch === 0 ? snapshot(baselineSession, 4, 8) : snapshot(postSession, 5, 8);
      maybeMutate?.(value, epoch);
      listeners.get("message")?.({ data: JSON.stringify({ event: "update", data: value }) });
    });
    return client;
  };
}

function fakePort(
  projection: unknown = readyProjection,
  configuration: {
    readonly omitProjection?: boolean;
    readonly sessionOutcome?: ProcessOutcome;
    readonly recoveryOutcome?: ProcessOutcome;
    readonly finalValidationOutcome?: ProcessOutcome;
  } = {},
): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    const command = spec.args[0] ?? "";
    if (command === "flash-monitor") {
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), [
        "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
        `runtime_origin session=${baselineSession} device_url=http://private-device.test redacted=true`,
      ].join("\n"));
      return ok();
    }
    if (command === "verify-settings-durability") {
      return ok(JSON.stringify({
        status: "passed",
        session: baselineSession,
        boot_ordinal: 4,
        device_url: "http://private-device.test",
      }));
    }
    if (command === "validate-operator-snapshot") return ok('{"status":"passed"}');
    if (command === "reboot-live") {
      if (configuration.omitProjection !== true) {
        const output = String(spec.args[spec.args.indexOf("--projection-output") + 1]);
        await writeFile(output, JSON.stringify(projection));
      }
      return configuration.sessionOutcome ?? ok();
    }
    if (command === "flash") return configuration.recoveryOutcome ?? ok();
    if (spec.program === "evidence-validator") return configuration.finalValidationOutcome ?? ok();
    throw new Error(`unexpected child command ${command}`);
  });
}

async function capture(value: Fixture, processPort: ProcessPort, factory = websocketFactory()) {
  return captureOperatorSnapshotEvidence(
    value.root, value.options, processPort, "flash", "parity", "device-session", "evidence-validator", factory,
  );
}

async function captureError(promise: Promise<unknown>): Promise<OperatorSnapshotEvidenceError> {
  try {
    await promise;
    assert.fail("expected operator snapshot capture to fail");
  } catch (error) {
    assert.ok(error instanceof OperatorSnapshotEvidenceError);
    return error;
  }
}

test("two substantive boot epochs and one ready restart emit closed evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const restore = installHttp();

  try {
    // Act
    const evidence = await capture(value, fakePort());

    // Assert
    assert.equal(evidence.distinct_boot_sessions, true);
    assert.equal(evidence.baseline_epoch.websocket_revision, 8);
    assert.equal(evidence.post_restart_epoch.websocket_revision, 8);
    const publicDocument = await readFile(value.projection, "utf8");
    assert.doesNotMatch(publicDocument, /private-device|private-hostname|private-ultra205|127\.0\.0\.1/u);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt-001"))).mode & 0o777, 0o700);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt-001", "device-session-intent.private.json"))).mode & 0o777, 0o600);
  } finally {
    restore();
  }
});

test("non-ready and malformed restart projections withhold final evidence", async () => {
  const nonReadyCategories = [
    "incomplete", "observer_unqualified", "restart_request_not_sent", "restart_attribution_ambiguous",
    "usb_identity_unavailable", "usb_identity_drift", "service_recovery_timeout", "boot_identity_invalid",
    "build_identity_mismatch", "session_not_advanced", "reset_reason_wrong", "ordinal_not_next",
    "postcondition_mismatch",
  ];
  const testCases = [
    ...nonReadyCategories.map((terminalCategory) => ({
      name: terminalCategory,
      projection: { ...readyProjection, terminal_category: terminalCategory },
      category: "hardware_blocked",
    })),
    { name: "malformed", projection: { ...readyProjection, private_origin: "http://secret" }, category: "evidence_invalid" },
    { name: "missing", projection: readyProjection, category: "evidence_invalid", omitProjection: true },
  ] as const;
  for (const testCase of testCases) {
    // Arrange
    const value = await fixture(testCase.name);
    const restore = installHttp();
    try {
      // Act
      const error = await captureError(capture(value, fakePort(testCase.projection, {
        ...("omitProjection" in testCase ? { omitProjection: testCase.omitProjection } : {}),
        sessionOutcome: failed(),
      })));

      // Assert
      assert.equal(error.category, testCase.category);
      assert.equal(error.publicValue["recovery_flash_used"], true);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      restore();
    }
  }
});

test("epoch mismatches fail evidence admission and preserve the primary failure through recovery", async () => {
  // Arrange
  const value = await fixture("mismatch");
  const restore = installHttp();
  const factory = websocketFactory((document, index) => {
    if (index === 1) document["sourceCommit"] = "f".repeat(40);
  });

  try {
    // Act
    const error = await captureError(capture(value, fakePort(readyProjection, { recoveryOutcome: failed() }), factory));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    assert.equal(error.publicValue["secondary_recovery_failure"], true);
    assert.doesNotMatch(JSON.stringify(error.publicValue), /private|sourceCommit|device/u);
  } finally {
    restore();
  }
});

test("same-epoch session and revision mismatches are rejected before publication", async () => {
  for (const testCase of [
    { name: "session", mutate: (document: Record<string, unknown>) => { document["bootSession"] = "3".repeat(32); } },
    { name: "revision", mutate: (document: Record<string, unknown>) => { document["operatorSnapshotRevision"] = 6; } },
  ]) {
    // Arrange
    const value = await fixture(testCase.name);
    const restore = installHttp();
    const factory = websocketFactory((document, index) => {
      if (index === 0) testCase.mutate(document);
    });

    try {
      // Act
      const error = await captureError(capture(value, fakePort(), factory));

      // Assert
      assert.equal(error.category, "evidence_invalid");
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      restore();
    }
  }
});

test("failed final contract validation never publishes the candidate projection", async () => {
  // Arrange
  const value = await fixture("final-validation");
  const restore = installHttp();

  try {
    // Act
    const error = await captureError(capture(value, fakePort(readyProjection, { finalValidationOutcome: failed() })));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    assert.equal((await stat(path.join(value.root, "scratch", "attempt-001", "final-evidence.private.json"))).mode & 0o777, 0o600);
  } finally {
    restore();
  }
});

test("a real child process provides every file-based transaction boundary", async () => {
  // Arrange
  const value = await fixture("real-child");
  const restore = installHttp();
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}
import { writeFile } from "node:fs/promises";
import path from "node:path";
const args = process.argv.slice(2);
if (args[0] === "flash-monitor") {
  const root = args[args.indexOf("--evidence-dir") + 1];
  await writeFile(path.join(root, "flash-monitor.classifier-input.log"), ${JSON.stringify(`safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=${baselineSession} device_url=http://private-device.test redacted=true\n`)});
} else if (args[0] === "verify-settings-durability") {
  process.stdout.write(${JSON.stringify(JSON.stringify({ status: "passed", session: baselineSession, boot_ordinal: 4, device_url: "http://private-device.test" }))});
} else if (args[0] === "reboot-live") {
  await writeFile(args[args.indexOf("--projection-output") + 1], ${JSON.stringify(JSON.stringify(readyProjection))});
} else if (args[0] !== "validate-operator-snapshot" && args.length !== 1) {
  process.exitCode = 91;
}
`);
  await chmod(child, 0o700);
  const port = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });

  try {
    // Act
    const evidence = await captureOperatorSnapshotEvidence(
      value.root, value.options, port, child, child, child, child, websocketFactory(),
    );

    // Assert
    assert.equal(evidence.schema_version, "bitaxe-operator-snapshot-evidence-v1");
    assert.equal((await readFile(value.projection, "utf8")).includes("private-device.test"), false);
  } finally {
    restore();
  }
});
