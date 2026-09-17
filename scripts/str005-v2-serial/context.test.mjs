import assert from "node:assert/strict";
import { chmod, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { contextFixture } from "./context-fixtures.mjs";
import { loadContext, preflight, recheckNative, verifyEffectInputs } from "./context.mjs";
import { AMENDMENT_PATH } from "./context-sources.mjs";

for (const scope of ["channel", "share"]) test(`${scope} protected preflight freezes pair and exact installation inventory`, async (t) => {
  const f = await contextFixture(t, { scope });
  assert.deepEqual(f.context.install_indices, scope === "channel" ? [0, 1, 2, 3, 4] : [1, 2, 3, 4]);
  assert.equal(Object.hasOwn(f.context, "mining_authorized"), false);
  assert.equal(JSON.parse(await readFile(resolve(f.root, "artifact-snapshot.json"))).files.length, 13);
  assert.deepEqual(await recheckNative(f.context, f.operations), f.context.native_readiness);
  if (scope === "share") { assert.equal(f.context.qualificationAttempt.id, f.context.attemptId); assert.equal(f.context.qualificationAttempt.ordinal, 18); }
  else assert.equal(Object.hasOwn(f.context, "qualificationAttempt"), false);
  await assert.rejects(preflight(f.options, f.operations));
});

test("missing native readiness cannot consume an assignment", async (t) => {
  const f = await contextFixture(t, { prepare: false });
  const native = f.operations.inspectNative;
  f.operations.inspectNative = async (input) => ({ ...await native(input), result: "unverified" });
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_native_readiness" });
  assert.deepEqual(await readdir(f.parent), [".synthetic-parent", "channel-001", "channel-001.permission-closure.json", "channel-002", "channel-002.successor-readiness.json", "channel-003", "channel-003.successor-readiness.json", "channel-ordinal-1.json", "channel-ordinal-2.json", "channel-ordinal-3.json"]);
});

test("interruption before mkdir consumes host and qualification assignments permanently", async (t) => {
  const f = await contextFixture(t, { scope: "share", prepare: false });
  f.operations.beforeCreate = () => { throw new Error("synthetic interruption"); };
  await assert.rejects(preflight(f.options, f.operations), /synthetic interruption/u);
  const names = await readdir(f.parent); assert(names.includes("share-ordinal-1.json")); assert(names.includes("qualification-ordinal-18.json"));
  assert(!names.includes("share-001"));
  delete f.operations.beforeCreate;
  await assert.rejects(preflight(f.options, f.operations));
});

test("archived task and contract drift reject before namespace mutation", async (t) => {
  const f = await contextFixture(t, { prepare: false });
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-str005-v2-serial-qualification | inactive\n");
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_live_task_inactive" });
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Active\n### task-str005-v2-serial-qualification | synthetic\n");
  await writeFile(resolve(f.options.firmwareRoot, AMENDMENT_PATH), "changed amendment");
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_contract_changed" });
  assert.deepEqual(await readdir(f.parent), [".synthetic-parent", "channel-001", "channel-001.permission-closure.json", "channel-002", "channel-002.successor-readiness.json", "channel-003", "channel-003.successor-readiness.json", "channel-ordinal-1.json", "channel-ordinal-2.json", "channel-ordinal-3.json"]);
});

test("fresh effect checks detect source drift independently on every call", async (t) => {
  const f = await contextFixture(t);
  await verifyEffectInputs(f.context, f.operations);
  await writeFile(resolve(f.options.firmwareRoot, "scripts/str005-v2-serial/context.mjs"), "source changed");
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "v2_effect_source_changed" });
  await assert.rejects(loadContext(f.root, { operations: f.operations }), { code: "v2_live_source_drift" });
  assert.equal((await loadContext(f.root, { historical: true, operations: f.operations })).scope, "channel");
});

