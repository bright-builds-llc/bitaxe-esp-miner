import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, requireCondition as check, writeNew } from "./contract.mjs";
import { proof, baseline } from "./cadence-premining-evidence.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { validateState } from "./judge.mjs";

export function validateRestartSummary(restart) {
  exactObject(restart, [
    "schema",
    "stage",
    "ackMatched",
    "expectedBootOrdinal",
    "nextBootOrdinal",
    "bootObserved",
    "softwareResetObserved",
    "identityObserved",
    "identityMatched",
    "runtimeReadyObserved",
    "records",
    "bytes",
    "durationMs",
    "portReopens",
    "streamInterrupted",
    "continuity",
  ]);
  check(
    restart.schema === "worker-qualification-restart-observation-v1" &&
      ["armed", "acknowledged", "reacquiring", "complete", "failed"].includes(restart.stage) &&
      [
        "ackMatched",
        "bootObserved",
        "softwareResetObserved",
        "identityObserved",
        "identityMatched",
        "runtimeReadyObserved",
        "streamInterrupted",
      ].every((key) => typeof restart[key] === "boolean") &&
      Number.isSafeInteger(restart.expectedBootOrdinal) &&
      restart.expectedBootOrdinal > 0 &&
      restart.expectedBootOrdinal < Number.MAX_SAFE_INTEGER &&
      restart.nextBootOrdinal === restart.expectedBootOrdinal + 1 &&
      ["records", "bytes", "durationMs"].every((key) => Number.isSafeInteger(restart[key]) && restart[key] >= 0) &&
      [0, 1].includes(restart.portReopens) &&
      ["uninterrupted", "interrupted", "same_port_reopened"].includes(restart.continuity),
    "restart_state_summary",
  );
}
export function validateRestartState(value, context) {
  const { restart, ...common } = value;
  if (restart !== undefined) validateRestartSummary(restart);
  // Reuse validation of the unchanged fields; retain the original new status/failure in the actual journal.
  const checked = { ...common, status: common.status === "restarting" ? "ready" : common.status };
  if (checked.failure === "qualification_restart_failed") {
    check(checked.status === "failed", "restart_state_failure");
    delete checked.failure;
  }
  validateState(checked, context);
  check(
    value.running === false &&
      value.renewalsConfirmed === 0 &&
      !["window_loaded", "running"].includes(value.status) &&
      value.cadence === undefined &&
      value.qualification === undefined,
    "restart_work_forbidden",
  );
  return value;
}
export async function readRestartStates(root, context) {
  const names = (await readdir(root)).filter((name) => /^no-mining-state-[0-9]{4}\.json$/u.test(name)).sort();
  check(names.length > 0 && names.length <= 512, "restart_states_missing");
  const records = [];
  for (const [index, name] of names.entries()) {
    const row = (await proof(resolve(root, name))).value;
    exactObject(row, ["schema", "context_sha256", "sequence", "state"]);
    check(
      row.schema === "fixed-usb-restart-state-v1" &&
        row.context_sha256 === digest(JSON.stringify(context)) &&
        row.sequence === index + 1 &&
        name === `no-mining-state-${String(index + 1).padStart(4, "0")}.json`,
      "restart_state_integrity",
    );
    validateRestartState(row.state, context);
    records.push(row);
  }
  return records;
}
export async function saveRestartAccounting(root, context, input) {
  exactObject(input, ["stage", "ledger", "original_budget", "state"]);
  check(["before", "after"].includes(input.stage), "restart_accounting_stage");
  requireIdleLedger(input.ledger, 17, 1380000);
  requireExhaustedOriginal(input.original_budget);
  validateRestartState(input.state, context);
  baseline(input.state, false);
  const row = (await readRestartStates(root, context)).at(-1);
  check(equal(row.state, input.state), "restart_accounting_state_missing");
  if (input.stage === "after") {
    const before = await readRestartAccounting(root, context, "before");
    check(
      equal(before.ledger, input.ledger) &&
        equal(before.original_budget, input.original_budget) &&
        before.state.preservation.baseline_id === input.state.preservation.baseline_id &&
        row.sequence > before.observed_sequence,
      "restart_accounting_changed",
    );
  }
  await writeNew(resolve(root, `no-mining-accounting-${input.stage}.json`), {
    schema: "fixed-usb-restart-accounting-v1",
    context_sha256: digest(JSON.stringify(context)),
    observed_sequence: row.sequence,
    ...input,
  });
  return { accounting_saved: true, stage: input.stage };
}
export async function readRestartAccounting(root, context, stage) {
  const value = (await proof(resolve(root, `no-mining-accounting-${stage}.json`))).value;
  exactObject(value, ["schema", "context_sha256", "observed_sequence", "stage", "ledger", "original_budget", "state"]);
  check(
    value.schema === "fixed-usb-restart-accounting-v1" &&
      value.context_sha256 === digest(JSON.stringify(context)) &&
      value.stage === stage &&
      Number.isSafeInteger(value.observed_sequence) &&
      value.observed_sequence > 0 &&
      value.observed_sequence <= 512,
    "restart_accounting_integrity",
  );
  requireIdleLedger(value.ledger, 17, 1380000);
  requireExhaustedOriginal(value.original_budget);
  validateRestartState(value.state, context);
  baseline(value.state, false);
  check(
    equal((await readRestartStates(root, context))[value.observed_sequence - 1]?.state, value.state),
    "restart_accounting_state_missing",
  );
  return value;
}
