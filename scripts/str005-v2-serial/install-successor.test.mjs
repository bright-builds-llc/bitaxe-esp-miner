import assert from "node:assert/strict";
import test from "node:test";
import { chmod, readFile, unlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { installSuccessorFixture, resealInstallFixture } from "./install-successor.test-helper.mjs";
import { inspectFailedInstallChannel } from "./install-successor-evidence.mjs";
import { collectCurrentInstallOwnership, inspectInstallOwnershipEvidence, validateInstallOwnershipObservation } from "./install-successor-ownership.mjs";
import { prepareChannelSuccessor, reviewChannelSuccessor, checkCurrentSuccessorOwnership, successorReceiptPath } from "./successor-readiness.mjs";
import { inventory, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { main } from "./main.mjs";
import { readJournal } from "./journal.mjs";
import { inspectInstall } from "../str005-noise-serial/install.mjs";
import { sha256 } from "./values.mjs";

test("sealed install failure yields only fresh-readiness facts and preserves missing after-baseline", async t => {
  // Arrange: production snapshot, initial accounting, install claim, failure finalizer and read-only review.
  const f = await installSuccessorFixture(t), before = await inventory(f.root);
  // Act
  const facts = await inspectFailedInstallChannel(f.root, f.operations);
  const ready = await prepareChannelSuccessor(f.root, f.operations);
  // Assert: no flat ledger aliases or retrospective device-baseline claim.
  assert.equal(ready.status, "unverified"); assert.equal(ready.afterBaselineObserved, false);
  assert.equal(ready.hardwareQualified, false); assert.equal(ready.historicalCleanupComplete, false);
  assert.equal(Object.hasOwn(ready, "ledger"), false); assert.equal(Object.hasOwn(ready, "original"), false);
  assert.equal(facts.initialAccounting.observedSequence, 4);
  assert.deepEqual(ready.beforeSource, { firmware_commit: f.context.firmware_commit, app_elf_sha256: f.context.app_elf_sha256 });
  assert.deepEqual(await reviewChannelSuccessor(f.root, f.operations), ready);
  await checkCurrentSuccessorOwnership(ready, f.operations);
  assert.deepEqual(await inventory(f.root), before);
  await assert.rejects(prepareChannelSuccessor(f.root, f.operations), { code: "private_path_exists" });
});

test("install readiness CLI is closed and historical review does not require current resources", async t => {
  // Arrange
  const f = await installSuccessorFixture(t);
  // Act
  const summary = await main(["prepare-channel-successor", "--private-root", f.root], f.operations);
  const historical = { ...f.operations, processSnapshot: () => assert.fail("historical kernel inspection"),
    execFileSync: () => assert.fail("historical lsof"), git: () => assert.fail("historical HEAD") };
  // Assert
  assert.deepEqual(Object.keys(summary).sort(), ["status", "classification", "hardware_qualified", "historical_cleanup_complete", "device_effects", "context_sha256", "receipt_sha256"].sort());
  assert.equal(summary.device_effects, false);
  assert.deepEqual(await main(["review-channel-successor", "--private-root", f.root], historical), summary);
});

test("historical review rejects evidence changed during checker identity inspection", async t => {
  // Arrange: mutate only after the initial complete failed-tree inspection.
  const f = await installSuccessorFixture(t);
  await prepareChannelSuccessor(f.root, f.operations);
  const path = resolve(f.root, "parent-observations/operator/parent.mjs");
  const original = await readFile(path);
  const operations = { ...f.operations, publishedCheckerSources: async (...args) => {
    const sources = await f.operations.publishedCheckerSources(...args);
    await writeFile(path, Buffer.concat([original, Buffer.from("\n// changed during review\n")]));
    return sources;
  } };
  // Act / Assert: no successful review or private capability escapes the late mutation.
  await assert.rejects(reviewChannelSuccessor(f.root, operations), { code: "noise_inventory_changed" });
});

test("owned parent boundary excludes live launcher ancestry but rejects owned groups and worker parents", async t => {
  // Arrange
  const f = await installSuccessorFixture(t), facts = await inspectFailedInstallChannel(f.root, f.operations);
  const external = { pid: f.parent.ppid, ppid: 1, pgid: 92000, startedAt: "external launcher" };
  // Act / Assert
  const observed = await collectCurrentInstallOwnership(facts.ownership, { ...f.operations, processSnapshot: async () => [external] });
  validateInstallOwnershipObservation(observed, facts.ownership);
  for (const row of [f.parent, f.server, { pid: 99000, ppid: f.parent.pid, pgid: 99000, startedAt: "child" },
    { pid: 99001, ppid: 1, pgid: f.parent.pgid, startedAt: "group member" }])
    await assert.rejects(collectCurrentInstallOwnership(facts.ownership, { ...f.operations, processSnapshot: async () => [row] }), { code: "noise_owner_remains" });
  assert(facts.ownership.parentPids.length > 0);
  await assert.rejects(collectCurrentInstallOwnership(facts.ownership, { ...f.operations,
    processSnapshot: async () => [{ pid: facts.ownership.parentPids[0], ppid: 0, pgid: 99999, startedAt: "unidentified worker parent" }] }),
  { code: "v2_install_successor_parent_present" });
  await assert.rejects(collectCurrentInstallOwnership(facts.ownership, { ...f.operations, execFileSync: () => "occupied" }), { code: "noise_resource_unproved" });
});

test("default failure pins reject synthetic history before ancestry callbacks or operational reads", async t => {
  // Arrange
  const f = await installSuccessorFixture(t);
  // Act / Assert
  await assert.rejects(inspectFailedInstallChannel(f.root), { code: "v2_install_successor_anchor" });
  await chmod(resolve(f.root, "context.json"), 0o644);
  await assert.rejects(inspectFailedInstallChannel(f.root, f.operations), { code: "private_path_policy" });
});

test("independent reviews and private ownership capability reject changed evidence and pending receipts", async t => {
  // Arrange
  const f = await installSuccessorFixture(t), ready = await prepareChannelSuccessor(f.root, f.operations);
  const path = resolve(f.root, "parent-observations/operator/parent.mjs"), bytes = await readFile(path);
  // Act / Assert
  await writeFile(path, Buffer.concat([bytes, Buffer.from("changed")]));
  await assert.rejects(reviewChannelSuccessor(f.root, f.operations), { code: "noise_inventory_changed" });
  await assert.rejects(checkCurrentSuccessorOwnership(ready, f.operations), { code: "noise_inventory_changed" });
  await writeFile(path, bytes);
  await assert.rejects(checkCurrentSuccessorOwnership({ ...ready }, f.operations), { code: "v2_successor_inspection_required" });
  await writeNew(`${successorReceiptPath(f.root)}.pending`, { interrupted: true });
  await assert.rejects(checkCurrentSuccessorOwnership(ready, f.operations), { code: "private_path_exists" });
  await unlink(`${successorReceiptPath(f.root)}.pending`);
  await checkCurrentSuccessorOwnership(ready, f.operations);
});

test("interrupted install-readiness publication is consumed rather than overwritten", async t => {
  // Arrange
  const f = await installSuccessorFixture(t);
  // Act / Assert
  await assert.rejects(prepareChannelSuccessor(f.root, { ...f.operations, beforeReadinessPublish: () => { throw new Error("synthetic interruption"); } }), /synthetic interruption/u);
  await assert.rejects(prepareChannelSuccessor(f.root, f.operations), { code: "private_path_exists" });
});

test("forensic classifier rejects conflicting facts even when synthetic inventory is consistently resealed", async t => {
  // Arrange: only this adverse test injects historical classification; all changed facts below use production readers.
  const f = await installSuccessorFixture(t), sealPath = resolve(f.root, "sealed-inventory.json"), sealBytes = await readFile(sealPath);
  const operations = { ...f.operations, inspectHistoricalInstallChannel: async (_root, pins) => ({ context: f.context,
    reviewed: { ...f.sealed, result_sha256: pins.result.sha256, sealed_inventory_sha256: pins.seal.sha256 } }) };
  const cases = [
    ["accounting-before-install.json", v => { v.observedSequence = 3; }, "v2_install_successor_initial_sequence"],
    ["accounting-before-install.json", v => { v.ledger.total_charged_ms = 1740000; }, null],
    ["parent-install-permissions.json", v => { v.entries[1].length++; }, "v2_install_successor_permission_bytes"],
    ["parent-install-permissions.json", v => { v.acceptanceRestored = true; }, "v2_install_successor_permission_provenance"],
    ["install-0/flash-command-evidence.json", v => { v.fixed_serial_assessment.startup_failed = true; }, "v2_install_successor_permission_bytes"],
    ["install-0.exit.json", v => { v.code = 1; }, "noise_install_exit"],
    ["install-0.claim.json", v => { v.argv.push("--factory"); }, "v2_install_successor_arguments"],
    ["state-0009.json", v => { v.state.qualification.submitted = 1; }, "v2_install_successor_counter_changed"],
    ["parent-observations/browser-provenance.json", v => { v.candidateFreshRestorationCollected = true; }, "v2_install_successor_browser_provenance"],
    ["parent-cleanup-supervisor.json", v => { v.exitedAtMs = v.stopRequestedAtMs + 5001; }, "v2_install_successor_supervisor_exit"],
    ["parent-cleanup-failure.json", v => { v.code = "v2_other_failure"; }, "v2_install_successor_cleanup_failure"],
  ];
  // Act / Assert: restore exact fixture bytes after each independent mutation.
  for (const [name, mutate, code] of cases) {
    const path = resolve(f.root, name), original = await readFile(path), value = JSON.parse(original);
    mutate(value); await writeFile(path, `${JSON.stringify(value, null, 2)}\n`); await resealInstallFixture(f);
    await assert.rejects(inspectFailedInstallChannel(f.root, operations), code ? { code } : undefined, name);
    await writeFile(path, original); await writeFile(sealPath, sealBytes);
  }
  for (const name of ["fixture-owner.json", "install-1.claim.json", "issued.json", "device-0001.json", "accounting-after.json", "install-0.review.json"]) {
    await writeNew(resolve(f.root, name), {}); await resealInstallFixture(f);
    await assert.rejects(inspectFailedInstallChannel(f.root, operations), { code: "v2_install_successor_unexpected_evidence" }, name);
    await unlink(resolve(f.root, name)); await writeFile(sealPath, sealBytes);
  }
  await inspectFailedInstallChannel(f.root, f.operations);
});

test("ownership rejects a non-detached campaign parent or mismatched supervisor-parent join", async t => {
  // Arrange: original production forensic inputs, with mutation confined to the synthetic fixture.
  const f = await installSuccessorFixture(t), states = await readJournal(f.root, f.context), installed = await inspectInstall(f.root, f.context, 0);
  const parentPath = resolve(f.root, "parent-observations/operator/parent-root.json"), parentBytes = await readFile(parentPath);
  const supportPath = resolve(f.root, "parent-observations/support-inventory.json"), supportBytes = await readFile(supportPath);
  const parent = JSON.parse(parentBytes); parent.pgid++;
  const changed = Buffer.from(`${JSON.stringify(parent, null, 2)}\n`), support = JSON.parse(supportBytes);
  const row = support.files.find(item => item.path === "parent-observations/operator/parent-root.json");
  row.sha256 = sha256(changed); row.length = changed.length;
  // Act / Assert
  await writeFile(parentPath, changed); await writeFile(supportPath, JSON.stringify(support));
  await assert.rejects(inspectInstallOwnershipEvidence(f.root, f.context, states, installed), { code: "v2_install_successor_parent_identity" });
  await writeFile(parentPath, parentBytes); await writeFile(supportPath, supportBytes);
  const serverPath = resolve(f.root, "server-owner.json"), serverBytes = await readFile(serverPath), server = JSON.parse(serverBytes);
  server.owner.ppid++;
  await writeFile(serverPath, JSON.stringify(server));
  await assert.rejects(inspectInstallOwnershipEvidence(f.root, f.context, states, installed), { code: "v2_install_successor_parent_identity" });
  await writeFile(serverPath, serverBytes);
  await inspectInstallOwnershipEvidence(f.root, f.context, states, installed);
});

test("another owned record referencing the external launcher retains that parent absence requirement", async t => {
  // Arrange: exclusion applies to the detached root-parent record, never a PID-wide allowlist.
  const f = await installSuccessorFixture(t), states = await readJournal(f.root, f.context), installed = await inspectInstall(f.root, f.context, 0);
  const path = resolve(f.root, "install-0.detect.observation.json"), bytes = await readFile(path), observation = JSON.parse(bytes);
  observation.seen.push({ pid: 99002, ppid: f.parent.ppid, pgid: 99002, startedAt: "other owned record" });
  await writeFile(path, JSON.stringify(observation));
  // Act
  const ownership = await inspectInstallOwnershipEvidence(f.root, f.context, states, installed);
  // Assert
  assert(ownership.parentPids.includes(f.parent.ppid));
  const external = { pid: f.parent.ppid, ppid: 1, pgid: 92000, startedAt: "external launcher" };
  await assert.rejects(collectCurrentInstallOwnership(ownership, { ...f.operations, processSnapshot: async () => [external] }),
    { code: "v2_install_successor_parent_present" });
  await writeFile(path, bytes);
});
