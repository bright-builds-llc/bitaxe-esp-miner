import assert from "node:assert/strict";
import test from "node:test";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { contextFixture } from "./context-fixtures.mjs";
import { loadContext, preflight, verifyEffectInputs, recheckNative, recheckSuccessorOwnership } from "./context.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { INSTALL_OWNERSHIP_AMENDMENT_PATH, INSTALL_OWNERSHIP_AMENDMENT_SHA256, sha256 } from "./values.mjs";

const unassigned = async f => {
  const names = await readdir(f.parent);
  assert(!names.includes("channel-ordinal-4.json")); assert(!names.includes("channel-004"));
};

test("Channel004 joins Noise ancestry separately from actual installed Channel003 identity", async t => {
  // Arrange / Act.
  const f = await contextFixture(t), context = f.context;
  // Assert.
  assert.equal(context.schema, "str005-v2-serial-context-v4"); assert.equal(context.hostOrdinal, 4);
  assert.equal(context.permissionSupersession, null);
  assert.deepEqual(context.contracts.installReviewOwnership, { path: INSTALL_OWNERSHIP_AMENDMENT_PATH, sha256: INSTALL_OWNERSHIP_AMENDMENT_SHA256 });
  assert.deepEqual(Object.keys(context.cleanupSupersession).sort(), ["failedRoot", "failedContextSha256", "failedResultSha256", "failedSealSha256", "receiptPath", "receiptSha256"].sort());
  assert.equal(context.predecessor.root, f.previous.root);
  const failed = (await proof(resolve(f.parent, "channel-003"), "context.json")).value.context;
  assert.deepEqual(context.before_source, { firmware_commit: failed.firmware_commit, app_elf_sha256: failed.app_elf_sha256 });
  assert.notEqual(context.before_source.firmware_commit, f.previous.context.firmware_commit);
  for (const name of ["permission-correction.json", "host-correction.json"]) {
    const file = await proof(f.root, name);
    assert((await proof(f.root, "preflight-inventory.json")).value.files.some(row => row.path === name && row.sha256 === file.sha256));
  }
});

test("Channel004 is the only new Channel assignment and all supersession alternatives reject", async t => {
  // Arrange.
  const f = await contextFixture(t, { prepare: false });
  // Act / Assert.
  for (const patch of [{ supersedeChannel: undefined }, { supersedePermission: "/private/old-closure.json" },
    { privateRoot: resolve(f.parent, "channel-005") }, { privateRoot: resolve(f.parent, "channel-001-new") },
    { scope: "share", privateRoot: resolve(f.parent, "share-001") }]) {
    await assert.rejects(preflight({ ...f.options, ...patch }, f.operations)); await unassigned(f);
  }
});

test("missing current ownership after software checks cannot consume an assignment", async t => {
  // Arrange.
  const f = await contextFixture(t, { prepare: false }); let checks = 0, signs = 0;
  const spawn = f.operations.spawnSync;
  f.operations.spawnSync = (...args) => { signs++; return spawn(...args); };
  f.operations.checkCurrentSuccessorOwnership = async () => { checks++; assert.equal(signs, 2); throw Object.assign(Error("synthetic holder"), { code: "v2_successor_owner_live" }); };
  // Act / Assert.
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_successor_owner_live" });
  assert.equal(checks, 1); await unassigned(f);
});

test("historical review never recollects ownership but live admission requires a fresh check", async t => {
  // Arrange.
  const f = await contextFixture(t);
  f.operations.checkCurrentSuccessorOwnership = async () => { throw Object.assign(Error("synthetic present owner"), { code: "v2_successor_owner_live" }); };
  // Act / Assert.
  assert.equal((await loadContext(f.root, { historical: true, operations: f.operations })).hostOrdinal, 4);
  await assert.rejects(loadContext(f.root, { operations: f.operations }), { code: "v2_successor_owner_live" });
});

