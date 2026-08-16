import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createFakeProcessPort, type ProcessPort } from "./process.js";
import {
  captureStatisticsHistoryEvidence,
  StatisticsHistoryEvidenceError,
} from "./statistics-history-evidence.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const appElfSha256 = "b".repeat(64);
const trace = "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=private device_url=http://private-device.test redacted=true\n";
const labels = [
  "hashrate", "hashrate_1m", "hashrate_10m", "hashrate_1h", "errorPercentage",
  "asicTemp", "asicTemp2", "vrTemp", "asicVoltage", "voltage", "power", "current",
  "fanSpeed", "fanRpm", "fan2Rpm", "wifiRssi", "freeHeap", "responseTime", "timestamp",
] as const;

type Fixture = {
  readonly root: string;
  readonly manifest: string;
  readonly credentials: string;
  readonly detectorOutput: string;
  readonly projection: string;
  readonly planSha256: string;
};

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

async function file(root: string, relative: string, document: string): Promise<void> {
  const output = path.join(root, relative);
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, document);
}

async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-statistics-history-${name}-`));
  await file(root, "MODULE.bazel", "module(name = \"fixture\")\n");
  const plan = [
    "# Parity work plan",
    "- Parity row: `STAT-002`",
    "- Active task: `task-parity-stat002-statistics-history`",
    "",
  ].join("\n");
  await file(root, "docs/parity/work-plans/20260816T204646Z-STAT-002/PLAN.md", plan);
  await file(root, "TASKS.md", [
    "### task-parity-stat002-statistics-history | fixture",
    "docs/parity/work-plans/20260816T204646Z-STAT-002/PLAN.md",
    "attempt-001",
    "",
  ].join("\n"));
  await file(root, "firmware/bitaxe/src/statistics_runtime.rs", [
    "pub const STATISTICS_CADENCE_MS: u64 = 1_000;",
    "record_statistics_sample(now_ms, frequency_seconds)",
  ].join("\n"));
  await file(root, "firmware/bitaxe/src/runtime_snapshot.rs", [
    "pub fn record_statistics_sample(timestamp_ms: u64, frequency_seconds: u16)",
    "statistics_response(timestamp_ms, None, &statistics_samples())",
  ].join("\n"));
  await file(root, "crates/bitaxe-api/src/statistics/history.rs", [
    "pub const MAX_STATISTICS_SAMPLES: usize = 720;",
    "if frequency_seconds == 0 {",
    "            return Ok(self.disable());",
  ].join("\n"));
  await file(root, "crates/bitaxe-api/src/statistics.rs", [
    "const ALL_COLUMNS: [StatisticsColumn; 18]",
    "labels.push(TIMESTAMP_LABEL.to_owned())",
    "voltage_millivolts: millivolts_from_volts",
    "current_milliamps: milliamps_from_amps",
  ].join("\n"));
  await file(root, "reference/esp-miner/main/tasks/statistics_task.c", [
    "#define DEFAULT_POLL_RATE 1000",
    "if (0 != configStatsFrequency)",
    "if (currentTime >= statsData.timestamp + 1000)",
    "addStatisticData(&statsData, configStatsFrequency);",
    "removeStatisticsBuffer();",
  ].join("\n"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await file(root, "inputs/package.json", JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
    artifacts: [{ kind: "factory_merged_image", sha256: "d".repeat(64) }],
  }));
  await file(root, "inputs/wifi.json", "{}\n");
  await chmod(credentials, 0o600);
  const wrapper = path.join(root, "scratch/stat002-statistics-history/wrapper-001");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  for (const name of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await writeFile(path.join(wrapper, name), "", { mode: 0o600 });
    await chmod(path.join(wrapper, name), 0o600);
  }
  return {
    root,
    manifest,
    credentials,
    detectorOutput: path.join(wrapper, "detector.stdout"),
    projection: path.join(
      root,
      "docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json",
    ),
    planSha256: sha256(plan),
  };
}

function options(value: Fixture) {
  return {
    privateRoot: "scratch/stat002-statistics-history/attempt-001",
    packageManifest: value.manifest,
    wifiCredentials: value.credentials,
    detectorOutput: value.detectorOutput,
    port: "/dev/private-sensitive-port",
    projection: value.projection,
    captureTimeoutSeconds: 360,
  } as const;
}

function row(timestamp: number): number[] {
  return [...Array.from({ length: 18 }, (_, index) => index + 0.25), timestamp];
}

function installApi(configuration: { readonly malformedHistory?: boolean } = {}) {
  let frequency = 0;
  let statisticsRead = 0;
  const patches: number[] = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (_input, init) => {
    if (init?.method === "PATCH") {
      const body = JSON.parse(String(init.body)) as Record<string, unknown>;
      assert.deepEqual(Object.keys(body), ["statsFrequency"]);
      frequency = Number(body["statsFrequency"]);
      patches.push(frequency);
      return new Response("ok", { status: 200 });
    }
    const url = String(_input);
    if (url.endsWith("/api/system/info")) {
      return new Response(JSON.stringify({
        statsFrequency: frequency,
        sourceCommit,
        referenceCommit,
        appElfSha256,
      }), { status: 200, headers: { "content-type": "application/json" } });
    }
    statisticsRead += 1;
    const statistics = frequency === 0
      ? []
      : statisticsRead <= 2
        ? [row(1_000), row(2_000), row(3_000)]
        : [row(1_000), row(2_000), row(3_000), row(4_000)];
    return new Response(JSON.stringify({
      currentTimestamp: 5_000,
      labels: configuration.malformedHistory === true ? ["private-invalid"] : labels,
      statistics,
    }), { status: 200, headers: { "content-type": "application/json" } });
  };
  return { patches, restore: () => { globalThis.fetch = originalFetch; } };
}

async function writeCompletedEffect(spec: Parameters<ProcessPort["run"]>[0]): Promise<void> {
  const effectPath = String(spec.environment?.["PHASE36_EFFECT_RESULT_PATH"]);
  await writeFile(effectPath, `${JSON.stringify({
    schema_version: "phase36-effect-result-v1",
    operation: "exact_package_flash",
    status: "completed",
    failure: null,
    package_identity_digest: spec.environment?.["PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST"],
    factory_image_digest: spec.environment?.["PHASE36_EFFECT_FACTORY_IMAGE_DIGEST"],
  })}\n`, { mode: 0o600 });
  await chmod(effectPath, 0o600);
}

function fakePort(): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      await writeCompletedEffect(spec);
      const evidenceRoot = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(evidenceRoot, "flash-monitor.classifier-input.log"), trace, { mode: 0o600 });
      await chmod(path.join(evidenceRoot, "flash-monitor.classifier-input.log"), 0o600);
      return { exitCode: 0, stdout: "", stderr: "", timedOut: false };
    }
    if (spec.program === "git") {
      const joined = spec.args.join(" ");
      const stdout = joined === "rev-parse HEAD" || joined === "rev-parse origin/main"
        ? `${sourceCommit}\n`
        : joined.includes("reference/esp-miner") && joined.endsWith("rev-parse HEAD")
          ? `${referenceCommit}\n`
          : "";
      return { exitCode: 0, stdout, stderr: "", timedOut: false };
    }
    if (spec.program === "validator") {
      return { exitCode: 0, stdout: "", stderr: "", timedOut: false };
    }
    throw new Error("unexpected child process");
  });
}

const noWait = async (_milliseconds: number): Promise<void> => {};

async function capture(value: Fixture): Promise<unknown> {
  return captureStatisticsHistoryEvidence(
    value.root,
    options(value),
    fakePort(),
    "flash",
    "git",
    "validator",
    value.planSha256,
    noWait,
  );
}

test("one-field cadence proof restores zero and emits aggregate-only evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const api = installApi();

  try {
    // Act
    const evidence = await capture(value) as { statistics_history: { sample_count: number } };
    const projection = await readFile(value.projection, "utf8");

    // Assert
    assert.equal(evidence.statistics_history.sample_count, 4);
    assert.deepEqual(api.patches, [1, 0]);
    assert.doesNotMatch(projection, /private-device|private-sensitive-port|0\.25/u);
    const privateRoot = path.join(value.root, "scratch/stat002-statistics-history/attempt-001");
    assert.equal((await stat(privateRoot)).mode & 0o777, 0o700);
    assert.equal((await stat(path.join(privateRoot, "final-evidence.private.json"))).mode & 0o777, 0o600);
  } finally {
    api.restore();
  }
});

test("malformed statistics preserve the primary failure after restoration", async () => {
  // Arrange
  const value = await fixture("malformed");
  const api = installApi({ malformedHistory: true });

  try {
    // Act
    let captured: StatisticsHistoryEvidenceError | undefined;
    try {
      await capture(value);
    } catch (error) {
      assert.ok(error instanceof StatisticsHistoryEvidenceError);
      captured = error;
    }

    // Assert
    assert.equal(captured?.category, "evidence_invalid");
    assert.equal(captured?.publicValue["restoration_complete"], true);
    assert.deepEqual(api.patches, [1, 0]);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    api.restore();
  }
});
