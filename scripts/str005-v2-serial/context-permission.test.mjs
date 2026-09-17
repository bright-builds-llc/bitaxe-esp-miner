import assert from "node:assert/strict";
import test from "node:test";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { contextFixture, legacyContextFixture } from "./context-fixtures.mjs";
import { loadContext, preflight, verifyEffectInputs } from "./context.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";

async function unchangedAssignment(f, original) {
  assert.equal((await proof(f.parent, "channel-ordinal-1.json")).sha256, original);
  const names = await readdir(f.parent);
  assert(!names.includes("channel-ordinal-2.json")); assert(!names.includes("channel-002"));
}

test("Channel002 binds exact permission supersession and correction snapshot", async t => {
  // Arrange / Act
  const f = await contextFixture(t), context = f.context;
  // Assert
  assert.equal(context.schema, "str005-v2-serial-context-v2"); assert.equal(context.hostOrdinal, 2);
  assert.deepEqual(Object.keys(context.permissionSupersession).sort(), ["failedRoot", "failedContextSha256", "closurePath", "closureSha256"].sort());
  assert.equal(context.permissionSupersession.failedRoot, resolve(f.parent, "channel-001"));
  const correction = await proof(f.root, "permission-correction.json");
  assert.equal(correction.value.gateCommit, context.gate_commit);
  const inventory = (await proof(f.root, "preflight-inventory.json")).value;
  assert(inventory.files.some(file => file.path === "permission-correction.json" && file.sha256 === correction.sha256));
  assert.equal(context.predecessor.root, f.previous.root);
});

test("only the one explicit Channel successor can consume a new host assignment", async t => {
  // Arrange
  const f = await contextFixture(t, { prepare: false }), old = (await proof(f.parent, "channel-ordinal-1.json")).sha256;
  // Act / Assert
  for (const options of [{ ...f.options, supersedePermission: undefined },
    { ...f.options, privateRoot: resolve(f.parent, "channel-003") },
    { ...f.options, scope: "share", privateRoot: resolve(f.parent, "share-001") }])
    await assert.rejects(preflight(options, f.operations), { code: "v2_permission_successor_required" });
  await unchangedAssignment(f, old);
});

test("failed correction leaves the old consumed marker intact without assigning002", async t => {
  // Arrange
  const f = await contextFixture(t, { prepare: false }), old = (await proof(f.parent, "channel-ordinal-1.json")).sha256;
  f.operations.spawnSync = () => ({ status: 1, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) });
  // Act / Assert
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_permission_correction_failed" });
  await unchangedAssignment(f, old);
});

test("failed preparation cannot replace the accepted Noise protocol ancestor", async t => {
  // Arrange
  const f = await contextFixture(t, { prepare: false }), read = f.operations.inspectPermissionClosure;
  f.operations.inspectPermissionClosure = async (...args) => {
    const value = await read(...args); return { ...value, predecessor: { ...value.predecessor, root: value.root } };
  };
  // Act / Assert
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_permission_predecessor" });
  assert(!(await readdir(f.parent)).includes("channel-ordinal-2.json"));
});

test("each changed-pair component is mandatory before the successor correction runs", async t => {
  // Arrange
  const f = await contextFixture(t, { prepare: false }), read = f.operations.inspectPermissionClosure;
  for (const [key, value] of [["firmware_commit", "a".repeat(40)], ["gate_commit", "b".repeat(40)],
    ["gate_bundle_sha256", sha256(await readFile(resolve(f.options.gateRoot, "dist/worker-serial-acceptance/worker-serial-acceptance.js")))]]) {
    f.operations.inspectPermissionClosure = async (...args) => {
      const result = await read(...args), context = { ...result.context, [key]: value };
      return { ...result, context, contextSha256: sha256(JSON.stringify(context)) };
    };
    // Act / Assert
    await assert.rejects(preflight(f.options, f.operations), { code: "v2_permission_pair_unchanged" });
  }
  assert(!(await readdir(f.parent)).includes("channel-ordinal-2.json"));
});

test("recursive supersession is rejected before any correction command", async t => {
  // Arrange
  const f = await contextFixture(t, { prepare: false }), read = f.operations.inspectPermissionClosure;
  f.operations.inspectPermissionClosure = async (...args) => {
    const result = await read(...args), context = { ...result.context, schema: "str005-v2-serial-context-v2", hostOrdinal: 2 };
    return { ...result, context, contextSha256: sha256(JSON.stringify(context)) };
  };
  // Act / Assert
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_permission_predecessor" });
});

