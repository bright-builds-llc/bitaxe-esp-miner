import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";
import { captureRuntimeHealthEvidence, RuntimeHealthEvidenceError } from "./runtime-health-evidence.js";
import type { WebSocketClient, WebSocketFactory } from "./websocket.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const session = "1".repeat(32);
const ok = (): ProcessOutcome => ({ exitCode: 0, stdout: "", stderr: "", timedOut: false });
const failed = (): ProcessOutcome => ({ exitCode: 1, stdout: "", stderr: "", timedOut: false });
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

function runtimeHealth(sequence: number) {
  return {
    selfTestState: "unavailable", supervisorAvailability: "available", checkpointCategory: "telemetry",
    checkpointSequence: sequence, checkpointAgeMillis: 100, checkpointHealth: "healthy",
    taskWatchdogParticipation: "participating", taskWatchdogReason: "feed_fresh",
    taskWatchdogFeedSequence: sequence + 2, taskWatchdogFeedAgeMillis: 50,
    taskWatchdogOwnerPhase: "waiting_inbox", taskWatchdogWaitState: "within_deadline",
  };
}

function snapshot(revision: number, sequence: number) {
  return { bootSession: session, operatorSnapshotRevision: revision, sourceCommit, referenceCommit, appElfSha256, runtimeHealth: runtimeHealth(sequence) };
}

function retained(revision: number, sequence: number): string {
  return `runtime_health boot_session=${session} operator_snapshot_revision=${String(revision)} self_test=unavailable supervisor=available checkpoint_category=telemetry checkpoint_sequence=${String(sequence)} checkpoint_age_millis=100 checkpoint_health=healthy task_watchdog_participation=participating task_watchdog_reason=feed_fresh task_watchdog_feed_sequence=${String(sequence + 2)} task_watchdog_feed_age_millis=50 task_watchdog_owner_phase=waiting_inbox task_watchdog_wait_state=within_deadline redacted=true`;
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-health-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({ source_commit: sourceCommit, reference_commit: referenceCommit, app_elf_sha256: appElfSha256 }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return {
    root,
    projection: path.join(root, "docs", "health.json"),
    options: {
      privateRoot: "scratch/attempt", packageManifest: manifest, wifiCredentials: credentials,
      port: "/dev/private-port", projection: path.join(root, "docs", "health.json"), captureTimeoutSeconds: 360,
    },
  };
}

function installHttp(configuration: { readonly health?: Record<string, unknown>; readonly omitRetained?: boolean } = {}) {
  const original = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname.endsWith("/logs")) {
      return new Response(configuration.omitRetained === true ? "" : `${retained(7, 9)}\n${retained(8, 10)}\n`, { status: 200 });
    }
    const value = snapshot(7, 9);
    if (configuration.health !== undefined) value.runtimeHealth = configuration.health as ReturnType<typeof runtimeHealth>;
    return new Response(JSON.stringify(value), { status: 200, headers: { "content-type": "application/json" } });
  };
  return () => { globalThis.fetch = original; };
}

function websocketFactory(value = snapshot(8, 10)): WebSocketFactory {
  return () => {
    const listeners = new Map<string, (event: { readonly data: unknown }) => void>();
    const client: WebSocketClient = { addEventListener(type, listener): void { listeners.set(type, listener); }, close(): void {} };
    queueMicrotask(() => listeners.get("message")?.({ data: JSON.stringify({ event: "update", data: value }) }));
    return client;
  };
}

function fakePort(configuration: { readonly recovery?: ProcessOutcome; readonly validator?: ProcessOutcome } = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=${session} device_url=http://private-device.test redacted=true\n`);
      return ok();
    }
    if (spec.args[0] === "flash") return configuration.recovery ?? ok();
    if (spec.program === "validator") return configuration.validator ?? ok();
    throw new Error("unexpected child");
  });
}

async function capture(value: Awaited<ReturnType<typeof fixture>>, port: ProcessPort, websocket = websocketFactory()) {
  return captureRuntimeHealthEvidence(value.root, value.options, port, "flash", "validator", websocket);
}

async function captureError(promise: Promise<unknown>): Promise<RuntimeHealthEvidenceError> {
  try { await promise; assert.fail("expected capture failure"); } catch (error) { assert.ok(error instanceof RuntimeHealthEvidenceError); return error; }
}

test("healthy HTTP WebSocket and retained tuples emit closed runtime evidence", async () => {
  const value = await fixture("ready");
  const restore = installHttp();
  try {
    const evidence = await capture(value, fakePort());
    const document = await readFile(value.projection, "utf8");
    assert.equal(evidence.runtime_health["checkpoint_sequence_not_regressed"], true);
    assert.doesNotMatch(document, /private-device|private-port|hostname/u);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt"))).mode & 0o777, 0o700);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt", "final-evidence.private.json"))).mode & 0o777, 0o600);
  } finally { restore(); }
});

test("unhealthy state and missing retained tuple fail closed with primary precedence", async () => {
  for (const testCase of [
    { name: "unhealthy", configuration: { health: { ...runtimeHealth(9), checkpointHealth: "stale" } }, category: "hardware_blocked" },
    { name: "retained", configuration: { omitRetained: true }, category: "evidence_invalid" },
  ] as const) {
    const value = await fixture(testCase.name);
    const restore = installHttp(testCase.configuration);
    try {
      const error = await captureError(capture(value, fakePort({ recovery: failed() })));
      assert.equal(error.category, testCase.category);
      assert.equal(error.publicValue["secondary_recovery_failure"], true);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally { restore(); }
  }
});

test("session and sequence mismatches fail closed before publication", async () => {
  for (const testCase of [
    { name: "session-mismatch", value: { ...snapshot(8, 10), bootSession: "2".repeat(32) } },
    { name: "sequence-regression", value: snapshot(8, 8) },
  ]) {
    const value = await fixture(testCase.name);
    const restore = installHttp();
    try {
      const error = await captureError(capture(value, fakePort(), websocketFactory(testCase.value)));
      assert.equal(error.category, "evidence_invalid");
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally { restore(); }
  }
});

test("final validator rejection withholds the public projection", async () => {
  const value = await fixture("validator-rejected");
  const restore = installHttp();
  try {
    const error = await captureError(capture(value, fakePort({ validator: failed() })));
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally { restore(); }
});

test("real child process supplies flash and final validation boundaries", async () => {
  const value = await fixture("real-child");
  const restore = installHttp();
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}\nimport { writeFile } from "node:fs/promises"; import path from "node:path"; const args=process.argv.slice(2); if(args[0]==="flash-monitor"){const root=args[args.indexOf("--evidence-dir")+1]; await writeFile(path.join(root,"flash-monitor.classifier-input.log"),${JSON.stringify(`safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=${session} device_url=http://private-device.test redacted=true\n`)});}\n`);
  await chmod(child, 0o700);
  try {
    const evidence = await captureRuntimeHealthEvidence(value.root, value.options, createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }), child, child, websocketFactory());
    assert.equal(evidence.schema_version, "bitaxe-runtime-health-evidence-v1");
  } finally { restore(); }
});
