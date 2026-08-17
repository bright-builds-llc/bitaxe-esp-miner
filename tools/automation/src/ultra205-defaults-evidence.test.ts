import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome } from "./process.js";
import {
  captureUltra205DefaultsEvidence,
  Ultra205DefaultsEvidenceError,
} from "./ultra205-defaults-evidence.js";
import type { WebSocketClient, WebSocketFactory } from "./websocket.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const session = "1".repeat(32);
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;
const attestation = "ultra205_config_defaults schema_version=1 matching_fields=27 total_fields=27 all_match=true mineonboot_disabled=true redacted=true";
const ok = (): ProcessOutcome => ({ exitCode: 0, stdout: "", stderr: "", timedOut: false });

type Contract = {
  readonly fields: Readonly<Record<string, { readonly type: string; readonly presence: string }>>;
};

async function sourceFile(relative: string): Promise<string> {
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
  throw new Error("test source file is missing");
}

function runtimeHealth(sequence: number) {
  return {
    selfTestState: "unavailable", supervisorAvailability: "available", checkpointCategory: "telemetry",
    checkpointSequence: sequence, checkpointAgeMillis: 100, checkpointHealth: "healthy",
    taskWatchdogParticipation: "participating", taskWatchdogReason: "feed_fresh",
    taskWatchdogFeedSequence: sequence + 2, taskWatchdogFeedAgeMillis: 50,
    taskWatchdogOwnerPhase: "waiting_inbox", taskWatchdogWaitState: "within_deadline",
  };
}

function retainedHealth(revision: number, sequence: number): string {
  return `runtime_health boot_session=${session} operator_snapshot_revision=${String(revision)} self_test=unavailable supervisor=available checkpoint_category=telemetry checkpoint_sequence=${String(sequence)} checkpoint_age_millis=100 checkpoint_health=healthy task_watchdog_participation=participating task_watchdog_reason=feed_fresh task_watchdog_feed_sequence=${String(sequence + 2)} task_watchdog_feed_age_millis=50 task_watchdog_owner_phase=waiting_inbox task_watchdog_wait_state=within_deadline redacted=true`;
}

function sampleFor(type: string): unknown {
  if (type === "array") return [];
  if (type === "boolean") return false;
  if (type === "number") return 0;
  if (type === "object") return {};
  return "fixture";
}

function configuredDefaults(): Readonly<Record<string, unknown>> {
  return {
    hostname: "bitaxe",
    stratumURL: "public-pool.io",
    stratumPort: 3333,
    stratumTLS: 0,
    stratumCert: "x",
    stratumUser: "bc1qnp980s5fpp8l94p5cvttmtdqy8rvrq74qly2yrfmzkdsntqzlc5qkc4rkq.bitaxe",
    stratumSuggestedDifficulty: 1000,
    stratumExtranonceSubscribe: false,
    fallbackStratumURL: "solo.ckpool.org",
    fallbackStratumPort: 3333,
    fallbackStratumTLS: 0,
    fallbackStratumCert: "x",
    fallbackStratumUser: "bc1qnp980s5fpp8l94p5cvttmtdqy8rvrq74qly2yrfmzkdsntqzlc5qkc4rkq.bitaxe",
    fallbackStratumSuggestedDifficulty: 1000,
    fallbackStratumExtranonceSubscribe: false,
    frequency: 485,
    coreVoltage: 1200,
    ASICModel: "BM1366",
    boardVersion: "205",
    rotation: 0,
    autofanspeed: 1,
    manualFanSpeed: 100,
    overheat_mode: 0,
    startMiningOnBoot: false,
    miningPaused: true,
  };
}

function snapshot(contract: Contract, revision: number, sequence: number, mismatch = false): Record<string, unknown> {
  const value: Record<string, unknown> = {};
  for (const [field, rule] of Object.entries(contract.fields)) {
    if (rule.presence === "always") value[field] = sampleFor(rule.type);
  }
  return {
    ...value,
    ...configuredDefaults(),
    ...(mismatch ? { coreVoltage: 1 } : {}),
    blockFound: 0,
    bootSession: session,
    operatorSnapshotRevision: revision,
    sourceCommit,
    referenceCommit,
    appElfSha256,
    runtimeHealth: runtimeHealth(sequence),
  };
}

