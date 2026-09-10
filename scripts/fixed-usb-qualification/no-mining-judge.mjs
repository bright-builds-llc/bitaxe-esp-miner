import { requireRecordedAccounting } from "./no-mining-accounting.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";
import { resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { digest, exactObject, protectedPath, readJson, requireCondition, REQUIRED_CYCLES, writeNew } from "./contract.mjs";
import { requireExhaustedOriginal, validateLedger } from "./iterative-contract.mjs";
import { validateCycle, validateState } from "./judge.mjs";
import { validateNoMiningContext } from "./no-mining-context.mjs";

export function validateNoMiningReview(input, context, finalCycle) {
  exactObject(input, ["schema", "ledger_before", "ledger_after", "original_budget_before", "original_budget_after",
    "recovery_before", "recovery_after", "final_state", "recovery_without_drain", "cleanup_complete"]);
  requireCondition(input.schema === "fixed-usb-no-mining-review-v1" && input.recovery_without_drain === true && input.cleanup_complete === true,
    "no_mining_review_shape");
  for (const ledger of [input.ledger_before, input.ledger_after]) {
    validateLedger(ledger);
    requireCondition(!ledger.pending, "no_mining_pending_reservation");
  }
  for (const original of [input.original_budget_before, input.original_budget_after]) requireExhaustedOriginal(original);
  requireCondition(isDeepStrictEqual(input.ledger_before, input.ledger_after) &&
    isDeepStrictEqual(input.original_budget_before, input.original_budget_after), "no_mining_accounting_changed");
  for (const state of [input.recovery_before, input.recovery_after, input.final_state]) {
    validateState(state, context);
    requireCondition(!state.running && state.renewalsConfirmed === 0 && !state.failure && state.deviceBaselineConfirmed === true && state.deviceLeaseInactive &&
      state.preservation?.baseline_id === finalCycle.baseline_id && state.preservation.device_identity_match &&
      state.preservation.settings_match && state.preservation.authorization_high_water_match && !state.preservation.mine_on_boot,
    "no_mining_safe_baseline_missing");
  }
  requireCondition(input.recovery_after.helloRecovery?.discardedRecords > 0 && input.recovery_after.helloRecovery.discardedBytes > 0 &&
    input.recovery_after.helloRecovery.discardedReplies > 0,
    "no_mining_stale_recovery_missing");
  requireCondition(input.recovery_before.connected && input.recovery_before.status === "ready" &&
    input.recovery_after.connected && input.recovery_after.status === "ready" && !input.final_state.connected &&
    input.final_state.serialOwnershipReleased && input.final_state.status === "closed", "no_mining_cleanup_missing");
  return { schema: "fixed-usb-no-mining-report-v1", context_sha256: digest(JSON.stringify(context)), no_mining_cycles: REQUIRED_CYCLES,
    accounting_unchanged: true, recovery_without_drain: true, cleanup_confirmed: true,
    mining_authorized: false, hardware_execution_claimed_by_supervisor: false };
}

export async function finishNoMining(root, context, inputPath) {
  validateNoMiningContext(context);
  let maybePrevious;
  for (let cycle = 1; cycle <= REQUIRED_CYCLES; cycle += 1) {
    const path = resolve(root, `cycle-${cycle}.json`);
    await protectedPath(path);
    maybePrevious = validateCycle(await readJson(path), context, maybePrevious);
  }
  await protectedPath(inputPath);
  const input = await readJson(inputPath);
  const result = { ...validateNoMiningReview(input, context, maybePrevious), review_sha256: digest(JSON.stringify(input)) };
  const accounting = await requireRecordedAccounting(root, context, input);
  await requireRecordedRecovery(root, context, input, accounting);
  await writeNew(resolve(root, "no-mining.result.json"), result);
  return result;
}

async function requireRecordedRecovery(root, context, input, accounting) {
  const records = await readNoMiningStates(root, context);
  const before = accounting.before.observed_sequence, after = accounting.after.observed_sequence;
  requireCondition(isDeepStrictEqual(records[before - 1]?.state, input.recovery_before) &&
    isDeepStrictEqual(records[after - 1]?.state, input.recovery_after), "no_mining_accounting_observation_missing");
  const released = records.slice(before, after - 1).some(({ state }) => !state.connected && !state.running &&
    state.serialOwnershipReleased && ["disconnected", "closed"].includes(state.status));
  requireCondition(released, "no_mining_interruption_missing");
  requireCondition(records.length > after && isDeepStrictEqual(records.at(-1).state, input.final_state), "no_mining_final_cleanup_missing");
}
