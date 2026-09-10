import { readNoMiningStates } from "./no-mining-records.mjs";
import { readFile, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { canonicalBase64, digest, exactObject, protectedPath, QualificationError, readJson, requireCondition, writeNew } from "./contract.mjs";
import { requireExhaustedOriginal, validateLedger } from "./iterative-contract.mjs";
import { validateState } from "./judge.mjs";

async function readExistingCampaign(path) {
  requireCondition(await realpath(path) === resolve(path), "campaign_path_alias");
  await protectedPath(dirname(path), true);
  await protectedPath(path);
  const bytes = await readFile(path);
  const value = JSON.parse(bytes.toString("utf8"));
  exactObject(value, ["schema", "campaign_id"]);
  requireCondition(value.schema === "fixed-usb-campaign-v1" && canonicalBase64(value.campaign_id, 16), "original_campaign_shape");
  return { value, sha256: digest(bytes) };
}

export async function inspectOriginalCampaign(path) {
  const { sha256 } = await readExistingCampaign(path);
  return { path: resolve(path), sha256 };
}

export async function readFrozenCampaign(context) {
  const record = context.original_campaign_record;
  requireCondition(record !== undefined, "original_campaign_record_missing");
  exactObject(record, ["path", "sha256"]);
  const { value, sha256 } = await readExistingCampaign(record.path);
  requireCondition(sha256 === record.sha256, "original_campaign_record_changed");
  return value.campaign_id;
}

export async function saveNoMiningAccounting(root, context, input) {
  exactObject(input, ["stage", "ledger", "original_budget", "state"]);
  requireCondition(["before", "after"].includes(input.stage), "no_mining_accounting_stage");
  validateLedger(input.ledger);
  requireExhaustedOriginal(input.original_budget);
  const state = validateState(input.state, context);
  requireCondition(!input.ledger.pending && state.status === "ready" && state.connected && !state.running && !state.failure &&
    state.renewalsConfirmed === 0 && state.deviceRestorationConfirmed && state.deviceLeaseInactive &&
    state.preservation?.device_identity_match && state.preservation.settings_match && state.preservation.authorization_high_water_match &&
    !state.preservation.mine_on_boot, "no_mining_accounting_baseline");
  const records = await readNoMiningStates(root, context);
  const observed = records.at(-1);
  requireCondition(isDeepStrictEqual(observed.state, state), "no_mining_accounting_observation_missing");
  const value = { schema: "fixed-usb-no-mining-accounting-v1", context_sha256: digest(JSON.stringify(context)),
    observed_sequence: observed.sequence, ...input };
  if (input.stage === "after") {
    const before = await loadAccounting(root, context, "before");
    requireCondition(isDeepStrictEqual(input.ledger, before.ledger) && isDeepStrictEqual(input.original_budget, before.original_budget) &&
      state.preservation.baseline_id === before.state.preservation.baseline_id && observed.sequence > before.observed_sequence, "no_mining_accounting_changed");
  }
  await writeNew(resolve(root, `no-mining-accounting-${input.stage}.json`), value);
  return { accounting_saved: true, stage: input.stage };
}

async function loadAccounting(root, context, stage) {
  const path = resolve(root, `no-mining-accounting-${stage}.json`);
  try { await protectedPath(path); }
  catch (error) {
    if (error.code === "ENOENT") throw new QualificationError("no_mining_accounting_evidence_missing");
    throw error;
  }
  const record = await readJson(path);
  exactObject(record, ["schema", "context_sha256", "stage", "ledger", "original_budget", "state", "observed_sequence"]);
  requireCondition(Number.isInteger(record.observed_sequence) && record.observed_sequence > 0 && record.observed_sequence <= 512, "no_mining_accounting_integrity");
  requireCondition(record.schema === "fixed-usb-no-mining-accounting-v1" && record.context_sha256 === digest(JSON.stringify(context)) &&
    record.stage === stage, "no_mining_accounting_integrity");
  return record;
}

export async function requireRecordedAccounting(root, context, review) {
  requireCondition(context.original_campaign_record !== undefined, "no_mining_accounting_evidence_missing");
  const receipts = {};
  for (const stage of ["before", "after"]) {
    const record = await loadAccounting(root, context, stage);
    requireCondition(isDeepStrictEqual(record.ledger, review[`ledger_${stage}`]) &&
      isDeepStrictEqual(record.original_budget, review[`original_budget_${stage}`]) &&
      isDeepStrictEqual(record.state, review[`recovery_${stage}`]), "no_mining_accounting_integrity");
    receipts[stage] = record;
  }
  requireCondition(receipts.before.observed_sequence < receipts.after.observed_sequence, "no_mining_accounting_order");
  return receipts;
}
