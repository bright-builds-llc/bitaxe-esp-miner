import { accountingBaselineAllowed } from "./accounting-baseline.mjs";
import { CONTEXT_V4 } from "./values.mjs";
import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { canonical, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { validateState } from "../fixed-usb-qualification/judge.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { baseline } from "../str005-v2-serial/journal.mjs";
import { contextHash, legacyView } from "./context.mjs";
import { check, object, schema, sha256, uint } from "./values.mjs";
export { baseline };
export function checkedState(state, context, phase) {
  check(["before", "candidate"].includes(phase), "bootstrap_phase");
  validateState(state, legacyView(context, phase));
  check(!state.running && !state.heartbeatSuppressed && state.renewalsConfirmed === 0 && !["running", "window_loaded"].includes(state.status), "bootstrap_mining_forbidden");
  return state;
}
export async function readJournal(root, context) {
  const names = (await readdir(root)).filter(name => /^state-[0-9]{4}\.json$/u.test(name)).sort(); check(names.length <= 1024, "bootstrap_journal_bound");
  const rows = [];
  for (const [index, name] of names.entries()) {
    check(name === `state-${String(index + 1).padStart(4, "0")}.json`, "bootstrap_journal_gap");
    const row = (await proof(root, name)).value; object(row, ["schema", "contextSha256", "sequence", "phase", "atHostMs", "state"]);
    check(row.schema === schema("state") && row.contextSha256 === contextHash(context) && row.sequence === index + 1, "bootstrap_journal_identity");
    uint(row.atHostMs); checkedState(row.state, context, row.phase);
    if (rows.length) check(row.atHostMs >= rows.at(-1).atHostMs && !(rows.at(-1).phase === "candidate" && row.phase === "before"), "bootstrap_journal_order");
    rows.push(row);
  }
  return rows;
}
export async function createJournal(root, context) {
  const rows = await readJournal(root, context); let last = rows.at(-1);
  return { last: () => last, async record(phase, state) {
    checkedState(state, context, phase); check(rows.length < 1024, "bootstrap_journal_bound");
    const row = { schema: schema("state"), contextSha256: contextHash(context), sequence: rows.length + 1, phase, atHostMs: Math.floor(performance.now()), state };
    await writeNew(resolve(root, `state-${String(row.sequence).padStart(4, "0")}.json`), row); rows.push(row); last = row; return { recorded: true, sequence: row.sequence };
  } };
}
function accountingBaseline(state, context, stage) {
  if (context.schema === CONTEXT_V4) check(accountingBaselineAllowed(state, stage), "bootstrap_accounting_baseline");
  else baseline(state);
}
export async function saveAccounting(root, context, input, last) {
  object(input, ["stage", "ledger", "original", "state"]); check(["before", "after"].includes(input.stage), "bootstrap_accounting_stage");
  requireIdleLedger(input.ledger, 18, 1560000); requireExhaustedOriginal(input.original); accountingBaseline(input.state, context, input.stage);
  check(last && last.phase === (input.stage === "before" ? "before" : "candidate") && canonical(last.state) === canonical(input.state), "bootstrap_accounting_join");
  check(canonical({ ledger: input.ledger, original: input.original }) === canonical(context.expectedAccounting), "bootstrap_ledger_changed");
  if (input.stage === "after") {
    const before = (await proof(root, "accounting-before.json")).value;
    check(input.state.preservation.baseline_id === before.state.preservation.baseline_id, "bootstrap_baseline_changed");
    for (const key of ["budget_reserved_ms", "submitted", "accepted", "rejected", "work_dispatched", "nonce_work_correlations"])
      check(input.state.qualification?.[key] === before.state.qualification?.[key], "bootstrap_work_observed");
  }
  const value = { schema: schema("accounting"), contextSha256: contextHash(context), ...input, observedSequence: last.sequence };
  await writeNew(resolve(root, `accounting-${input.stage}.json`), value);
  if (input.stage === "after") await writeNew(resolve(root, "restoration.json"), { schema: schema("restoration"), contextSha256: contextHash(context),
    observedSequence: last.sequence, stateSha256: sha256(canonical(last)), accountingSha256: sha256(`${JSON.stringify(value, null, 2)}\n`), confirmed: true });
  return { accounting_saved: true, stage: input.stage };
}
export async function verifyAccounting(root, context) {
  const rows = await readJournal(root, context), before = (await proof(root, "accounting-before.json")).value, after = (await proof(root, "accounting-after.json")).value;
  for (const [stage, value] of [["before", before], ["after", after]]) {
    object(value, ["schema", "contextSha256", "stage", "ledger", "original", "state", "observedSequence"]);
    const row = rows[value.observedSequence - 1];
    check(value.schema === schema("accounting") && value.contextSha256 === contextHash(context) && value.stage === stage && row &&
      canonical(row.state) === canonical(value.state) && row.phase === (stage === "before" ? "before" : "candidate"), "bootstrap_accounting_join");
    accountingBaseline(value.state, context, stage); check(canonical({ ledger: value.ledger, original: value.original }) === canonical(context.expectedAccounting), "bootstrap_ledger_changed");
  }
  check(before.state.preservation.baseline_id === after.state.preservation.baseline_id && after.observedSequence > before.observedSequence, "bootstrap_baseline_changed");
  for (const key of ["budget_reserved_ms", "submitted", "accepted", "rejected", "work_dispatched", "nonce_work_correlations"])
    check(before.state.qualification?.[key] === after.state.qualification?.[key], "bootstrap_work_observed");
  const restore = (await proof(root, "restoration.json")).value;
  check(restore.confirmed === true && restore.contextSha256 === contextHash(context) && restore.observedSequence === after.observedSequence &&
    restore.stateSha256 === sha256(canonical(rows[after.observedSequence - 1])) && restore.accountingSha256 === (await proof(root, "accounting-after.json")).sha256, "bootstrap_restoration");
  return { before, after, last: rows.at(-1) };
}
