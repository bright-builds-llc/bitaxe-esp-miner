import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { exactObject } from "../fixed-usb-qualification/contract.mjs";
import { validateState } from "../fixed-usb-qualification/judge.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { baseline } from "../fixed-usb-qualification/cadence-premining-evidence.mjs";
import { parseNoiseStatusV2, validateNoiseProgressV2 } from "./device-v2.mjs";
import { check, digest, proof, writeNew } from "./files.mjs";
export { baseline };

export function stateContext(context, phase) {
  check(["before", "candidate"].includes(phase), "noise_phase");
  return { ...context, ...(phase === "before" ? context.before_source : {}) };
}
export function checkedState(state, context, phase) {
  if (state.status === "restoration_pending") {
    check(phase === "candidate" && !state.running, "noise_pending_restoration_shape");
    validateState({ ...state, status: "restoration_unconfirmed" }, stateContext(context, phase));
  } else validateState(state, stateContext(context, phase));
  check(!state.running && !state.heartbeatSuppressed && state.renewalsConfirmed === 0 &&
    !["running", "window_loaded"].includes(state.status), "noise_mining_forbidden");
  return state;
}
export function healthy(state) {
  baseline(state, false);
  const q = state.qualification;
  check(q && q.watchdog_alive && !q.mine_on_boot && q.voltage_fresh && q.power_fresh &&
    q.temperature_fresh && q.fan_fresh, "noise_fresh_health_required");
}
export async function readJournal(root, context) {
  const names = (await readdir(root)).filter((name) => /^state-[0-9]{4}\.json$/u.test(name)).sort();
  check(names.length <= 1024, "noise_journal_bound");
  const rows = [];
  for (const [index, name] of names.entries()) {
    const record = (await proof(root, name)).value;
    exactObject(record, ["schema", "contextSha256", "sequence", "phase", "atHostMs", "state"]);
    check(record.schema === "noise-serial-state-v2" && record.contextSha256 === digest(JSON.stringify(context)) &&
      record.sequence === index + 1 && name === `state-${String(index + 1).padStart(4, "0")}.json` &&
      Number.isSafeInteger(record.atHostMs) && record.atHostMs >= 0, "noise_journal_identity");
    checkedState(record.state, context, record.phase);
    if (rows.length) check(record.atHostMs >= rows.at(-1).atHostMs &&
      !(rows.at(-1).phase === "candidate" && record.phase === "before"), "noise_journal_order");
    rows.push(record);
  }
  return rows;
}
export async function recordState(root, context, phase, state, now) {
  checkedState(state, context, phase);
  const rows = await readJournal(root, context), sequence = rows.length + 1;
  check(sequence <= 1024 && Number.isSafeInteger(now) && now >= (rows.at(-1)?.atHostMs ?? 0), "noise_journal_bound");
  await writeNew(resolve(root, `state-${String(sequence).padStart(4, "0")}.json`), {
    schema: "noise-serial-state-v2", contextSha256: digest(JSON.stringify(context)), sequence, phase, atHostMs: now, state });
  return { recorded: true, sequence };
}
export async function saveAccounting(root, context, input) {
  exactObject(input, ["stage", "state", "ledger", "original_budget"]);
  check(["before-install", "before", "after"].includes(input.stage), "noise_accounting_stage");
  requireIdleLedger(input.ledger, context.expected_ledger.next_ordinal, context.expected_ledger.total_charged_ms);
  requireExhaustedOriginal(input.original_budget);
  const rows = await readJournal(root, context), last = rows.at(-1);
  check(last && equal(last.state, input.state), "noise_accounting_unpublished"); baseline(input.state, false);
  check(last.phase === (input.stage === "before-install" ? "before" : "candidate"), "noise_accounting_phase");
  if (input.stage !== "before-install") {
    const prior = (await proof(root, "accounting-before-install.json")).value;
    check(equal(prior.ledger, input.ledger) && equal(prior.original_budget, input.original_budget) &&
      prior.state.preservation.baseline_id === input.state.preservation.baseline_id && last.sequence > prior.observedSequence, "noise_accounting_changed");
  }
  if (input.stage === "after") {
    const before = (await proof(root, "accounting-before.json")).value;
    const restored = (await proof(root, "restoration.json")).value;
    check(last.sequence > restored.observedSequence && restored.observedSequence > before.observedSequence, "noise_accounting_order");
  }
  await writeNew(resolve(root, `accounting-${input.stage}.json`), { schema: "noise-serial-accounting-v2",
    contextSha256: digest(JSON.stringify(context)), observedSequence: last.sequence, ...input });
  return { accounting_saved: true, stage: input.stage };
}
export async function readNoiseJournal(root, context) {
  const names = (await readdir(root)).filter((name) => /^noise-[0-9]{4}\.json$/u.test(name)).sort();
  check(names.length <= 600, "noise_status_bound");
  const rows = [];
  for (const [index, name] of names.entries()) {
    const row = (await proof(root, name)).value;
    exactObject(row, ["schema", "contextSha256", "sequence", "atHostMs", "status"]);
    check(row.schema === "noise-serial-status-record-v2" && row.contextSha256 === digest(JSON.stringify(context)) &&
      row.sequence === index + 1 && name === `noise-${String(index + 1).padStart(4, "0")}.json` &&
      Number.isSafeInteger(row.atHostMs) && row.atHostMs >= (rows.at(-1)?.atHostMs ?? 0), "noise_status_order");
    parseNoiseStatusV2(row.status);
    if (rows.length) validateNoiseProgressV2(rows.at(-1).status, row.status);
    rows.push(row);
  }
  return rows;
}
export async function recordNoise(root, context, status, now) {
  parseNoiseStatusV2(status);
  check(status.job === null || status.job.attemptId === context.attempt_id, "noise_status_attempt");
  const rows = await readNoiseJournal(root, context), sequence = rows.length + 1;
  check(sequence <= 600, "noise_status_bound");
  if (rows.length) validateNoiseProgressV2(rows.at(-1).status, status);
  await writeNew(resolve(root, `noise-${String(sequence).padStart(4, "0")}.json`), {
    schema: "noise-serial-status-record-v2", contextSha256: digest(JSON.stringify(context)), sequence, atHostMs: now, status });
  return { recorded: true, sequence };
}
