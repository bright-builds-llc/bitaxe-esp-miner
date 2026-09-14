import { dirname, resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, fileDigest, missing, requireCondition as check } from "./contract.mjs";
import { readPremining, preminingClosurePath } from "./cadence-premining.mjs";
import { validateCadenceContext } from "./cadence-preflight.mjs";
import { readPrevious } from "./iterative-preflight.mjs";
import { requireIdleLedger } from "./iterative-contract.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { baseline, inventory, observerProof, proof, verifyPreparationCycles } from "./cadence-premining-evidence.mjs";
import { parseSamples } from "./sample-seal.mjs";
import { validateCadenceReview } from "./cadence-evidence.mjs";
import { jsonLines, idleEvidence, usbEvidence } from "./cadence-premining-usb-evidence.mjs";

export const ACCEPTED_USB_PREMINING_INVENTORY = "1936ad2be4fffa59dbc650fd833bc335e280dd31394ab3378e5d2760aa1ab580";
export const USB_PREMINING_AUDITOR = "66653e866e924d7811acf1b13b3889f20fec2c92c5c31588d00f4f040b171d11";

async function operatorFailure(root, context, final) {
  const failureSaved = await proof(resolve(root, "operator-failure.json")),
    failure = failureSaved.value;
  exactObject(failure, [
    "schema",
    "source",
    "client_failure",
    "replayed_validator_failure",
    "first_rejected_review_sequence",
    "first_rejected_review_sha256",
    "qualified",
    "usb_probes_completed",
    "mining_started",
    "allowance_issued",
    "observations",
    "ledger",
    "ledger_source",
    "retained_review_after_fresh_reconnect",
    "final_state",
  ]);
  check(
    failure.schema === "cpu0-cadence-usb-failure-v1" &&
      failure.source === "parent-observed" &&
      equal(failure.client_failure, { stage: "usb_review", message: "cadence_supervisor_rejected" }) &&
      failure.qualified === false &&
      failure.usb_probes_completed === 12 &&
      failure.mining_started === false &&
      failure.allowance_issued === false &&
      failure.ledger_source === "fresh_authenticated_workerAcceptance.reviewQualificationAttempts",
    "operator_failure_binding",
  );
  requireIdleLedger(failure.ledger, context.qualification_attempt.ordinal, context.expected_charged_ms);
  check(
    equal(
      failure.final_state,
      Object.fromEntries(
        ["status", "connected", "running", "deviceBaselineConfirmed", "deviceLeaseInactive", "serialOwnershipReleased"].map((key) => [
          key,
          final[key],
        ]),
      ),
    ),
    "final_state_binding",
  );
  return { failureSaved, failure };
}

