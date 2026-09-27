import assert from "node:assert/strict";
import test from "node:test";
import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { contextFixture, installContextFixture } from "./context-fixtures.mjs";
import { loadContext, loadOperatorContext, preflight, verifyEffectInputs } from "./context.mjs";
import { proof } from "../str005-noise-serial/files.mjs";

for (const scope of ["channel", "share"]) test(`v4 ${scope} remains historical with unchanged closed contracts`, async t => {
  // Arrange.
  const f = await installContextFixture(t, { scope });
  // Act / Assert.
  assert.equal((await loadContext(f.root, { historical: true, operations: f.operations })).schema, "str005-v2-serial-context-v4");
  assert.equal(Object.hasOwn(f.context, "shareSupersession"), false);
  assert.equal(Object.hasOwn(f.context.contracts, "operatorSurvival"), false);
  await assert.rejects(loadContext(f.root, { operations: f.operations }), { code: "v2_legacy_context_read_only" });
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "v2_legacy_context_read_only" });
});

test("Share002 exclusively appends ordinal18 successor without changing original bytes", async t => {
  // Arrange.
  const f = await contextFixture(t, { scope: "share", prepare: false });
  const path = resolve(f.parent, "qualification-ordinal-18.json"), before = await readFile(path);
  // Act.
  await preflight(f.options, f.operations);
  // Assert.
  assert.deepEqual(await readFile(path), before);
  const marker = (await proof(f.parent, "qualification-ordinal-18.successor-2.json")).value;
  assert.equal(marker.ordinal, 18); assert.equal(marker.failedContextSha256, (await proof(resolve(f.parent, "share-001"), "context.json")).value.sha256);
  assert.equal(marker.contextSha256, (await proof(f.root, "context.json")).value.sha256);
  await assert.rejects(preflight(f.options, f.operations));
});

test("interrupted ordinal successor publication blocks assignment without replacing old marker", async t => {
  // Arrange.
  const f = await contextFixture(t, { scope: "share", prepare: false }), old = await readFile(resolve(f.parent, "qualification-ordinal-18.json"));
  await f.put(resolve(f.parent, "qualification-ordinal-18.successor-2.json.pending"), "{}");
  // Act / Assert.
  await assert.rejects(preflight(f.options, f.operations));
  assert(!(await readdir(f.parent)).includes("share-ordinal-2.json"));
  assert.deepEqual(await readFile(resolve(f.parent, "qualification-ordinal-18.json")), old);
});

test("daemon bootstrap checks source and snapshot without live supervisor or repeated ancestry", async t => {
  // Arrange.
  const f = await contextFixture(t);
  f.operations.inspectPredecessor = () => assert.fail("no complete ancestry review during bootstrap");
  f.operations.inspectShareSuccessor = () => assert.fail("no complete failure review during bootstrap");
  // Act.
  const context = await loadOperatorContext(f.root, f.operations);
  // Assert.
  assert.equal(context.schema, "str005-v2-serial-context-v6");
  assert(!(await readdir(f.root)).includes("server.claim.json"));
});
