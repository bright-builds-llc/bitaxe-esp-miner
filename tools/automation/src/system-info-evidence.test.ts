import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";
import { captureSystemInfoEvidence, SystemInfoEvidenceError } from "./system-info-evidence.js";
import type { WebSocketClient, WebSocketFactory } from "./websocket.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const session = "1".repeat(32);
const ok = (): ProcessOutcome => ({ exitCode: 0, stdout: "", stderr: "", timedOut: false });
const failed = (): ProcessOutcome => ({ exitCode: 1, stdout: "", stderr: "", timedOut: false });
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

type Contract = {
  readonly schema_version: string;
  readonly fields: Readonly<Record<string, { readonly type: string; readonly presence: string }>>;
};

function runtimeHealth(sequence: number) {
  return {
    selfTestState: "unavailable", supervisorAvailability: "available", checkpointCategory: "telemetry",
    checkpointSequence: sequence, checkpointAgeMillis: 100, checkpointHealth: "healthy",
    taskWatchdogParticipation: "participating", taskWatchdogReason: "feed_fresh",
    taskWatchdogFeedSequence: sequence + 2, taskWatchdogFeedAgeMillis: 50,
    taskWatchdogReadOutcome: "stable", taskWatchdogOwnerPhase: "waiting_inbox", taskWatchdogWaitState: "within_deadline",
  };
}

function retained(revision: number, sequence: number): string {
  return `runtime_health boot_session=${session} operator_snapshot_revision=${String(revision)} self_test=unavailable supervisor=available checkpoint_category=telemetry checkpoint_sequence=${String(sequence)} checkpoint_age_millis=100 checkpoint_health=healthy task_watchdog_participation=participating task_watchdog_reason=feed_fresh task_watchdog_feed_sequence=${String(sequence + 2)} task_watchdog_feed_age_millis=50 task_watchdog_read_outcome=stable task_watchdog_owner_phase=waiting_inbox task_watchdog_wait_state=within_deadline redacted=true`;
}

function sampleFor(type: string): unknown {
  if (type === "array") return [];
  if (type === "boolean") return false;
  if (type === "number") return 0;
  if (type === "object") return {};
  return "";
}

function snapshot(contract: Contract, revision: number, sequence: number): Record<string, unknown> {
  const value: Record<string, unknown> = {};
  for (const [field, rule] of Object.entries(contract.fields)) {
    if (rule.presence === "always") value[field] = sampleFor(rule.type);
  }
  return {
    ...value,
    blockFound: 0,
    bootSession: session,
    operatorSnapshotRevision: revision,
    sourceCommit,
    referenceCommit,
    appElfSha256,
    runtimeHealth: runtimeHealth(sequence),
  };
}

async function sourceContract(): Promise<string> {
  const relative = path.join("crates", "bitaxe-api", "fixtures", "api", "system-info-contract-v1.json");
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  const candidates = [
    ...(maybeRunfiles === undefined ? [] : [path.join(maybeRunfiles, "_main", relative)]),
    path.join(process.cwd(), relative),
    path.resolve(process.cwd(), "..", "..", relative),
  ];
  for (const candidate of candidates) {
    try {
      return await readFile(candidate, "utf8");
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  throw new Error("system info field contract test input is missing");
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-system-info-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  await mkdir(path.join(root, "crates", "bitaxe-api", "fixtures", "api"), { recursive: true });
  const contractDocument = await sourceContract();
  const contract = JSON.parse(contractDocument) as Contract;
  await writeFile(path.join(root, "crates", "bitaxe-api", "fixtures", "api", "system-info-contract-v1.json"), contractDocument);
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({ source_commit: sourceCommit, reference_commit: referenceCommit, app_elf_sha256: appElfSha256 }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return {
    root,
    contract,
    projection: path.join(root, "docs", "system-info.json"),
    options: {
      privateRoot: "scratch/attempt", packageManifest: manifest, wifiCredentials: credentials,
      port: "/dev/private-port", projection: path.join(root, "docs", "system-info.json"), captureTimeoutSeconds: 360,
    },
  };
}

function installHttp(contract: Contract, configuration: { readonly removeField?: string; readonly blockFound?: number } = {}) {
  const original = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname.endsWith("/logs")) return new Response(`${retained(7, 9)}\n${retained(8, 10)}\n`, { status: 200 });
    const value = snapshot(contract, 7, 9);
    if (configuration.removeField !== undefined) delete value[configuration.removeField];
    if (configuration.blockFound !== undefined) value["blockFound"] = configuration.blockFound;
    return new Response(JSON.stringify(value), { status: 200, headers: { "content-type": "application/json" } });
  };
  return () => { globalThis.fetch = original; };
}

function websocketFactory(value: Record<string, unknown>): WebSocketFactory {
  return () => {
    const listeners = new Map<string, (event: { readonly data: unknown }) => void>();
    const client: WebSocketClient = { addEventListener(type, listener): void { listeners.set(type, listener); }, close(): void {} };
    queueMicrotask(() => listeners.get("message")?.({ data: JSON.stringify({ event: "update", data: value }) }));
    return client;
  };
}

function fakePort(configuration: {
  readonly flash?: ProcessOutcome;
  readonly recovery?: ProcessOutcome;
  readonly validator?: ProcessOutcome;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      if (configuration.launchFailure === true) throw new Error("launch canary");
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=${session} device_url=http://private-device.test redacted=true\n`);
      return configuration.flash ?? ok();
    }
    if (spec.args[0] === "flash") return configuration.recovery ?? ok();
    if (spec.program === "validator") return configuration.validator ?? ok();
    throw new Error("unexpected child");
  });
}

