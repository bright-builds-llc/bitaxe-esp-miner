import { writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { resetOriginFixture, resetDiagnostics } from "./reset-origin-fixtures.mjs";
import { recoveryState, recordRecoveryState } from "./cadence-startup-fixtures.mjs";
import { saveNoMiningAccounting } from "./no-mining-accounting.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { resetOriginPreflight } from "./reset-origin-context.mjs";
import { RESET_ORIGIN_JOURNAL_REGRESSION } from "./reset-origin-journal-failure.mjs";
import { RESET_ORIGIN_FINALIZATION_REGRESSION } from "./reset-origin-finalization-failure.mjs";
import { inspectResetOriginObservation } from "./reset-origin-observation-review.mjs";
import { createResetOriginObservation } from "./reset-origin-observation.mjs";
export async function unusedObservation(t) {
  const f = await resetOriginFixture(t),
    hash = digest(JSON.stringify(f.context));
  await writeNew(resolve(f.root, "page-serving-failure.json"), {
    schema: "reset-origin-page-serving-failure-v1",
    source: "parent-observed",
    http_status: 200,
    content_type: "text/html",
    body_sha256: "a".repeat(64),
    json_encoded_html: true,
    browser_control_opened: false,
    browser_closed: true,
    first_failure: "html_response_json_encoded",
  });
  await writeNew(resolve(f.root, "unused-host-cleanup.json"), {
    schema: "reset-origin-unused-host-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    browser_control_opened: false,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
    observation_started: false,
  });
  await writeNew(resolve(f.root, "reset-origin-server-claim.json"), {
    schema: "fixed-usb-reset-origin-server-claim-v1",
    context_sha256: hash,
  });
  await writeNew(resolve(f.root, "reset-origin-failure.json"), {
    schema: "fixed-usb-reset-origin-failure-v1",
    context_sha256: hash,
    code: "reset_origin_server_closed_before_end",
  });
  const seal = {
    schema: "fixed-usb-reset-origin-unstarted-failed-inventory-v1",
    outcome: "unverified",
    first_failure: "html_response_json_encoded",
    secondary_failure: "reset_origin_server_closed_before_end",
    observation_started: false,
    qualification_pass: false,
    device_recovery_claimed: false,
    continuation_authorized: false,
    context_sha256: hash,
    artifact_snapshot_sha256: await fileDigest(resolve(f.root, "artifact-snapshot.json")),
    page_failure_sha256: await fileDigest(resolve(f.root, "page-serving-failure.json")),
    cleanup_sha256: await fileDigest(resolve(f.root, "unused-host-cleanup.json")),
    secondary_failure_sha256: await fileDigest(resolve(f.root, "reset-origin-failure.json")),
    inventory: await inventory(f.root),
  };
  await writeNew(resolve(f.root, "failed-inventory.json"), seal);
  f.operations.expectedResetOriginUnstartedSeal = await fileDigest(resolve(f.root, "failed-inventory.json"));
  const plan = resolve(dirname(f.root), "observation-correction.json");
  await writeNew(plan, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [f.operations.expectedResetOriginUnstartedSeal],
  });
  f.successor = {
    ...f.options,
    privateRoot: resolve(dirname(f.root), "observation-2"),
    input: plan,
    qualificationSourceCommit: "f".repeat(40),
    supersedeUnstarted: f.root,
  };
  return f;
}

