import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { authorityCall } from "../fixed-usb-qualification/authority.mjs";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";

async function signer(t, body) {
  const root = await mkdtemp(join(tmpdir(), "v2-authority-")); t.after(() => rm(root, { recursive: true }));
  await mkdir(join(root, "scripts"));
  await writeFile(join(root, "scripts/worker-development-authority.ts"), body);
  return root;
}

test("filtered signer environment and actual exit are observed without persisting input", async t => {
  // Arrange
  const root = await signer(t, 'process.stdin.resume(); process.stdin.on("end", () => process.stdout.write(JSON.stringify({filtered: process.env.V2_TEST_SENTINEL === undefined})));');
  const previous = process.env.V2_TEST_SENTINEL; process.env.V2_TEST_SENTINEL = "synthetic-private-value";
  t.after(() => { if (previous === undefined) delete process.env.V2_TEST_SENTINEL; else process.env.V2_TEST_SENTINEL = previous; });
  let receipt;
  // Act
  const result = await authorityCall(root, root, "sign-start", { synthetic: "private-input" }, process.execPath,
    { maybeEnvironment: { PATH: "/usr/bin:/bin", ...nodeRuntimeEnvironment() }, maybeObserveExit: value => { receipt = value; } });
  // Assert
  assert.equal(result.filtered, true); assert.equal(receipt.code, 0); assert.equal(receipt.signal, null);
  assert.equal(receipt.inputFailed, false); assert.equal(receipt.overflow, false); assert.ok(receipt.pid > 0);
  assert.equal(JSON.stringify(receipt).includes("private-input"), false);
});

test("legacy authority callers retain their existing inherited environment and output", async t => {
  const root = await signer(t, 'process.stdin.resume(); process.stdin.on("end", () => process.stdout.write(JSON.stringify({ inherited: process.env.PATH !== undefined })));');
  assert.deepEqual(await authorityCall(root, root, "public-trust", undefined, process.execPath), { inherited: true });
});

test("failed exit evidence prevents a successful authorization from escaping", async t => {
  const root = await signer(t, 'process.stdin.resume(); process.stdin.on("end", () => process.stdout.write("{}"));');
  await assert.rejects(authorityCall(root, root, "public-trust", undefined, process.execPath,
    { maybeObserveExit: () => { throw Error("synthetic collector failure"); } }), { code: "authority_exit_evidence" });
});
