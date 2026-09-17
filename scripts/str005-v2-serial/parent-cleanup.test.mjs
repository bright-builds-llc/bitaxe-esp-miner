import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { firstFailure, failureOutcome } from "./disposition.mjs";
import { sha256 } from "./values.mjs";
import { completedFixture } from "./completed-fixture.mjs";
import { finalize, review } from "./finalize.mjs";

async function fixture(t, schema = "str005-v2-serial-context-v4") {
  const root = await mkdtemp(join(tmpdir(), "v2-parent-cleanup-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const context = { schema, scope: "channel" };
  const parent = { schema: "str005-v2-parent-cleanup-failure-v1", source: "parent-observed",
    contextSha256: sha256(JSON.stringify(context)), stage: "record", code: "v2_pool_listener_present", observedAtUnixMs: 1750000000000 };
  return { root, context, parent };
}

test("parent cleanup failure retains its provenance without mixing clock domains", async t => {
  // Arrange
  const f = await fixture(t); await writeNew(join(f.root, "parent-cleanup-failure.json"), f.parent);
  // Act
  const result = await firstFailure(f.root, f.context, "v2_parent_cleanup_failed");
  // Assert
  assert.equal(result.source, "parent"); assert.equal(result.code, "v2_pool_listener_present");
  assert.equal(result.ordering, "parent-observed"); assert.equal(result.observedAtHostMs, null);
  assert.equal(result.sourceSequence, null); assert.equal(result.cause, null);
  assert.equal(failureOutcome(result, result.code), "stop_evidence_incomplete");
});

test("later parent cleanup failure cannot replace the earlier supervisor cause", async t => {
  // Arrange
  const f = await fixture(t);
  await writeNew(join(f.root, "parent-cleanup-failure.json"), f.parent);
  await writeNew(join(f.root, "failure.json"), { schema: "str005-v2-first-failure-v1",
    contextSha256: sha256(JSON.stringify(f.context)), code: "v2_client_start_timeout", atHostMs: 42,
    deviceCause: null, sourceSequence: null });
  // Act
  const result = await firstFailure(f.root, f.context, "v2_parent_cleanup_failed");
  // Assert
  assert.equal(result.source, "supervisor"); assert.equal(result.code, "v2_client_start_timeout");
  assert.equal(result.observedAtHostMs, 42);
});

for (const [label, change] of [
  ["context", { contextSha256: "0".repeat(64) }],
  ["source", { source: "browser" }],
  ["stage", { stage: "invented" }],
  ["code", { code: "v2_arbitrary_untrusted_detail" }],
]) test(`unjoinable parent ${label} is not trusted as observed provenance`, async t => {
  // Arrange
  const f = await fixture(t); await writeNew(join(f.root, "parent-cleanup-failure.json"), { ...f.parent, ...change });
  // Act
  const result = await firstFailure(f.root, f.context, "v2_parent_cleanup_failed");
  // Assert
  assert.equal(result.source, "judge"); assert.equal(result.code, "v2_parent_cleanup_record");
  assert.equal(result.ordering, "unproved");
});

test("legacy disposition does not reinterpret an artifact introduced by v3", async t => {
  // Arrange
  const f = await fixture(t, "str005-v2-serial-context-v2");
  await writeNew(join(f.root, "parent-cleanup-failure.json"), f.parent);
  // Act
  const result = await firstFailure(f.root, f.context, "v2_baseline");
  // Assert
  assert.equal(result.source, "judge"); assert.equal(result.code, "v2_baseline");
  assert.equal(result.ordering, "no-producer-cause");
});

test("a parent failure prevents qualification even when complete cleanup receipts exist", async t => {
  // Arrange: otherwise complete software evidence, never a real hardware run.
  const f = await completedFixture(t);
  assert.equal(f.context.schema, "str005-v2-serial-context-v4");
  await writeNew(join(f.root, "parent-cleanup-failure.json"), {
    schema: "str005-v2-parent-cleanup-failure-v1", source: "parent-observed",
    contextSha256: sha256(JSON.stringify(f.context)), stage: "record",
    code: "v2_parent_cleanup_failed", observedAtUnixMs: 1750000000000,
  });
  // Act
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  // Assert: later complete receipts cannot erase a sticky first failure.
  assert.equal(result.status, "unverified"); assert.equal(result.hardware_qualified, false);
  assert.equal((await proof(f.root, "final-result.json")).value.firstFailure.source, "parent");
  assert.deepEqual(await review(f.root, f.operations), result);
});


test("historical v3 retains its parent failure interpretation", async t => {
  // Arrange
  const f = await fixture(t, "str005-v2-serial-context-v3");
  await writeNew(join(f.root, "parent-cleanup-failure.json"), f.parent);
  // Act
  const result = await firstFailure(f.root, f.context, "v2_parent_cleanup_failed");
  // Assert
  assert.equal(result.source, "parent");
  assert.equal(result.code, "v2_pool_listener_present");
  assert.equal(result.ordering, "parent-observed");
});
