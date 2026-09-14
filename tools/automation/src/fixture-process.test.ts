import assert from "node:assert/strict";
import { mkdtemp, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { internalCommandSpec } from "./contracts.generated.js";
import { createFixtureProcessPort } from "./fixture-process.test-support.js";

async function fixture(source: string, interpreter: "node" | "shell" = "node", timeoutMs = 5_000) {
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-interpreted-fixture-"));
  const script = path.join(root, "fixture-source.mjs");
  await writeFile(script, source, { mode: 0o600 });
  const port = createFixtureProcessPort({ cwd: root, timeoutMs }, { [script]: interpreter });
  return { script, port };
}

test("explicit Node fixture runs non-executable source with unchanged arguments and environment", async () => {
  // Arrange
  const value = await fixture("process.stdout.write(JSON.stringify([process.argv.slice(2), process.env.PHASE36_EFFECT_OPERATION])); process.stderr.write('child-stderr');");
  const spec = internalCommandSpec(value.script, ["a b", "--literal"], (input) => input, { PHASE36_EFFECT_OPERATION: "fixture-operation" });
  // Act
  const outcome = await value.port.run(spec);
  // Assert
  assert.equal(outcome.exitCode, 0, outcome.stderr);
  assert.deepEqual(JSON.parse(outcome.stdout), [["a b", "--literal"], "fixture-operation"]);
  assert.equal(outcome.stderr, "child-stderr");
});

test("explicit shell fixture does not infer interpreter from its mjs extension", async () => {
  const value = await fixture("printf '%s' \"$1\"\n", "shell");
  const outcome = await value.port.run(internalCommandSpec(value.script, ["shell-output"], (input) => input));
  assert.equal(outcome.stdout, "shell-output");
  assert.equal(outcome.exitCode, 0, outcome.stderr);
});

test("interpreted fixture preserves a real nonzero child exit", async () => {
  const value = await fixture("process.exitCode = 17;");
  const outcome = await value.port.run(internalCommandSpec(value.script, [], (input) => input));
  assert.equal(outcome.exitCode, 17);
  assert.equal(outcome.timedOut, false);
});

test("interpreted fixture preserves per-child timeout override", async () => {
  const value = await fixture("setInterval(() => {}, 1000);");
  const outcome = await value.port.run(internalCommandSpec(value.script, [], (input) => input), 50);
  assert.equal(outcome.timedOut, true);
  assert.notEqual(outcome.exitCode, 0);
});

test("interpreted fixture preserves operator-gated lifetime", async () => {
  const value = await fixture("setTimeout(() => process.stdout.write('completed'), 60);", "node", 20);
  const outcome = await value.port.run(internalCommandSpec(value.script, [], (input) => input), "operator-gated");
  assert.equal(outcome.stdout, "completed");
  assert.equal(outcome.timedOut, false);
});

test("unregistered native executable remains unchanged", async () => {
  const value = await fixture("throw new Error('must not run');");
  const outcome = await value.port.run(internalCommandSpec("/usr/bin/printf", ["%s", "native output"], (input) => input));
  assert.equal(outcome.stdout, "native output");
  assert.equal(outcome.exitCode, 0, outcome.stderr);
});