test("closure changes during native checking cannot enter an assigned context", async t => {
  // Arrange
  const f = await contextFixture(t, { prepare: false }), audit = f.operations.inspectNative;
  f.operations.inspectNative = async input => {
    const result = await audit(input); await writeFile(f.options.supersedePermission, "changed closure after initial read"); return result;
  };
  // Act / Assert
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_permission_closure_changed" });
  assert(!(await readdir(f.parent)).includes("channel-ordinal-2.json"));
});

test("Share001 inherits the exact accepted Channel proof without rerunning or reserving another ordinal", async t => {
  // Arrange
  const f = await contextFixture(t, { scope: "share", prepare: false });
  const prior = await proof(resolve(f.parent, "channel-002"), "permission-correction.json");
  f.operations.spawnSync = () => { assert.fail("Share cannot rerun Channel correction"); };
  // Act
  await preflight(f.options, f.operations); const context = await loadContext(f.root, { operations: f.operations });
  // Assert
  assert.equal(context.schema, "str005-v2-serial-context-v2"); assert.equal(context.hostOrdinal, 1);
  assert.equal(context.permissionSupersession, null); assert.equal(context.qualificationAttempt.ordinal, 18);
  assert.equal(context.expectedLedgerBefore.total_charged_ms, 1560000);
  assert.equal((await proof(f.root, "permission-correction.json")).sha256, prior.sha256);
});

test("missing or altered inherited correction cannot reserve Share allowance identity", async t => {
  // Arrange
  const f = await contextFixture(t, { scope: "share", prepare: false });
  const path = resolve(f.parent, "channel-002/permission-correction.json"), changed = JSON.parse(await readFile(path));
  changed.command = ["bun", "test", "unrelated.test.ts"]; await writeFile(path, JSON.stringify(changed));
  // Act / Assert
  await assert.rejects(preflight(f.options, f.operations), { code: "v2_permission_correction_binding" });
  assert(!(await readdir(f.parent)).includes("qualification-ordinal-18.json"));
});

test("v1 snapshots remain historical-readable after archival but never regain live eligibility", async t => {
  // Arrange
  const f = await legacyContextFixture(t);
  await f.put(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-str005-v2-serial-qualification | archived\n");
  // Act / Assert
  assert.equal((await loadContext(f.root, { historical: true, operations: f.operations })).schema, "str005-v2-serial-context-v1");
  await assert.rejects(loadContext(f.root, { operations: f.operations }), { code: "v2_legacy_context_read_only" });
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "v2_legacy_context_read_only" });
});

test("hot effect checks rehash the pinned closure without replaying historical artifacts", async t => {
  // Arrange: full closure review already succeeded through preflight/load.
  const f = await contextFixture(t);
  f.operations.inspectPermissionClosure = () => { assert.fail("no exhaustive replay inside the Start window"); };
  // Act / Assert
  await verifyEffectInputs(f.context, f.operations);
  await writeFile(f.options.supersedePermission, JSON.stringify({ changed: true }));
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "v2_permission_closure_changed" });
});

test("hot effect pin rejects a changed failed context without treating it as a new verdict", async t => {
  // Arrange
  const f = await contextFixture(t);
  f.operations.inspectPermissionClosure = () => { assert.fail("hot path cannot classify a replacement"); };
  const path = resolve(f.context.permissionSupersession.failedRoot, "context.json");
  const changed = JSON.parse(await readFile(path)); changed.context.hostOrdinal = 2;
  await writeFile(path, JSON.stringify(changed));
  // Act / Assert
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "v2_permission_closure_changed" });
});

test("hot effect pin rejects byte changes that preserve the parsed failed context", async t => {
  // Arrange
  const f = await contextFixture(t), path = resolve(f.context.permissionSupersession.failedRoot, "context.json");
  const envelope = JSON.parse(await readFile(path));
  const originalDigest = sha256(JSON.stringify(envelope.context));
  await writeFile(path, `${JSON.stringify(envelope, null, 2)}\n`);
  // Act / Assert
  assert.equal(sha256(JSON.stringify(JSON.parse(await readFile(path)).context)), originalDigest);
  await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "v2_permission_closure_changed" });
});
