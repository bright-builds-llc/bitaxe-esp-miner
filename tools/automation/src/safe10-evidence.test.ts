import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { toolProgram } from "./cli-tools.js";
import {
  projectSafe10Evidence,
  Safe10EvidenceError,
  type Safe10EvidenceOptions,
} from "./safe10-evidence.js";
import {
  safe10EvaluatorFragments,
  safe10CurrentInventory,
  safe10ProductionFragments,
  safe10ReferenceFragments,
} from "./safe10-source-inventory.js";
import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome } from "./process.js";

const workspace = process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd();
const repositoryRoot = process.env["RUNFILES_DIR"] === undefined
  ? workspace
  : path.join(process.env["RUNFILES_DIR"], "_main");
const attemptSourceCommit = "60a56d4935ced15eeb5ec6950b1ad4ea35fdf223";
const currentSourceCommit = "b".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const plan = "docs/parity/work-plans/20260818T132739Z-SAFE-10/PLAN.md";
const attemptPlan = "docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md";
const attemptClosure = "docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md";

type Fixture = Readonly<{
  root: string;
  options: Safe10EvidenceOptions;
  sourceDocuments: ReadonlyMap<string, string>;
  validatorProgram: string;
}>;

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

async function writeProtected(candidate: string, document: string): Promise<void> {
  await writeFile(candidate, document, { mode: 0o600 });
  await chmod(candidate, 0o600);
}

async function fixture(
  name: string,
  options: Readonly<{ powerFresh?: boolean }> = {},
): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-safe10-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), 'module(name = "fixture")\n');
  const sourceDocuments = new Map<string, string>();
  for (const [relative, fragments] of [
    ...safe10ProductionFragments,
    ...safe10EvaluatorFragments,
    ...safe10ReferenceFragments,
  ] as const) {
    const document = `${fragments.join("\n")}\n`;
    sourceDocuments.set(relative, document);
    const candidate = path.join(root, relative);
    await mkdir(path.dirname(candidate), { recursive: true });
    await writeFile(candidate, document);
  }
  for (const relative of [plan, attemptPlan, attemptClosure]) {
    const candidate = path.join(root, relative);
    await mkdir(path.dirname(candidate), { recursive: true });
    await writeFile(candidate, await readFile(path.join(repositoryRoot, relative), "utf8"));
  }
  await writeFile(path.join(root, "TASKS.md"), [
    "## Active",
    "### task-parity-safe10-prerequisite-readiness | fixture",
    `Plan: \`${plan}\`.`,
  ].join("\n"));

  const attemptRoot = path.join(root, "scratch/stat003-scoreboard/attempt-003");
  const campaignRoot = path.join(attemptRoot, "campaign");
  const wrapperRoot = path.join(root, "scratch/stat003-scoreboard/wrapper-003");
  await mkdir(campaignRoot, { recursive: true, mode: 0o700 });
  await mkdir(wrapperRoot, { recursive: true, mode: 0o700 });
  await chmod(path.join(root, "scratch/stat003-scoreboard"), 0o700);
  await chmod(attemptRoot, 0o700);
  await chmod(campaignRoot, 0o700);
  await chmod(wrapperRoot, 0o700);
  await writeProtected(
    path.join(wrapperRoot, "detector.stdout"),
    "espflash_version: 4.3.0\nport: /dev/cu.usbmodem101\nusb_session: ready\n",
  );
  const observations = `${JSON.stringify({
    schema: "mining-campaign-observations-v4",
    terminal_marker: {
      readiness_transition: {
        current_blocker: "none",
        session_phase: "running_primary",
        hardware_state: "ready",
        safety_sample: "fresh",
        observation_epoch: "advanced",
        pending_observation_recovered: false,
      },
    },
  })}\n`;
  const network = `${JSON.stringify({
    schema: "mining-campaign-network-continuity-v12",
    status: "accepted",
    correlation_failure: "none",
    watchdog_failure: "none",
    required_window_count: 20,
    covered_window_count: 20,
    work_renewal_valid: true,
    active_state_valid: true,
    safety_valid: true,
    watchdog_valid: true,
    terminal_http_valid: true,
    terminal_websocket_valid: true,
    terminal_pool_persisted: true,
    final_terminal_consumed: true,
    serial_finished_observed: true,
  })}\n`;
  const diagnostics = `${JSON.stringify({
    runtime_attestation_mixed_reset_reason: "none",
    panic_signature: "none",
    panic_signature_count: 0,
  })}\n`;
  const required = {
    power_watts: true,
    bus_voltage_volts: true,
    current_amps: true,
    chip_temp_celsius: true,
    vr_temp_celsius: false,
    fan_rpm: true,
  };
  const result = `${JSON.stringify({
    schema: "mining-campaign-result-v16",
    status: "accepted",
    stage: "live-share",
    profile: "conservative",
    duration_seconds: 600,
    active_ms: 600_000,
    runtime_identity: "trusted",
    network_status: "accepted",
    safety: "fresh",
    safe_stop: "confirmed",
    usb_cleanup: "ready",
    qualified_candidate_count: 1,
    submit_outcome: "accepted",
    fresh_observation_count: 5,
    observation_requirements: required,
    observation_freshness: { ...required, power_watts: options.powerFresh ?? true },
    network_continuity_sha256: sha256(network),
    observations_sha256: sha256(observations),
    diagnostics_sha256: sha256(diagnostics),
  })}\n`;
  for (const [file, document] of [
    ["campaign-result.json", result],
    ["campaign-result.sha256", `${sha256(result)}\n`],
    ["campaign-network.private.json", network],
    ["campaign-observations.private.json", observations],
    ["campaign-diagnostics.private.json", diagnostics],
  ] as const) {
    await writeProtected(path.join(campaignRoot, file), document);
  }
  return {
    root,
    sourceDocuments,
    validatorProgram: toolProgram(workspace, "crates/bitaxe-automation-contracts/validate_safe10_evidence"),
    options: {
      attemptRoot: "scratch/stat003-scoreboard/attempt-003",
      detectorOutput: "scratch/stat003-scoreboard/wrapper-003/detector.stdout",
      attemptPlan,
      attemptClosure,
      projection: "docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json",
    },
  };
}

