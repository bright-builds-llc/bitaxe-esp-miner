import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { checkHostCorrection, validateHostCorrection, HOST_CORRECTION_COMMAND, HOST_CORRECTION_CHECKER } from "./host-correction.mjs";
import { CLEANUP_AMENDMENT_PATH, sha256 } from "./values.mjs";
const repo = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
async function setup(t) {
  const root = await mkdtemp(resolve(tmpdir(), "v2-host-correction-")); t.after(() => rm(root, { recursive: true }));
  const context = { firmware_root: resolve(root, "firmware"), gate_root: resolve(root, "gate"),
    firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40) };
  const put = async (path, bytes) => { await mkdir(dirname(path), { recursive: true, mode: 0o700 }); await writeFile(path, bytes, { mode: 0o600 }); };
  const bundle = `${context.gate_commit} synthetic bundle`, checker = await readFile(resolve(repo, HOST_CORRECTION_CHECKER));
  await put(resolve(context.firmware_root, "MODULE.bazel"), `strip_prefix = "bitaxe-turnstile-system-${context.gate_commit}"`);
  await put(resolve(context.firmware_root, CLEANUP_AMENDMENT_PATH), await readFile(resolve(repo, CLEANUP_AMENDMENT_PATH)));
  await put(resolve(context.firmware_root, HOST_CORRECTION_CHECKER), checker);
  const bundlePath = resolve(context.gate_root, "dist/worker-serial-acceptance/worker-serial-acceptance.js");
  await put(bundlePath, bundle); context.gate_bundle_sha256 = sha256(bundle);
  context.evaluator = [{ path: HOST_CORRECTION_CHECKER, sha256: sha256(checker) }];
  return { context, bundlePath, operations: { hostPlatform: "darwin", cleanPushed() {}, hostCorrectionNow: () => 100,
    spawnSync: () => ({ status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) }) } };
}
test("host correction runs the exact bounded command with filtered environment and no output receipt", async t => {
  // Arrange
  const f = await setup(t), output = Buffer.from("private-test-output"); let calls = 0;
  f.operations.spawnSync = (program, args, options) => {
    calls++; assert.deepEqual([program, ...args], HOST_CORRECTION_COMMAND);
    assert.equal(options.cwd, f.context.firmware_root); assert.equal(options.timeout, 180000); assert.equal(options.maxBuffer, 65536);
    assert.deepEqual(Object.keys(options.env).sort(), ["LANG", "LC_ALL", "PATH"]);
    return { status: 0, signal: null, stdout: output, stderr: Buffer.alloc(0) };
  };
  // Act
  const receipt = await checkHostCorrection(f.context, f.operations);
  // Assert
  assert.equal(calls, 1); assert.equal(validateHostCorrection(receipt, f.context), receipt);
  assert(!JSON.stringify(receipt).includes("private-test-output")); assert(output.every(byte => byte === 0));
});
for (const [name, change] of [["exit", { status: 1 }], ["signal", { signal: "SIGKILL" }],
  ["spawn", { error: Error("private-detail") }], ["combined output", { stdout: Buffer.alloc(40000), stderr: Buffer.alloc(40000) }]])
  test(`host correction rejects ${name} without returning a passing verdict`, async t => {
    // Arrange
    const f = await setup(t); f.operations.spawnSync = () => ({ status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0), ...change });
    // Act / Assert
    await assert.rejects(checkHostCorrection(f.context, f.operations), { code: "v2_host_correction_failed" });
  });
test("elapsed timeout and reversed clocks cannot satisfy the host bound", async t => {
  // Arrange
  const f = await setup(t);
  for (const end of [180101, 99]) {
    let first = true; f.operations.hostCorrectionNow = () => { if (first) { first = false; return 100; } return end; };
    // Act / Assert
    await assert.rejects(checkHostCorrection(f.context, f.operations), { code: "v2_host_correction_failed" });
  }
});
test("unsupported platform rejects before reading context or running skipped tests", async () => {
  // Arrange / Act / Assert
  await assert.rejects(checkHostCorrection({}, { hostPlatform: "linux", spawnSync() { assert.fail("no command"); } }), { code: "v2_host_platform_unsupported" });
});
test("stale bundle and source mutation cannot gain a host correction receipt", async t => {
  // Arrange
  const f = await setup(t);
  f.operations.spawnSync = () => { writeFileSync(f.bundlePath, "changed"); return { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) }; };
  // Act / Assert
  await assert.rejects(checkHostCorrection(f.context, f.operations), { code: "v2_host_correction_source" });
  f.context.gate_bundle_sha256 = sha256("changed");
  await assert.rejects(checkHostCorrection(f.context, f.operations), { code: "v2_host_correction_source" });
});
test("retained host correction has a closed source-bound schema", async t => {
  // Arrange
  const f = await setup(t), receipt = await checkHostCorrection(f.context, f.operations);
  // Act / Assert
  for (const change of [{ command: ["node", "--test"] }, { exitCode: 1 }, { signal: "SIGTERM" }, { elapsedMs: 180001 },
    { checkerSha256: "0".repeat(64) }, { firmwareCommit: null }, { gateBundleSha256: "invalid" }, { rawOutput: "forbidden" }])
    assert.throws(() => validateHostCorrection({ ...receipt, ...change }, f.context));
});
