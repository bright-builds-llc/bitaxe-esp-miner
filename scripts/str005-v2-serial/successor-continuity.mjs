import { validateCycle } from "../fixed-usb-qualification/judge.mjs";
import { inspectInstall } from "../str005-noise-serial/install.mjs";
import { canonical, proof } from "../str005-noise-serial/files.mjs";
import { baseline } from "./journal.mjs";
import { inspectProbe } from "./install.mjs";
import { check, object, sha256, uint } from "./values.mjs";

/** Forensic facts only: this does not run or replace the frozen historical verdict. */
export async function inspectAvailableContinuity(root, context, rows, accounting) {
  const contextSha256 = sha256(JSON.stringify(context)), initial = rows[accounting.initial.observedSequence - 1];
  baseline(initial?.state);
  check(initial.phase === "before" && canonical(initial.state) === canonical(accounting.initial.state), "v2_successor_initial_join");
  const baselineId = initial.state.preservation.baseline_id;
  const previousClaim = await proof(context.predecessor.root, "install-0.claim.json");
  const previousSeal = (await proof(context.predecessor.root, "sealed-inventory.json")).value;
  check(previousSeal.files.some(row => row.path === "install-0.claim.json" && row.sha256 === previousClaim.sha256), "v2_successor_physical_ancestry");
  const physical = previousClaim.value.detector.physical;
  let previousSequence = initial.sequence, maybeReport;
  check(canonical(context.install_indices) === canonical([0, 1, 2, 3, 4]), "v2_successor_installations");
  for (const index of context.install_indices) {
    const installed = await inspectInstall(root, context, index), reviewed = await proof(root, `install-${index}.review.json`), review = reviewed.value;
    object(review, ["schema", "contextSha256", "index", "beforeSequence", "atHostMs", "flashSha256", "observationSha256", "startupCaptureSha256", "startupMemory"]);
    uint(review.atHostMs);
    check(review.schema === "str005-v2-install-review-v1" && review.contextSha256 === contextSha256 && review.index === index &&
      review.beforeSequence === installed.claim.beforeSequence && review.flashSha256 === installed.flashSha256 &&
      review.observationSha256 === installed.observationSha256 && review.startupCaptureSha256 === installed.startupCaptureSha256 &&
      canonical(review.startupMemory) === canonical(installed.startupMemory) && installed.claim.detector.physical === physical, "v2_successor_install_join");
    const before = rows[review.beforeSequence - 1]; baseline(before?.state, true);
    check(before.sequence > previousSequence && before.sequence < accounting.before.observedSequence &&
      before.state.preservation.baseline_id === baselineId && installed.claim.beforeStateSha256 === sha256(canonical(before)), "v2_successor_before_install");
    if (index === 0) { check(before.phase === "before", "v2_successor_initial_phase"); previousSequence = before.sequence; continue; }
    const cycle = (await proof(root, `cycle-${index}.json`)).value;
    object(cycle, ["schema", "contextSha256", "beforeSequence", "afterSequence", "installReviewSha256", "report"]);
    const after = rows[cycle.afterSequence - 1], probe = await inspectProbe(root, context, index, rows); baseline(after?.state);
    check(cycle.schema === "str005-v2-cycle-v1" && cycle.contextSha256 === contextSha256 && cycle.beforeSequence === before.sequence &&
      cycle.installReviewSha256 === reviewed.sha256 && after.sequence > before.sequence && after.phase === "candidate" &&
      after.atHostMs > review.atHostMs && after.sequence < accounting.before.observedSequence && probe.beforeSequence > before.sequence &&
      probe.afterSequence === after.sequence && after.state.preservation.baseline_id === baselineId, "v2_successor_cycle_join");
    maybeReport = validateCycle(cycle.report, context, maybeReport);
    check(cycle.report.cycle === index && cycle.report.baseline_id === baselineId, "v2_successor_cycle_report");
    previousSequence = after.sequence;
  }
  check(maybeReport?.cycle === 4 && rows.every(row => !row.state.preservation || row.state.preservation.baseline_id === baselineId),
    "v2_successor_preservation");
  return { installations: 5, cycles: 4 };
}
