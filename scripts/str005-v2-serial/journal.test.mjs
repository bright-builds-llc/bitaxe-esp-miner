import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { baseline, createJournal, readJournal, restoredBaseline, saveAccounting } from "./journal.mjs";

async function setup(t, scope = "channel") {
  const root = await mkdtemp(join(tmpdir(), "v2-journal-")); t.after(() => rm(root, { recursive: true }));
  const context = { scope, attemptId: Buffer.alloc(16, 1).toString("base64url"), gate_commit: "b".repeat(40),
    firmware_commit: "a".repeat(40), app_elf_sha256: "c".repeat(64),
    before_source: { firmware_commit: "d".repeat(40), app_elf_sha256: "e".repeat(64) } };
  return { root, context, writer: await createJournal(root, context) };
}

test("before/candidate journal retains one immutable sequence and blocks phase regression", async (t) => {
  // Arrange
  const { root, context, writer } = await setup(t);
  // Act
  await writer.state("before", state(context, "before"), 1);
  await writer.state("candidate", state(context), 2);
  // Assert
  assert.deepEqual((await readJournal(root, context)).map((row) => row.phase), ["before", "candidate"]);
  await assert.rejects(writer.state("before", state(context, "before"), 3), { code: "v2_journal_order" });
});

test("Channel cannot record mining or a loaded work window as a valid state", async (t) => {
  const { context, writer } = await setup(t);
  const candidate = state(context); candidate.status = "window_loaded";
  await assert.rejects(writer.state("candidate", candidate, 1), { code: "v2_mining_forbidden" });
});

test("separate Share baseline still requires unchanged pre-install and pre-work ledgers", async (t) => {
  // Arrange
  const { root, context, writer } = await setup(t, "share");
  const before = state(context, "before"); await writer.state("before", before, 1);
  await saveAccounting(root, context, { stage: "before-install", state: before, ledger, original_budget: original }, writer.lastState());
  const candidate = state(context); await writer.state("candidate", candidate, 2);
  // Act / Assert
  await assert.rejects(saveAccounting(root, context, { stage: "before", state: candidate,
    ledger: { ...ledger, pending: true }, original_budget: original }, writer.lastState()));
  await saveAccounting(root, context, { stage: "before", state: candidate, ledger, original_budget: original }, writer.lastState());
  assert.equal(JSON.parse(await readFile(join(root, "accounting-before.json"))).ledger.next_ordinal, 18);
});

test("failed Share preserves the real unconsumed after-ledger without fabricating a charge", async (t) => {
  // Arrange
  const { root, context, writer } = await setup(t, "share");
  const before = state(context, "before"); await writer.state("before", before, 1);
  await saveAccounting(root, context, { stage: "before-install", state: before, ledger, original_budget: original }, writer.lastState());
  const candidate = state(context); await writer.state("candidate", candidate, 2);
  await saveAccounting(root, context, { stage: "before", state: candidate, ledger, original_budget: original }, writer.lastState());
  await writer.state("candidate", candidate, 3);
  await writeNew(join(root, "restoration.json"), { observedSequence: 3 });
  await writer.state("candidate", candidate, 4);
  // Act
  await saveAccounting(root, context, { stage: "after", state: candidate, ledger, original_budget: original }, writer.lastState());
  // Assert
  assert.equal(JSON.parse(await readFile(join(root, "accounting-after.json"))).ledger.total_charged_ms, 1560000);
});

test("Share restoration uses its matched checkpoint without changing the original high-water result", async (t) => {
  // Arrange
  const { context } = await setup(t, "share"), restored = state(context);
  restored.preservation.authorization_high_water_match = false;
  restored.qualification = { generation: 7 };
  restored.authorizationRecovery = { schema: "worker-authorization-recovery-v1", checkpointId: Buffer.alloc(16, 3).toString("base64url"),
    generation: 7, matched: true };
  // Act / Assert
  assert.doesNotThrow(() => restoredBaseline(restored, context));
  assert.equal(restored.preservation.authorization_high_water_match, false);
  assert.throws(() => baseline(restored), { code: "v2_baseline" });
  assert.throws(() => restoredBaseline(restored, { ...context, scope: "channel" }), { code: "v2_baseline" });
  restored.authorizationRecovery.generation = 8;
  assert.throws(() => restoredBaseline(restored, context), { code: "v2_authorization_restoration" });
});
