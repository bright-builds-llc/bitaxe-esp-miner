import assert from "node:assert/strict";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { fixture } from "./test-fixture.mjs";
import { preflight, loadContext, publicationPath } from "./context.mjs";

test("real protected preflight freezes thirteen artifacts and reserves before child creation", async (t) => {
  const f = await fixture(t);
  assert.equal(f.context.schema, "noise-serial-context-v2");
  assert.equal(f.context.mining_authorized, false);
  assert.equal(JSON.parse(await readFile(resolve(f.root, "artifact-snapshot.json"))).files.length, 13);
  assert((await readdir(f.parent)).includes("ordinal-1.json"));
});
test("missing native readiness stops before any assignment or child", async (t) => {
  const f = await fixture(t, { prepare: false });
  await assert.rejects(preflight(f.options, { ...f.operations, inspectNative: async () => { throw new Error("native missing"); } }), /native missing/u);
  assert.deepEqual(await readdir(f.parent), []);
});
test("interruption after reservation cannot reassign the ordinal", async (t) => {
  const f = await fixture(t, { prepare: false });
  await assert.rejects(preflight(f.options, { ...f.operations, beforeCreate: () => { throw new Error("interrupted"); } }), /interrupted/u);
  assert.deepEqual(await readdir(f.parent), ["ordinal-1.json"]);
  await assert.rejects(preflight(f.options, f.operations), { code: "EEXIST" });
});
test("live source mutation is rejected on a fresh independent load", async (t) => {
  const f = await fixture(t);
  await loadContext(f.root, { operations: f.operations });
  await writeFile(resolve(f.options.gateRoot, "dist/worker-serial-acceptance/worker-serial-acceptance.js"), "changed");
  await assert.rejects(loadContext(f.root, { operations: f.operations }));
});
test("completed or archived task cannot admit live effects", async (t) => {
  const f = await fixture(t, { prepare: false });
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-str005-noise-auth-205 | blocked\n");
  await assert.rejects(preflight(f.options, f.operations), { code: "noise_live_task_inactive" });
  assert.deepEqual(await readdir(f.parent), []);
});

