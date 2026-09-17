import assert from "node:assert/strict";
import test from "node:test";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { cleanupContextFixture } from "./context-fixtures.mjs";
import { loadContext, loadEffectContext, preflight, verifyEffectInputs } from "./context.mjs";
import { proof } from "../str005-noise-serial/files.mjs";

test("v3 Channel003 remains exactly readable with old four contracts and v1 readiness", async t => {
  // Arrange / Act.
  const f = await cleanupContextFixture(t), context = f.context;
  // Assert.
  assert.equal(context.schema, "str005-v2-serial-context-v3"); assert.equal(context.hostOrdinal, 3);
  assert.equal(context.permissionSupersession, null);
  assert.deepEqual(Object.keys(context.contracts).sort(), ["amendment", "base", "cleanup", "permission"]);
  assert.equal(context.cleanupSupersession.failedRoot, resolve(f.parent, "channel-002"));
  const receipt = (await proof(f.parent, context.cleanupSupersession.receiptPath)).value;
  assert.equal(receipt.schema, "str005-v2-channel-successor-readiness-v1");
  assert.equal(Object.hasOwn(receipt, "initialAccounting"), false);
  assert.equal((await proof(f.root, "host-correction.json")).value.schema, "str005-v2-host-correction-v1");
});

test("v3 historical load survives archival without restoring any live or child effect eligibility", async t => {
  // Arrange.
  const f = await cleanupContextFixture(t);
  await f.put(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-str005-v2-serial-qualification | archived\n");
  // Act / Assert.
  assert.equal((await loadContext(f.root, { historical: true, operations: f.operations })).hostOrdinal, 3);
  await assert.rejects(loadContext(f.root, { operations: f.operations }), { code: "v2_legacy_context_read_only" });
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_legacy_context_read_only" });
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "v2_legacy_context_read_only" });
});

test("v3 Share still reads only its exact v3 Channel pair and original correction receipts", async t => {
  // Arrange / Act.
  const f = await cleanupContextFixture(t, { scope: "share" });
  // Assert.
  assert.equal(f.context.cleanupSupersession, null); assert.equal(f.context.permissionSupersession, null);
  assert.equal(f.context.predecessor.root, resolve(f.parent, "channel-003"));
  for (const file of ["permission-correction.json", "host-correction.json"])
    assert.equal((await proof(f.root, file)).sha256, (await proof(resolve(f.parent, "channel-003"), file)).sha256);
});

test("retired v3 admission cannot consume another Channel003 assignment", async t => {
  // Arrange.
  const f = await cleanupContextFixture(t, { prepare: false }), before = await readdir(f.parent);
  // Act / Assert.
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_cleanup_successor_required" });
  assert.deepEqual(await readdir(f.parent), before);
});

test("historical v3 correction remains strict after the v4 writer changes", async t => {
  // Arrange.
  const f = await cleanupContextFixture(t), path = resolve(f.root, "host-correction.json");
  const value = JSON.parse(await readFile(path)); value.command = ["node", "--test", "different.mjs"];
  await writeFile(path, JSON.stringify(value));
  // Act / Assert.
  await assert.rejects(loadContext(f.root, { historical: true, operations: f.operations }));
});
