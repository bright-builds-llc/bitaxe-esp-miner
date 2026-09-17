import assert from "node:assert/strict";
import test from "node:test";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { legacyContextFixture, permissionContextFixture } from "./context-fixtures.mjs";
import { loadContext, preflight, verifyEffectInputs } from "./context.mjs";
import { proof } from "../str005-noise-serial/files.mjs";

for (const [name, fixture] of [["v1", legacyContextFixture], ["v2", permissionContextFixture]]) {
  test(`${name} protected historical context remains readable after archival but never effect-eligible`, async t => {
    // Arrange.
    const f = await fixture(t);
    await f.put(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-str005-v2-serial-qualification | archived\n");
    // Act / Assert.
    assert.equal((await loadContext(f.root, { historical: true, operations: f.operations })).schema, `str005-v2-serial-context-${name}`);
    await assert.rejects(loadContext(f.root, { operations: f.operations }), { code: "v2_legacy_context_read_only" });
    await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "v2_legacy_context_read_only" });
  });
}

test("v2 Channel preserves its original exact permission metadata and correction snapshot", async t => {
  // Arrange / Act.
  const f = await permissionContextFixture(t), context = f.context;
  // Assert.
  assert.equal(context.hostOrdinal, 2); assert.equal(Object.hasOwn(context, "cleanupSupersession"), false);
  assert.deepEqual(Object.keys(context.contracts).sort(), ["amendment", "base", "permission"]);
  assert.deepEqual(Object.keys(context.permissionSupersession).sort(), ["failedRoot", "failedContextSha256", "closurePath", "closureSha256"].sort());
  const correction = await proof(f.root, "permission-correction.json");
  assert.equal(correction.value.gateCommit, context.gate_commit);
  assert((await proof(f.root, "preflight-inventory.json")).value.files.some(row => row.path === "permission-correction.json" && row.sha256 === correction.sha256));
  assert(!(await readdir(f.root)).includes("host-correction.json"));
});

test("v2 Share preserves its exact accepted Channel pair and inherited Gate correction", async t => {
  // Arrange / Act.
  const f = await permissionContextFixture(t, { scope: "share" });
  // Assert.
  assert.equal(f.context.hostOrdinal, 1); assert.equal(f.context.permissionSupersession, null);
  assert.equal(f.context.qualificationAttempt.ordinal, 18);
  assert.equal((await proof(f.root, "permission-correction.json")).sha256,
    (await proof(resolve(f.parent, "channel-002"), "permission-correction.json")).sha256);
  await assert.rejects(loadContext(f.root, { operations: f.operations }), { code: "v2_legacy_context_read_only" });
});

test("old permission flag cannot allocate a fresh Channel002 after the v3 policy takes effect", async t => {
  // Arrange.
  const f = await permissionContextFixture(t, { prepare: false }), before = await readdir(f.parent);
  // Act / Assert.
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_cleanup_successor_required" });
  assert.deepEqual(await readdir(f.parent), before);
});

test("historical v2 correction and closure mutations remain invalid", async t => {
  // Arrange.
  const f = await permissionContextFixture(t), path = resolve(f.root, "permission-correction.json"), original = await readFile(path);
  // Act / Assert.
  const changed = JSON.parse(original); changed.command = ["bun", "test", "unrelated.test.ts"];
  await writeFile(path, JSON.stringify(changed));
  await assert.rejects(loadContext(f.root, { historical: true, operations: f.operations }), { code: "v2_permission_correction_binding" });
  await writeFile(path, original);
  await writeFile(f.options.supersedePermission, "changed closure");
  await assert.rejects(loadContext(f.root, { historical: true, operations: f.operations }), { code: "v2_permission_closure_changed" });
});
