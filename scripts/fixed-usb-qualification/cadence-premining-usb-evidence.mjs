import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, protectedPath, requireCondition as check } from "./contract.mjs";
import { baseline, proof } from "./cadence-premining-evidence.mjs";
import { validateCadenceReview, validateCadenceProbe, requireCadencePhase } from "./cadence-evidence.mjs";

export async function jsonLines(path, limit) {
  await protectedPath(path);
  const bytes = await readFile(path);
  check(bytes.length > 0 && bytes.length <= limit && bytes.at(-1) === 10, "journal_bytes");
  return { rows: bytes.toString("utf8").trim().split("\n").map(JSON.parse), sha256: digest(bytes), bytes };
}
function armBinding(saved, phase, contextHash, records) {
  exactObject(saved, ["context_sha256", "phase", "arm", "started_sequence", "started_at_unix_ms"]);
  exactObject(saved.arm, ["schema", "phase", "armedAtUs", "generation"]);
  check(
    saved.context_sha256 === contextHash &&
      saved.phase === phase &&
      saved.arm.schema === "worker-telemetry-cadence-arm-v1" &&
      saved.arm.phase === phase &&
      saved.arm.generation === 0 &&
      Number.isSafeInteger(saved.arm.armedAtUs) &&
      saved.arm.armedAtUs > 0 &&
      Number.isSafeInteger(saved.started_sequence) &&
      saved.started_sequence > 0 &&
      saved.started_sequence <= records.length &&
      Number.isSafeInteger(saved.started_at_unix_ms) &&
      saved.started_at_unix_ms > 0,
    "arm_binding",
  );
  baseline(records[saved.started_sequence - 1].state, false);
}
export async function idleEvidence(root, contextHash, records, cycles) {
  const arm = (await proof(resolve(root, "cadence-idle-arm.json"))).value;
  armBinding(arm, "idle", contextHash, records);
  const saved = await proof(resolve(root, "cadence-idle.json")),
    idle = saved.value;
  exactObject(idle, [
    "schema",
    "context_sha256",
    "phase",
    "arm",
    "started_sequence",
    "finished_sequence",
    "started_at_unix_ms",
    "finished_at_unix_ms",
    "collected_at_unix_ms",
    "measurement_end_sequence",
    "review",
    "observer_connected",
  ]);
  check(
    idle.schema === "worker-cadence-phase-v1" &&
      idle.context_sha256 === contextHash &&
      idle.phase === "idle" &&
      equal(idle.arm, arm.arm) &&
      idle.started_sequence === arm.started_sequence &&
      idle.started_at_unix_ms === arm.started_at_unix_ms &&
      idle.started_sequence >= cycles.last_ready_sequence &&
      idle.finished_sequence > idle.started_sequence &&
      idle.measurement_end_sequence === idle.finished_sequence &&
      idle.finished_at_unix_ms === idle.collected_at_unix_ms &&
      idle.finished_at_unix_ms > idle.started_at_unix_ms &&
      idle.observer_connected === true &&
      equal(records[idle.finished_sequence - 1]?.state.cadence?.review, idle.review),
    "idle_phase_binding",
  );
  baseline(records[idle.finished_sequence - 1].state, false);
  const summary = requireCadencePhase(idle.review, "idle");
  check(summary.armedAtUs === arm.arm.armedAtUs && summary.generation === 0, "idle_summary_binding");
  return { record: idle, summary, sha256: saved.sha256 };
}
export async function usbEvidence(root, contextHash, records, idle, failure) {
  const armSaved = await proof(resolve(root, "cadence-usb-arm.json")),
    arm = armSaved.value;
  armBinding(arm, "usb", contextHash, records);
  check(arm.started_sequence > idle.record.finished_sequence && arm.started_at_unix_ms >= idle.record.finished_at_unix_ms, "phase_order");
  const first = records.find((row) => row.state.cadence?.review?.phases[1].state === "complete");
  check(first && first.sequence === failure.first_rejected_review_sequence && first.sequence > arm.started_sequence, "earliest_usb_review");
  const review = validateCadenceReview(first.state.cadence.review),
    reviewHash = digest(JSON.stringify(review)),
    summary = review.phases[1];
  baseline(first.state, false);
  check(
    reviewHash === failure.first_rejected_review_sha256 &&
      equal({ ...review.phases[0], passed: idle.summary.passed }, idle.summary) &&
      idle.summary.passed === true &&
      review.phases[0].passed === false &&
      summary.armedAtUs === arm.arm.armedAtUs &&
      summary.generation === 0 &&
      summary.startedAtUs >= summary.armedAtUs &&
      summary.endedAtUs - summary.startedAtUs >= 60000000 &&
      summary.endedAtUs - summary.startedAtUs <= 62000000 &&
      summary.maxProbeCount === 12 &&
      summary.firstMaxProbeAtUs >= summary.startedAtUs &&
      summary.lastMaxProbeAtUs >= summary.firstMaxProbeAtUs &&
      summary.lastMaxProbeAtUs - summary.startedAtUs < 60000000,
    "usb_review_binding",
  );
  // Published 912e7a summary.refresh_passed uses global dropped==0; retain both derived pass flags.
  let failureCode;
  try {
    requireCadencePhase(review, "usb");
  } catch (error) {
    failureCode = error.code;
  }
  check(failureCode === "cadence_capture_loss" && failure.replayed_validator_failure === failureCode, "usb_review_must_fail");
  check(
    equal(failure.observations, {
      interval_count: summary.intervalCount,
      within_750ms: summary.intervalBuckets[0] + summary.intervalBuckets[1],
      maximum_interval_us: summary.maximumIntervalUs,
      maximum_execution_us: summary.maximumExecutionUs,
      dropped_observations: review.droppedObservations,
      pending_sends: summary.pendingSends,
    }) &&
      review.droppedObservations === 1 &&
      summary.pendingSends === 2 &&
      equal(summary.intervalBuckets, [0, 85, 6, 0]) &&
      summary.intervalCount === 91 &&
      (summary.intervalBuckets[0] + summary.intervalBuckets[1]) * 100 < summary.intervalCount * 95,
    "usb_failure_numbers",
  );
  const probes = await jsonLines(resolve(root, "cadence-probes.jsonl"), 16384),
    witnesses = await jsonLines(resolve(root, "cadence-probe-witnesses.jsonl"), 16384);
  check(probes.rows.length === 12 && witnesses.rows.length === 12, "probe_count");
  let prior, priorWitness;
  for (const [index, probe] of probes.rows.entries()) {
    prior = validateCadenceProbe(probe, prior);
    const w = witnesses.rows[index];
    exactObject(w, ["ordinal", "observedAtUnixMs", "sequence"]);
    check(
      w.ordinal === probe.ordinal &&
        Number.isSafeInteger(w.observedAtUnixMs) &&
        w.observedAtUnixMs >= arm.started_at_unix_ms &&
        w.observedAtUnixMs < arm.started_at_unix_ms + 60000 &&
        Number.isSafeInteger(w.sequence) &&
        w.sequence >= arm.started_sequence &&
        w.sequence < first.sequence &&
        (!priorWitness || (w.observedAtUnixMs >= priorWitness.observedAtUnixMs && w.sequence >= priorWitness.sequence)),
      "probe_witness_binding",
    );
    priorWitness = w;
  }
  const released = records.find(
    (row) => row.sequence > first.sequence && row.state.status === "closed" && row.state.serialOwnershipReleased,
  );
  check(released, "released_boundary");
  baseline(released.state, true);
  const later = records
    .filter((row) => row.sequence > released.sequence && row.state.status === "ready" && row.state.connected && row.state.cadence?.review)
    .at(-1);
  check(later && later.sequence > released.sequence && failure.retained_review_after_fresh_reconnect === true, "later_recovery_review");
  baseline(later.state, false);
  validateCadenceReview(later.state.cadence.review);
  check(equal(later.state.cadence.review, review), "later_review_changed");
  return {
    arm,
    summary,
    first_sequence: first.sequence,
    first_sha256: reviewHash,
    later_sequence: later.sequence,
    later_sha256: digest(JSON.stringify(later.state.cadence.review)),
    released_sequence: released.sequence,
    arm_sha256: armSaved.sha256,
    probes_sha256: probes.sha256,
    witnesses_sha256: witnesses.sha256,
  };
}
