import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { digest, exactObject, fileDigest, protectedPath, QualificationError, readJson, requireCondition, writeNew } from "./contract.mjs";
import { judgeWindow, validateState } from "./judge.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { requireReleasedState, validateIterativeContext } from "./iterative-preflight.mjs";

export function judgeIterative(context, records, fault) {
  const attempt = context.qualification_attempt;
  for (const record of records) validateState(record.state, context);
  const observed = records.filter((record) => record.state.qualification?.attempt?.ordinal === attempt.ordinal);
  requireCondition(observed.length > 0, "iterative_observation_missing");
  for (const record of observed) {
    const q = record.state.qualification, a = q.attempt;
    requireCondition(a.purpose === attempt.purpose && a.maximum_active_ms === attempt.maximumActiveMilliseconds &&
      a.reserved_ms === attempt.maximumActiveMilliseconds && q.budget_reserved_ms === 240000 &&
      q.budget_complete === true, "iterative_observation_binding");
  }
  requireCondition(observed.at(-1).state.qualification.attempt.complete === true, "iterative_stop_incomplete");
  const index = attempt.purpose === "foreground_loss" ? 1 : attempt.purpose === "heartbeat_loss" ? 2 : 0;
  const translatedFault = fault ? { ...fault, window: index } : undefined;
  const result = judgeWindow(index, observed, translatedFault, { iterativePurpose: attempt.purpose });
  if (attempt.purpose === "normal") requireCondition(result.accepted_share_verified, "no_accepted_share_within_allowance");
  return { ...result, purpose: attempt.purpose, diagnostic_only: attempt.purpose === "diagnostic" };
}
export async function finishIterative(root, context, inputPath) {
  await validateIterativeContext(root, context);
  await protectedPath(inputPath);
  const input = await readJson(inputPath);
  exactObject(input, ["ledger_before", "ledger_after", "original_budget", "final_state"]);
  const a = context.qualification_attempt;
  await protectedPath(resolve(root, "issued.json"));
  await protectedPath(resolve(root, "consumed.json"));
  const issuance = await readJson(resolve(root, "issued.json"));
  const consumption = await readJson(resolve(root, "consumed.json"));
  requireCondition(issuance.context_sha256 === digest(JSON.stringify(context)) && issuance.ordinal === a.ordinal &&
    consumption.ordinal === a.ordinal && consumption.delivery_attempted === true, "iterative_issuance_evidence");
  requireCondition(JSON.stringify(issuance.ledger_before) === JSON.stringify(input.ledger_before), "iterative_ledger_before_changed");
  requireIdleLedger(input.ledger_before, a.ordinal, context.expected_charged_ms);
  requireIdleLedger(input.ledger_after, a.ordinal + 1, context.expected_charged_ms + a.maximumActiveMilliseconds);
  requireExhaustedOriginal(input.original_budget);
  requireReleasedState(input.final_state, context);
  const samplePath = resolve(root, "iterative.samples.jsonl");
  await protectedPath(samplePath);
  const records = (await readFile(samplePath, "utf8")).trim().split("\n").filter(Boolean).map((line) => JSON.parse(line));
  let fault;
  try { await protectedPath(resolve(root, "iterative.fault.json")); fault = await readJson(resolve(root, "iterative.fault.json")); }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  const earliest = records.find((record) => record.state.failure);
  let firstFailure;
  if (earliest) {
    const path = resolve(root, "first-failure.json");
    await protectedPath(path);
    const details = await readJson(path);
    const expected = { schema: "worker-iterative-first-failure-v1", ordinal: a.ordinal, sequence: earliest.sequence,
      browser: earliest.state.failure, serial: earliest.state.serialFailureCategory ?? null, admission: earliest.state.admissionFailureStage ?? null };
    requireCondition(JSON.stringify(details) === JSON.stringify(expected), "iterative_first_failure_changed");
    firstFailure = { details, sha256: await fileDigest(path) };
  }
  let judgment, failure;
  try { judgment = judgeIterative(context, records, fault); }
  catch (error) { if (!(error instanceof QualificationError)) throw error; failure = error.code; }
  const receipt = { schema: "worker-iterative-result-v1", context, context_sha256: digest(JSON.stringify(context)),
    original_campaign_id: context.original_campaign_id, next_ordinal: a.ordinal + 1,
    total_charged_ms: input.ledger_after.total_charged_ms, cleanup_confirmed: true, result: judgment ? "passed" : "unverified",
    ...(judgment ? { judgment } : {}), first_failure: firstFailure?.details.browser ?? null,
    first_failure_evidence: firstFailure ?? null, judgment_failure: failure ?? null, ...input,
    samples_sha256: await fileDigest(samplePath), progress_sha256: context.progress_sha256 };
  await writeNew(resolve(root, "result.json"), { receipt, sha256: digest(JSON.stringify(receipt)) });
  return { result: receipt.result, ordinal: a.ordinal, purpose: a.purpose, first_failure: receipt.first_failure, judgment_failure: receipt.judgment_failure,
    cumulative_charged_ms: receipt.total_charged_ms, cleanup_confirmed: true };
}
