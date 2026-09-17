import assert from "node:assert/strict";
import { chmod, readFile, unlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { contextFixture } from "./context-fixtures.mjs";
import { finalize, review } from "./finalize.mjs";
import { collectInputs } from "./inputs.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { channelFixture } from "./protocol-judge.test-helper.mjs";
import { sha256 } from "./values.mjs";

test("partial run seals available inputs and never emits an accepted projection", async t => {
  // Arrange: real protected context/snapshot with explicit synthetic native/predecessor readers.
  const f = await contextFixture(t);
  await writeNew(resolve(f.root, "accounting-before-install.json"), { incompleteSyntheticObservation: true });
  // Act: absence of a running owner is established before finalization.
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  // Assert: missing proof is retained; neither firstFailure nor source input is fabricated as a pass.
  assert.equal(result.status, "unverified"); assert.equal(result.hardware_qualified, false);
  const saved = (await proof(f.root, "final-result.json")).value;
  assert.equal(saved.firstFailure.source, "judge"); assert.equal(saved.inputs.accounting.length, 1);
  assert.equal(saved.inputs.stateJournal, null);
  assert.deepEqual(await review(f.root, f.operations), result);
  await assert.rejects(readFile(resolve(f.root, "projection.json")), { code: "ENOENT" });
  await assert.rejects(finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations));
});

test("independent review detects mutation, new membership and permissive modes", async t => {
  const f = await contextFixture(t);
  await writeNew(resolve(f.root, "accounting-before-install.json"), { partial: true });
  await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  const path = resolve(f.root, "accounting-before-install.json"), bytes = await readFile(path);
  await review(f.root, f.operations);
  await writeFile(path, `${bytes} `);
  await assert.rejects(review(f.root, f.operations), { code: "noise_inventory_changed" });
  await writeFile(path, bytes);
  await writeNew(resolve(f.root, "unexpected.json"), { partial: true });
  await assert.rejects(review(f.root, f.operations), { code: "noise_inventory_changed" });
  await unlink(resolve(f.root, "unexpected.json")); await chmod(path, 0o644);
  await assert.rejects(review(f.root, f.operations));
  await chmod(path, 0o600); await review(f.root, f.operations);
});

test("native failure is retained without a fabricated native completion receipt", async t => {
  const f = await contextFixture(t), operations = { ...f.operations,
    inspectNative: async () => { throw Object.assign(new Error("synthetic native rejection"), { code: "v2_native_stack" }); } };
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, operations);
  assert.equal(result.status, "unverified");
  assert.equal((await proof(f.root, "judgment-failure.json")).value.nativeCode, "v2_native_stack");
  await assert.rejects(readFile(resolve(f.root, "native/final-readiness.json")), { code: "ENOENT" });
  assert.deepEqual(await review(f.root, f.operations), result);
});

test("inputs retain malformed source bytes rather than replacing all partial evidence with null", async t => {
  const f = await contextFixture(t);
  await f.put(resolve(f.root, "state-0001.json"), "{incomplete");
  const inputs = await collectInputs(f.root);
  assert.equal(inputs.stateJournal[0].length, 11);
  assert.equal(inputs.deviceJournal, null);
  assert(inputs.native.length > 0);
});

test("typed causes survive while an earlier generic first observation keeps its provenance", async t => {
  const f = await contextFixture(t), contextSha256 = sha256(JSON.stringify(f.context));
  const admitted = channelFixture().deviceRecords[0]; admitted.attemptId = f.context.attemptId;
  const failed = { ...structuredClone(admitted), state: "terminal", outcome: "rejected", terminalAtDeviceUs: 1100, observedAtUs: 1100,
    firstFailure: { stage: "authenticated", category: "authentication", atDeviceUs: 1100 } };
  for (const [index, record] of [admitted, failed].entries()) await writeNew(resolve(f.root, `device-000${index + 1}.json`), {
    schema: "str005-v2-device-record-v1", contextSha256, sequence: index + 1, atHostMs: index + 1, record });
  await writeNew(resolve(f.root, "failure.json"), { schema: "str005-v2-first-failure-v1", contextSha256,
    code: "v2_operation_failed", atHostMs: 0, deviceCause: null, sourceSequence: null });
  const fixtureCause = { stage: "setup_received", category: "eof", atFixtureUs: 500 };
  await f.put(resolve(f.root, "fixture-run/fixture-terminal.json"), JSON.stringify({ ...channelFixture().fixtureTerminal,
    outcome: "unverified", firstFailure: fixtureCause }));
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  assert.equal(result.outcome, "stop_hardware_blocker");
  const saved = (await proof(f.root, "final-result.json")).value;
  assert.equal(saved.firstFailure.source, "supervisor"); assert.equal(saved.firstFailure.observedAtHostMs, 0);
  assert.equal(saved.firstFailure.code, "v2_operation_failed"); assert.equal(saved.firstFailure.cause, null);
  assert.deepEqual(saved.firstFailure.availableCauses.device.cause, failed.firstFailure);
  assert.deepEqual(saved.firstFailure.availableCauses.fixture, fixtureCause);
  assert.deepEqual(await review(f.root, f.operations), result);
});
