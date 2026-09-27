import assert from "node:assert/strict";
import test from "node:test";
import { readFile, writeFile, chmod } from "node:fs/promises";
import { resolve } from "node:path";
import { proof } from "../str005-noise-serial/files.mjs";
import { inspectFailedShare } from "./share-successor-evidence.mjs";
import { prepareShareSuccessor, reviewShareSuccessor } from "./share-successor-readiness.mjs";
import { shareSuccessorFailureFixture, resealShareFixture } from "./share-successor.test-helper.mjs";

 test("interrupted Share preserves unknown exits, one trust lookup and missing restoration", async t => {
  // Arrange.
  const f = await shareSuccessorFailureFixture(t);
  // Act.
  const value = await inspectFailedShare(f.root, f.operations);
  // Assert.
  assert.equal(value.afterBaselineObserved, false); assert.equal(value.initialAccounting.ledger.next_ordinal, 18);
  assert.equal(value.ordinalMarker.path, "qualification-ordinal-18.json");
  assert(value.ownership.parentPids.includes(81999));
});

test("Share readiness is exclusive, effect-free and historically read-only", async t => {
  // Arrange.
  const f = await shareSuccessorFailureFixture(t), before = await proof(f.root, "sealed-inventory.json");
  f.operations.spawnSync = () => ({ status: 1, stdout: "", stderr: "" });
  // Act.
  const created = await prepareShareSuccessor(f.root, f.operations);
  f.operations.processSnapshot = () => assert.fail("historical review must not collect live processes");
  const reviewed = await reviewShareSuccessor(f.root, f.operations);
  // Assert.
  assert.equal(created.receiptSha256, reviewed.receiptSha256); assert.equal(reviewed.hardwareQualified, false);
  assert.equal((await proof(f.root, "sealed-inventory.json")).sha256, before.sha256);
  await assert.rejects(prepareShareSuccessor(f.root, f.operations));
});

for (const name of ["issuance.claim", "issuance.claim.json", "fixture-start.json", "signer-02.exit.json", "device-record-0001.json"])
  test(`Share successor rejects added ${name}`, async t => {
    // Arrange.
    const f = await shareSuccessorFailureFixture(t); await f.put(resolve(f.root, name), "{}");
    // Act / Assert.
    await assert.rejects(inspectFailedShare(f.root, f.operations));
  });

test("Share successor rejects changed evidence bytes and unsafe permission", async t => {
  // Arrange.
  const f = await shareSuccessorFailureFixture(t), path = resolve(f.root, "signer-01.exit.json"), bytes = await readFile(path);
  // Act / Assert.
  await writeFile(path, `${bytes.toString()}\n`); await assert.rejects(inspectFailedShare(f.root, f.operations));
  await writeFile(path, bytes); await chmod(path, 0o644); await assert.rejects(inspectFailedShare(f.root, f.operations));
});

for (const [name, change] of [
  ["signer-01.exit.json", value => { value.operation = "sign-work-lease"; }],
  ["parent-observations/resumed-cleanup.json", value => { value.supervisorExitCode = 0; }],
  ["parent-observations/interruption.json", value => { value.freshPostInstallBaselineObserved = true; }],
  ["accounting-before-install.json", value => { value.ledger.pending = true; }],
]) test(`Share classifier rejects inconsistent ${name} even in a rebuilt synthetic seal`, async t => {
  // Arrange.
  const f = await shareSuccessorFailureFixture(t), path = resolve(f.root, name), value = JSON.parse(await readFile(path));
  change(value); await writeFile(path, JSON.stringify(value)); await resealShareFixture(f);
  // Act / Assert.
  await assert.rejects(inspectFailedShare(f.root, f.operations));
});

test("Share readiness blocks an occupied signer PID without inventing its start identity", async t => {
  // Arrange.
  const f = await shareSuccessorFailureFixture(t);
  f.operations.processSnapshot = async () => [{ pid: 81999, ppid: 1, pgid: 81999, startedAt: "new unrelated process" }];
  // Act / Assert.
  await assert.rejects(prepareShareSuccessor(f.root, f.operations));
});
