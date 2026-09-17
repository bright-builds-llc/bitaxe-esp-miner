import assert from "node:assert/strict";
import test from "node:test";
import { readFile } from "node:fs/promises";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { contextFixture } from "./context-fixtures.mjs";
import { checkPermissionCorrection, validatePermissionCorrection, CORRECTION_COMMAND, CORRECTION_CHECKER } from "./permission-correction.mjs";
import { sha256 } from "./values.mjs";

async function setup(t) {
  const f = await contextFixture(t, { prepare: false });
  const source = { firmware_root: f.options.firmwareRoot, gate_root: f.options.gateRoot,
    firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40),
    gate_bundle_sha256: sha256(await readFile(resolve(f.options.gateRoot, "dist/worker-serial-acceptance/worker-serial-acceptance.js"))) };
  return { ...f, source };
}
test("fixed correction command binds publication and discards all raw output", async t => {
  // Arrange
  const f = await setup(t), stdout = Buffer.from("synthetic private test output"), stderr = Buffer.from("synthetic stderr");
  const calls = []; let time = 100;
  f.operations.correctionNow = () => time;
  f.operations.spawnSync = (program, args, options) => {
    calls.push([program, ...args]); assert.equal(options.cwd, f.source.gate_root);
    assert.equal(options.timeout, 30000); assert.equal(options.killSignal, "SIGKILL"); assert.equal(options.maxBuffer, 65536);
    assert.deepEqual(options.stdio, ["ignore", "pipe", "pipe"]);
    assert.deepEqual(Object.keys(options.env).sort(), ["LANG", "LC_ALL", "PATH"]); time = 150;
    return { status: 0, signal: null, stdout, stderr };
  };
  // Act
  const receipt = await checkPermissionCorrection(f.source, f.operations);
  // Assert
  assert.deepEqual(calls, [CORRECTION_COMMAND]); assert.equal(receipt.elapsedMs, 50);
  assert(!JSON.stringify(receipt).includes("private test output")); assert(stdout.every(byte => byte === 0)); assert(stderr.every(byte => byte === 0));
  assert.deepEqual(Object.keys(receipt).sort(), ["schema", "firmwareCommit", "gateCommit", "gateBundleSha256", "checkerSha256", "command", "exitCode", "signal", "elapsedMs"].sort());
});
for (const [name, change] of [
  ["nonzero exit", { status: 1 }], ["signal", { signal: "SIGTERM" }],
  ["timeout", { error: Object.assign(Error("private exception"), { code: "ETIMEDOUT" }) }],
  ["output bound", { stdout: Buffer.alloc(65537) }],
]) test(`correction ${name} fails with a closed code before authorization`, async t => {
  // Arrange
  const f = await setup(t);
  f.operations.spawnSync = () => ({ status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0), ...change });
  // Act / Assert
  await assert.rejects(checkPermissionCorrection(f.source, f.operations), { code: "v2_permission_correction_failed" });
});
test("elapsed timeout and reversed host clock cannot become a passed correction", async t => {
  // Arrange
  const f = await setup(t);
  for (const end of [30101, 99]) {
    let first = true; f.operations.correctionNow = () => { if (first) { first = false; return 100; } return end; };
    // Act / Assert
    await assert.rejects(checkPermissionCorrection(f.source, f.operations), { code: "v2_permission_correction_failed" });
  }
});
test("publication changes during a passing command invalidate its receipt", async t => {
  // Arrange
  const f = await setup(t);
  f.operations.spawnSync = () => {
    writeFileSync(resolve(f.source.gate_root, "dist/worker-serial-acceptance/worker-serial-acceptance.js"), "changed during command");
    return { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) };
  };
  // Act / Assert
  await assert.rejects(checkPermissionCorrection(f.source, f.operations), { code: "v2_permission_correction_source" });
});
test("historical receipt validation binds exact command and checker without rerunning tests", async t => {
  // Arrange
  const f = await setup(t), receipt = await checkPermissionCorrection(f.source, f.operations);
  const context = { ...f.source, evaluator: [{ path: CORRECTION_CHECKER, sha256: receipt.checkerSha256 }] };
  // Act / Assert
  assert.equal(validatePermissionCorrection(receipt, context), receipt);
  for (const changed of [{ ...receipt, command: ["bun", "test"] }, { ...receipt, checkerSha256: "0".repeat(64) },
    { ...receipt, elapsedMs: 30001 }, { ...receipt, signal: "SIGTERM" }, { ...receipt, extra: true }])
    assert.throws(() => validatePermissionCorrection(changed, context));
});

test("a self-consistent stale bundle hash cannot bind corrected published source", async t => {
  // Arrange
  const f = await setup(t), stale = "old ignored bundle without the admitted Gate commit";
  writeFileSync(resolve(f.source.gate_root, "dist/worker-serial-acceptance/worker-serial-acceptance.js"), stale);
  f.source.gate_bundle_sha256 = sha256(stale);
  f.operations.spawnSync = () => { assert.fail("stale compiled bundle must reject before tests"); };
  // Act / Assert
  await assert.rejects(checkPermissionCorrection(f.source, f.operations), { code: "v2_permission_correction_source" });
});

test("closed correction receipt validates digest types even without a live source check", async t => {
  // Arrange
  const f = await setup(t), receipt = await checkPermissionCorrection(f.source, f.operations);
  // Act / Assert
  for (const change of [{ firmwareCommit: null }, { gateCommit: "short" }, { gateBundleSha256: "not-a-digest" }]) {
    const changed = { ...receipt, ...change };
    const context = { firmware_commit: changed.firmwareCommit, gate_commit: changed.gateCommit,
      gate_bundle_sha256: changed.gateBundleSha256, evaluator: [{ path: CORRECTION_CHECKER, sha256: changed.checkerSha256 }] };
    assert.throws(() => validatePermissionCorrection(changed, context));
  }
});
