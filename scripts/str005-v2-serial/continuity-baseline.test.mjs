import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { contextFixture } from "./context-fixtures.mjs";
import { preparedFixture } from "./completed-fixture.mjs";
import { state } from "../str005-noise-serial/test-fixture.mjs";
import { createJournal, readJournal } from "./journal.mjs";
import { readAccounting } from "./accounting-judge.mjs";
import { judgeContinuity } from "./continuity-judge.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";

const SOURCE = "scripts/str005-v2-serial/continuity-judge.mjs";
const HISTORICAL = { path: SOURCE, sha256: "d49635695c66849ab053343f32c5d681c20aea3ca144e4f100a673cb88b2b640", length: 3883 };
async function realisticPreparation(t) {
  const f = await contextFixture(t), journal = await createJournal(f.root, f.context);
  const configured = state(f.context, "before"); delete configured.preservation;
  Object.assign(configured, { status: "configured", connected: false, running: false, deviceBaselineConfirmed: false,
    deviceRestorationConfirmed: false, deviceLeaseInactive: false, serialOwnershipReleased: true });
  await journal.state("before", configured, 1);
  // Gate represents connecting as configured with ownership acquisition, before fresh admission.
  await journal.state("before", { ...configured, serialOwnershipReleased: false }, 2);
  await preparedFixture(t, f);
  const rows = await readJournal(f.root, f.context), initial = await readAccounting(f.root, f.context, rows, "before-install");
  const before = await readAccounting(f.root, f.context, rows, "before");
  return { ...f, rows, initial, before };
}

test("new evaluator selects the authenticated accounting row after configured/connecting states", async t => {
  // Arrange: use real installation/probe/cycle producers after realistic page initialization.
  const f = await realisticPreparation(t), implementation = f.context.evaluator.find(row => row.path === SOURCE);
  assert.notEqual(implementation.sha256, HISTORICAL.sha256);
  assert.equal(implementation.sha256, sha256(await readFile(new URL("./continuity-judge.mjs", import.meta.url))));
  assert.equal(f.initial.observedSequence, 3); assert.equal(f.rows[0].state.deviceBaselineConfirmed, false);
  // Act / Assert: every installation and fresh cycle still receives its independent checks.
  const result = await judgeContinuity(f.root, f.context, f.rows, f.before, f.initial);
  assert.equal(result.installations, 5); assert.equal(result.cycles, 4);
});

test("new selector rejects missing, stale, cross-context or mismatched accounting pointers", async t => {
  // Arrange: preserve the complete journal and source-produced accounting bytes.
  const f = await realisticPreparation(t), inspect = (initial = f.initial, rows = f.rows) => judgeContinuity(f.root, f.context, rows, f.before, initial);
  // Act / Assert: neither missing joins nor a self-consistent but inadmissible pointer can pass.
  await assert.rejects(judgeContinuity(f.root, f.context, f.rows, f.before), { code: "v2_initial_accounting_join" });
  const path = resolve(f.root, "accounting-before-install.json"), bytes = await readFile(path);
  for (const mutate of [
    value => { value.observedSequence = 1; value.state = f.rows[0].state; },
    value => { value.contextSha256 = "0".repeat(64); },
    value => { value.stage = "before"; },
    value => { value.observedSequence = f.before.observedSequence; },
    value => { value.state = { ...value.state, connected: false }; },
  ]) {
    const value = JSON.parse(bytes); mutate(value); await writeFile(path, JSON.stringify(value));
    await assert.rejects(inspect()); await assert.rejects(inspect(value)); await writeFile(path, bytes);
  }
  const rows = structuredClone(f.rows); rows[2].contextSha256 = "0".repeat(64);
  await assert.rejects(inspect(f.initial, rows), { code: "v2_initial_accounting_join" });
  await unlink(path); await assert.rejects(inspect(), { code: "ENOENT" }); await writeFile(path, bytes, { mode: 0o600 });
  assert.equal((await inspect()).cycles, 4);
});

test("only the exact historical evaluator entry retains the old first-before-row rejection", async t => {
  // Arrange: the two original captures bind this exact published implementation digest and size.
  const f = await realisticPreparation(t), historic = { ...f.context, evaluator: f.context.evaluator.map(row => row.path === SOURCE ? HISTORICAL : row) };
  // Act / Assert: historical selection stays rejected; malformed source identities cannot opt into it.
  await assert.rejects(judgeContinuity(f.root, historic, f.rows, f.before, f.initial), { code: "v2_baseline" });
  for (const evaluator of [undefined, {}, [], [...historic.evaluator, HISTORICAL],
    historic.evaluator.map(row => row.path === SOURCE ? { ...HISTORICAL, length: 3884 } : row),
    historic.evaluator.map(row => row.path === SOURCE ? { ...HISTORICAL, sha256: "invalid" } : row)]) {
    await assert.rejects(judgeContinuity(f.root, { ...f.context, evaluator }, f.rows, f.before, f.initial), { code: "v2_continuity_evaluator" });
  }
  assert.equal((await proof(f.root, "accounting-before-install.json")).value.observedSequence, 3);
});