test("sealed or failed roots reject live loading while historical inspection remains valid", async (t) => {
  const f = await contextFixture(t);
  await f.put(resolve(f.root, "failed-inventory.json"), "{}");
  await assert.rejects(loadContext(f.root, { operations: f.operations }));
  assert.equal((await loadContext(f.root, { historical: true, operations: f.operations })).scope, "channel");
});

test("snapshot artifact, SDK and mode mutations cannot serve", async (t) => {
  const f = await contextFixture(t);
  const path = resolve(f.root, "native/bitaxe-firmware.sdkconfig"), original = await readFile(path);
  await writeFile(path, "changed SDK"); await assert.rejects(loadContext(f.root, { operations: f.operations }));
  await writeFile(path, original); await chmod(path, 0o644);
  await assert.rejects(loadContext(f.root, { operations: f.operations }));
});

test("native auditor-source mismatch fails before reservation", async (t) => {
  const f = await contextFixture(t, { prepare: false }), native = f.operations.inspectNative;
  f.operations.inspectNative = async (input) => { const value = await native(input); value.auditorSources[0].sha256 = "0".repeat(64); return value; };
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_native_source_join" });
  assert.deepEqual(await readdir(f.parent), [".synthetic-parent", "channel-001", "channel-001.permission-closure.json", "channel-002", "channel-002.successor-readiness.json", "channel-003", "channel-003.successor-readiness.json", "channel-ordinal-1.json", "channel-ordinal-2.json", "channel-ordinal-3.json"]);
});

test("missing final preflight inventory cannot serve a partially created context", async (t) => {
  const f = await contextFixture(t);
  await rm(resolve(f.root, "preflight-inventory.json"));
  await assert.rejects(loadContext(f.root, { operations: f.operations }));
});

test("Share rejects a different pair and unsigned native changes after Channel", async (t) => {
  const f = await contextFixture(t, { scope: "share", prepare: false });
  const prior = f.operations.inspectPredecessor;
  f.operations.inspectPredecessor = async (...args) => { const p = await prior(...args); return { ...p, context: { ...p.context, gate_commit: "e".repeat(40) } }; };
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_share_pair_changed" });
  assert(!(await readdir(f.parent)).includes("share-ordinal-1.json"));
});

test("native auditing cannot race changed task or package into an assignment", async (t) => {
  const f = await contextFixture(t, { prepare: false }), audit = f.operations.inspectNative;
  f.operations.inspectNative = async (input) => { const value = await audit(input);
    await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-str005-v2-serial-qualification | moved\n"); return value; };
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_live_task_inactive" });
  assert.deepEqual(await readdir(f.parent), [".synthetic-parent", "channel-001", "channel-001.permission-closure.json", "channel-002", "channel-002.successor-readiness.json", "channel-003", "channel-003.successor-readiness.json", "channel-ordinal-1.json", "channel-ordinal-2.json", "channel-ordinal-3.json"]);
});

test("observer identity is mandatory before assignment and immutable in snapshot", async (t) => {
  const f = await contextFixture(t, { prepare: false });
  const path = resolve(f.options.firmwareRoot, "bazel-bin/tools/http-transport/v2-observer-build-identity.json");
  const original = await readFile(path), bad = JSON.parse(original); bad.sourceCommit = "f".repeat(40);
  await writeFile(path, JSON.stringify(bad));
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_observer_build_identity" });
  assert.deepEqual(await readdir(f.parent), [".synthetic-parent", "channel-001", "channel-001.permission-closure.json", "channel-002", "channel-002.successor-readiness.json", "channel-003", "channel-003.successor-readiness.json", "channel-ordinal-1.json", "channel-ordinal-2.json", "channel-ordinal-3.json"]);
  await writeFile(path, original); await preflight(f.options, f.operations);
  const context = await loadContext(f.root, { operations: f.operations });
  assert(context.cadence_observer.path.endsWith("/tools/http-transport/cadence_observer"));
  await writeFile(resolve(f.root, "observer/observer.bin"), "tampered observer");
  await assert.rejects(loadContext(f.root, { historical: true, operations: f.operations }), { code: "v2_observer_snapshot_changed" });
});
