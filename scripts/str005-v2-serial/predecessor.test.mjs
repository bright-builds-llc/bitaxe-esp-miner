import assert from "node:assert/strict";
import { chmod, readFile, symlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { contextFixture } from "./context-fixtures.mjs";
import { inspectPredecessor } from "./predecessor.mjs";
import { sha256 } from "./values.mjs";

async function predecessorFixture(t) {
  const f = await contextFixture(t, { prepare: false }), root = f.previous.root;
  await f.put(resolve(root, "context.json"), JSON.stringify({ context: { ...f.previous.context, schema: "str005-v2-serial-context-v1", scope: "channel" }, sha256: "synthetic" }));
  await f.put(resolve(root, "sealed-inventory.json"), "{\"synthetic\":true}\n");
  await f.put(resolve(root, "accounting-after.json"), JSON.stringify({ schema: "str005-v2-accounting-v1", contextSha256: "synthetic", stage: "after", ledger: f.previous.ledger, original_budget: f.previous.original }));
  const expected = { status: "passed", outcome: "complete", hardware_qualified: true, scope: "channel",
    result_sha256: sha256(await readFile(resolve(root, "final-result.json"))), sealed_inventory_sha256: sha256(await readFile(resolve(root, "sealed-inventory.json"))) };
  return { f, root, path: resolve(root, "final-result.json"), expected };
}

test("predecessor requires the independent reader and exact unused ordinal ledger", async (t) => {
  const p = await predecessorFixture(t); let reviewed = 0;
  const operations = { reviewChannel: async () => { reviewed++; return p.expected; } };
  const result = await inspectPredecessor(p.path, "share", operations);
  assert.equal(reviewed, 1); assert.equal(result.ledger.next_ordinal, 18);
  await writeFile(resolve(p.root, "accounting-after.json"), JSON.stringify({
    schema: "str005-v2-accounting-v1", contextSha256: "synthetic", stage: "after", ledger: { ...p.f.previous.ledger, next_ordinal: 19 }, original_budget: p.f.previous.original,
  }));
  await assert.rejects(inspectPredecessor(p.path, "share", operations), { code: "iterative_ledger_admission" });
});

test("unsafe or unprotected predecessor rejects before invoking its reader", async (t) => {
  const p = await predecessorFixture(t); let reviewed = 0;
  const operations = { reviewChannel: async () => { reviewed++; return p.expected; } };
  const alias = resolve(p.f.base, "alias"); await symlink(p.root, alias);
  await assert.rejects(inspectPredecessor(resolve(alias, "final-result.json"), "share", operations));
  await chmod(p.path, 0o644); await assert.rejects(inspectPredecessor(p.path, "share", operations));
  assert.equal(reviewed, 0);
});

test("failed predecessor or changing source bytes cannot gain acceptance", async (t) => {
  const p = await predecessorFixture(t);
  await assert.rejects(inspectPredecessor(p.path, "share", { reviewChannel: async () => ({ ...p.expected, status: "unverified", hardware_qualified: false }) }),
    { code: "v2_predecessor_unverified" });
  await assert.rejects(inspectPredecessor(p.path, "share", { reviewChannel: async () => {
    await writeFile(resolve(p.root, "context.json"), "{}"); return p.expected;
  } }), { code: "v2_predecessor_unverified" });
});

test("Share cannot recursively use a Share predecessor", async (t) => {
  const p = await predecessorFixture(t); let reviewed = false;
  await writeFile(resolve(p.root, "context.json"), JSON.stringify({ context: { schema: "str005-v2-serial-context-v1", scope: "share" } }));
  await assert.rejects(inspectPredecessor(p.path, "share", { reviewChannel: async () => { reviewed = true; return { ...p.expected, scope: "channel" }; } }),
    { code: "v2_channel_predecessor_required" });
  assert.equal(reviewed, false);
});

test("default Noise admission rejects coherently constructed but unanchored predecessor", async (t) => {
  const p = await predecessorFixture(t);
  await assert.rejects(inspectPredecessor(p.path, "channel"), { code: "v2_noise_predecessor_anchor" });
});
