import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { toolProgram } from "./cli-tools.js";
import {
  Cfg07EvidenceError,
  projectCfg07Evidence,
  type Cfg07EvidenceOptions,
} from "./cfg07-evidence.js";
import {
  cfg07CurrentInventory,
  cfg07EvaluatorFragments,
  cfg07ProductionFragments,
  cfg07ReferenceFragments,
} from "./cfg07-source-inventory.js";
import { parseInvocation } from "./invocation.js";
import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome } from "./process.js";

const workspace = process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd();
const repositoryRoot = process.env["RUNFILES_DIR"] === undefined
  ? workspace
  : path.join(process.env["RUNFILES_DIR"], "_main");
const attemptSourceCommit = "60a56d4935ced15eeb5ec6950b1ad4ea35fdf223";
const currentSourceCommit = "b".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const plan = "docs/parity/work-plans/20260818T150603Z-CFG-07/PLAN.md";
const safe10Projection =
  "docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json";
const attemptPlan = "docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md";
const attemptClosure = "docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md";

type Fixture = Readonly<{
  root: string;
  options: Cfg07EvidenceOptions;
  sourceDocuments: ReadonlyMap<string, string>;
  safe10ValidatorProgram: string;
  cfg07ValidatorProgram: string;
}>;

async function fixture(name: string, tamperSafe10 = false): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-cfg07-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), 'module(name = "fixture")\n');
  const sourceDocuments = new Map<string, string>();
  for (const [relative, fragments] of [
    ...cfg07ProductionFragments,
    ...cfg07EvaluatorFragments,
    ...cfg07ReferenceFragments,
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
  const safe10Candidate = path.join(root, safe10Projection);
  await mkdir(path.dirname(safe10Candidate), { recursive: true });
  const safe10 = JSON.parse(
    await readFile(path.join(repositoryRoot, safe10Projection), "utf8"),
  ) as Record<string, unknown>;
  if (tamperSafe10) {
    const prerequisites = safe10["prerequisites"] as Record<string, unknown>;
    prerequisites["accepted_submit_observed"] = false;
  }
  await writeFile(safe10Candidate, `${JSON.stringify(safe10, undefined, 2)}\n`);
  await writeFile(path.join(root, "TASKS.md"), [
    "## Active",
    "### task-parity-cfg07-runtime-credentials | fixture",
    `Plan: \`${plan}\`.`,
  ].join("\n"));
  return {
    root,
    sourceDocuments,
    safe10ValidatorProgram: toolProgram(
      workspace,
      "crates/bitaxe-automation-contracts/validate_safe10_evidence",
    ),
    cfg07ValidatorProgram: toolProgram(
      workspace,
      "crates/bitaxe-automation-contracts/validate_cfg07_evidence",
    ),
    options: {
      safe10Projection,
      attemptPlan,
      attemptClosure,
      projection:
        "docs/parity/evidence/cfg07-runtime-credentials/runtime-credentials-projection.json",
    },
  };
}

function outcome(stdout = ""): ProcessOutcome {
  return { exitCode: 0, stdout, stderr: "", timedOut: false };
}

function processPort(value: Fixture, drift = false) {
  const local = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });
  return createFakeProcessPort(async (spec, maybeLifetime) => {
    if (spec.program === value.safe10ValidatorProgram
      || spec.program === value.cfg07ValidatorProgram) {
      return local.run(spec, maybeLifetime);
    }
    const args = spec.args;
    if (args[0] === "rev-parse" && args[1] === "HEAD") return outcome(`${currentSourceCommit}\n`);
    if (args[0] === "rev-parse" && args[1] === "origin/main") return outcome(`${currentSourceCommit}\n`);
    if (args[0] === "status") return outcome();
    if (args[0] === "-C") return outcome(`${referenceCommit}\n`);
    if (args[0] === "show") {
      const relative = args[1]?.slice(attemptSourceCommit.length + 1);
      const document = relative === undefined ? undefined : value.sourceDocuments.get(relative);
      if (document === undefined) return { ...outcome(), exitCode: 1 };
      if (drift && relative === cfg07ProductionFragments.keys().next().value) {
        return outcome("credential semantics removed\n");
      }
      return outcome(document);
    }
    return { ...outcome(), exitCode: 2 };
  });
}

async function captureError(promise: Promise<unknown>): Promise<Cfg07EvidenceError> {
  try {
    await promise;
    assert.fail("expected CFG-07 evidence failure");
  } catch (error) {
    assert.ok(error instanceof Cfg07EvidenceError);
    return error;
  }
}

test("complete public same-chain proof publishes independently validated CFG-07 evidence", async () => {
  // Arrange
  const value = await fixture("accepted");
  try {
    // Act
    const evidence = await projectCfg07Evidence(
      value.root,
      value.options,
      processPort(value),
      "git",
      value.safe10ValidatorProgram,
      value.cfg07ValidatorProgram,
    );

    // Assert
    assert.equal(evidence.source.source_path_count, 17);
    assert.equal(evidence.credentials.runtime_credentials_input, "local-owner-supplied");
    assert.equal(evidence.credentials.credential_contents_read_by_projector, false);
    const projection = path.join(value.root, value.options.projection);
    assert.equal((await stat(projection)).mode & 0o777, 0o644);
    assert.doesNotMatch(
      await readFile(projection, "utf8"),
      /credential_path|pool_url|pool_user|pool_password|wifi_ssid|wifi_pass|endpoint|device_url/u,
    );
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("checked-in CFG-07 source inventory is complete", async () => {
  // Arrange
  // Act
  const inventory = await cfg07CurrentInventory(repositoryRoot);

  // Assert
  assert.equal(inventory.pathCount, 17);
});

test("invalid live proof or attempt-source drift withholds CFG-07 projection", async () => {
  for (const [name, tamper, drift] of [
    ["live-proof", true, false],
    ["source", false, true],
  ] as const) {
    // Arrange
    const value = await fixture(name, tamper);
    try {
      // Act
      const error = await captureError(projectCfg07Evidence(
        value.root,
        value.options,
        processPort(value, drift),
        "git",
        value.safe10ValidatorProgram,
        value.cfg07ValidatorProgram,
      ));

      // Assert
      assert.equal(error.category, "evidence_invalid");
      await assert.rejects(stat(path.join(value.root, value.options.projection)));
    } finally {
      await rm(value.root, { recursive: true });
    }
  }
});

test("CFG-07 invocation accepts only committed public evidence paths", () => {
  // Arrange
  const args = [
    "project-cfg07-evidence",
    "--safe10-projection", safe10Projection,
    "--attempt-plan", attemptPlan,
    "--attempt-closure", attemptClosure,
    "--projection", "docs/projection.json",
  ];

  // Act / Assert
  assert.equal(parseInvocation(args).command, "project-cfg07-evidence");
  assert.throws(() => parseInvocation([...args, "--wifi-credentials", "private.json"]));
  assert.throws(() => parseInvocation(args.slice(0, -2)));
});