/** Recognize only the accepted USB failure and independently reconstruct its evidence. */
export async function reviewUsbPreminingEvidence(root, operations = {}) {
  const accepted = await proof(resolve(root, "failed-inventory.json"));
  check(
    accepted.sha256 === (operations.expectedUsbPreminingInventorySha256 ?? ACCEPTED_USB_PREMINING_INVENTORY),
    "cadence_usb_premining_accepted_seal_required",
  );
  const saved = await proof(resolve(root, "context.json"));
  exactObject(saved.value, ["context", "sha256"]);
  const context = saved.value.context,
    contextHash = digest(JSON.stringify(context));
  check(
    saved.value.sha256 === contextHash &&
      context.schema === "fixed-usb-cadence-context-v1" &&
      context.preparation_attempt === 3 &&
      context.premining_predecessor,
    "context_binding",
  );
  await validateCadenceContext(root, context, { historical: true, operations });
  const snapshot = await (operations.verifyArtifactSnapshot ?? verifyArtifactSnapshot)(root, context),
    previous = await (operations.readPrevious ?? readPrevious)(context.previous_receipt);
  const priorPath = preminingClosurePath(context.premining_predecessor.root);
  check((await fileDigest(priorPath)) === context.premining_predecessor.closure_sha256, "predecessor_hash");
  const predecessor = await readPremining(priorPath, operations);
  check(
    predecessor.schema === "worker-cadence-premining-closure-v1" &&
      predecessor.context.preparation_attempt === 2 &&
      dirname(predecessor.root) === dirname(root) &&
      previous.next_ordinal === context.qualification_attempt.ordinal &&
      previous.total_charged_ms === context.expected_charged_ms,
    "predecessor_lineage",
  );
  for (const name of [
    "issued.json",
    "consumed.json",
    "result.json",
    "cadence-usb.json",
    "cadence-mining-arm.json",
    "cadence-mining.json",
    "cadence-mining-measurement-end.json",
    "iterative.fault.json",
    "sample-seal-intent.json",
    "sealed.samples.jsonl",
    "sealed-inventory.json",
    "unissued-closure.json",
    "first-failure.json",
  ])
    await missing(resolve(root, name));
  const journal = await jsonLines(resolve(root, "iterative.samples.jsonl"), 33554432),
    records = parseSamples(journal.bytes, context),
    final = records.at(-1).state;
  baseline(final, true);
  check(
    records.every(
      ({ state: s }) =>
        !s.running &&
        s.status !== "window_loaded" &&
        s.renewalsConfirmed === 0 &&
        !s.failure &&
        !s.serialFailureCategory &&
        !s.admissionFailureStage &&
        (!s.qualification?.attempt || s.qualification.attempt.ordinal < context.qualification_attempt.ordinal) &&
        (!s.cadence || (!s.cadence.suppressionRequested && s.cadence.firstWorkObservedAtMs === undefined && !s.cadence.latestWork)),
    ),
    "unexpected_funding_or_work",
  );
  for (const row of records) {
    const review = row.state.cadence?.review;
    if (review) {
      validateCadenceReview(review);
      check(
        review.phases[2].state === "empty" && review.phases[2].intervalCount === 0 && review.phases[2].generation === 0,
        "mining_phase_observed",
      );
    }
  }
  const cycles = await verifyPreparationCycles(root, context, records);
  const { failureSaved, failure } = await operatorFailure(root, context, final);
  const idle = await idleEvidence(root, contextHash, records, cycles),
    usb = await usbEvidence(root, contextHash, records, idle, failure),
    observer = await observerProof(root, usb.arm);
  check(
    observer.result.connectedAtUnixMs <= idle.record.started_at_unix_ms && observer.terminal_observed_at >= idle.record.finished_at_unix_ms,
    "observer_phase_coverage",
  );
  const cleanupSaved = await proof(resolve(root, "operator-cleanup.json")),
    cleanup = cleanupSaved.value;
  exactObject(cleanup, [
    "schema",
    "source",
    "browser_closed",
    "supervisor_exited",
    "supervisor_exit_code",
    "listener_absent",
    "owned_children_absent",
    "serial_holders_absent",
  ]);
  check(
    cleanup.schema === "worker-cadence-host-cleanup-v1" &&
      cleanup.source === "parent-observed" &&
      cleanup.supervisor_exit_code === 0 &&
      ["browser_closed", "supervisor_exited", "listener_absent", "owned_children_absent", "serial_holders_absent"].every(
        (key) => cleanup[key] === true,
      ),
    "host_cleanup",
  );
  const files = await inventory(root);
  const seal = {
    schema: "cpu0-cadence-usb-failed-inventory-v1",
    outcome: "unverified_usb_cadence_failure",
    qualification_pass: false,
    continuation_authority: false,
    auditor_sha256: USB_PREMINING_AUDITOR,
    context_sha256: contextHash,
    artifact_snapshot_sha256: snapshot.receipt_sha256,
    premining_closure_sha256: context.premining_predecessor.closure_sha256,
    samples_sha256: journal.sha256,
    operator_failure_sha256: failureSaved.sha256,
    operator_cleanup_sha256: cleanupSaved.sha256,
    idle_phase_sha256: idle.sha256,
    observer_result_sha256: observer.sha256,
    first_rejected_review: { sequence: usb.first_sequence, sha256: usb.first_sha256 },
    later_retained_review: { sequence: usb.later_sequence, sha256: usb.later_sha256, after_released_sequence: usb.released_sequence },
    usb_arm_sha256: usb.arm_sha256,
    probes_sha256: usb.probes_sha256,
    probe_witnesses_sha256: usb.witnesses_sha256,
    inventory: files,
  };
  check(equal(accepted.value, seal), "cadence_usb_premining_legacy_seal_changed");
  return {
    failure_class: "usb_cadence_failure",
    context,
    context_sha256: contextHash,
    failed_inventory_sha256: accepted.sha256,
    operator_failure_sha256: failureSaved.sha256,
    earliest_failure: failure.client_failure,
    ledger: failure.ledger,
    cleanup,
    first_rejected_review: seal.first_rejected_review,
    later_retained_review: seal.later_retained_review,
    samples_sha256: journal.sha256,
    final_sequence: records.at(-1).sequence,
    artifact_snapshot_sha256: snapshot.receipt_sha256,
    observer_result_sha256: observer.sha256,
    file_count: files.filter((row) => row.type === "file").length,
    cycle_count: cycles.cycle_count,
    flash_count: cycles.flash_count,
  };
}