test("changed ancestry, before identity, accounting and checker identity block admission", async t => {
  // Arrange.
  const f = await contextFixture(t, { prepare: false }), inspect = f.operations.inspectChannelSuccessor;
  const changes = [value => ({ ...value, predecessor: { ...value.predecessor, root: value.root } }),
    value => ({ ...value, beforeSource: { ...value.beforeSource, firmware_commit: f.previous.context.firmware_commit } }),
    value => ({ ...value, initialAccounting: { ...value.initialAccounting, ledger: { ...value.initialAccounting.ledger, pending: true } } }),
    value => ({ ...value, checkerIdentity: { ...value.checkerIdentity, firmwareCommit: "0".repeat(40) } }),
    value => ({ ...value, context: { ...value.context, schema: "str005-v2-serial-context-v4", hostOrdinal: 4 } })];
  // Act / Assert.
  for (const change of changes) {
    f.operations.inspectChannelSuccessor = async (...args) => change(await inspect(...args));
    await assert.rejects(preflight(f.options, f.operations)); await unassigned(f);
  }
});

test("firmware, package and evaluator must each change while Gate may remain the same", async t => {
  // Arrange.
  const f = await contextFixture(t, { prepare: false }), inspect = f.operations.inspectChannelSuccessor;
  const { inspectSources, nativeInterface } = await import("./context-sources.mjs");
  const source = await inspectSources(f.options, await nativeInterface(f.operations), f.operations);
  // Act / Assert.
  for (const key of ["firmware_commit", "manifest_sha256", "evaluator"]) {
    f.operations.inspectChannelSuccessor = async (...args) => {
      const value = await inspect(...args), context = { ...value.context, [key]: source[key] };
      const beforeSource = { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 };
      return { ...value, context, beforeSource, contextSha256: sha256(JSON.stringify(context)) };
    };
    await assert.rejects(preflight(f.options, f.operations), { code: "v2_cleanup_correction_unchanged" }); await unassigned(f);
  }
});

test("bounded effect pins reject exact-byte changes without replaying historical inspection", async t => {
  // Arrange.
  const f = await contextFixture(t); f.operations.inspectChannelSuccessor = () => assert.fail("no historical replay inside effect window");
  await verifyEffectInputs(f.context, f.operations);
  const root = f.context.cleanupSupersession.failedRoot;
  // Act / Assert.
  for (const name of ["context.json", "final-result.json", "sealed-inventory.json", "accounting-before-install.json"]) {
    const path = resolve(root, name), original = await readFile(path);
    await writeFile(path, `${original.toString()}\n`);
    await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: name === "accounting-before-install.json" ? "v2_install_successor_accounting" : "v2_cleanup_receipt_changed" });
    await writeFile(path, original);
  }
  await writeFile(f.options.supersedeChannel, "changed readiness");
  await assert.rejects(verifyEffectInputs(f.context, f.operations));
});

test("Share001 inherits both same-pair corrections and no independent supersession", async t => {
  // Arrange.
  const f = await contextFixture(t, { scope: "share", prepare: false });
  f.operations.spawnSync = () => assert.fail("Share must inherit checked software receipts");
  // Act.
  await preflight(f.options, f.operations); const context = await loadContext(f.root, { operations: f.operations });
  // Assert.
  assert.equal(context.permissionSupersession, null); assert.equal(context.cleanupSupersession, null);
  assert.equal(context.hostOrdinal, 1); assert.equal(context.qualificationAttempt.ordinal, 18);
  for (const name of ["permission-correction.json", "host-correction.json"])
    assert.equal((await proof(f.root, name)).sha256, (await proof(resolve(f.parent, "channel-004"), name)).sha256);
  await verifyEffectInputs(context, f.operations);
});

