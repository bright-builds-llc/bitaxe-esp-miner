import { validateCycle } from "../fixed-usb-qualification/judge.mjs";
import { inspectInstall } from "../str005-noise-serial/install.mjs";
import { canonical, proof } from "../str005-noise-serial/files.mjs";
import { baseline } from "./journal.mjs";
import { inspectProbe } from "./install.mjs";
import { check, object, sha256, uint } from "./values.mjs";

const SOURCE = "scripts/str005-v2-serial/continuity-judge.mjs";
const HISTORICAL_SOURCE = { sha256: "d49635695c66849ab053343f32c5d681c20aea3ca144e4f100a673cb88b2b640", length: 3883 };

async function initialBaseline(root, context, rows, beforeWork, initialAccounting) {
  check(Array.isArray(context.evaluator), "v2_continuity_evaluator");
  const entries = context.evaluator.filter(entry => entry?.path === SOURCE);
  check(entries?.length === 1 && /^[a-f0-9]{64}$/u.test(entries[0].sha256) && Number.isSafeInteger(entries[0].length) &&
    entries[0].length > 0, "v2_continuity_evaluator");
  if (entries[0].sha256 === HISTORICAL_SOURCE.sha256) {
    check(entries[0].length === HISTORICAL_SOURCE.length, "v2_continuity_evaluator");
    // Both frozen 001/002 snapshots used this exact implementation. Preserve its verdict,
    // including the sealed 002 baseline-selection rejection; never upgrade historical evidence.
    const initial = rows.find(row => row.phase === "before");
    check(initial, "v2_initial_baseline_missing"); baseline(initial.state); return initial;
  }
  const saved = (await proof(root, "accounting-before-install.json")).value;
  const work = (await proof(root, "accounting-before.json")).value;
  const keys = ["schema", "contextSha256", "observedSequence", "stage", "state", "ledger", "original_budget"];
  object(saved, keys); object(work, keys);
  const hash = sha256(JSON.stringify(context));
  uint(saved.observedSequence); uint(work.observedSequence);
  const initial = rows[saved.observedSequence - 1], workRow = rows[work.observedSequence - 1];
  check(initialAccounting !== undefined && canonical(initialAccounting) === canonical(saved) && canonical(beforeWork) === canonical(work) &&
    saved.schema === "str005-v2-accounting-v1" && work.schema === saved.schema && saved.contextSha256 === hash && work.contextSha256 === hash &&
    saved.stage === "before-install" && work.stage === "before" && saved.observedSequence > 0 && saved.observedSequence < work.observedSequence &&
    initial && initial.sequence === saved.observedSequence && initial.contextSha256 === hash && initial.phase === "before" &&
    canonical(initial.state) === canonical(saved.state) && workRow && workRow.sequence === work.observedSequence &&
    workRow.contextSha256 === hash && workRow.phase === "candidate" && canonical(workRow.state) === canonical(work.state) &&
    Number.isSafeInteger(initial.atHostMs) && initial.atHostMs >= 0 && Number.isSafeInteger(workRow.atHostMs) &&
    workRow.atHostMs >= initial.atHostMs, "v2_initial_accounting_join");
  baseline(initial.state); return initial;
}

/** Re-derive each flash and fresh probe; recorded cycle pass flags alone confer no continuity. */
export async function judgeContinuity(root, context, rows, beforeWork, initialAccounting) {
  const contextSha256 = sha256(JSON.stringify(context));
  const initial = await initialBaseline(root, context, rows, beforeWork, initialAccounting);
  const baselineId = initial.state.preservation.baseline_id;
  const predecessorClaim = await proof(context.predecessor.root, "install-0.claim.json");
  const predecessorSeal = (await proof(context.predecessor.root, "sealed-inventory.json")).value;
  check(predecessorSeal.files.some((entry) => entry.path === "install-0.claim.json" && entry.sha256 === predecessorClaim.sha256), "v2_predecessor_physical_proof");
  const physical = predecessorClaim.value.detector.physical;
  let maybePreviousReport, previousAfter = initial.sequence;
  for (const index of context.install_indices) {
    const installed = await inspectInstall(root, context, index), reviewProof = await proof(root, `install-${index}.review.json`), review = reviewProof.value;
    object(review, ["schema", "contextSha256", "index", "beforeSequence", "atHostMs", "flashSha256", "observationSha256", "startupCaptureSha256", "startupMemory"]);
    check(review.schema === "str005-v2-install-review-v1" && review.contextSha256 === contextSha256 && review.index === index &&
      review.beforeSequence === installed.claim.beforeSequence && review.flashSha256 === installed.flashSha256 &&
      review.observationSha256 === installed.observationSha256 && review.startupCaptureSha256 === installed.startupCaptureSha256 &&
      canonical(review.startupMemory) === canonical(installed.startupMemory) && installed.claim.detector.physical === physical,
      "v2_install_review_join"); uint(review.atHostMs);
    const before = rows[review.beforeSequence - 1]; baseline(before?.state, true);
    check(before.sequence > previousAfter && before.sequence < beforeWork.observedSequence &&
      before.state.preservation.baseline_id === baselineId && installed.claim.beforeStateSha256 === sha256(canonical(before)), "v2_install_before_join");
    if (index === 0) { check(before.phase === "before", "v2_initial_install_phase"); previousAfter = before.sequence; continue; }
    const cycle = (await proof(root, `cycle-${index}.json`)).value;
    object(cycle, ["schema", "contextSha256", "beforeSequence", "afterSequence", "installReviewSha256", "report"]);
    const after = rows[cycle.afterSequence - 1], probe = await inspectProbe(root, context, index, rows);
    baseline(after?.state);
    check(cycle.schema === "str005-v2-cycle-v1" && cycle.contextSha256 === contextSha256 && cycle.beforeSequence === before.sequence &&
      cycle.installReviewSha256 === reviewProof.sha256 && cycle.afterSequence > before.sequence && after.phase === "candidate" &&
      after.atHostMs > review.atHostMs && after.sequence < beforeWork.observedSequence && probe.beforeSequence > before.sequence &&
      probe.afterSequence === after.sequence && after.state.preservation.baseline_id === baselineId, "v2_cycle_evidence_join");
    maybePreviousReport = validateCycle(cycle.report, context, maybePreviousReport);
    check(cycle.report.cycle === index && cycle.report.baseline_id === baselineId, "v2_cycle_report_join");
    previousAfter = after.sequence;
  }
  check(maybePreviousReport?.cycle === 4, "v2_four_cycles_required");
  return { installations: context.install_indices.length, cycles: 4, baselineId, physical };
}
