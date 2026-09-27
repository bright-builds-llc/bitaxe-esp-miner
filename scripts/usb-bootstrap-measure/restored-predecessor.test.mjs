import test from "node:test";
import assert from "node:assert/strict";
import { validateRestoredFacts } from "./restored-predecessor.mjs";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { BEFORE_V4, MEASUREMENT_003, CONTEXT_V3 } from "./values.mjs";
function fixture() {
  const context = { schema: CONTEXT_V3, firmwareRoot: "/synthetic", package: BEFORE_V4, expectedAccounting: { ledger, original } };
  const baseline = state({ ...BEFORE_V4, gate_commit: "a".repeat(40) });
  const rows = [{ phase: "before", state: baseline }, ...["ready", "stopping", "baseline_confirmed", "closed"].map(status => ({ phase: "candidate", state: { ...structuredClone(baseline), status, connected: status !== "closed", serialOwnershipReleased: status === "closed" } }))];
  const failure = { source: "browser", stage: "browser", code: "bootstrap_client_failed", observationSha256: null };
  return { root: "/synthetic/scratch/usb-bootstrap-measure/attempt-003", context, failure, rows, last: rows.at(-1),
    accounting: { stage: "before", contextSha256: MEASUREMENT_003.contextSha256, ledger, original, state: baseline, observedSequence: 1 },
    result: { status: "unverified", correction: { accepted: false, checks: { captureQualified: true, hostTimingComplete: true, readerHeadroom: true, nativeBootstrapComplete: true, zeroTxFailures: true, preservationAndAccounting: false, cleanupComplete: false } },
      firstFailure: failure, cleanup: { complete: false }, restoration: { confirmed: false }, capture: { qualified: true, exitCode: 0 }, hardware_qualified: false, mining_authorized: false, qualification_credit: "none" } };
}
test("failed restored predecessor retains prior-before-only accounting and negative result", () => { const facts = fixture(); assert.doesNotThrow(() => validateRestoredFacts(facts)); assert.equal(facts.result.status, "unverified"); });
for (const [name, mutate] of Object.entries({
  promoted_result: f => { f.result.status = "measurement_complete"; },
  wrong_failure: f => { f.failure.code = "other"; },
  invented_cleanup: f => { f.result.cleanup.complete = true; },
  invented_after_ledger: f => { f.accounting.stage = "after"; },
  wrong_source: f => { f.context.package = { ...BEFORE_V4, firmware_commit: "b".repeat(40) }; },
  missing_restore: f => { f.last.state.deviceRestorationConfirmed = false; },
  changed_ledger: f => { f.accounting.ledger = { ...ledger, next_ordinal: 19 }; },
  observed_renewal: f => { f.rows[2].state.renewalsConfirmed = 1; },
  missing_ordered_stop: f => { f.rows[2].state.status = "ready"; },
})) test(`guard rejects ${name}`, () => { const f = fixture(); mutate(f); assert.throws(() => validateRestoredFacts(f)); });

import { mkdtemp, chmod, writeFile, rm, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { restoredPins } from "./restored-predecessor.mjs";
async function privateFixture(t) {
  const root = await realpath(await mkdtemp(join(tmpdir(), "bootstrap-predecessor-"))); await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true })); return root;
}
test("missing predecessor artifacts reject without writing replacement evidence", async t => {
  const root = await privateFixture(t); await assert.rejects(restoredPins(root), { code: "ENOENT" });
});
test("altered anchored context rejects before any ancestry or resource admission", async t => {
  const root = await privateFixture(t);
  for (const name of ["context.json", "final-result.json", "sealed-inventory.json", "accounting-before.json", "failure.json"])
    await writeFile(join(root, name), JSON.stringify(name === "context.json" ? { context: {}, sha256: "a".repeat(64) } : {}), { mode: 0o600 });
  await assert.rejects(restoredPins(root), { code: "bootstrap_restored_anchor" });
});