for (const [name, archive, tasks, code] of [
  ["missing", "", null, "noise_preparation_receipt_missing"],
  ["cancelled", "### task-str005-noise-runtime-readiness | fixture\nStatus: Cancelled\n", null, "noise_preparation_not_complete"],
  ["moved to Future", null, "## Active\n### task-str005-noise-auth-205 | fixture\n## Future\n### task-str005-noise-runtime-readiness | fixture\n", "noise_preparation_incomplete"],
]) test(`preparation ${name} is not completed native authority`, async (t) => {
  const f = await fixture(t, { prepare: false });
  if (archive !== null) await writeFile(resolve(f.options.firmwareRoot, "TASKS.archive.md"), archive);
  if (tasks !== null) await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), tasks);
  await assert.rejects(preflight(f.options, f.operations), { code });
  assert.deepEqual(await readdir(f.parent), []);
});
test("same amendment label with changed bytes cannot reserve an ordinal", async (t) => {
  const f = await fixture(t, { prepare: false }), path = resolve(f.options.firmwareRoot, "docs/hardware/str005-noise-parity-scope-amendment.md");
  await writeFile(path, `${await readFile(path, "utf8")}\n`);
  await assert.rejects(preflight(f.options, f.operations), { code: "noise_base_contract_changed" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("old fixture binary build provenance cannot claim a new published source", async (t) => {
  const f = await fixture(t, { prepare: false }), path = resolve(f.options.firmwareRoot, "bazel-bin/tools/stratum-v2-fixture/noise-serial-build-identity.json");
  const record = JSON.parse(await readFile(path)); record.sourceCommit = "e".repeat(40); await writeFile(path, JSON.stringify(record));
  await assert.rejects(preflight(f.options, f.operations), { code: "noise_fixture_build_identity" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("native auditor receipt must match the pre-frozen source inventory before assignment", async (t) => {
  const f = await fixture(t, { prepare: false }), inspect = f.operations.inspectNative;
  await assert.rejects(preflight(f.options, { ...f.operations, inspectNative: async (input) => {
    const value = await inspect(input); value.auditorSources[0].sha256 = "0".repeat(64); return value;
  } }), { code: "noise_native_auditor_join" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("native receipt changes preserving ELF identity still fail historical input validation", async (t) => {
  const f = await fixture(t), path = resolve(f.root, "native/readiness.json");
  const value = JSON.parse(await readFile(path)); value.auditorSources[0].sha256 = "0".repeat(64); await writeFile(path, JSON.stringify(value));
  await assert.rejects(loadContext(f.root, { historical: true, operations: f.operations }), { code: "noise_native_snapshot_changed" });
});

test("device helper profile freezes its task gate, successor amendment and recovery ledger", async (t) => {
  const f = await fixture(t, { profile: "device-noise-helper" });
  assert.equal(f.context.profile, "device-noise-helper");
  assert.deepEqual(f.context.expected_ledger, { next_ordinal: 22, last_ordinal: 21, total_charged_ms: 2280000 });
  assert.equal(f.context.contracts.binding.successor.path, "docs/hardware/device-noise-helper-amendment.md");
  assert.equal(f.context.maximum_installations, 5);
});
test("device helper profile without its exact enabled line reserves nothing", async (t) => {
  const f = await fixture(t, { prepare: false, profile: "device-noise-helper" });
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Active\n### task-device-noise-worker-stack | synthetic\n\nDevice noise serial hardware: disabled.\n");
  await assert.rejects(preflight(f.options, f.operations), { code: "noise_live_task_disabled" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("device helper profile rejects a predecessor basis with another ledger", async (t) => {
  const f = await fixture(t, { prepare: false, profile: "device-noise-helper" }), inspect = f.operations.inspectPredecessor;
  await assert.rejects(preflight(f.options, { ...f.operations, inspectPredecessor: async (path) => {
    const value = await inspect(path); return { ...value, previous: { ...value.previous, next_ordinal: 23 } };
  } }), { code: "noise_predecessor" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("changed successor amendment bytes cannot reserve an ordinal", async (t) => {
  const f = await fixture(t, { prepare: false, profile: "device-noise-helper" }), path = resolve(f.options.firmwareRoot, "docs/hardware/device-noise-helper-amendment.md");
  await writeFile(path, `${await readFile(path, "utf8")}\n`);
  await assert.rejects(preflight(f.options, f.operations), { code: "noise_base_contract_changed" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("historical contexts keep their original bindings and need no successor", async (t) => {
  const f = await fixture(t);
  assert.equal(f.context.profile, undefined);
  assert.equal(f.context.contracts.binding.successor, undefined);
  assert.deepEqual(f.context.expected_ledger, { next_ordinal: 18, last_ordinal: 17, total_charged_ms: 1560000 });
});
test("a recovery predecessor with other bytes fails its exact anchor", async (t) => {
  const { inspectRecoveryPredecessor } = await import("./predecessor.mjs");
  const { mkdtemp, mkdir, realpath, rm } = await import("node:fs/promises");
  const { tmpdir } = await import("node:os");
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "noise-recovery-")));
  t.after(() => rm(base, { recursive: true, force: true }));
  const root = resolve(base, "attempt"); await mkdir(root, { mode: 0o700 });
  await writeFile(resolve(root, "result.json"), JSON.stringify({ current_safe_recovery: true }), { mode: 0o600 });
  await assert.rejects(inspectRecoveryPredecessor(resolve(root, "result.json")), /noise_predecessor_anchor/u);
});
test("a device continuation without its sealed prior attempt reserves nothing", async (t) => {
  const f = await fixture(t, { prepare: false, profile: "device-noise-helper" });
  const options = { ...f.options, privateRoot: resolve(f.parent, "attempt-002"), attemptOrdinal: 2 };
  await assert.rejects(preflight(options, f.operations));
  assert.deepEqual(await readdir(f.parent), []);
});
test("the historical profile still refuses a later ordinal", async (t) => {
  const f = await fixture(t, { prepare: false });
  const options = { ...f.options, privateRoot: resolve(f.parent, "attempt-002"), attemptOrdinal: 2 };
  await assert.rejects(preflight(options, f.operations), { code: "noise_retry_progress_unverified" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("an ordinal without a reviewed continuation is refused", async (t) => {
  const f = await fixture(t, { prepare: false, profile: "device-noise-helper" });
  const options = { ...f.options, privateRoot: resolve(f.parent, "attempt-004"), attemptOrdinal: 4 };
  await assert.rejects(preflight(options, f.operations), { code: "noise_retry_progress_unverified" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("an installed continuation without its prior attempt reserves nothing", async (t) => {
  const f = await fixture(t, { prepare: false, profile: "device-noise-helper" });
  const options = { ...f.options, privateRoot: resolve(f.parent, "attempt-003"), attemptOrdinal: 3 };
  await assert.rejects(preflight(options, f.operations));
  assert.deepEqual(await readdir(f.parent), []);
});
test("control stack profile freezes its task gate, successor amendment and helper-pass ledger", async (t) => {
  // Arrange / Act
  const f = await fixture(t, { profile: "control-stack-port-reuse" });
  // Assert
  assert.equal(f.context.profile, "control-stack-port-reuse");
  assert.deepEqual(f.context.expected_ledger, { next_ordinal: 22, last_ordinal: 21, total_charged_ms: 2280000 });
  assert.equal(f.context.contracts.binding.successor.path, "docs/hardware/control-stack-port-reuse-amendment.md");
  assert.equal(f.context.continuation, undefined);
});
test("control stack profile without its exact enabled line reserves nothing", async (t) => {
  // Arrange
  const f = await fixture(t, { prepare: false, profile: "control-stack-port-reuse" });
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Active\n### task-control-stack-port-reuse-run | synthetic\n\nControl stack port reuse hardware: disabled.\n");
  // Act / Assert
  await assert.rejects(preflight(f.options, f.operations), { code: "noise_live_task_disabled" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("control stack profile refuses the recovery basis its predecessor profile used", async (t) => {
  // Arrange
  const f = await fixture(t, { prepare: false, profile: "control-stack-port-reuse" }), inspect = f.operations.inspectPredecessor;
  // Act / Assert
  await assert.rejects(preflight(f.options, { ...f.operations, inspectPredecessor: async (path) => {
    const value = await inspect(path); return { ...value, previous: { ...value.previous, basis: "current_safe_recovery" } };
  } }), { code: "noise_predecessor" });
  assert.deepEqual(await readdir(f.parent), []);
});
test("control stack publication never shares the historical attempt directory", async (t) => {
  // Arrange
  const f = await fixture(t, { profile: "control-stack-port-reuse" }), historical = await fixture(t);
  // Act
  const path = publicationPath(f.context);
  // Assert
  assert.equal(path, resolve(f.options.firmwareRoot, "docs/parity/evidence/control-stack-port-reuse/attempt-001.json"));
  assert.equal(publicationPath(historical.context), resolve(historical.options.firmwareRoot, "docs/parity/evidence/str005-noise-serial/attempt-001.json"));
});
test("a helper pass predecessor with other bytes fails its exact anchor", async (t) => {
  // Arrange
  const { inspectHelperPassPredecessor } = await import("./predecessor.mjs");
  const { mkdtemp, mkdir, realpath, rm } = await import("node:fs/promises");
  const { tmpdir } = await import("node:os");
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "noise-helper-pass-")));
  t.after(() => rm(base, { recursive: true, force: true }));
  const root = resolve(base, "attempt"); await mkdir(root, { mode: 0o700 });
  await writeFile(resolve(root, "final-result.json"), JSON.stringify({ status: "passed" }), { mode: 0o600 });
  // Act / Assert
  await assert.rejects(inspectHelperPassPredecessor(resolve(root, "final-result.json")), /noise_predecessor_anchor/u);
});
