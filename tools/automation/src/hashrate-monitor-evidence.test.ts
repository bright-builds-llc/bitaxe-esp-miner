import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { toolProgram } from "./cli-tools.js";
import {
  captureHashrateMonitorEvidence,
  HashrateMonitorEvidenceError,
  validateHashrateMonitorTaskAndSources,
  type HashrateMonitorEvidenceOptions,
} from "./hashrate-monitor-evidence.js";
import { createLocalProcessPort } from "./process.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;
const workspace = process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd();
const validatorProgram = toolProgram(
  workspace,
  "crates/bitaxe-automation-contracts/validate_hashrate_monitor_evidence",
);
const sourceDocuments = new Map<string, string>([
  ["crates/bitaxe-core/src/hashrate.rs", [
    "const HASHRATE_REGISTER_UNIT_HASHES: f64 = 1_048_576.0;",
    "const HASH_COUNTER_UNIT_HASHES: f64 = 4_294_967_296.0;",
    "const MIN_COUNTER_INTERVAL_US: u64 = 1_000_000;",
  ].join("\n")],
  ["crates/bitaxe-stratum/src/v1/state.rs", "pub hashrate_inputs: HashrateInputs"],
  ["crates/bitaxe-api/src/mining.rs", [
    "hash_rate: hashrate.current_ghs,",
    "hashrate_monitor: HashrateMonitorWire {",
  ].join("\n")],
  ["crates/bitaxe-api/src/wire.rs", [
    '#[serde(rename = "hashRate")]',
    '#[serde(rename = "hashrateMonitor")]',
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/hashrate.rs", [
    "const HASHRATE_CADENCE_MS: u64 = 1_000;",
    "const BM1366_HASH_DOMAIN_COUNT: usize = 4;",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/asic_worker.rs", [
    "request_hashrate_monitor_register_reads_tx()",
    "emit(AsicWorkerEvent::RegisterRead {",
  ].join("\n")],
  ["firmware/bitaxe/src/runtime_snapshot.rs", "publish_hashrate_snapshot"],
]);

const okResult = {
  schema: "mining-campaign-result-v9",
  status: "accepted",
  stage: "live-share",
  profile: "conservative",
  duration_seconds: 600,
  runtime_identity: "trusted",
  safe_stop: "confirmed",
  usb_cleanup: "ready",
};

type Fixture = {
  readonly root: string;
  readonly planSha256: string;
  readonly options: HashrateMonitorEvidenceOptions;
};

function sha256(document: string): string {
  return createHash("sha256").update(document).digest("hex");
}

async function writeProtected(candidate: string, document: string): Promise<void> {
  await writeFile(candidate, document, { mode: 0o600 });
  await chmod(candidate, 0o600);
}

async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-hashrate-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), 'module(name = "fixture")\n');
  for (const [relative, document] of sourceDocuments) {
    const candidate = path.join(root, relative);
    await mkdir(path.dirname(candidate), { recursive: true });
    await writeFile(candidate, `${document}\n`);
  }
  const reference = path.join(root, "reference/esp-miner/main/tasks/hashrate_monitor_task.c");
  await mkdir(path.dirname(reference), { recursive: true });
  await writeFile(reference, [
    "#define HASHRATE_UNIT 0x100000uLL",
    "#define POLL_RATE 1000",
    "#define HASHRATE_1M_SIZE (60000 / POLL_RATE)",
    "void update_hash_counter(measurement_t * measurement, uint32_t value, uint64_t time_us)",
    ...Array.from({ length: 7 }, () => "update_hash_counter"),
    "ASIC_read_registers(GLOBAL_STATE);",
  ].join("\n"));
  const counterReference = path.join(root, "reference/esp-miner/components/stratum/utils.c");
  await mkdir(path.dirname(counterReference), { recursive: true });
  await writeFile(counterReference, [
    "#define HASH_CNT_LSB 0x100000000uLL",
    "float hashCounterToGhs(uint64_t duration_us, uint32_t counter)",
  ].join("\n"));
  const planRelative = "docs/parity/work-plans/20260816T022946Z-STAT-001/PLAN.md";
  const plan = "- Parity row: `STAT-001`\n- Active task: `task-parity-stat001-hashrate-monitor`\n";
  await mkdir(path.dirname(path.join(root, planRelative)), { recursive: true });
  await writeFile(path.join(root, planRelative), plan);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-stat001-hashrate-monitor | fixture",
    `Plan: \`${planRelative}\`.`,
    "Attempt: `attempt-003`.",
  ].join("\n"));
  const inputs = path.join(root, "inputs");
  await mkdir(inputs);
  await writeFile(path.join(inputs, "package.json"), JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
  }));
  const wrapper = path.join(root, "scratch/stat001-hashrate-monitor/wrapper-003");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  for (const output of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await writeProtected(path.join(wrapper, output), "");
  }
  return {
    root,
    planSha256: sha256(plan),
    options: {
      privateRoot: "scratch/stat001-hashrate-monitor/attempt-003",
      packageManifest: "inputs/package.json",
      wifiCredentials: "inputs/wifi.json",
      poolCredentials: "inputs/pool.json",
      detectorOutput: "scratch/stat001-hashrate-monitor/wrapper-003/detector.stdout",
      port: "/dev/private-port",
      projection: "docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json",
      durationSeconds: 600,
      captureTimeoutSeconds: 30,
    },
  };
}