function outcome(stdout = ""): ProcessOutcome {
  return { exitCode: 0, stdout, stderr: "", timedOut: false };
}

function processPort(value: Fixture, drift = false) {
  const local = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });
  return createFakeProcessPort(async (spec, maybeLifetime) => {
    if (spec.program === value.validatorProgram) return local.run(spec, maybeLifetime);
    const args = spec.args;
    if (args[0] === "rev-parse" && args[1] === "HEAD") return outcome(`${currentSourceCommit}\n`);
    if (args[0] === "rev-parse" && args[1] === "origin/main") return outcome(`${currentSourceCommit}\n`);
    if (args[0] === "status") return outcome();
    if (args[0] === "-C") return outcome(`${referenceCommit}\n`);
    if (args[0] === "show") {
      const relative = args[1]?.slice(attemptSourceCommit.length + 1);
      const document = relative === undefined ? undefined : value.sourceDocuments.get(relative);
      if (document === undefined) return { ...outcome(), exitCode: 1 };
      return outcome(drift && relative === safe10ProductionFragments.keys().next().value
        ? `${document}drift\n`
        : document);
    }
    return { ...outcome(), exitCode: 2 };
  });
}

async function captureError(promise: Promise<unknown>): Promise<Safe10EvidenceError> {
  try {
    await promise;
    assert.fail("expected SAFE-10 evidence failure");
  } catch (error) {
    assert.ok(error instanceof Safe10EvidenceError);
    return error;
  }
}

test("complete protected attempt publishes independently validated SAFE-10 evidence", async () => {
  // Arrange
  const value = await fixture("accepted");
  try {
    // Act
    const evidence = await projectSafe10Evidence(
      value.root,
      value.options,
      processPort(value),
      "git",
      value.validatorProgram,
    );

    // Assert
    assert.equal(evidence.prerequisites.fresh_observation_count, 5);
    assert.equal(evidence.source.source_path_count, 19);
    const projection = path.join(value.root, value.options.projection);
    assert.equal((await stat(projection)).mode & 0o777, 0o644);
    assert.doesNotMatch(
      await readFile(projection, "utf8"),
      /private_|lease_id|boot_session|endpoint|sensor_value|serial_(?:port|log|bytes)|http_body/u,
    );
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("checked-in SAFE-10 source inventory is complete", async () => {
  // Arrange
  const root = process.env["RUNFILES_DIR"] === undefined
    ? workspace
    : path.join(process.env["RUNFILES_DIR"], "_main");

  // Act
  const inventory = await safe10CurrentInventory(root);

  // Assert
  assert.equal(inventory.pathCount, 19);
});

for (const [name, before, after] of [
  ["nonzero-fan", "*sample.value() > 0", "*sample.value() >= 0"],
  ["freshness-conjunction", "&& observations", "|| observations"],
] as const) {
  test(`SAFE-10 source admission rejects weakened ${name}`, async () => {
    // Arrange
    const value = await fixture(name);
    const relative = "firmware/bitaxe/src/production_mining_session.rs";
    const source = value.sourceDocuments.get(relative);
    assert.ok(source !== undefined);
    assert.ok(source.includes(before));
    await writeFile(path.join(value.root, relative), source.replace(before, after));

    // Act / Assert
    try { await assert.rejects(safe10CurrentInventory(value.root), /source semantics are invalid/u); }
    finally { await rm(value.root, { recursive: true }); }
  });
}

test("prerequisite or attempt-source drift withholds SAFE-10 projection", async () => {
  for (const [name, powerFresh, drift] of [
    ["prerequisite", false, false],
    ["source", true, true],
  ] as const) {
    // Arrange
    const value = await fixture(name, { powerFresh });
    try {
      // Act
      const error = await captureError(projectSafe10Evidence(
        value.root,
        value.options,
        processPort(value, drift),
        "git",
        value.validatorProgram,
      ));

      // Assert
      assert.equal(error.category, "evidence_invalid");
      await assert.rejects(readFile(path.join(value.root, value.options.projection), "utf8"), {
        code: "ENOENT",
      });
    } finally {
      await rm(value.root, { recursive: true });
    }
  }
});
