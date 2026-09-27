import test from "node:test";
import assert from "node:assert/strict";
import { readdir, readFile, writeFile, mkdir, realpath } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fixture } from "./test-fixture.mjs";
import { preflight, load, contextHash, validate } from "./context.mjs";
import { validateLayoutCheck, COMMAND } from "./source-layout-check.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
test("fixed regression executes the admitted Node and closed arguments before assignment", async t => {
  const f = await fixture(t, { prepare: false }), admittedNode = await realpath(process.execPath); let calls = 0;
  f.operations.runSourceLayoutCheck = (program, args, options) => {
    calls++; assert.equal(program, admittedNode); assert.deepEqual(args, [...COMMAND]); assert.equal(options.timeout, 60000); assert.equal(options.maxBuffer, 65536);
    const result = spawnSync(program, args, options);
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.status, 0, Buffer.concat([result.stdout, result.stderr]).toString());
    return result;
  };
  await preflight(f.options, f.operations); assert.equal(calls, 1);
  const context = await load(f.root, { operations: f.operations }); const receipt = (await proof(f.root, "snapshot/source-layout-check.json")).value;
  validateLayoutCheck(receipt, context); assert.throws(() => validateLayoutCheck({ ...receipt, exitCode: 1 }, context));
});
test("failed correction cannot consume a new successor assignment", async t => {
  const f = await fixture(t, { prepare: false }); f.operations.runSourceLayoutCheck = () => ({ status: 1, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) });
  await assert.rejects(preflight(f.options, f.operations), { code: "bootstrap_layout_check_failed" });
  assert(!(await readdir(f.parent)).includes("attempt-ordinal-2.json"));
});
test("v1 closed context remains parseable but cannot reach a live loader", async t => {
  const f = await fixture(t), context = structuredClone(f.context); context.schema = "usb-bootstrap-measure-context-v1"; context.attempt.ordinal = 1;
  delete context.preflightAmendment; delete context.preflightSupersession; validate(context);
  await writeFile(resolve(f.root, "context.json"), JSON.stringify({ context, sha256: contextHash(context) }));
  await assert.rejects(load(f.root, { operations: f.operations }), { code: "bootstrap_v1_read_only" });
});

test("complete snapshots are checked in a non-authoritative staging directory before assignment", async t => {
  const f = await fixture(t, { prepare: false }); let staged;
  f.operations.beforeSnapshotReview = async stage => {
    staged = stage; await assert.rejects(load(stage, { operations: f.operations }), { code: "bootstrap_context_changed" });
    await writeFile(resolve(stage, "snapshot/source/unindexed.bin"), "unexpected", { mode: 0o600 });
  };
  await assert.rejects(preflight(f.options, f.operations)); assert(staged);
  assert(!(await readdir(f.parent)).includes("attempt-ordinal-2.json"));
});
test("late final-directory collision cannot overwrite preexisting bytes", async t => {
  const f = await fixture(t, { prepare: false });
  f.operations.beforeCreate = async () => { await mkdir(f.root, { mode: 0o700 }); await writeFile(resolve(f.root, "sentinel"), "existing", { mode: 0o600 }); };
  await assert.rejects(preflight(f.options, f.operations), { code: "EEXIST" });
  assert.equal(await readFile(resolve(f.root, "sentinel"), "utf8"), "existing");
  assert((await readdir(f.parent)).includes("attempt-ordinal-2.json"));
});
test("interrupted final publication remains assigned but cannot load without final inventory", async t => {
  const f = await fixture(t, { prepare: false }); f.operations.beforeInventoryPublish = () => { throw Error("publish interrupted"); };
  await assert.rejects(preflight(f.options, f.operations), /publish interrupted/u);
  assert((await readdir(f.root)).includes("snapshot"));
  await assert.rejects(load(f.root, { operations: f.operations }), { code: "ENOENT" });
  delete f.operations.beforeInventoryPublish; await assert.rejects(preflight(f.options, f.operations));
});
