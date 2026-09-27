import test from "node:test";
import assert from "node:assert/strict";
import { fixture } from "./test-fixture.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { validateWriter } from "./native-writer.mjs";
test("writer receipt cannot hide invalid frames, sums, roles or omitted source dependencies", async t => {
  const f = await fixture(t), stored = (await proof(f.root, "snapshot/native-writer.json")).value;
  assert.equal(validateWriter(stored, f.context), stored);
  for (const change of [v => { v.selectedFrameSumBytes = -1; }, v => { v.frames[0].entryBytes = Number.NaN; },
    v => { v.selectedFrameSumBytes -= 1; }, v => { v.sources.pop(); }, v => { v.roles.emit = "standalone_measured"; },
    v => { v.sources[0].secret = "private"; }, v => { v.frames[0].secret = "private"; }]) {
    const value = structuredClone(stored); change(value); assert.throws(() => validateWriter(value, f.context));
  }
});