export async function preparationReview(t) {
  const f = await unusedObservation(t);
  await resetOriginPreflight(f.successor, f.operations);
  const root = f.successor.privateRoot,
    context = (await readJson(resolve(root, "context.json"))).context,
    inner = context.no_mining_context,
    hash = digest(JSON.stringify(context));
  for (const status of ["configured", "configured", "ready"]) {
    const state = recoveryState(inner);
    if (status === "configured")
      Object.assign(state, { status, connected: false, deviceBaselineConfirmed: false, deviceLeaseInactive: false });
    await recordRecoveryState(root, inner, state);
  }
  await saveNoMiningAccounting(root, inner, {
    stage: "before",
    ledger: f.ledger,
    original_budget: f.original,
    state: recoveryState(inner),
  });
  for (const status of ["closing", "closing", "closed"]) await recordRecoveryState(root, inner, { ...recoveryState(inner, true), status });
  await writeNew(resolve(root, "parent-failure-review.json"), {
    schema: "reset-origin-parent-failure-review-v1",
    source: "parent-observed",
    first_failure: "reset_origin_preparation_receipt_review_required",
    capture_started: false,
    browser_closed: true,
    observer: { stage: "failed", started: true, cleanupFailed: false },
    worker: {
      status: "closed",
      connected: false,
      running: false,
      serialOwnershipReleased: true,
      baseline: true,
      inactive: true,
      failure: null,
    },
    preparation: [
      { authoritative: false, category: "worker_preparation_receipt", origin: "previous_boot", status: "wrong_firmware" },
      { authoritative: false, category: "worker_preparation_receipt", origin: "current_boot", status: "unavailable" },
    ],
  });
  await writeNew(resolve(root, "host-cleanup.json"), {
    schema: "worker-reset-origin-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
  await writeNew(resolve(root, "reset-origin-server-claim.json"), {
    schema: "fixed-usb-reset-origin-server-claim-v1",
    context_sha256: hash,
  });
  await writeNew(resolve(root, "reset-origin-failure.json"), {
    schema: "fixed-usb-reset-origin-failure-v1",
    context_sha256: hash,
    code: "reset_origin_preparation_receipt_review_required",
  });
  await writeNew(resolve(root, "failed-inventory.json"), {
    schema: "fixed-usb-reset-origin-preparation-review-failed-inventory-v1",
    outcome: "unverified",
    first_failure: "reset_origin_preparation_receipt_review_required",
    capture_started: false,
    observation_pass: false,
    device_recovery_claimed: false,
    continuation_authorized: false,
    after_accounting_observed: false,
    context_sha256: hash,
    artifact_snapshot_sha256: await fileDigest(resolve(root, "artifact-snapshot.json")),
    parent_failure_sha256: await fileDigest(resolve(root, "parent-failure-review.json")),
    cleanup_sha256: await fileDigest(resolve(root, "host-cleanup.json")),
    before_accounting_sha256: await fileDigest(resolve(root, "no-mining-accounting-before.json")),
    inventory: await inventory(root),
  });
  f.operations.expectedResetOriginPreparationReviewSeal = await fileDigest(resolve(root, "failed-inventory.json"));
  const input = resolve(dirname(root), "preparation-review-correction.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [f.operations.expectedResetOriginPreparationReviewSeal],
  });
  return {
    ...f,
    reviewRoot: root,
    reviewContext: context,
    third: {
      ...f.options,
      privateRoot: resolve(dirname(root), "observation-3"),
      qualificationSourceCommit: "1".repeat(40),
      input,
      supersedePreparationReview: root,
    },
  };
}

export async function journalFailure(t) {
  const f = await preparationReview(t);
  await resetOriginPreflight(f.third, f.operations);
  const root = f.third.privateRoot,
    context = (await readJson(resolve(root, "context.json"))).context,
    inner = context.no_mining_context,
    hash = digest(JSON.stringify(context));
  for (let n = 1; n <= 83; n++) {
    const state = recoveryState(inner);
    if (n < 3) Object.assign(state, { status: "configured", connected: false, deviceBaselineConfirmed: false, deviceLeaseInactive: false });
    await recordRecoveryState(root, inner, state);
    if (n === 3) await saveNoMiningAccounting(root, inner, { stage: "before", ledger: f.ledger, original_budget: f.original, state });
  }
  const start = {
    schema: "fixed-usb-reset-origin-start-v1",
    context_sha256: hash,
    hostMonotonicMs: 100,
    observed_sequence: 3,
    primeObservations: resetDiagnostics(context, 1000),
  };
  await writeNew(resolve(root, "reset-origin-start.json"), start);
  const reducer = createResetOriginObservation({
    firmwareCommit: context.firmware_commit,
    appElfSha256: context.app_elf_sha256,
    minimumSpanMs: 120000,
    maximumGapMs: 6000,
    startedAtHostMonotonicMs: 100,
  });
  let sequence = 0;
  for (let n = 0; n < 446; n++) {
    const values = resetDiagnostics(context, 1000 + n * 293);
    await writeNew(resolve(root, `diagnostic-export-${String(n).padStart(4, "0")}.json`), {
      schema: "fixed-usb-reset-origin-batch-v1",
      context_sha256: hash,
      sequence: n,
      hostMonotonicMs: 100 + n * 293,
      observations: values,
    });
    for (const d of values.filter((d) => (n === 0 ? !["boot", "startup"].includes(d.category) : ["boot", "startup"].includes(d.category))))
      reducer.observe({ sequence: ++sequence, hostMonotonicMs: 100 + n * 293, diagnostic: d });
  }
  await writeNew(resolve(root, "parent-journal-failure.json"), {
    schema: "reset-origin-parent-journal-failure-v1",
    source: "parent-observed",
    browser_closed: true,
    observer: { stage: "failed", started: true, cleanupFailed: true },
    worker: {
      status: "closed",
      connected: false,
      running: false,
      serialOwnershipReleased: true,
      baseline: true,
      inactive: true,
      failure: null,
    },
    causes: ["no_mining_record_rejected", "no_mining_record_rejected"],
    saved_state_rows: 83,
    saved_diagnostic_batches: 446,
    end_record_present: false,
    after_accounting_present: false,
    failed_transport_cause: "not_retained",
  });
  await writeNew(resolve(root, "host-cleanup.json"), {
    schema: "worker-reset-origin-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
  await writeNew(resolve(root, "reset-origin-server-claim.json"), {
    schema: "fixed-usb-reset-origin-server-claim-v1",
    context_sha256: hash,
  });
  await writeNew(resolve(root, "reset-origin-failure.json"), {
    schema: "fixed-usb-reset-origin-failure-v1",
    context_sha256: hash,
    code: "observer_client_failed",
  });
  await writeNew(resolve(root, "failed-inventory.json"), {
    schema: "fixed-usb-reset-origin-journal-failed-inventory-v1",
    outcome: "unverified",
    first_failure: "no_mining_record_rejected",
    failed_transport_cause: "not_retained",
    observer_cleanup_failed: true,
    host_cleanup_confirmed: true,
    parent_observed_worker_closed: true,
    final_closed_state_journal_present: false,
    end_record_present: false,
    after_accounting_observed: false,
    observation_pass: false,
    device_recovery_claimed: false,
    continuation_authorized: false,
    context_sha256: hash,
    artifact_snapshot_sha256: await fileDigest(resolve(root, "artifact-snapshot.json")),
    parent_failure_sha256: await fileDigest(resolve(root, "parent-journal-failure.json")),
    cleanup_sha256: await fileDigest(resolve(root, "host-cleanup.json")),
    before_accounting_sha256: await fileDigest(resolve(root, "no-mining-accounting-before.json")),
    partial_summary_end_basis: "last_saved_batch_not_end_record",
    partial_observation_summary: reducer.finish({ hostMonotonicMs: 100 + 445 * 293 }),
    maximum_batch_gap_ms: 293,
    inventory: await inventory(root),
  });
  f.operations.expectedResetOriginJournalFailureSeal = await fileDigest(resolve(root, "failed-inventory.json"));
  const input = resolve(dirname(root), "journal-correction.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [f.operations.expectedResetOriginJournalFailureSeal, RESET_ORIGIN_JOURNAL_REGRESSION],
  });
  return {
    ...f,
    journalRoot: root,
    journalContext: context,
    fourth: {
      ...f.options,
      privateRoot: resolve(dirname(root), "observation-4"),
      qualificationSourceCommit: "2".repeat(40),
      input,
      supersedeJournalFailure: root,
    },
  };
}

export async function finalizationFailure(t) {
  const f = await journalFailure(t);
  await resetOriginPreflight(f.fourth, f.operations);
  const root = f.fourth.privateRoot,
    saved = await readJson(resolve(root, "context.json")),
    context = saved.context;
  // Represent a previously published observer, not the corrected observer under test.
  context.qualification_driver.client_sha256 = digest("fixture prior finalization observer");
  const hash = digest(JSON.stringify(context)),
    inner = context.no_mining_context;
  await writeFile(resolve(root, "context.json"), JSON.stringify({ context, sha256: hash }));
  const snapshot = await readJson(resolve(root, "artifact-snapshot.json"));
  snapshot.context_sha256 = hash;
  await writeFile(resolve(root, "artifact-snapshot.json"), JSON.stringify(snapshot));
  await writeFile(
    `${f.sourceRoot}.reset-origin-assignment-4.json`,
    JSON.stringify({
      schema: "fixed-usb-reset-origin-assignment-v1",
      context_sha256: hash,
      observation_root: root,
    }),
  );
  for (let n = 1; n <= 118; n++) {
    const state = recoveryState(inner, n > 115);
    if (n < 3) Object.assign(state, { status: "configured", connected: false, deviceBaselineConfirmed: false, deviceLeaseInactive: false });
    if (n === 116 || n === 117) state.status = "closing";
    await recordRecoveryState(root, inner, state);
    if (n === 3 || n === 115)
      await saveNoMiningAccounting(root, inner, {
        stage: n === 3 ? "before" : "after",
        ledger: f.ledger,
        original_budget: f.original,
        state,
      });
  }
  const start = {
    schema: "fixed-usb-reset-origin-start-v1",
    context_sha256: hash,
    hostMonotonicMs: 100,
    observed_sequence: 3,
    primeObservations: resetDiagnostics(context, 1000),
  };
  const end = { schema: "fixed-usb-reset-origin-end-v1", context_sha256: hash, hostMonotonicMs: 130400, observed_sequence: 115 };
  await writeNew(resolve(root, "reset-origin-start.json"), start);
  await writeNew(resolve(root, "reset-origin-end.json"), end);
  for (let n = 0; n < 453; n++)
    await writeNew(resolve(root, `diagnostic-export-${String(n).padStart(4, "0")}.json`), {
      schema: "fixed-usb-reset-origin-batch-v1",
      context_sha256: hash,
      sequence: n,
      hostMonotonicMs: 100 + n * 288,
      observations: resetDiagnostics(context, 1000 + n * 288),
    });
  await writeNew(resolve(root, "parent-review-failure.json"), {
    schema: "reset-origin-parent-review-failure-v1",
    source: "independent-judge-observed",
    first_failure: "reset_origin_journal_order",
    judge_log_sha256: "f88c6b5ce192ad32b663b32b43214a5948d7d372fb8010fe31b332c3f41a7b13",
    start_sequence: 3,
    end_sequence: 115,
    after_accounting_sequence: 115,
    final_sequence: 118,
    final_status: "closed",
    browser_failure_observed: false,
    observation_qualified: false,
    device_recovery_claimed: false,
  });
  await writeNew(resolve(root, "host-cleanup.json"), {
    schema: "worker-reset-origin-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
  await writeNew(resolve(root, "reset-origin-server-claim.json"), {
    schema: "fixed-usb-reset-origin-server-claim-v1",
    context_sha256: hash,
  });
  const seal = {
    schema: "fixed-usb-reset-origin-finalization-failed-inventory-v1",
    outcome: "unverified",
    first_failure: "reset_origin_journal_order",
    browser_failure_observed: false,
    observation_qualified: false,
    device_recovery_claimed: false,
    continuation_authorized: false,
    final_closed_journal: true,
    accounting_values_unchanged: true,
    context_sha256: hash,
    artifact_snapshot_sha256: await fileDigest(resolve(root, "artifact-snapshot.json")),
    parent_failure_sha256: await fileDigest(resolve(root, "parent-review-failure.json")),
    cleanup_sha256: await fileDigest(resolve(root, "host-cleanup.json")),
    before_accounting_sha256: await fileDigest(resolve(root, "no-mining-accounting-before.json")),
    after_accounting_sha256: await fileDigest(resolve(root, "no-mining-accounting-after.json")),
    start_sha256: await fileDigest(resolve(root, "reset-origin-start.json")),
    end_sha256: await fileDigest(resolve(root, "reset-origin-end.json")),
    observation_capture: await inspectResetOriginObservation(root, context, start, end),
    inventory: await inventory(root),
  };
  await writeNew(resolve(root, "failed-inventory.json"), seal);
  f.operations.expectedResetOriginFinalizationSeal = await fileDigest(resolve(root, "failed-inventory.json"));
  const input = resolve(dirname(root), "finalization-correction.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [f.operations.expectedResetOriginFinalizationSeal, RESET_ORIGIN_FINALIZATION_REGRESSION],
  });
  return {
    ...f,
    finalizationRoot: root,
    finalizationContext: context,
    fifth: {
      ...f.options,
      privateRoot: resolve(dirname(root), "observation-5"),
      qualificationSourceCommit: "3".repeat(40),
      input,
      supersedeFinalization: root,
    },
  };
}
