import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { fixtureBuildIdentity } from "./build-identity.mjs";

async function inputs(t) {
  const root = await mkdtemp(join(tmpdir(), "noise-build-identity-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const status = join(root, "status"), binary = join(root, "fixture");
  await writeFile(status, `STABLE_BITAXE_SOURCE_COMMIT ${"a".repeat(40)}\nSTABLE_BITAXE_SOURCE_DIRTY false\n`);
  await writeFile(binary, "explicit synthetic fixture binary");
  return { root, status, binary };
}

test("build identity binds exact fixture bytes and rejects ambiguous source status", async (t) => {
  const f = await inputs(t);
  const result = await fixtureBuildIdentity(f.status, f.binary);
  assert.equal(result.sourceCommit, "a".repeat(40));
  assert.equal(result.sourceDirty, false);
  assert.equal(result.fixtureSha256, createHash("sha256").update(await readFile(f.binary)).digest("hex"));
  await writeFile(f.status, `STABLE_BITAXE_SOURCE_COMMIT ${"a".repeat(40)}\nSTABLE_BITAXE_SOURCE_DIRTY false\nSTABLE_BITAXE_SOURCE_DIRTY true\n`);
  await assert.rejects(fixtureBuildIdentity(f.status, f.binary), /noise_build_status_field/u);
});

test("relative CLI invocation writes an exclusive build receipt", async (t) => {
  const f = await inputs(t), script = fileURLToPath(new URL("./build-identity.mjs", import.meta.url));
  const output = join(f.root, "receipt.json");
  const invoke = () => spawnSync(process.execPath, [basename(script), f.status, f.binary, output], { cwd: dirname(script), encoding: "utf8" });
  const first = invoke();
  assert.equal(first.status, 0, first.stderr);
  const bytes = await readFile(output);
  assert.equal(JSON.parse(bytes).schema, "noise-serial-fixture-build-v2");
  assert.notEqual(invoke().status, 0);
  assert.deepEqual(await readFile(output), bytes);
});
