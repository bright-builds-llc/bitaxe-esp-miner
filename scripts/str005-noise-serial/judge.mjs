import { isDeepStrictEqual as equal } from "node:util";
import { resolve } from "node:path";
import { exactObject, missing } from "../fixed-usb-qualification/contract.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { validateCycle } from "../fixed-usb-qualification/judge.mjs";
import { baseline, healthy, readJournal, readNoiseJournal } from "./journal.mjs";
import { inspectInstall, inspectProbe } from "./install.mjs";
import { inspectCleanup } from "./cleanup.mjs";
import { collectInputs } from "./inputs.mjs";
import { inspectProtocolV2 } from "./protocol-v2.mjs";
import { canonical, check, digest, proof } from "./files.mjs";

async function accounting(root, context, stage, rows) {
  const record = await proof(root, `accounting-${stage}.json`), v = record.value;
  exactObject(v, ["schema", "contextSha256", "observedSequence", "stage", "state", "ledger", "original_budget"]);
  check(v.schema === "noise-serial-accounting-v2" && v.contextSha256 === digest(JSON.stringify(context)) && v.stage === stage &&
    equal(rows[v.observedSequence - 1]?.state, v.state), "noise_accounting_join");
  baseline(v.state, false);
  requireIdleLedger(v.ledger, context.expected_ledger.next_ordinal, context.expected_ledger.total_charged_ms);
  requireExhaustedOriginal(v.original_budget);
  return record;
}
async function continuity(root, context, rows) {
  let previous, previousAfter = 0;
  const first = await inspectInstall(root, context, 0);
  const initial = rows[first.claim.beforeSequence - 1]; baseline(initial.state, true);
  check(first.claim.beforeStateSha256 === digest(canonical(initial)), "noise_initial_claim_state");
  check(initial.phase === "before", "noise_initial_baseline_phase");
  for (let index = 1; index <= 4; index++) {
    const installed = await inspectInstall(root, context, index), cycle = (await proof(root, `cycle-${index}.json`)).value;
    exactObject(cycle, ["schema", "contextSha256", "beforeSequence", "afterSequence", "installReviewSha256", "report"]);
    const review = await proof(root, `install-${index}.review.json`);
    check(cycle.schema === "noise-serial-cycle-v2" && cycle.contextSha256 === digest(JSON.stringify(context)) &&
      cycle.installReviewSha256 === review.sha256 && cycle.beforeSequence === installed.claim.beforeSequence &&
      cycle.afterSequence > cycle.beforeSequence && cycle.beforeSequence > previousAfter &&
      installed.claim.detector.physical === first.claim.detector.physical, "noise_cycle_binding");
    const probe = await inspectProbe(root, context, index, rows);
    check(probe.beforeSequence > cycle.beforeSequence && probe.afterSequence === cycle.afterSequence, "noise_cycle_fresh_probe");
    const before = rows[cycle.beforeSequence - 1], after = rows[cycle.afterSequence - 1];
    check(before && after && after.phase === "candidate" && after.atHostMs > review.value.atHostMs, "noise_cycle_observations");
    baseline(before.state, true); baseline(after.state, false);
    check(installed.claim.beforeStateSha256 === digest(canonical(before)), "noise_claim_state_changed");
    check(after.state.probe?.requestPayloadBytes === 65536 && after.state.probe.responsePayloadBytes === 65536 &&
      after.state.preservation.baseline_id === initial.state.preservation.baseline_id &&
      before.state.preservation.baseline_id === initial.state.preservation.baseline_id, "noise_cycle_preservation");
    previous = validateCycle(cycle.report, context, previous); previousAfter = after.sequence;
  }
  return { baselineId: initial.state.preservation.baseline_id, lastCycleSequence: previousAfter };
}
/** Re-derive acceptance from protected original observations, not supplied pass flags. */
export async function judge(root, context, cleanupPath, operations = {}) {
  await missing(resolve(root, "failure.json"));
  const rows = await readJournal(root, context);
  check(rows.length > 0 && rows.every((row) => !row.state.failure && !row.state.ownerResourceFailure), "noise_browser_failure");
  const cycle = await continuity(root, context, rows);
  const initial = await accounting(root, context, "before-install", rows), before = await accounting(root, context, "before", rows),
    after = await accounting(root, context, "after", rows);
  check(before.value.observedSequence > cycle.lastCycleSequence && after.value.observedSequence > before.value.observedSequence &&
    equal(before.value.ledger, after.value.ledger) && equal(initial.value.ledger, after.value.ledger) &&
    equal(before.value.original_budget, after.value.original_budget) && equal(initial.value.original_budget, after.value.original_budget), "noise_unchanged_accounting");
  healthy(before.value.state); healthy(after.value.state);
  const serverClaim = (await proof(root, "server.claim.json")).value;
  check(serverClaim.schema === "noise-serial-server-claim-v2" && serverClaim.contextSha256 === digest(JSON.stringify(context)), "noise_server_claim_join");
  const start = (await proof(root, "start.claim.json")).value;
  exactObject(start, ["schema", "contextSha256", "atHostMs", "observedSequence", "inputSha256", "start", "observation"]);
  check(start.schema === "noise-serial-start-claim-v2" && start.contextSha256 === digest(JSON.stringify(context)) &&
    start.start.attemptId === context.attempt_id && start.inputSha256 === digest(canonical(start.start)) &&
    start.observedSequence >= before.value.observedSequence && start.observation.state === "idle" &&
    start.observation.observation.observedAtUs === start.start.networkObservedAtUs, "noise_start_claim_join");
  const statuses = await readNoiseJournal(root, context);
  check(statuses[0]?.status.job?.bootOrdinal === start.observation.observation.bootOrdinal &&
    statuses[0].status.job.workerGeneration === start.observation.observation.workerGeneration &&
    statuses[0].status.job.transportEpoch === start.observation.observation.transportEpoch, "noise_admission_binding");
  check(statuses.length > 1 && statuses[0].atHostMs >= start.atHostMs, "noise_start_record_order");
  const ready = await proof(root, "fixture-run/ready.json"), terminal = await proof(root, "fixture-run/terminal.json");
  const protocol = inspectProtocolV2(start.start, statuses.map((row) => row.status), ready.value, terminal.value);
  const readyObservation = (await proof(root, "fixture-ready-observation.json")).value,
    fixtureClaim = (await proof(root, "fixture-start.claim.json")).value,
    fixtureExit = (await proof(root, "fixture-exit.json")).value, fixtureReap = (await proof(root, "fixture-reap.json")).value;
  check(readyObservation.readySha256 === ready.sha256 && readyObservation.atHostMs >= fixtureClaim.atHostMs &&
    readyObservation.atHostMs - fixtureClaim.atHostMs <= 5000 && start.atHostMs >= readyObservation.atHostMs &&
    fixtureExit.code === 0 && fixtureExit.lifetimeMs >= 0 && fixtureExit.lifetimeMs <= 150000 &&
    fixtureReap.schema === "noise-serial-fixture-reap-v2" && fixtureReap.contextSha256 === digest(JSON.stringify(context)) &&
    fixtureReap.durationMs >= 0 && fixtureReap.durationMs <= 5000 && fixtureReap.completedAtHostMs - fixtureReap.startedAtHostMs === fixtureReap.durationMs && fixtureExit.atHostMs - readyObservation.atHostMs === fixtureExit.lifetimeMs,
  "noise_fixture_chronology");
  const complete = (await proof(root, "diagnostic-complete.json")).value, restored = (await proof(root, "restoration.json")).value;
  const completionStatus = statuses.filter((row) => row.atHostMs <= complete.atHostMs).at(-1);
  check(completionStatus?.status.job?.terminal?.outcome === "accepted" &&
    complete.statusSha256 === digest(canonical(completionStatus.status)) && rows[complete.observedSequence - 1]?.atHostMs <= complete.atHostMs,
    "noise_completion_status_join");
  check(complete.contextSha256 === digest(JSON.stringify(context)) && complete.observedSequence > before.value.observedSequence &&
    complete.fixtureSha256 === digest(canonical(terminal.value)) && restored.contextSha256 === complete.contextSha256 &&
    restored.closedSequence > complete.observedSequence && restored.observedSequence > restored.closedSequence &&
    after.value.observedSequence > restored.observedSequence, "noise_restoration_order");
  const closed = rows[restored.closedSequence - 1], restoredState = rows[restored.observedSequence - 1];
  baseline(closed.state, true); baseline(restoredState.state, false);
  check(equal(restored.state, restoredState.state) && restored.status.job.attemptId === context.attempt_id &&
    restored.status.observation.bootOrdinal === protocol.job.bootOrdinal &&
    restored.status.observation.transportEpoch !== protocol.job.transportEpoch &&
    statuses.some((row) => equal(row.status, restored.status) && row.atHostMs > closed.atHostMs && row.atHostMs <= restoredState.atHostMs), "noise_fresh_restoration_join");
  for (const value of [initial.value.state, before.value.state, after.value.state, restored.state, rows.at(-1).state])
    check(value.preservation.baseline_id === cycle.baselineId, "noise_page_baseline_changed");
  const a = before.value.state.qualification, b = after.value.state.qualification;
  check(a && b && ["generation", "work_dispatched", "submitted", "accepted", "rejected"].every((key) => a[key] === b[key]), "noise_new_work");
  check(rows.at(-1).sequence > after.value.observedSequence, "noise_final_journal_order"); baseline(rows.at(-1).state, true);
  const cleanup = await inspectCleanup(root, context, cleanupPath, operations);
  return { protocol, cleanup, inputs: await collectInputs(root, operations.cleanupSnapshot),
    hostTiming: { fixture_lifetime: fixtureExit.lifetimeMs, host_cleanup: Math.max(fixtureReap.durationMs, cleanup.cleanupMs) } };
}
