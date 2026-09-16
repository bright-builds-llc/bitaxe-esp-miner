import assert from "node:assert/strict";
import { readFile, readdir, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { completedFixture } from "./completed-fixture.mjs";
import { finalize, review } from "./finalize.mjs";
import { judge } from "./judge.mjs";

test("independent finalizer derives normal-path result only from complete protected observations", async (t) => {
  const f = await completedFixture(t);
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  assert.equal(result.status, "passed");
  assert.deepEqual(await review(f.root, f.operations), result);
  await assert.rejects(finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations));
});
test("a missing fresh probe cannot be replaced by retained state probe counts", async (t) => {
  const f = await completedFixture(t);
  await rm(resolve(f.root, "cycle-3.probe.json"));
  await assert.rejects(judge(f.root, f.context, `${f.root}.cleanup/receipt.json`, f.operations), { code: "ENOENT" });
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  assert.equal(result.status, "unverified");
  const saved = JSON.parse(await readFile(resolve(f.root, "final-result.json")));
  assert(saved.inputs.deviceJournal && saved.inputs.accounting && saved.inputs.fixtureReady);
  await assert.rejects(readFile(resolve(f.context.firmware_root, "docs/parity/evidence/str005-noise-serial/attempt-001.json")), { code: "ENOENT" });
});
test("still-running owners prevent any result or seal write", async (t) => {
  const f = await completedFixture(t), owner = JSON.parse(await readFile(resolve(f.root, "server-owner.json"))).owner;
  await assert.rejects(finalize(f.root, `${f.root}.cleanup/receipt.json`, { ...f.operations, processSnapshot: async () => [owner] }), { code: "noise_owner_remains" });
  assert(!(await readdir(f.root)).includes("final-result.json"));
});
test("sealed byte mutation fails read-only review", async (t) => {
  const f = await completedFixture(t);
  await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  await writeFile(resolve(f.root, "cycle-1.json"), "{}");
  await assert.rejects(review(f.root, f.operations), { code: "noise_inventory_changed" });
});
