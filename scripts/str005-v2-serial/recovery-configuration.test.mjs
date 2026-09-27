import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { promisify } from "node:util";

test("real Gate parser and transition reject direct candidate and accept recovery bootstrap", async () => {
  // Arrange: actual Gate configuration modules; simulated serial close and native reconnect.
  const here = dirname(fileURLToPath(import.meta.url));
  const gateRoot = process.argv[2] ? dirname(resolve(process.argv[2])) : resolve(here, "../../../bitaxe-turnstile-system");
  // Act
  const result = await promisify(execFile)("bun", [resolve(here, "recovery-configuration.fixture.ts"), gateRoot], { timeout: 15000 });
  // Assert
  assert.equal(result.stdout.trim(), "gate_configuration_boundary_passed");
  assert.equal(result.stderr, "");
});
