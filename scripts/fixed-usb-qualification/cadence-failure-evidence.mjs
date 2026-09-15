import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, fileDigest, missing, requireCondition as check } from "./contract.mjs";
import { proof, inventory, baseline, verifyPreparationCycles, observerProof } from "./cadence-premining-evidence.mjs";
import { idleEvidence, jsonLines } from "./cadence-premining-usb-evidence.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { parseSamples } from "./sample-seal.mjs";
import { validateCadenceReview, validateCadenceProbe, requireCadencePhase, requireCadenceDiagnostics } from "./cadence-evidence.mjs";
import { requireIdleLedger, validateCooling } from "./iterative-contract.mjs";
async function initialEvidence(root, context) {
  const hash = digest(JSON.stringify(context));
  for (const name of [
    "issued.json",
    "consumed.json",
    "result.json",
    "iterative.fault.json",
    "first-failure.json",
    "cadence-usb.json",
    "cadence-mining-arm.json",
    "cadence-mining.json",
    "cadence-mining-measurement-end.json",
    "completion-input.json",
    "sample-seal-intent.json",
  ])
    await missing(resolve(root, name));
  const snapshot = await verifyArtifactSnapshot(root, context),
    journalPath = resolve(root, "iterative.samples.jsonl"),
    bytes = await readFile(journalPath),
    records = parseSamples(bytes, context),
    final = records.at(-1);
  baseline(final.state, true);
  check(
    final.state.deviceRestorationConfirmed === true &&
      records.every(
        (r) =>
          !r.state.running &&
          !r.state.qualification &&
          !r.state.failure &&
          !r.state.serialFailureCategory &&
          !r.state.admissionFailureStage &&
          !r.state.ownerResourceFailure &&
          r.state.renewalsConfirmed === 0 &&
          !r.state.heartbeatSuppressed &&
          r.state.status !== "window_loaded" &&
          r.state.cadence?.suppressionRequested !== true &&
          r.state.cadence?.firstWorkObservedAtMs === undefined &&
          r.state.cadence?.latestWork === undefined,
      ),
    "unexpected_work_or_failure",
  );
  for (const row of records) {
    const review = row.state.cadence?.review;
    if (!review) continue;
    validateCadenceReview(review);
    requireCadenceDiagnostics(context, review);
    const mining = review.phases[2];
    check(
      mining.state === "empty" &&
        mining.generation === 0 &&
        mining.intervalCount === 0 &&
        mining.armedAtUs === 0 &&
        mining.startedAtUs === 0 &&
        mining.endedAtUs === 0,
      "mining_phase_not_empty",
    );
  }
  const cycles = await verifyPreparationCycles(root, context, records);
  const idle = await idleEvidence(root, hash, records, cycles);
  const coolingProof = await proof(resolve(root, "cooling.json")),
    cooling = coolingProof.value;
  check(cooling.schema === "worker-iterative-cooling-v1" && cooling.context_sha256 === hash, "cooling_context");
  validateCooling(cooling.proof, cooling.restoration);
  baseline(cooling.state, false);
  requireIdleLedger(cooling.budget_before, 17, 1380000);
  requireIdleLedger(cooling.budget_after, 17, 1380000);
  check(equal(cooling.budget_before, cooling.budget_after), "cooling_accounting_changed");
  return { snapshot, journalPath, records, final, cycles, idle, coolingProof, cooling };
}
async function usbEvidence(root, context, records, idle) {
  const hash = digest(JSON.stringify(context));
  const first = records.find((r) => r.state.cadence?.review?.phases[1]?.state === "complete");
  check(first && first.sequence > idle.record.finished_sequence, "usb_complete_review_missing");
  baseline(first.state, false);
  const review = validateCadenceReview(first.state.cadence.review),
    usb = review.phases[1];
  check(
    equal(review.phases[0], idle.summary) &&
      review.snapshotAvailable === true &&
      review.droppedObservations === 0 &&
      usb.state === "complete" &&
      usb.passed === false &&
      !usb.overflow &&
      usb.generation === 0 &&
      usb.armedAtUs <= usb.startedAtUs &&
      usb.endedAtUs - usb.startedAtUs >= 60000000 &&
      usb.endedAtUs - usb.startedAtUs <= 62000000 &&
      usb.intervalCount >= 60 &&
      usb.intervalBuckets.reduce((a, b) => a + b, 0) === usb.intervalCount &&
      usb.intervalBuckets[3] === 0 &&
      usb.maximumIntervalUs <= 1500000 &&
      usb.maximumExecutionUs <= 500000 &&
      usb.maximumLiveUs <= usb.maximumExecutionUs &&
      usb.maximumLogsUs <= usb.maximumExecutionUs &&
      usb.maximumPruneUs <= usb.maximumExecutionUs &&
      usb.sendsQueued === usb.sendsCompleted &&
      [
        "cpuMismatchCount",
        "priorityMismatchCount",
        "subscriberMismatchCount",
        "noSubscriberCount",
        "projectionFailures",
        "serializationFailures",
        "queueFailures",
        "sendFailures",
        "pendingSends",
        "clockFailures",
      ].every((k) => usb[k] === 0),
    "usb_nonpercentile_predicate",
  );
  check(
    usb.worstInterval.previousExecutionUs + usb.worstInterval.gapUs === usb.maximumIntervalUs &&
      usb.worstInterval.previousLiveStagesUs.reduce((a, b) => a + b, 0) <= usb.worstInterval.previousExecutionUs &&
      usb.maximumLiveStagesUs.every((v) => v <= usb.maximumLiveUs),
    "usb_live_coherence",
  );
  const within = usb.intervalBuckets[0] + usb.intervalBuckets[1];
  check(
    within === 89 &&
      usb.intervalCount === 95 &&
      equal(usb.intervalBuckets, [0, 89, 6, 0]) &&
      within * 100 < usb.intervalCount * 95 &&
      usb.maximumIntervalUs === 828450 &&
      usb.maximumExecutionUs === 320724,
    "usb_percentile_failure",
  );
  let rejected;
  try {
    requireCadencePhase(review, "usb");
  } catch (error) {
    rejected = error.code;
  }
  check(rejected === "cadence_phase_unqualified", "usb_review_must_fail");
  const armProof = await proof(resolve(root, "cadence-usb-arm.json")),
    arm = armProof.value;
  exactObject(arm, ["context_sha256", "phase", "arm", "started_sequence", "started_at_unix_ms"]);
  exactObject(arm.arm, ["schema", "phase", "armedAtUs", "generation"]);
  check(
    arm.context_sha256 === hash &&
      arm.phase === "usb" &&
      arm.arm.schema === "worker-telemetry-cadence-arm-v1" &&
      arm.arm.phase === "usb" &&
      arm.arm.generation === 0 &&
      arm.arm.armedAtUs === usb.armedAtUs &&
      Number.isSafeInteger(arm.started_sequence) &&
      arm.started_sequence > idle.record.finished_sequence &&
      arm.started_sequence < first.sequence &&
      Number.isSafeInteger(arm.started_at_unix_ms) &&
      arm.started_at_unix_ms >= idle.record.finished_at_unix_ms,
    "usb_arm_binding",
  );
  baseline(records[arm.started_sequence - 1].state, false);
  check(
    usb.maxProbeCount === 12 &&
      usb.firstMaxProbeAtUs >= usb.startedAtUs &&
      usb.lastMaxProbeAtUs >= usb.firstMaxProbeAtUs &&
      usb.lastMaxProbeAtUs - usb.startedAtUs < 60000000,
    "device_probe_witness",
  );
  return { first, review, usb, within, rejected, armProof, arm };
}
async function probeEvidence(root, arm, first) {
  const probes = await jsonLines(resolve(root, "cadence-probes.jsonl"), 16384),
    witnesses = await jsonLines(resolve(root, "cadence-probe-witnesses.jsonl"), 16384);
  check(probes.rows.length === 12 && witnesses.rows.length === 12, "probe_count");
  let previous, priorWitness;
  for (const [i, p] of probes.rows.entries()) {
    previous = validateCadenceProbe(p, previous);
    const w = witnesses.rows[i];
    exactObject(w, ["ordinal", "observedAtUnixMs", "sequence"]);
    check(
      w.ordinal === p.ordinal &&
        Number.isSafeInteger(w.observedAtUnixMs) &&
        w.observedAtUnixMs >= arm.started_at_unix_ms &&
        w.observedAtUnixMs < arm.started_at_unix_ms + 60000 &&
        Number.isSafeInteger(w.sequence) &&
        w.sequence >= arm.started_sequence &&
        w.sequence < first.sequence &&
        (!priorWitness || (w.observedAtUnixMs >= priorWitness.observedAtUnixMs && w.sequence >= priorWitness.sequence)),
      "probe_observation_binding",
    );
    priorWitness = w;
  }
  return { probes, witnesses };
}
async function operatorEvidence(root, context, final, usb, within) {
  const hash = digest(JSON.stringify(context));
  const operatorProof = await proof(resolve(root, "operator-failure.json")),
    failure = operatorProof.value;
  exactObject(failure, [
    "schema",
    "source",
    "context_sha256",
    "browser_error",
    "browser_error_location",
    "browser_cleanup_promise",
    "phase",
    "journal_sequence",
    "within_750_ms",
    "intervals",
    "maximum_interval_us",
    "maximum_execution_us",
    "all_twelve_probes_completed",
    "mining_issued",
    "mining_consumed",
    "qualified",
  ]);
  check(
    equal(failure, {
      schema: "parent-observed-cadence-failure-v1",
      source: "parent-observed",
      context_sha256: hash,
      browser_error: "cadence_supervisor_rejected",
      browser_error_location: "cadence-client.mjs:31 via runQualification usb phase",
      browser_cleanup_promise: "fulfilled-empty-errors",
      phase: "usb",
      journal_sequence: final.sequence,
      within_750_ms: within,
      intervals: usb.intervalCount,
      maximum_interval_us: usb.maximumIntervalUs,
      maximum_execution_us: usb.maximumExecutionUs,
      all_twelve_probes_completed: true,
      mining_issued: false,
      mining_consumed: false,
      qualified: false,
    }),
    "parent_failure_provenance",
  );
  const cleanupProof = await proof(resolve(root, "operator-cleanup.json"));
  check(
    equal(cleanupProof.value, {
      schema: "worker-cadence-host-cleanup-v1",
      source: "parent-observed",
      browser_closed: true,
      supervisor_exited: true,
      supervisor_exit_code: 0,
      listener_absent: true,
      owned_children_absent: true,
      serial_holders_absent: true,
    }),
    "actual_cleanup",
  );
  const supplemental = await proof(resolve(root, "supervisor.observation.json")),
    supp = supplemental.value;
  check(
    supp.schema === "hello-passive-command-observation-v1" &&
      supp.rootObserved === true &&
      supp.complete === false &&
      supp.observations === 1133 &&
      equal(supp.failures, []) &&
      supp.remaining === undefined &&
      supp.finished_at_unix_ms === undefined,
    "supplement_changed",
  );
  return { operatorProof, cleanupProof, supplemental, supp };
}
/** Reconstruct this closed pre-mining failure; no caller pass flag supplies a missing fact. */
export async function inspectCadenceFailureEvidence(root, context, restart, producer) {
  const hash = digest(JSON.stringify(context));
  const { snapshot, journalPath, records, final, cycles, idle, coolingProof, cooling } = await initialEvidence(root, context);
  const { first, review, usb, within, rejected, armProof, arm } = await usbEvidence(root, context, records, idle);
  const { probes, witnesses } = await probeEvidence(root, arm, first);
  check(
    final.sequence > first.sequence &&
      records.filter((r) => r.sequence >= first.sequence && r.state.cadence?.review).every((r) => equal(r.state.cadence.review, review)),
    "retained_review_changed",
  );
  const observer = await observerProof(root, (await proof(resolve(root, "cadence-idle-arm.json"))).value);
  check(observer.terminal_observed_at >= arm.started_at_unix_ms + 60000, "usb_observer_coverage");
  const { operatorProof, cleanupProof, supplemental, supp } = await operatorEvidence(root, context, final, usb, within);
  const seal = {
    schema: "fixed-usb-cadence-usb-percentile-failed-inventory-v1",
    outcome: "unverified_premining_usb_percentile_failure",
    auditor_sha256: producer,
    context_sha256: hash,
    restart_receipt_sha256: restart.sha256,
    artifact_snapshot_sha256: snapshot.receipt_sha256,
    journal_sha256: await fileDigest(journalPath),
    journal_rows: records.length,
    first_rejected_review_sequence: first.sequence,
    first_rejected_review_sha256: digest(JSON.stringify(review)),
    parent_failure_sha256: operatorProof.sha256,
    cleanup_sha256: cleanupProof.sha256,
    idle_phase_sha256: idle.sha256,
    usb_arm_sha256: armProof.sha256,
    probe_sha256: probes.sha256,
    probe_witness_sha256: witnesses.sha256,
    observer_result_sha256: observer.sha256,
    cooling_sha256: coolingProof.sha256,
    last_observed_ledger: cooling.budget_after,
    accounting_provenance: "cooling_before_and_after_before_phases",
    post_failure_ledger_observed: false,
    original_budget: restart.value.receipt.original_budget,
    original_budget_provenance: "accepted_stage_b_004",
    cadence_usb_file_absent: true,
    mining_phase_empty: true,
    allowance_issued: false,
    allowance_consumed: false,
    mining_authorized: false,
    qualification_pass: false,
    continuation_authority: false,
    final_sequence: final.sequence,
    final_state_sha256: digest(JSON.stringify(final.state)),
    observations: {
      idle_intervals: idle.summary.intervalCount,
      idle_within_750_ms: idle.summary.intervalBuckets[0] + idle.summary.intervalBuckets[1],
      idle_maximum_interval_us: idle.summary.maximumIntervalUs,
      usb_intervals: usb.intervalCount,
      usb_within_750_ms: within,
      usb_maximum_interval_us: usb.maximumIntervalUs,
      usb_maximum_execution_us: usb.maximumExecutionUs,
      usb_interval_buckets: usb.intervalBuckets,
      usb_probes: probes.rows.length,
      dropped_observations: 0,
      pending_sends: 0,
      strict_validator_failure: rejected,
      observer_elapsed_ms: observer.result.closedAtUnixMs - observer.result.startedAtUnixMs,
    },
    supplemental_observer: {
      sha256: supplemental.sha256,
      complete: false,
      observations: supp.observations,
      cleanup_proven_by_supplement: false,
    },
    files: await inventory(root),
  };
  return seal;
}