test("Share cannot inherit an altered host correction or a historical Channel003", async t => {
  // Arrange.
  const f = await contextFixture(t, { scope: "share", prepare: false });
  const path = resolve(f.parent, "channel-004/host-correction.json"), changed = JSON.parse(await readFile(path));
  changed.command = ["node", "--test", "unrelated.mjs"]; await writeFile(path, JSON.stringify(changed));
  // Act / Assert.
  await assert.rejects(preflight(f.options, f.operations));
  assert(!(await readdir(f.parent)).includes("qualification-ordinal-18.json"));
});


test("post-native serve admission rechecks present ownership before any server claim", async t => {
  // Arrange.
  const f = await contextFixture(t), inspect = f.operations.inspectNative; let occupied = false;
  f.operations.inspectNative = async input => { const value = await inspect(input); occupied = true; return value; };
  f.operations.checkCurrentSuccessorOwnership = async () => {
    if (occupied) throw Object.assign(Error("synthetic owner appeared during native audit"), { code: "v2_successor_owner_live" });
  };
  // Act / Assert.
  await recheckNative(f.context, f.operations);
  await assert.rejects(recheckSuccessorOwnership(f.context, f.operations), { code: "v2_successor_owner_live" });
  assert(!(await readdir(f.root)).includes("server.claim.json"));
});

test("interrupted Channel004 assignment remains consumed without changing older markers", async t => {
  // Arrange.
  const f = await contextFixture(t, { prepare: false });
  const previous = await Promise.all([1, 2, 3].map(index => proof(f.parent, `channel-ordinal-${index}.json`)));
  f.operations.beforeCreate = () => { throw Error("synthetic interrupted child creation"); };
  // Act / Assert.
  await assert.rejects(preflight(f.options, f.operations), /synthetic interrupted child creation/u);
  const names = await readdir(f.parent);
  assert(names.includes("channel-ordinal-4.json")); assert(!names.includes("channel-004"));
  for (const [index, old] of previous.entries()) assert.equal((await proof(f.parent, `channel-ordinal-${index + 1}.json`)).sha256, old.sha256);
  delete f.operations.beforeCreate; await assert.rejects(preflight(f.options, f.operations));
});


test("late external parent cleanup failure blocks live loading and hot effects without blocking history", async t => {
  // Arrange.
  const f = await contextFixture(t);
  await verifyEffectInputs(f.context, f.operations);
  await f.put(resolve(f.root, "parent-cleanup.claim.json"), "{}\n");
  await verifyEffectInputs(f.context, f.operations); // Collection preparation is not a failure.
  await f.put(resolve(f.root, "parent-cleanup-failure.json"), "{\"synthetic\":true}\n");
  // Act / Assert.
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "private_path_exists" });
  await assert.rejects(loadContext(f.root, { operations: f.operations }), { code: "private_path_exists" });
  assert.equal((await loadContext(f.root, { historical: true, operations: f.operations })).hostOrdinal, 4);
});


test("Channel004 requires tagged pre-install accounting without an invented after baseline", async t => {
  // Arrange.
  const f = await contextFixture(t, { prepare: false }), inspect = f.operations.inspectChannelSuccessor;
  // Act / Assert.
  for (const change of [value => ({ ...value, afterBaselineObserved: true }),
    value => ({ ...value, ledger: value.initialAccounting.ledger }),
    value => ({ ...value, initialAccounting: { ...value.initialAccounting, path: "accounting-after.json" } }),
    value => ({ ...value, initialAccounting: { ...value.initialAccounting, observedSequence: 0 } })]) {
    f.operations.inspectChannelSuccessor = async (...args) => change(await inspect(...args));
    await assert.rejects(preflight(f.options, f.operations)); await unassigned(f);
  }
});


test("ownership clarification drift blocks v4 assignment before any new context exists", async t => {
  // Arrange.
  const f = await contextFixture(t, { prepare: false });
  await writeFile(resolve(f.options.firmwareRoot, INSTALL_OWNERSHIP_AMENDMENT_PATH), "changed ownership clarification");
  // Act / Assert.
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_contract_changed" });
  await unassigned(f);
});