async function capture(value: Awaited<ReturnType<typeof fixture>>, port: ProcessPort) {
  return captureSystemInfoEvidence(
    value.root,
    value.options,
    port,
    "flash",
    "validator",
    websocketFactory(snapshot(value.contract, 8, 10)),
  );
}

async function captureError(promise: Promise<unknown>): Promise<SystemInfoEvidenceError> {
  try { await promise; assert.fail("expected capture failure"); } catch (error) { assert.ok(error instanceof SystemInfoEvidenceError); return error; }
}

test("complete inactive system-info contract emits aggregate-only evidence", async () => {
  const value = await fixture("ready");
  const restore = installHttp(value.contract);
  try {
    const evidence = await capture(value, fakePort());
    const document = await readFile(value.projection, "utf8");
    assert.equal(evidence.system_info["required_field_count"], 94);
    assert.equal(evidence.system_info["conditional_field_count"], 7);
    assert.doesNotMatch(document, /private-device|private-port|hostname|stratumURL|ssid/u);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt"))).mode & 0o777, 0o700);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt", "final-evidence.private.json"))).mode & 0o777, 0o600);
  } finally { restore(); }
});

test("missing field and active block fail closed with typed primary precedence", async () => {
  for (const testCase of [
    { name: "missing", configuration: { removeField: "cpuUsage" }, category: "evidence_invalid" },
    { name: "block-active", configuration: { blockFound: 1 }, category: "hardware_blocked" },
  ] as const) {
    const value = await fixture(testCase.name);
    const restore = installHttp(value.contract, testCase.configuration);
    try {
      const error = await captureError(capture(value, fakePort({ recovery: failed() })));
      assert.equal(error.category, testCase.category);
      assert.equal(error.publicValue["secondary_recovery_failure"], true);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally { restore(); }
  }
});

test("child timeout launch failure and validator rejection preserve categories", async () => {
  for (const testCase of [
    { name: "timeout", port: fakePort({ flash: { ...ok(), timedOut: true } }), category: "timeout" },
    { name: "launch", port: fakePort({ launchFailure: true }), category: "process_failed" },
    { name: "validator", port: fakePort({ validator: failed() }), category: "evidence_invalid" },
  ] as const) {
    const value = await fixture(testCase.name);
    const restore = installHttp(value.contract);
    try {
      const error = await captureError(capture(value, testCase.port));
      assert.equal(error.category, testCase.category);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally { restore(); }
  }
});

test("real child process supplies flash and validation boundaries", async () => {
  const value = await fixture("real-child");
  const restore = installHttp(value.contract);
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}\nimport { writeFile } from "node:fs/promises"; import path from "node:path"; const args=process.argv.slice(2); if(args[0]==="flash-monitor"){const root=args[args.indexOf("--evidence-dir")+1]; await writeFile(path.join(root,"flash-monitor.classifier-input.log"),${JSON.stringify(`safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=${session} device_url=http://private-device.test redacted=true\n`)});}\n`);
  await chmod(child, 0o700);
  try {
    const evidence = await captureSystemInfoEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      websocketFactory(snapshot(value.contract, 8, 10)),
    );
    assert.equal(evidence.schema_version, "bitaxe-system-info-evidence-v1");
  } finally { restore(); }
});
