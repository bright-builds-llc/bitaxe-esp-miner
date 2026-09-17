import assert from "node:assert/strict";
import { chmod, readFile, unlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { successorFixture } from "./successor-readiness.test-helper.mjs";
import { inspectFailedChannel } from "./successor-evidence.mjs";
import { prepareChannelSuccessor, reviewChannelSuccessor, checkCurrentSuccessorOwnership, successorReceiptPath } from "./successor-readiness.mjs";
import { inventory, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { main } from "./main.mjs";
import { inspectOwnershipEvidence } from "./successor-ownership.mjs";
import { readJournal } from "./journal.mjs";
import { sha256 } from "./values.mjs";

test("real historical failure stays unverified while source-bound readiness records only present ownership", async t => {
  // Arrange: real protected v2 context, original source evaluator and actual finalizer rejection.
  const f = await successorFixture(t), before = await inventory(f.root);
  assert.equal((await proof(f.root, "judgment-failure.json")).value.code, "v2_baseline");
  // Act: forensic facts and readiness never rewrite the sealed result.
  const facts = await inspectFailedChannel(f.root, f.operations);
  const ready = await prepareChannelSuccessor(f.root, f.operations);
  // Assert: old pool-port absence and historical cleanup remain unproved.
  assert.equal(ready.status, "unverified"); assert.equal(ready.hardwareQualified, false); assert.equal(ready.historicalCleanupComplete, false);
  assert.deepEqual(ready.beforeSource, { firmware_commit: f.context.firmware_commit, app_elf_sha256: f.context.app_elf_sha256 });
  const receipt = (await proof(f.parent, successorReceiptPath(f.root))).value;
  assert.equal(receipt.currentOwnership.ownerCount, facts.ownership.owners.length);
  assert(receipt.nonClaims.includes("historical-pool-port-absence"));
  assert.deepEqual(await inventory(f.root), before);
  assert.deepEqual(await reviewChannelSuccessor(f.root, f.operations), ready);
  await checkCurrentSuccessorOwnership(ready, f.operations);
  await assert.rejects(prepareChannelSuccessor(f.root, f.operations), { code: "private_path_exists" });
});

test("CLI exposes only non-promotional summary and historical review ignores current owners and HEAD", async t => {
  // Arrange
  const f = await successorFixture(t);
  // Act
  const value = await main(["prepare-channel-successor", "--private-root", f.root], f.operations);
  const historical = { ...f.operations, git: () => assert.fail("historical current HEAD"),
    processSnapshot: () => assert.fail("historical process inspection"), execFileSync: () => assert.fail("historical lsof") };
  // Assert
  assert.deepEqual(Object.keys(value).sort(), ["status", "classification", "hardware_qualified", "historical_cleanup_complete", "device_effects", "context_sha256", "receipt_sha256"].sort());
  assert.equal(value.hardware_qualified, false); assert.equal(value.historical_cleanup_complete, false);
  assert.equal(value.device_effects, false); assert.equal(value.status, "unverified");
  assert.deepEqual(await main(["review-channel-successor", "--private-root", f.root], historical), value);
  await writeFile(resolve(f.context.firmware_root, "scripts/str005-v2-serial/successor-sources.mjs"), "// synthetic future HEAD\n");
  assert.deepEqual(await main(["review-channel-successor", "--private-root", f.root], historical), value);
});

test("fresh ownership covers unclaimed group, unidentified parent and all serial holders", async t => {
  // Arrange
  const f = await successorFixture(t), ready = await prepareChannelSuccessor(f.root, f.operations);
  // Act / Assert
  for (const current of [f.unclaimedOwner, { pid: 99001, ppid: f.unclaimedOwner.pid, pgid: 99001, startedAt: "synthetic-descendant" }])
    await assert.rejects(checkCurrentSuccessorOwnership(ready, { ...f.operations, processSnapshot: async () => [current] }), { code: "noise_owner_remains" });
  await assert.rejects(checkCurrentSuccessorOwnership(ready, { ...f.operations,
    processSnapshot: async () => [{ pid: f.unclaimedOwner.ppid, ppid: 1, pgid: 99999, startedAt: "unidentified-current-process" }] }),
  { code: "v2_successor_parent_reference_present" });
  await assert.rejects(checkCurrentSuccessorOwnership(ready, { ...f.operations, execFileSync: () => "synthetic occupied resource" }), { code: "noise_resource_unproved" });
  await assert.rejects(checkCurrentSuccessorOwnership({ ...ready }, f.operations), { code: "v2_successor_inspection_required" });
});

test("private inspection capability does not cache changed evidence across a current ownership check", async t => {
  // Arrange
  const f = await successorFixture(t), ready = await prepareChannelSuccessor(f.root, f.operations);
  const path = resolve(f.root, "parent-observations/operator/parent.mjs"), bytes = await readFile(path);
  // Act / Assert
  await writeFile(path, Buffer.concat([bytes, Buffer.from("\n// changed\n")]));
  await assert.rejects(checkCurrentSuccessorOwnership(ready, f.operations), { code: "noise_inventory_changed" });
  await assert.rejects(reviewChannelSuccessor(f.root, f.operations), { code: "noise_inventory_changed" });
  await writeFile(path, bytes); await checkCurrentSuccessorOwnership(ready, f.operations);
  await writeNew(`${successorReceiptPath(f.root)}.pending`, { interrupted: true });
  await assert.rejects(checkCurrentSuccessorOwnership(ready, f.operations), { code: "private_path_exists" });
  await unlink(`${successorReceiptPath(f.root)}.pending`);
  const receipt = successorReceiptPath(f.root), value = JSON.parse(await readFile(receipt));
  value.historicalCleanupComplete = true; await writeFile(receipt, JSON.stringify(value));
  await assert.rejects(checkCurrentSuccessorOwnership(ready, f.operations), { code: "v2_successor_receipt_drift" });
  await assert.rejects(reviewChannelSuccessor(f.root, f.operations), { code: "v2_successor_receipt_binding" });
});

test("default class rejects synthetic pins and unsafe/unknown failed evidence", async t => {
  // Arrange
  const f = await successorFixture(t);
  // Act / Assert
  await assert.rejects(inspectFailedChannel(f.root), { code: "v2_successor_failure_anchor" });
  const path = resolve(f.root, "accounting-after.json"); await chmod(path, 0o644);
  await assert.rejects(inspectFailedChannel(f.root, f.operations), { code: "private_path_policy" }); await chmod(path, 0o600);
  await writeNew(resolve(f.root, "issued.json"), { synthetic: true });
  await assert.rejects(inspectFailedChannel(f.root, f.operations), { code: "noise_inventory_changed" }); await unlink(resolve(f.root, "issued.json"));
});

test("interrupted publication stays consumed and duplicate creation cannot overwrite", async t => {
  // Arrange
  const f = await successorFixture(t);
  // Act / Assert
  await assert.rejects(prepareChannelSuccessor(f.root, { ...f.operations, beforeReadinessPublish: () => { throw new Error("synthetic interruption"); } }));
  await assert.rejects(readFile(successorReceiptPath(f.root)), { code: "ENOENT" });
  assert((await readFile(`${successorReceiptPath(f.root)}.pending`)).length > 0);
  await assert.rejects(prepareChannelSuccessor(f.root, f.operations), { code: "private_path_exists" });
  await assert.rejects(reviewChannelSuccessor(f.root, f.operations), { code: "private_path_exists" });
});

test("ownership inspector rejects malformed readiness and invented historical exit even with consistent support hashes", async t => {
  // Arrange: direct atomic inspection supplements outer seal-tamper tests.
  const f = await successorFixture(t), states = await readJournal(f.root, f.context);
  const protocol = { fixtureTerminal: (await proof(f.root, "fixture-run/fixture-terminal.json")).value };
  const inspect = () => inspectOwnershipEvidence(f.root, f.context, states, protocol);
  await inspect();
  const readyPath = resolve(f.root, "fixture-ready.json"), readyBytes = await readFile(readyPath);
  // Act / Assert: no permissive partial shape or mismatched instance may support readiness.
  for (const change of [value => { value.schema = "wrong"; }, value => { value.readyAtMs = "100"; }, value => { value.extra = true; }]) {
    const value = JSON.parse(readyBytes); change(value); await writeFile(readyPath, JSON.stringify(value));
    await assert.rejects(inspect()); await writeFile(readyPath, readyBytes);
  }
  const launchPath = resolve(f.root, "parent-observations/operator/launch-001-unclaimed.json");
  const launch = JSON.parse(await readFile(launchPath)); launch.childExitCode = 0;
  const bytes = Buffer.from(JSON.stringify(launch)); await writeFile(launchPath, bytes);
  const parentPath = resolve(f.root, "parent-observations/parent-observation.json"), parent = JSON.parse(await readFile(parentPath));
  Object.assign(parent.operatorFiles.find(row => row.path === "launch-001-unclaimed.json"), { sha256: sha256(bytes), length: bytes.length });
  await writeFile(parentPath, JSON.stringify(parent));
  await assert.rejects(inspect(), { code: "v2_successor_unclaimed_launch" });
});

test("checker dependency omissions or altered historical source cannot attest readiness", async t => {
  // Arrange
  const f = await successorFixture(t); await prepareChannelSuccessor(f.root, f.operations);
  const path = successorReceiptPath(f.root), bytes = await readFile(path);
  // Act / Assert
  const omitted = JSON.parse(bytes); omitted.checkerIdentity.sources.pop(); await writeFile(path, JSON.stringify(omitted));
  await assert.rejects(reviewChannelSuccessor(f.root, f.operations), { code: "v2_successor_checker_membership" });
  await writeFile(path, bytes);
  const altered = JSON.parse(bytes); altered.checkerIdentity.sources[0].sha256 = "0".repeat(64); await writeFile(path, JSON.stringify(altered));
  await assert.rejects(reviewChannelSuccessor(f.root, f.operations), { code: "v2_successor_checker_source" });
});
