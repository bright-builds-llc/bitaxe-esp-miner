import test from "node:test";
import assert from "node:assert/strict";
import { readdir } from "node:fs/promises";
import { fixture } from "./test-fixture.mjs";
import { preflight, load, validate } from "./context.mjs";
import { parseArgs } from "./main.mjs";

test("one context binds the measurement and cannot be reassigned", async t => {
  const f = await fixture(t);
  assert.deepEqual(f.context.installIndices, [0]); assert.equal(f.context.miningAuthorized, false);
  assert.equal((await load(f.root, { operations: f.operations })).attempt.ordinal, 4);
  await assert.rejects(preflight(f.options, f.operations));
});
test("unsupported authority, scope and unmanaged serve reject before any path read", () => {
  for (const args of [["serve", "--private-root", "/missing"], ["operator-start", "--private-root", "/missing", "--authority-directory", "/secret"],
    ["preflight", "--private-root", "/missing", "--scope", "share"]]) assert.throws(() => parseArgs(args));
});
test("interrupted creation consumes the single assignment", async t => {
  const f = await fixture(t, { prepare: false }); f.operations.beforeCreate = () => { throw Error("interrupted"); };
  await assert.rejects(preflight(f.options, f.operations), /interrupted/u);
  const entries = await readdir(f.parent);
  assert(entries.includes("attempt-ordinal-4.json")); assert(!entries.includes("attempt-004"));
  assert.equal(entries.filter(name => name.startsWith("attempt-004.preparation-")).length, 1); delete f.operations.beforeCreate;
  await assert.rejects(preflight(f.options, f.operations));
});

test("closed nested records reject private extras and invalid inventory rows", async t => {
  const f = await fixture(t);
  for (const change of [c => { c.package.secret = "private"; }, c => { c.nativeReadiness.noise.secret = "private"; },
    c => { c.nativeReadiness.telemetry.nodes.push({ symbol: "node", address: 1, entry_bytes: 1, secret: "private" }); },
    c => { c.sourceInventory.push(c.sourceInventory[0]); }, c => { c.sourceInventory[0].length = "1"; }, c => { c.expectedAccounting.original.secret = "private"; }]) {
    const value = structuredClone(f.context); change(value); assert.throws(() => validate(value));
  }
});
