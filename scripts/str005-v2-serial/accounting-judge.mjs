import { readdir } from "node:fs/promises";
import { canonical, proof } from "../str005-noise-serial/files.mjs";
import { requireExhaustedOriginal, requireIdleLedger, validateCooling } from "../fixed-usb-qualification/iterative-contract.mjs";
import { baseline, restoredBaseline } from "./journal.mjs";
import { signerExitProofs } from "./host-resources.mjs";
import { check, object, sha256, uint } from "./values.mjs";

/** Accounting is credited only at the matching authenticated journal observation. */
export async function readAccounting(root, context, rows, stage) {
  const value = (await proof(root, `accounting-${stage}.json`)).value;
  object(value, ["schema", "contextSha256", "observedSequence", "stage", "state", "ledger", "original_budget"]);
  const row = rows[value.observedSequence - 1];
  check(value.schema === "str005-v2-accounting-v1" && value.contextSha256 === sha256(JSON.stringify(context)) &&
    value.stage === stage && row && canonical(row.state) === canonical(value.state) &&
    row.phase === (stage === "before-install" ? "before" : "candidate"), "v2_accounting_journal_join");
  if (stage === "after") restoredBaseline(value.state, context); else baseline(value.state);
  requireExhaustedOriginal(value.original_budget);
  requireIdleLedger(value.ledger, stage === "after" && context.scope === "share" ? 19 : 18,
    stage === "after" && context.scope === "share" ? 1740000 : 1560000);
  return value;
}

export async function judgeAccounting(root, context, rows, restoration) {
  const initial = await readAccounting(root, context, rows, "before-install");
  const before = await readAccounting(root, context, rows, "before");
  const after = await readAccounting(root, context, rows, "after");
  check(initial.observedSequence < before.observedSequence && before.observedSequence < restoration.observedSequence &&
    restoration.observedSequence < after.observedSequence && after.observedSequence < rows.at(-1).sequence,
  "v2_accounting_chronology");
  for (const value of [before, after]) check(canonical(value.original_budget) === canonical(initial.original_budget) &&
    value.state.preservation.baseline_id === initial.state.preservation.baseline_id, "v2_accounting_continuity");
  check(canonical(initial.ledger) === canonical(before.ledger), "v2_accounting_before_changed");
  return { initial, before, after };
}

/** Channel's absence of signing/work is checked independently of the protocol record. */
export async function requireChannelNoWork(root, rows, before, after) {
  const forbidden = /^(?:issuance\.claim|issued|consumed|cooling|fault(?:\.claim|-confirmed)?|observer-start\.claim|observer-stop|signer-[0-9]{2}\.exit|share-network-[0-9]{4}|share-start-observed)\.json$/u;
  check(!(await readdir(root)).some(name => forbidden.test(name)), "v2_channel_authority_evidence");
  const metrics = ["generation", "work_dispatched", "submitted", "accepted", "rejected"];
  const expected = before.state.qualification;
  check(expected, "v2_channel_counters_missing");
  for (const row of rows.filter(row => row.sequence >= before.observedSequence)) {
    check(!row.state.running && !row.state.heartbeatSuppressed && row.state.renewalsConfirmed === 0 &&
      !row.state.failure && !row.state.serialFailureCategory && !row.state.ownerResourceFailure, "v2_channel_work_observed");
    const q = row.state.qualification;
    check(q && metrics.every(key => q[key] === expected[key]), "v2_channel_counter_changed");
  }
  check(canonical(before.ledger) === canonical(after.ledger), "v2_channel_ledger_changed");
}

/** Signed delivery is a host fact, never substituted for a device reservation. */
export async function judgeIssuance(root, context, rows, before, firstDevice) {
  const hash = sha256(JSON.stringify(context));
  const claim = (await proof(root, "issuance.claim.json")).value;
  object(claim, ["schema", "contextSha256", "ordinal", "atHostMs", "reviewFile", "observedSequence", "deviceReservationObserved"]);
  check(claim.schema === "str005-v2-issuance-claim-v1" && claim.contextSha256 === hash && claim.ordinal === 18 &&
    claim.deviceReservationObserved === false && /^budget-review-[0-9]{4}\.json$/u.test(claim.reviewFile), "v2_issuance_claim");
  const review = (await proof(root, claim.reviewFile)).value;
  object(review, ["schema", "contextSha256", "observedSequence", "atHostMs", "expiresAtHostMs", "ledger"]);
  check(review.schema === "str005-v2-budget-review-v1" && review.contextSha256 === hash &&
    review.observedSequence === claim.observedSequence && claim.observedSequence >= before.observedSequence &&
    canonical(review.ledger) === canonical(before.ledger), "v2_issuance_review");
  baseline(rows[claim.observedSequence - 1]?.state);
  const issued = (await proof(root, "issued.json")).value, consumed = (await proof(root, "consumed.json")).value;
  object(issued, ["schema", "contextSha256", "ordinal", "atHostMs", "ledgerBefore", "authorizationCount", "privatePayloadPersisted", "deviceReservationObserved"]);
  object(consumed, ["schema", "contextSha256", "ordinal", "atHostMs", "deliveryAttempted", "deviceReservationObserved"]);
  check(issued.schema === "str005-v2-issuance-v1" && issued.contextSha256 === hash && issued.ordinal === 18 &&
    issued.authorizationCount === 10 && issued.privatePayloadPersisted === false && issued.deviceReservationObserved === false &&
    canonical(issued.ledgerBefore) === canonical(before.ledger) && consumed.schema === "str005-v2-artifact-delivery-v1" &&
    consumed.contextSha256 === hash && consumed.ordinal === 18 && consumed.deliveryAttempted === true && consumed.deviceReservationObserved === false,
  "v2_issuance_delivery");
  for (const time of [review.atHostMs, review.expiresAtHostMs, claim.atHostMs, issued.atHostMs, consumed.atHostMs]) uint(time);
  check(review.atHostMs <= claim.atHostMs && claim.atHostMs <= issued.atHostMs && issued.atHostMs <= consumed.atHostMs &&
    issued.atHostMs < review.expiresAtHostMs && consumed.atHostMs <= firstDevice.atHostMs, "v2_issuance_chronology");
  const cooling = (await proof(root, "cooling.json")).value;
  object(cooling, ["schema", "contextSha256", "scopeChallengeId", "observedSequence", "atHostMs", "proof", "restoration", "budget_before", "budget_after", "state"]);
  check(cooling.schema === "str005-v2-cooling-v1" && cooling.contextSha256 === hash &&
    canonical(cooling.state) === canonical(rows[cooling.observedSequence - 1]?.state) && cooling.observedSequence <= claim.observedSequence &&
    canonical(cooling.budget_before) === canonical(before.ledger) && canonical(cooling.budget_after) === canonical(before.ledger), "v2_cooling_join");
  validateCooling(cooling.proof, cooling.restoration); baseline(cooling.state);
  const signers = await signerExitProofs(root, context);
  check(signers.length === 11, "v2_signing_proof_missing");
  for (const [index, item] of signers.entries()) {
    const signed = (await proof(root, item.path)).value, observed = signed.observation;
    check(signed.operation === (index === 0 ? "public-trust" : index === 1 ? "start" : "renew") && observed.pid !== null &&
      observed.code === 0 && observed.signal === null && !observed.inputFailed && !observed.overflow && observed.stdoutBytes > 0,
    "v2_signing_exit_failed");
  }
  return { claim, issued, consumed };
}
