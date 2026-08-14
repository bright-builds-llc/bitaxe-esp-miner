import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  ApiCommandEffectsError,
  captureApiCommandEffects,
  type ApiCommandEffectsOptions,
} from "./api-command-effects.js";
import { toolProgram } from "./cli-tools.js";
import { createLocalProcessPort } from "./process.js";

const discardCheckpoint = () => undefined;

async function fixture(): Promise<{ root: string; options: ApiCommandEffectsOptions }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "api-command-effects-process-"));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const wifi = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: "a".repeat(40),
    reference_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64),
  }));
  await writeFile(wifi, "{}\n");
  return {
    root,
    options: {
      privateRoot: "scratch/attempt-001",
      packageManifest: manifest,
      wifiCredentials: wifi,
      port: "/dev/private-sensitive-port",
      projection: path.join(root, "docs", "api-command-effects.json"),
      durationSeconds: 600,
    },
  };
}

test("the deployed fixture executable crosses the sanitized real-process boundary", {
  skip: process.platform !== "darwin",
}, async () => {
  // Arrange
  const value = await fixture();
  const processPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 20_000 });

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    processPort,
    toolProgram(value.root, "scripts/api_command_effects_stratum_pool_/api_command_effects_stratum_pool"),
    "/usr/bin/false",
    "/usr/bin/false",
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "hardware_blocked");
  const attempt = path.join(value.root, "scratch", "attempt-001");
  assert.equal((await stat(path.join(attempt, "fixture-ready.private.json"))).mode & 0o777, 0o600);
  assert.equal((await stat(path.join(attempt, "fixture-report.private.json"))).mode & 0o777, 0o600);
  const diagnosticPath = path.join(attempt, "fixture-process.private.json");
  assert.equal((await stat(diagnosticPath)).mode & 0o777, 0o600);
  const diagnostic = JSON.parse(await readFile(diagnosticPath, "utf8")) as Record<string, unknown>;
  assert.equal(diagnostic["terminal_category"], "complete");
  assert.equal(diagnostic["raw_output_persisted"], false);
  assert(!JSON.stringify(diagnostic).includes("192.0.2"));
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});

test("an early real child exit is process_failed instead of a readiness timeout", {
  skip: process.platform !== "darwin",
}, async () => {
  // Arrange
  const value = await fixture();
  const processPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 20_000 });
  const startedAt = Date.now();

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    processPort,
    "/usr/bin/false",
    "/usr/bin/false",
    "/usr/bin/false",
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "process_failed");
  assert(Date.now() - startedAt < 5_000);
  const diagnosticPath = path.join(value.root, "scratch", "attempt-001", "fixture-process.private.json");
  assert.equal((await stat(diagnosticPath)).mode & 0o777, 0o600);
  const diagnostic = JSON.parse(await readFile(diagnosticPath, "utf8")) as Record<string, unknown>;
  assert.equal(diagnostic["terminal_category"], "nonzero_exit");
  assert(!JSON.stringify(diagnostic).includes("/usr/bin/false"));
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});

test("a running real child without readiness times out and receives cleanup", {
  skip: process.platform !== "darwin",
}, async () => {
  // Arrange
  const value = await fixture();
  const silentFixture = path.join(value.root, "silent-fixture.sh");
  await writeFile(silentFixture, [
    "#!/bin/sh",
    "stop_file=''",
    "while [ \"$#\" -gt 0 ]; do",
    "  if [ \"$1\" = '--stop-file' ]; then stop_file=\"$2\"; fi",
    "  shift 2",
    "done",
    "while [ ! -f \"$stop_file\" ]; do sleep 0.05; done",
    "",
  ].join("\n"), { mode: 0o700 });
  await chmod(silentFixture, 0o700);
  const processPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 20_000 });

  // Act
  const error = await captureApiCommandEffects(
    value.root,
    value.options,
    processPort,
    silentFixture,
    "/usr/bin/false",
    "/usr/bin/false",
    discardCheckpoint,
  ).then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "timeout");
  const attempt = path.join(value.root, "scratch", "attempt-001");
  assert.equal((await stat(path.join(attempt, "fixture.stop.private"))).mode & 0o777, 0o600);
  const diagnostic = JSON.parse(
    await readFile(path.join(attempt, "fixture-process.private.json"), "utf8"),
  ) as Record<string, unknown>;
  assert.equal(diagnostic["terminal_category"], "complete");
  assert.equal(diagnostic["raw_output_persisted"], false);
  await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
});