function websocketFactory(value: Record<string, unknown>): WebSocketFactory {
  return () => {
    const listeners = new Map<string, (event: { readonly data: unknown }) => void>();
    const client: WebSocketClient = {
      addEventListener(type, listener): void { listeners.set(type, listener); },
      close(): void {},
    };
    queueMicrotask(() => listeners.get("message")?.({ data: JSON.stringify({ event: "update", data: value }) }));
    return client;
  };
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-ultra205-defaults-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const contractDocument = await sourceFile("crates/bitaxe-api/fixtures/api/system-info-contract-v1.json");
  const seedDocument = await sourceFile("crates/bitaxe-config/fixtures/ultra-205-defaults.csv");
  await mkdir(path.join(root, "crates", "bitaxe-api", "fixtures", "api"), { recursive: true });
  await mkdir(path.join(root, "crates", "bitaxe-config", "fixtures"), { recursive: true });
  await mkdir(path.join(root, "inputs"));
  await writeFile(path.join(root, "crates", "bitaxe-api", "fixtures", "api", "system-info-contract-v1.json"), contractDocument);
  await writeFile(path.join(root, "crates", "bitaxe-config", "fixtures", "ultra-205-defaults.csv"), seedDocument);
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({ source_commit: sourceCommit, reference_commit: referenceCommit, app_elf_sha256: appElfSha256 }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return {
    root,
    contract: JSON.parse(contractDocument) as Contract,
    projection: path.join(root, "docs", "defaults.json"),
    options: {
      privateRoot: "scratch/attempt",
      packageManifest: manifest,
      wifiCredentials: credentials,
      port: "/dev/private-port",
      projection: path.join(root, "docs", "defaults.json"),
      captureTimeoutSeconds: 360,
    },
  };
}

function installHttp(contract: Contract, includeAttestation: boolean, mismatch = false) {
  const original = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname.endsWith("/logs")) {
      const lines = [retainedHealth(7, 9), retainedHealth(8, 10), ...(includeAttestation ? [attestation] : [])];
      return new Response(`${lines.join("\n")}\n`, { status: 200 });
    }
    return new Response(JSON.stringify(snapshot(contract, 7, 9, mismatch)), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };
  return () => { globalThis.fetch = original; };
}

function fakeProcessPort() {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=${session} device_url=http://private-device.test redacted=true\n`, { mode: 0o600 });
    }
    return ok();
  });
}

test("real process capture emits only closed Ultra 205 defaults evidence", async () => {
  const value = await fixture("real-child");
  const restore = installHttp(value.contract, true);
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}\nimport { writeFile } from "node:fs/promises"; import path from "node:path"; const args=process.argv.slice(2); if(args[0]==="flash-monitor"){const root=args[args.indexOf("--evidence-dir")+1]; await writeFile(path.join(root,"flash-monitor.classifier-input.log"),${JSON.stringify(`safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=${session} device_url=http://private-device.test redacted=true\n`)},{mode:0o600});}\n`);
  await chmod(child, 0o700);
  try {
    const evidence = await captureUltra205DefaultsEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      websocketFactory(snapshot(value.contract, 8, 10)),
    );
    const document = await readFile(value.projection, "utf8");
    assert.equal(evidence.defaults["firmware_matching_field_count"], 27);
    assert.equal(evidence.defaults["api_visible_default_field_count"], 23);
    assert.doesNotMatch(document, /public-pool|solo\.ckpool|bc1q|bitaxe"|private-device|private-port/u);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt"))).mode & 0o777, 0o700);
  } finally {
    restore();
  }
});

test("missing attestation and one mismatched API value withhold public evidence", async () => {
  for (const testCase of [
    { name: "marker", includeAttestation: false, mismatch: false },
    { name: "value", includeAttestation: true, mismatch: true },
  ]) {
    const value = await fixture(testCase.name);
    const restore = installHttp(value.contract, testCase.includeAttestation, testCase.mismatch);
    try {
      await assert.rejects(
        captureUltra205DefaultsEvidence(
          value.root,
          value.options,
          fakeProcessPort(),
          "flash",
          "system-validator",
          "defaults-validator",
          websocketFactory(snapshot(value.contract, 8, 10, testCase.mismatch)),
        ),
        (error: unknown) => error instanceof Ultra205DefaultsEvidenceError && error.category === "evidence_invalid",
      );
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      restore();
    }
  }
});