async function childProgram(value: Fixture, malformed = false): Promise<string> {
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}
import { createHash } from "node:crypto";
import { chmod, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
const args = process.argv.slice(2);
const digest = (value) => createHash("sha256").update(value).digest("hex");
if (args[0] === "mining-campaign") {
  if (args[args.indexOf("--stage") + 1] !== "live-share" || args[args.indexOf("--profile") + 1] !== "conservative") process.exit(5);
  const root = args[args.indexOf("--evidence-dir") + 1];
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
  const transport = { active_sample_count: 3, positive_coherent_count: 3, distinct_positive_count: 2, warm_rolling_window_count: 2, terminal_zero_confirmed: true };
  const network = JSON.stringify({ schema: "mining-campaign-network-continuity-v4", status: "accepted", required_window_count: 20, covered_window_count: 20, hashrate_monitor: { monitor_cadence_ms: 1000, asic_count: 1, domain_count: 4, http: transport, websocket: ${malformed ? "{ ...transport, distinct_positive_count: 1 }" : "transport"} } }) + "\\n";
  const result = JSON.stringify({ ...${JSON.stringify(okResult)}, network_continuity_sha256: digest(network) }) + "\\n";
  const files = new Map([["campaign-diagnostics.private.json", "{}\\n"], ["campaign-flash.private.json", "{}\\n"], ["campaign-mining-diagnostics.private.json", "{}\\n"], ["campaign-network.private.json", network], ["campaign-observations.private.json", "{}\\n"], ["campaign-result.json", result], ["campaign-result.sha256", digest(result) + "\\n"]]);
  for (const [name, document] of files) { const candidate = path.join(root, name); await writeFile(candidate, document, { mode: 0o600 }); await chmod(candidate, 0o600); }
} else if (args[0] === "-C") {
  process.stdout.write(${JSON.stringify(`${referenceCommit}\n`)});
} else if (args[0] === "status") {
  process.stdout.write("");
} else if (args[0] === "rev-parse") {
  process.stdout.write(${JSON.stringify(`${sourceCommit}\n`)});
} else {
  process.exitCode = 2;
}
`);
  await chmod(child, 0o700);
  return child;
}

async function captureError(promise: Promise<unknown>): Promise<HashrateMonitorEvidenceError> {
  try {
    await promise;
    assert.fail("expected hashrate evidence failure");
  } catch (error) {
    assert.ok(error instanceof HashrateMonitorEvidenceError);
    return error;
  }
}

test("admissible conservative campaign and independent validator publish only closed evidence", async () => {
  // Arrange
  const value = await fixture("real-child");
  const child = await childProgram(value);
  try {
    // Act
    const evidence = await captureHashrateMonitorEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      validatorProgram,
      value.planSha256,
    );

    // Assert
    assert.equal(evidence.hashrate.http.distinct_positive_count, 2);
    assert.equal(evidence.source.source_path_count, 7);
    assert.equal((await stat(path.join(value.root, value.options.projection))).mode & 0o777, 0o644);
    assert.doesNotMatch(
      await readFile(path.join(value.root, value.options.projection), "utf8"),
      /private-port|credential|pool_url|worker|device_url|serial/u,
    );
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("current immutable task and production/reference sources pass admission", async () => {
  // Arrange
  const root = process.env["RUNFILES_DIR"] === undefined
    ? workspace
    : path.join(process.env["RUNFILES_DIR"], "_main");

  // Act / Assert
  await validateHashrateMonitorTaskAndSources(
    root,
    "876d0ba3dce066985d0e71f3b76732b4d603c6048b399dd085074b45bd7ba71f",
  );
});

test("incomplete transport evidence is rejected before publication", async () => {
  // Arrange
  const value = await fixture("incomplete");
  const child = await childProgram(value, true);

  try {
    // Act
    const error = await captureError(captureHashrateMonitorEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      validatorProgram,
      value.planSha256,
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(path.join(value.root, value.options.projection), "utf8"), {
      code: "ENOENT",
    });
  } finally {
    await rm(value.root, { recursive: true });
  }
});
