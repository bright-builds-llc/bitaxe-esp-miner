import assert from "node:assert/strict";
import { chmod, readFile, symlink, unlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { permissionFixture } from "./permission-closure.test-helper.mjs";
import { closePermission, inspectPermissionClosure, reviewPermission } from "./permission-closure.mjs";
import { closurePath, inspectPermissionInputs } from "./permission-closure-inputs.mjs";
import { inventory, proof, writeNew } from "../str005-noise-serial/files.mjs";

test("permission closure retains immutable failure without inventing device baseline or ledger", async t => {
  // Arrange: synthetic publication/kernel adapters; actual protected snapshot and witness readers.
  const f = await permissionFixture(t), before = await inventory(f.root), operatorBefore = await inventory(f.operator);
  // Act: only the deterministic sibling is created.
  const result = await closePermission(f.root, f.operations), receipt = (await proof(f.parent, closurePath(f.root))).value;
  // Assert: historical review neither checks today's ownership nor reruns correction tests.
  assert.equal(result.status, "unverified"); assert.equal(result.hardware_qualified, false);
  assert.equal(receipt.facts.boundary, "before_native_open"); assert.equal(receipt.facts.deviceAccounting, "not_collected");
  assert.equal(receipt.facts.deviceBaseline, "not_collected"); assert.equal(receipt.facts.savedOperatorBrowserBranchUsed, false);
  assert.equal(receipt.facts.supervisorStopToCloseMs, 13);
  const historical = { ...f.operations, processSnapshot: () => assert.fail("historical process scan"),
    execFileSync: () => assert.fail("historical lsof"), spawnSync: () => assert.fail("historical Gate test") };
  assert.deepEqual(await reviewPermission(f.root, historical), result);
  assert.deepEqual(await inventory(f.root), before); assert.deepEqual(await inventory(f.operator), operatorBefore);
  await writeNew(resolve(f.parent, "unrelated-successor.json"), { synthetic: true });
  await inspectPermissionClosure(closurePath(f.root), historical);
  await assert.rejects(closePermission(f.root, f.operations), { code: "private_path_exists" });
});

test("classifier rejects later admission, device evidence and conflicting first cause", async t => {
  const f = await permissionFixture(t);
  const changes = [
    ["state-0003.json", row => { row.state.admissionFailureStage = "opening"; }],
    ["state-0001.json", row => { row.state.connected = true; }],
    ["state-0004.json", row => { row.state.deviceBaselineConfirmed = true; }],
    ["state-0004.json", row => { row.state.failure = "probe_failed"; }],
    ["state-0006.json", row => { row.state.deviceLeaseInactive = true; }],
    ["failure.json", row => { row.sourceSequence = 3; }],
  ];
  for (const [name, change] of changes) {
    const path = resolve(f.root, name), bytes = await readFile(path), value = JSON.parse(bytes);
    change(value); await writeFile(path, JSON.stringify(value));
    await assert.rejects(inspectPermissionInputs(f.root, f.operations)); await writeFile(path, bytes);
  }
  for (const name of ["accounting-before.json", "device-0001.json", "fixture-owner.json", "install-0.claim.json", "issued.json", "state-0007.json", "unknown.json"]) {
    await writeNew(resolve(f.root, name), { unexpected: true });
    await assert.rejects(inspectPermissionInputs(f.root, f.operations), { code: "v2_permission_inventory_membership" });
    await unlink(resolve(f.root, name));
  }
  await inspectPermissionInputs(f.root, f.operations);
  await assert.rejects(readFile(closurePath(f.root)), { code: "ENOENT" });
});

test("all artifacts and independently observed closure witnesses remain mandatory", async t => {
  const f = await permissionFixture(t);
  const artifact = (await proof(f.root, "artifact-snapshot.json")).value.files[0];
  const path = resolve(f.root, "qualified-artifacts", artifact.path), bytes = await readFile(path);
  await writeFile(path, Buffer.concat([bytes, Buffer.from("changed")]));
  await assert.rejects(inspectPermissionInputs(f.root, f.operations)); await writeFile(path, bytes);
  const changes = [
    ["browser.json", row => { row.lastStateSha256 = "0".repeat(64); }],
    ["supervisor-exit.json", row => { row.code = 1; }],
    ["supervisor-exit.json", row => { row.exitedAtMs = row.stopRequestedAtMs + 5001; }],
    ["no-admission-resources.json", row => { row.observations[0].stdoutBytes = 1; }],
    ["no-admission-resources.json", row => { row.observations[1].args[2] = "/dev/cu.wrong"; }],
  ];
  for (const [name, change] of changes) {
    const target = resolve(f.operator, name), original = await readFile(target), value = JSON.parse(original);
    change(value); await writeFile(target, JSON.stringify(value));
    await assert.rejects(inspectPermissionInputs(f.root, f.operations)); await writeFile(target, original);
  }
});

test("creation rejects current owned processes and unproved resource absence", async t => {
  const f = await permissionFixture(t);
  await assert.rejects(closePermission(f.root, { ...f.operations, processSnapshot: async () => [f.owner] }), { code: "noise_owner_remains" });
  await assert.rejects(closePermission(f.root, { ...f.operations, execFileSync: () => "22111\n" }), { code: "noise_resource_unproved" });
  await assert.rejects(readFile(closurePath(f.root)), { code: "ENOENT" });
});

test("unsafe paths, permissions and interrupted closure cannot become completed receipts", async t => {
  const f = await permissionFixture(t), path = resolve(f.root, "failure.json");
  await chmod(path, 0o644); await assert.rejects(closePermission(f.root, f.operations), { code: "private_path_policy" }); await chmod(path, 0o600);
  const alias = resolve(f.parent, "alias"); await symlink(f.root, alias);
  await assert.rejects(closePermission(alias, f.operations));
  await assert.rejects(closePermission(f.root, { ...f.operations, beforeClosurePublish: () => { throw new Error("synthetic interrupted publication"); } }));
  await assert.rejects(readFile(closurePath(f.root)), { code: "ENOENT" });
  assert((await readFile(`${closurePath(f.root)}.pending`)).length > 0);
  await assert.rejects(reviewPermission(f.root, f.operations));
  await assert.rejects(closePermission(f.root, f.operations), { code: "private_path_exists" });
});

test("review catches independent-call drift in witnesses, root membership and closure bytes", async t => {
  const f = await permissionFixture(t); await closePermission(f.root, f.operations);
  const file = resolve(f.operator, "parent.mjs"), bytes = await readFile(file);
  await reviewPermission(f.root, f.operations); await writeFile(file, Buffer.concat([bytes, Buffer.from("\n")]));
  await assert.rejects(reviewPermission(f.root, f.operations), { code: "v2_permission_closure_binding" }); await writeFile(file, bytes);
  await writeNew(resolve(f.operator, "added.json"), {});
  await assert.rejects(reviewPermission(f.root, f.operations), { code: "v2_permission_inventory_membership" }); await unlink(resolve(f.operator, "added.json"));
  const receipt = (await proof(f.parent, closurePath(f.root))).value; receipt.resources.supervisorAbsent = false;
  await writeFile(closurePath(f.root), JSON.stringify(receipt));
  await assert.rejects(reviewPermission(f.root, f.operations), { code: "v2_permission_closure_resources" });
});

test("production class anchor cannot be replaced by an unpinned synthetic failure", async t => {
  const f = await permissionFixture(t);
  await assert.rejects(inspectPermissionInputs(f.root), { code: "v2_permission_failure_anchor" });
});

test("concurrent closure publication has one winner and no overwriting path", async t => {
  const f = await permissionFixture(t);
  const results = await Promise.allSettled([closePermission(f.root, f.operations), closePermission(f.root, f.operations)]);
  assert.equal(results.filter(row => row.status === "fulfilled").length, 1);
  assert.equal(results.filter(row => row.status === "rejected").length, 1);
  await reviewPermission(f.root, f.operations);
  await assert.rejects(readFile(`${closurePath(f.root)}.pending`), { code: "ENOENT" });
});

test("evidence drift during correction checks cannot publish a closure", async t => {
  const f = await permissionFixture(t), path = resolve(f.operator, "parent.mjs");
  const original = await readFile(path);
  const operations = { ...f.operations, processSnapshot: async () => {
    await writeFile(path, Buffer.concat([original, Buffer.from("\n// synthetic concurrent drift\n")])); return [];
  } };
  await assert.rejects(closePermission(f.root, operations), { code: "v2_permission_input_drift" });
  await assert.rejects(readFile(closurePath(f.root)), { code: "ENOENT" });
  await assert.rejects(readFile(`${closurePath(f.root)}.pending`), { code: "ENOENT" });
});

test("closure cannot label the unchanged failed bundle as the correction", async t => {
  const f = await permissionFixture(t); await closePermission(f.root, f.operations);
  const receipt = (await proof(f.parent, closurePath(f.root))).value;
  receipt.createdBy.correction.gateBundleSha256 = f.context.gate_bundle_sha256;
  await writeFile(closurePath(f.root), JSON.stringify(receipt));
  await assert.rejects(reviewPermission(f.root, f.operations), { code: "v2_permission_correction_pair" });
});
