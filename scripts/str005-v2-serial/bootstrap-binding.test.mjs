import test from "node:test";
import assert from "node:assert/strict";
import { bootstrapBinding, bootstrapMetadata, requireBootstrapBinding, BOOTSTRAP_BEFORE } from "./bootstrap-binding.mjs";
import { contextFixture } from "./context-fixtures.mjs";
import { preflight } from "./context.mjs";
import { readdir } from "node:fs/promises";
test("closed bootstrap binding admits only exact finite roots and pinned identities", () => {
  const value = bootstrapBinding("/synthetic"); assert.doesNotThrow(() => bootstrapMetadata(value, "/synthetic"));
  for (const change of [v => { v.acceptedRoot += "-other"; }, v => { v.acceptedResultSha256 = "f".repeat(64); }, v => { v.endpoint = "private"; }]) {
    const changed = structuredClone(value); change(changed); assert.throws(() => bootstrapMetadata(changed, "/synthetic"));
  }
});
test("v6 Channel binds accepted bootstrap installed image without transferring cycles or mining authority", async t => {
  const f = await contextFixture(t); assert.equal(f.context.schema, "str005-v2-serial-context-v6"); assert.equal(f.context.hostOrdinal, 6);
  assert.deepEqual(f.context.before_source, BOOTSTRAP_BEFORE); assert.deepEqual(f.context.install_indices, [0, 1, 2, 3, 4]); assert.equal(f.context.qualificationAttempt, undefined);
  const changed = structuredClone(f.context); changed.before_source.firmware_commit = "f".repeat(40);
  await assert.rejects(requireBootstrapBinding(changed, f.operations), { code: "v2_bootstrap_before" });
});
test("old supersession flags and conflicting bootstrap bindings fail before a fresh assignment", async t => {
  const f = await contextFixture(t, { prepare: false });
  await assert.rejects(preflight({ ...f.options, supersedeShare: "/old" }, f.operations));
  assert(!(await readdir(f.parent)).includes("channel-ordinal-6.json"));
  const prior = f.operations.bootstrapPins; f.operations.bootstrapPins = async root => ({ ...await prior(root), shareSupersession: {} });
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_bootstrap_pair" });
});
