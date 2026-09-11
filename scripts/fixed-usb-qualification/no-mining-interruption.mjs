import { resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { digest, exactObject, QualificationError, requireCondition, writeNew } from "./contract.mjs";
import { validateState } from "./judge.mjs";
import { validateNoMiningContext } from "./no-mining-context.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";

function validateClosedInput(input, context) {
  validateNoMiningContext(context);
  exactObject(input, ["receipt", "before", "after"]);
  exactObject(input.receipt, ["schema", "interrupted", "request_consumed", "response_pending", "ownership_released"]);
  requireCondition(input.receipt.schema === "worker-read-interruption-v1" &&
    ["interrupted", "request_consumed", "response_pending", "ownership_released"].every((key) => typeof input.receipt[key] === "boolean"),
    "read_only_interruption_shape");
  validateState(input.before, context);
  validateState(input.after, context);
}

export function validateNoMiningReadOnlyInterruption(input, context, records) {
  validateClosedInput(input, context);
  const receipt = input.receipt;
  requireCondition(receipt.schema === "worker-read-interruption-v1" && receipt.interrupted === true && receipt.request_consumed === true &&
    receipt.response_pending === true && receipt.ownership_released === true, "read_only_interruption_not_observed");
  for (const state of [input.before, input.after]) {
    validateState(state, context);
    requireCondition(!state.running && !state.failure && state.renewalsConfirmed === 0 && state.deviceBaselineConfirmed === true &&
      state.deviceLeaseInactive && state.preservation?.device_identity_match && state.preservation.settings_match &&
      state.preservation.authorization_high_water_match && !state.preservation.mine_on_boot, "read_only_interruption_baseline");
  }
  requireCondition(input.before.status === "ready" && input.before.connected && !input.before.serialOwnershipReleased &&
    input.after.status === "closed" && !input.after.connected && input.after.serialOwnershipReleased &&
    input.after.preservation.baseline_id === input.before.preservation.baseline_id, "read_only_interruption_release");
  requireCondition(Array.isArray(records) && records.length >= 2 && records.length <= 512, "read_only_interruption_journal");
  for (const [index, record] of records.entries()) {
    exactObject(record, ["schema", "context_sha256", "sequence", "state"]);
    requireCondition(record.schema === "fixed-usb-no-mining-state-v1" && record.context_sha256 === digest(JSON.stringify(context)) &&
      record.sequence === index + 1, "read_only_interruption_journal");
    validateState(record.state, context);
    requireCondition(!record.state.running && record.state.renewalsConfirmed === 0, "read_only_interruption_journal");
  }
  const after = records.at(-1);
  requireCondition(isDeepStrictEqual(after.state, input.after), "read_only_interruption_journal");
  const beforeIndex = records.slice(0, -1).findLastIndex((record) => isDeepStrictEqual(record.state, input.before));
  requireCondition(beforeIndex >= 0, "read_only_interruption_journal");
  return { schema: "fixed-usb-no-mining-read-only-interruption-v1", context_sha256: digest(JSON.stringify(context)),
    receipt_sha256: digest(JSON.stringify(receipt)), before_sha256: digest(JSON.stringify(input.before)), after_sha256: digest(JSON.stringify(input.after)),
    before_sequence: beforeIndex + 1, after_sequence: after.sequence, ...input, hardware_execution_claimed_by_supervisor: false };
}

export async function saveNoMiningReadOnlyInterruption(root, context, input) {
  validateClosedInput(input, context);
  const records = await readNoMiningStates(root, context);
  let maybeRecord, maybeFailure;
  try { maybeRecord = validateNoMiningReadOnlyInterruption(input, context, records); }
  catch (error) {
    if (!(error instanceof QualificationError)) throw error;
    maybeFailure = error;
  }
  await writeNew(resolve(root, "no-mining-read-only-interruption-attempt.json"), {
    schema: "fixed-usb-no-mining-read-only-interruption-attempt-v1", context_sha256: digest(JSON.stringify(context)),
    input_sha256: digest(JSON.stringify(input)), input,
    outcome: maybeFailure ? "unqualified" : "qualified_observation", ...(maybeFailure ? { failure: maybeFailure.code } : {}),
    hardware_execution_claimed_by_supervisor: false });
  if (maybeFailure) throw maybeFailure;
  await writeNew(resolve(root, "no-mining-read-only-interruption.json"), maybeRecord);
  return { read_only_interruption_saved: true, receipt_sha256: maybeRecord.receipt_sha256 };
}
