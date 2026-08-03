import assert from "node:assert/strict";
import test from "node:test";

import { internalCommandSpec } from "./contracts.generated.js";
import { allowedEnvironment, createLocalProcessPort } from "./process.js";

const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

test("process environment uses an exact secret-safe allowlist", () => {
  // Arrange
  const source = {
    PATH: "/usr/bin",
    CARGO_TARGET_DIR: "/workspace/.target",
    CARGO_UNSTABLE_AMBIENT: "must-not-pass",
    SERVICE_TOKEN: "must-not-pass",
  };

  // Act
  const environment = allowedEnvironment(source);

  // Assert
  assert.deepEqual(environment, {
    PATH: "/usr/bin",
    CARGO_TARGET_DIR: "/workspace/.target",
  });
});

test("process adapter captures stdout and stderr separately", async () => {
  // Arrange
  const processPort = createLocalProcessPort({ cwd: process.cwd(), timeoutMs: 5_000 });
  const spec = internalCommandSpec(
    nodeProgram,
    ["-e", "process.stdout.write('public'); process.stderr.write('private')"],
    (value) => value,
  );

  // Act
  const outcome = await processPort.run(spec);

  // Assert
  assert.equal(outcome.exitCode, 0, outcome.stderr);
  assert.equal(outcome.stdout, "public");
  assert.equal(outcome.stderr, "private");
  assert.equal(outcome.timedOut, false);
});

test("process adapter terminates a timed out child", async () => {
  // Arrange
  const processPort = createLocalProcessPort({ cwd: process.cwd(), timeoutMs: 50 });
  const spec = internalCommandSpec(
    nodeProgram,
    ["-e", "setInterval(() => undefined, 1_000)"],
    (value) => value,
  );

  // Act
  const outcome = await processPort.run(spec);

  // Assert
  assert.equal(outcome.timedOut, true, outcome.stderr);
  assert.notEqual(outcome.exitCode, 0);
});

test("process adapter can acquire a long-running process before awaiting it", async () => {
  // Arrange
  const processPort = createLocalProcessPort({ cwd: process.cwd(), timeoutMs: 5_000 });
  const spec = internalCommandSpec(nodeProgram, ["-e", "setTimeout(() => process.stdout.write('done'), 25)"], (value) => value);

  // Act
  const running = processPort.start(spec);
  const outcome = await running.wait();

  // Assert
  assert.equal(outcome.exitCode, 0);
  assert.equal(outcome.stdout, "done");
});
