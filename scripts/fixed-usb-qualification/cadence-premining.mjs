import { lstat, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import {
  canonicalDirectory,
  digest,
  exactObject,
  fileDigest,
  missing,
  protectedPath,
  requireCondition as check,
  writeNew,
} from "./contract.mjs";
import { requireCadenceTask } from "./cadence-contract.mjs";
import { validateCadenceContext } from "./cadence-preflight.mjs";
import { readUnissued } from "./cadence-unissued.mjs";
import { readPrevious } from "./iterative-preflight.mjs";
import { requireIdleLedger } from "./iterative-contract.mjs";
import { parseSamples } from "./sample-seal.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { baseline, inventory, observerProof, proof, verifyPreparationCycles } from "./cadence-premining-evidence.mjs";

// Recognize the retained producer, then independently rederive every supported fact.
export const LEGACY_PREMINING_AUDITOR = "53849ff5e24230bb3ff7181fda40bc9ab61b2722ea96152b32824e95846673fe";
export const ACCEPTED_PREMINING_INVENTORY = "6895f2fcfe161a0d7617b51b02b9f12fcdf49454c9ee540295c31f40b463a5a7";
export const preminingClosurePath = (root) => `${resolve(root)}.premining-closure.json`;

async function operatorFailure(root, context, records, final) {
  const failureSaved = await proof(resolve(root, "operator-failure.json")),
    failure = failureSaved.value;
  exactObject(failure, [
    "schema",
    "source",
    "earliest_failure",
    "diagnostic_recovery",
    "qualification_pass",
    "usb_phase_started",
    "mining_phase_started",
    "allowance_issued",
    "ledger",
    "final_state",
    "raw_network_payloads_persisted",
  ]);
  check(
    failure.schema === "cpu0-cadence-operator-failure-v1" &&
      failure.source === "parent-observed" &&
      failure.qualification_pass === false &&
      failure.usb_phase_started === false &&
      failure.mining_phase_started === false &&
      failure.allowance_issued === false &&
      failure.raw_network_payloads_persisted === false,
    "operator_failure_shape",
  );
  check(
    isDeepStrictEqual(failure.earliest_failure, {
      stage: "idle_review",
      category: "probe_admission",
      class: "SerialFailure",
      method: "workerAcceptance.cadenceReview",
      guard: "before_possession_refresh",
    }) &&
      isDeepStrictEqual(failure.diagnostic_recovery, {
        fresh_authenticated_reconnect: true,
        review_result: "probe_admission",
        guard: "after_possession_refresh",
        frozen_summary_collected: false,
      }),
    "operator_failure_boundary",
  );
  requireIdleLedger(failure.ledger, context.qualification_attempt.ordinal, context.expected_charged_ms);
  const selected = Object.fromEntries(
    ["status", "connected", "running", "deviceBaselineConfirmed", "deviceLeaseInactive", "serialOwnershipReleased"].map((key) => [
      key,
      final[key],
    ]),
  );
  check(isDeepStrictEqual(failure.final_state, selected), "operator_final_state_binding");
  check(
    !records.some((row) => row.state.failure || row.state.serialFailureCategory || row.state.admissionFailureStage),
    "unexpected_earlier_browser_failure",
  );
  await missing(resolve(root, "first-failure.json"));
  const cleanup = (await proof(resolve(root, "operator-cleanup.json"))).value;
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

  return { failureSaved, failure, cleanup };
}

async function reviewEvidence(root, operations) {
  const acceptedSeal = await proof(resolve(root, "failed-inventory.json"));
  check(
    acceptedSeal.sha256 === (operations.expectedPreminingInventorySha256 ?? ACCEPTED_PREMINING_INVENTORY),
    "cadence_premining_accepted_seal_required",
  );
  const saved = await proof(resolve(root, "context.json"));
  exactObject(saved.value, ["context", "sha256"]);
  const context = saved.value.context;
  check(context.schema === "fixed-usb-cadence-context-v1" && saved.value.sha256 === digest(JSON.stringify(context)), "context_integrity");
  await validateCadenceContext(root, context, { historical: true, operations });
  const previous = await (operations.readPrevious ?? readPrevious)(context.previous_receipt),
    snapshot = await (operations.verifyArtifactSnapshot ?? verifyArtifactSnapshot)(root, context);
  check(
    previous.next_ordinal === context.qualification_attempt.ordinal && previous.total_charged_ms === context.expected_charged_ms,
    "previous_ledger",
  );
  check(context.unissued_predecessor && context.preparation_attempt === 2, "preparation_lineage");
  const predecessorPath = resolve(context.unissued_predecessor.root, "unissued-closure.json");
  check((await fileDigest(predecessorPath)) === context.unissued_predecessor.closure_sha256, "unissued_predecessor_digest");
  await readUnissued(predecessorPath, operations);
  for (const name of [
    "issued.json",
    "consumed.json",
    "result.json",
    "sample-seal-intent.json",
    "sealed.samples.jsonl",
    "cadence-idle.json",
    "cadence-usb-arm.json",
    "cadence-usb.json",
    "cadence-mining-arm.json",
    "cadence-mining.json",
    "cadence-probes.jsonl",
    "cadence-probe-witnesses.jsonl",
    "cadence-mining-measurement-end.json",
    "iterative.fault.json",
    "unissued-closure.json",
    "sealed-inventory.json",
  ])
    await missing(resolve(root, name));
  const journalPath = resolve(root, "iterative.samples.jsonl");
  await protectedPath(journalPath);
  const bytes = await readFile(journalPath),
    records = parseSamples(bytes, context),
    final = records.at(-1).state;
  baseline(final, true);
  check(
    records.every(
      ({ state }) =>
        !state.running &&
        state.status !== "window_loaded" &&
        state.renewalsConfirmed === 0 &&
        (!state.qualification?.attempt || state.qualification.attempt.ordinal < context.qualification_attempt.ordinal) &&
        (!state.cadence ||
          (!state.cadence.review &&
            state.cadence.firstWorkObservedAtMs === undefined &&
            !state.cadence.latestWork &&
            !state.cadence.suppressionRequested)),
    ),
    "unexpected_work_or_collected_summary",
  );
  const cycles = await verifyPreparationCycles(root, context, records);
  const armSaved = await proof(resolve(root, "cadence-idle-arm.json")),
    arm = armSaved.value;
  exactObject(arm, ["context_sha256", "phase", "arm", "started_sequence", "started_at_unix_ms"]);
  exactObject(arm.arm, ["schema", "phase", "armedAtUs", "generation"]);
  check(
    arm.context_sha256 === saved.value.sha256 &&
      arm.phase === "idle" &&
      arm.arm.schema === "worker-telemetry-cadence-arm-v1" &&
      arm.arm.phase === "idle" &&
      arm.arm.generation === 0 &&
      Number.isSafeInteger(arm.arm.armedAtUs) &&
      arm.arm.armedAtUs >= 0 &&
      Number.isSafeInteger(arm.started_at_unix_ms) &&
      arm.started_at_unix_ms > 0 &&
      Number.isInteger(arm.started_sequence) &&
      arm.started_sequence >= cycles.last_ready_sequence &&
      arm.started_sequence < records.length,
    "idle_arm_binding",
  );
  baseline(records[arm.started_sequence - 1].state, false);
  const observer = await observerProof(root, arm);
  const { failureSaved, failure, cleanup } = await operatorFailure(root, context, records, final);

  const legacyPath = resolve(root, "failed-inventory.json"),
    legacy = await proof(legacyPath),
    files = await inventory(root);
  const expected = {
    schema: "cpu0-cadence-failed-inventory-v1",
    outcome: "failed_preparation",
    qualification_pass: false,
    continuation_authority: false,
    auditor_sha256: LEGACY_PREMINING_AUDITOR,
    context_sha256: saved.value.sha256,
    artifact_snapshot_sha256: snapshot.receipt_sha256,
    unissued_closure_sha256: context.unissued_predecessor.closure_sha256,
    samples_sha256: digest(bytes),
    operator_failure_sha256: failureSaved.sha256,
    idle_arm_sha256: armSaved.sha256,
    observer_result_sha256: observer.sha256,
    inventory: files,
  };
  check(isDeepStrictEqual(legacy.value, expected), "cadence_premining_legacy_seal_changed");
  return {
    context,
    context_sha256: saved.value.sha256,
    failed_inventory_sha256: legacy.sha256,
    operator_failure_sha256: failureSaved.sha256,
    earliest_failure: failure.earliest_failure,
    ledger: failure.ledger,
    cleanup,
    samples_sha256: digest(bytes),
    final_sequence: records.at(-1).sequence,
    observer_result_sha256: observer.sha256,
    artifact_snapshot_sha256: snapshot.receipt_sha256,
    file_count: files.filter((row) => row.type === "file").length,
    cycle_count: cycles.cycle_count,
    flash_count: cycles.flash_count,
  };
}

function summary(evidence) {
  return {
    result: "unverified",
    classification: "premining_failure",
    qualification_pass: false,
    continuation_authority: false,
    device_effects: false,
    ordinal: evidence.context.qualification_attempt.ordinal,
    preparation_attempt: evidence.context.preparation_attempt,
    file_count: evidence.file_count,
    cycle_count: evidence.cycle_count,
    flash_count: evidence.flash_count,
    failed_inventory_sha256: evidence.failed_inventory_sha256,
  };
}

/** Validate an already sealed failed preparation without adding evidence or authority. */
export async function reviewPremining(root, operations = {}) {
  root = await canonicalDirectory(root);
  await protectedPath(root, true);
  let siblingExists = false;
  try {
    await lstat(preminingClosurePath(root));
    siblingExists = true;
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  return summary(siblingExists ? await readPremining(preminingClosurePath(root), operations) : await reviewEvidence(root, operations));
}

/** Append a sibling classification receipt; the sealed preparation is never edited. */
export async function closePremining(root, operations = {}) {
  root = await canonicalDirectory(root);
  await protectedPath(root, true);
  await protectedPath(dirname(root), true);
  const path = preminingClosurePath(root);
  await missing(path);
  const evidence = await reviewEvidence(root, operations);
  await requireCadenceTask(evidence.context.firmware_root);
  const receipt = { schema: "worker-cadence-premining-closure-v1", root, ...summary(evidence), ...evidence };
  // Catch a changed or partially rewritten source before creating the exclusive sibling.
  check(isDeepStrictEqual(await reviewEvidence(root, operations), evidence), "cadence_premining_concurrent_change");
  await requireCadenceTask(evidence.context.firmware_root);
  await writeNew(path, { receipt, sha256: digest(JSON.stringify(receipt)) });
  return summary(evidence);
}

/** Revalidate sibling provenance and all immutable sources for a future preflight. */
export async function readPremining(path, operations = {}) {
  path = resolve(path);
  await protectedPath(path);
  await protectedPath(dirname(path), true);
  const saved = await proof(path);
  exactObject(saved.value, ["receipt", "sha256"]);
  const receipt = saved.value.receipt;
  check(typeof receipt?.root === "string" && path === preminingClosurePath(receipt.root), "cadence_premining_closure_path");
  const root = await canonicalDirectory(receipt.root),
    ancestors = operations.preminingAncestors ?? [];
  check(!ancestors.includes(root), "cadence_premining_recursive_lineage");
  operations = { ...operations, preminingAncestors: [...ancestors, root] };
  check(saved.value.sha256 === digest(JSON.stringify(receipt)), "cadence_premining_closure_integrity");
  const evidence = await reviewEvidence(root, operations);
  const expected = { schema: "worker-cadence-premining-closure-v1", root, ...summary(evidence), ...evidence };
  check(isDeepStrictEqual(receipt, expected), "cadence_premining_closure_changed");
  return receipt;
}
