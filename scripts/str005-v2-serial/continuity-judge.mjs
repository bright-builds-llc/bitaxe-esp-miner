import { validateCycle } from "../fixed-usb-qualification/judge.mjs";
import { inspectInstall } from "../str005-noise-serial/install.mjs";
import { canonical, proof } from "../str005-noise-serial/files.mjs";
import { baseline } from "./journal.mjs";
import { inspectProbe } from "./install.mjs";
import { check, object, sha256, uint } from "./values.mjs";

/** Re-derive each flash and fresh probe; recorded cycle pass flags alone confer no continuity. */
export async function judgeContinuity(root, context, rows, beforeWork) {
  const contextSha256 = sha256(JSON.stringify(context)), initial = rows.find((row) => row.phase === "before");
  check(initial, "v2_initial_baseline_missing"); baseline(initial.state);
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
