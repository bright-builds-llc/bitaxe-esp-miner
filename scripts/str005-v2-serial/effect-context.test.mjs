import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { readFile, writeFile, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { contextFixture, permissionContextFixture } from "./context-fixtures.mjs";
import { loadEffectContext, verifyEffectInputs } from "./context.mjs";
import { syntheticSupervisor } from "./effect-context-fixture.mjs";

async function ready(t, scope = "channel") {
  const f = await contextFixture(t, { scope });
  const owner = await syntheticSupervisor(f);
  for (const name of ["inspectPredecessor", "inspectChannelSuccessor", "inspectNative", "checkCurrentSuccessorOwnership"])
    f.operations[name] = () => assert.fail(`bounded effect path cannot rerun ${name}`);
  return { ...f, owner };
}
for (const scope of ["channel", "share"]) test(`${scope} effect loading checks admitted snapshots without replaying historical or native judgments`, async t => {
  // Arrange.
  const f = await ready(t, scope);
  // Act / Assert.
  assert.deepEqual(await loadEffectContext(f.root, f.operations), f.context);
});

test("missing or mismatched full-admission supervisor receipt cannot admit an effect", async t => {
  // Arrange.
  const f = await ready(t), path = resolve(f.root, "server.claim.json"), old = await readFile(path);
  // Act / Assert.
  await unlink(path); await assert.rejects(loadEffectContext(f.root, f.operations), { code: "ENOENT" });
  const changed = JSON.parse(old); changed.contextSha256 = "0".repeat(64);
  await writeFile(path, JSON.stringify(changed), { mode: 0o600 });
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_effect_supervisor_binding" });
});

test("dead, stopped, zombie or reused supervisor identity never reaches the loopback request", async t => {
  // Arrange.
  const f = await ready(t); let requests = 0;
  f.operations.fetch = async () => { requests++; throw Error("must not request"); };
  // Act / Assert.
  for (const rows of [[], [{ ...f.owner, startedAt: "reused PID" }], [{ ...f.owner, state: "Z" }], [{ ...f.owner, state: "T" }]]) {
    f.operations.processSnapshot = async () => rows;
    await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_effect_supervisor_not_live" });
  }
  assert.equal(requests, 0);
});

test("live process with closed, oversized or failed HTTP state remains unavailable without raw errors", async t => {
  // Arrange.
  const f = await ready(t);
  const sources = [async () => { throw Error("private-address:32125"); }, async () => new Response("x".repeat(4097)),
    async () => new Response(JSON.stringify({ scope: "channel", phase: "before", failed: true })),
    async () => new Response(JSON.stringify({ scope: "channel", phase: "before", failed: false, private: "no" }))];
  // Act / Assert.
  for (const fetch of sources) {
    f.operations.fetch = fetch;
    await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_effect_supervisor_unavailable", message: "v2_effect_supervisor_unavailable" });
  }
});

test("supervisor identity is rechecked after its bounded fixed-origin HTTP response", async t => {
  // Arrange.
  const f = await ready(t); let snapshots = 0;
  f.operations.processSnapshot = async () => ++snapshots === 1 ? [f.owner] : [];
  f.operations.fetch = async (url, options) => {
    assert.equal(url, "http://127.0.0.1:32125/supervisor-state"); assert.equal(options.redirect, "error"); assert.equal(options.method, "GET");
    assert(options.signal instanceof AbortSignal);
    return new Response(JSON.stringify({ scope: "channel", phase: "before", failed: false }));
  };
  // Act / Assert.
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_effect_supervisor_not_live" });
});

test("default Noise pin reader rejects changed real bytes despite an unrelated supplied digest", async t => {
  // Arrange: placeholder files intentionally cannot stand in for the real accepted campaign.
  const f = await ready(t); delete f.operations.readNoiseAnchorProof;
  await f.put(resolve(f.previous.root, "sealed-inventory.json"), "{\"synthetic\":true}\n");
  f.operations.expectedNoiseResultSha256 = f.context.predecessor.resultSha256;
  // Act / Assert.
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_noise_predecessor_anchor" });
});

test("archived tasks, changed source and changed predecessor pins reject the bounded effect path", async t => {
  // Arrange.
  const f = await ready(t), task = resolve(f.options.firmwareRoot, "TASKS.md"), original = await readFile(task);
  // Act / Assert.
  await writeFile(task, "## Future\n### task-str005-v2-serial-qualification | archived\n");
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_live_task_inactive" }); await writeFile(task, original);
  const source = resolve(f.options.firmwareRoot, "scripts/str005-v2-serial/context.mjs"), before = await readFile(source);
  await writeFile(source, "changed source");
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_effect_source_changed" }); await writeFile(source, before);
  await writeFile(f.options.supersedeChannel, "changed readiness");
  await assert.rejects(loadEffectContext(f.root, f.operations));
});

test("parent cleanup failure appearing during hot checks blocks return but its claim does not", async t => {
  // Arrange.
  const f = await ready(t);
  await f.put(resolve(f.root, "parent-cleanup.claim.json"), "{}\n");
  await loadEffectContext(f.root, f.operations);
  f.operations.cleanPushed = () => writeFileSync(resolve(f.root, "parent-cleanup-failure.json"), "{}\n", { mode: 0o600 });
  // Act / Assert.
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "private_path_exists" });
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "private_path_exists" });
});

test("historical v2 cannot use the bounded loader even with a live supervisor witness", async t => {
  // Arrange.
  const f = await permissionContextFixture(t); await syntheticSupervisor(f);
  // Act / Assert.
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_legacy_context_read_only" });
});


test("a different process owning the reused HTTP port cannot supply supervisor proof", async t => {
  // Arrange: expected supervisor is alive and an HTTP response looks valid, but listener ownership differs.
  const f = await ready(t);
  f.operations.spawnSync = () => ({ status: 0, signal: null, stdout: "p68001\nf9\nn127.0.0.1:32125\n", stderr: "" });
  // Act / Assert.
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_effect_supervisor_listener" });
  f.operations.spawnSync = () => ({ status: 0, signal: null, stdout: "malformed-private-listener", stderr: "" });
  await assert.rejects(loadEffectContext(f.root, f.operations), { code: "v2_effect_supervisor_listener", message: "v2_effect_supervisor_listener" });
});
