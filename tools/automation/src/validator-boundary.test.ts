import assert from "node:assert/strict";
import { chmod, mkdtemp, readFile, realpath, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { internalCommandSpec } from "./contracts.generated.js";
import { createLocalProcessPort } from "./process.js";

function workspaceRoot(): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? (process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd())
    : path.join(maybeRunfiles, "_main");
}

test("mining criteria validator passes one existing absolute path through Bazel", async () => {
  // Arrange
  const workspace = workspaceRoot();
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-validator-boundary-"));
  const fixture = path.join(root, "projection.json");
  const bazel = path.join(root, "bazel");
  const captured = path.join(root, "captured-path");
  await writeFile(fixture, "{}\n");
  await writeFile(bazel, [
    "#!/usr/bin/env bash",
    "set -euo pipefail",
    "test \"$#\" -eq 4",
    "test \"$1\" = run",
    "test \"$2\" = //crates/bitaxe-automation-contracts:validate_mining_criteria_evidence",
    "test \"$3\" = --",
    "case \"$4\" in /*) ;; *) exit 1 ;; esac",
    "test -f \"$4\"",
    `printf '%s\\n' \"$4\" > ${JSON.stringify(captured)}`,
  ].join("\n") + "\n");
  await chmod(bazel, 0o700);
  const processPort = createLocalProcessPort({ cwd: workspace, timeoutMs: 10_000 });
  const justfile = await readFile(path.join(workspace, "Justfile"), "utf8");
  assert.ok(justfile.includes('projection_path="$(/bin/realpath "$projection")"'));
  assert.ok(justfile.includes('validate_mining_criteria_evidence -- "$projection_path"'));

  // Act
  const outcome = await processPort.run(internalCommandSpec(
    "/bin/bash",
    [
      "-c",
      'set -euo pipefail; projection="$1"; test -f "$projection"; projection_path="$(/bin/realpath "$projection")"; bazel run //crates/bitaxe-automation-contracts:validate_mining_criteria_evidence -- "$projection_path"',
      "validator-boundary-test",
      path.relative(workspace, fixture),
    ],
    (value) => value,
    { PATH: `${root}:${process.env["PATH"] ?? ""}` },
  ));

  // Assert
  assert.equal(outcome.exitCode, 0, outcome.stderr);
  assert.equal((await readFile(captured, "utf8")).trim(), await realpath(fixture));
});
