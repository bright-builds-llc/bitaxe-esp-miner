import { readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, fileDigest, missing, protectedPath, readJson, requireCondition as check, writeNew } from "./contract.mjs";
import { inventory, proof, baseline } from "./cadence-premining-evidence.mjs";
import { loadRestartContext, restartInnerContext } from "./reset-origin-restart-context.mjs";
import { requireRestartInstallation } from "./reset-origin-restart-install.mjs";
import { readRestartStates, readRestartAccounting } from "./reset-origin-restart-state.mjs";
import { inspectResetOriginObservation } from "./reset-origin-observation-review.mjs";
import { saveRestartFailure } from "./reset-origin-restart-failure.mjs";
import { validateRestartEvidence } from "./reset-origin-restart-evidence.mjs";

function cleanup(value) {
  exactObject(value, [
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
    value.schema === "worker-restart-host-cleanup-v1" &&
      value.source === "parent-observed" &&
      value.supervisor_exit_code === 0 &&
      ["browser_closed", "supervisor_exited", "listener_absent", "owned_children_absent", "serial_holders_absent"].every(
        (key) => value[key] === true,
      ),
    "restart_host_cleanup",
  );
}
async function scopes(root, context) {
  const priorRoot = resolve(root, "before-install"),
    afterRoot = resolve(root, "after-install");
  const priorContext = restartInnerContext(root, context, "before-install"),
    afterContext = restartInnerContext(root, context, "after-install");
  const oldRecords = await readRestartStates(priorRoot, priorContext),
    records = await readRestartStates(afterRoot, afterContext);
  const preInstall = await readRestartAccounting(priorRoot, priorContext, "before"),
    before = await readRestartAccounting(afterRoot, afterContext, "before"),
    after = await readRestartAccounting(afterRoot, afterContext, "after");
  const baselineId = preInstall.state.preservation.baseline_id;
  for (const value of [before, after])
    check(
      equal(preInstall.ledger, value.ledger) &&
        equal(preInstall.original_budget, value.original_budget) &&
        value.state.preservation.baseline_id === baselineId,
      "restart_preservation_accounting_changed",
    );
  baseline(oldRecords.at(-1).state, true);
  baseline(records.at(-1).state, true);
  check(
    oldRecords.at(-1).sequence > preInstall.observed_sequence &&
      records.at(-1).sequence > after.observed_sequence &&
      oldRecords.at(-1).state.preservation.baseline_id === baselineId &&
      records.at(-1).state.preservation.baseline_id === baselineId,
    "restart_closed_journal_order",
  );
  check(
    [...oldRecords, ...records].every(
      (row) =>
        !row.state.failure && !row.state.serialFailureCategory && !row.state.admissionFailureStage && !row.state.ownerResourceFailure,
    ),
    "restart_recorded_failure",
  );
  return { oldRecords, records, preInstall, before, after, baselineId, afterRoot };
}
async function evidence(root, context, hostCleanup) {
  cleanup(hostCleanup);
  const hash = digest(JSON.stringify(context));
  for (const name of [
    "issued.json",
    "consumed.json",
    "restart-failure.json",
    "restart-failed-observation.json",
    "failed-inventory.json",
    "iterative.fault.json",
  ])
    await missing(resolve(root, name));
  check(!(await readdir(root)).some((name) => /^(?:cycle|flash)-[0-9]/u.test(name)), "restart_unexpected_effect");
  for (const [name, schema] of [
    ["restart-server-claim.json", "fixed-usb-restart-server-claim-v1"],
    ["install-phase-advanced.json", "fixed-usb-restart-phase-v1"],
  ])
    check(equal((await proof(resolve(root, name))).value, { schema, context_sha256: hash }), "restart_phase_claim");
  const installation = await requireRestartInstallation(root, context),
    s = await scopes(root, context);
  const start = (await proof(resolve(s.afterRoot, "reset-origin-start.json"))).value,
    end = (await proof(resolve(s.afterRoot, "reset-origin-end.json"))).value;
  const observed = await inspectResetOriginObservation(s.afterRoot, context, start, end);
  check(
    observed.summary.initialBootOrdinal === installation.startup.boot_ordinal &&
      observed.summary.initialResetReason === installation.startup.initial_reset_category,
    "restart_install_boot_changed",
  );
  const review = (await proof(resolve(root, "pre-restart-observation.json"))).value;
  check(
    equal(review, { schema: "fixed-usb-pre-restart-observation-v1", context_sha256: hash, observed }),
    "restart_preobservation_changed",
  );
  const claimProof = await proof(resolve(root, "restart-consumed.json")),
    claim = claimProof.value;
  exactObject(claim, [
    "schema",
    "context_sha256",
    "request_nonce_sha256",
    "expected_boot_ordinal",
    "before_sequence",
    "baseline_id",
    "hostMonotonicMs",
  ]);
  check(
    claim.schema === "fixed-usb-restart-consumed-v1" &&
      claim.context_sha256 === hash &&
      claim.request_nonce_sha256 === digest(context.request_nonce) &&
      claim.expected_boot_ordinal === observed.summary.initialBootOrdinal &&
      claim.baseline_id === s.baselineId &&
      Number.isSafeInteger(claim.before_sequence) &&
      claim.before_sequence > end.observed_sequence &&
      Number.isSafeInteger(claim.hostMonotonicMs) &&
      claim.hostMonotonicMs >= end.hostMonotonicMs,
    "restart_claim_binding",
  );
  check(
    start.observed_sequence === s.before.observed_sequence &&
      end.observed_sequence >= start.observed_sequence &&
      claim.before_sequence <= s.records.length,
    "restart_preobservation_order",
  );
  check(
    [...s.oldRecords, ...s.records.slice(0, claim.before_sequence)].every(
      (row) => row.state.restart === undefined && row.state.status !== "restarting",
    ),
    "restart_unclaimed_state",
  );
  for (const row of s.records.slice(s.before.observed_sequence - 1, claim.before_sequence)) {
    baseline(row.state, false);
    check(row.state.preservation.baseline_id === s.baselineId, "restart_preobservation_baseline");
  }
  const captured = (await proof(resolve(root, "restart-observation.json"))).value;
  exactObject(captured, ["schema", "context_sha256", "claim_sha256", "after_sequence", "hostMonotonicMs", "evidence"]);
  check(
    captured.schema === "fixed-usb-restart-observation-v1" &&
      captured.context_sha256 === hash &&
      captured.claim_sha256 === claimProof.sha256 &&
      Number.isSafeInteger(captured.after_sequence) &&
      captured.after_sequence > claim.before_sequence &&
      captured.after_sequence < s.after.observed_sequence &&
      Number.isSafeInteger(captured.hostMonotonicMs) &&
      captured.hostMonotonicMs >= claim.hostMonotonicMs &&
      captured.hostMonotonicMs - claim.hostMonotonicMs <= 30000,
    "restart_result_order",
  );
  const transition = validateRestartEvidence(captured.evidence, context, claim.expected_boot_ordinal);
  check(equal(transition, captured.evidence), "restart_unsanitized_evidence");
  const ready = s.records[captured.after_sequence - 1]?.state;
  baseline(ready, false);
  check(ready.preservation.baseline_id === s.baselineId && equal(ready.restart, transition.summary), "restart_authenticated_state_missing");
  const finished = (await proof(resolve(root, "restart-finished.json"))).value;
  check(
    equal(finished, { schema: "fixed-usb-restart-finished-v1", context_sha256: hash, final_sequence: s.records.at(-1).sequence }),
    "restart_finished_journal",
  );
  return {
    installation,
    observation: observed,
    transition,
    ledger: s.after.ledger,
    original_budget: s.after.original_budget,
    final_state: s.records.at(-1).state,
    final_sequence: s.records.at(-1).sequence,
    pre_install_accounting_sha256: await fileDigest(resolve(root, "before-install/no-mining-accounting-before.json")),
    before_accounting_sha256: await fileDigest(resolve(root, "after-install/no-mining-accounting-before.json")),
    after_accounting_sha256: await fileDigest(resolve(root, "after-install/no-mining-accounting-after.json")),
    claim_sha256: claimProof.sha256,
  };
}
function result(e) {
  return {
    result: "controlled_restart_verified",
    device_recovered: true,
    prior_reset_attribution: "unknown",
    qualification_pass: false,
    mining_authorized: false,
    cadence_admission_authorized: false,
    host_span_ms: e.observation.summary.hostSpanMs,
    restart_duration_ms: e.transition.summary.durationMs,
    continuity: e.transition.summary.continuity,
    next_ordinal: e.ledger.next_ordinal,
    total_charged_ms: e.ledger.total_charged_ms,
    cleanup_confirmed: true,
  };
}
export async function judgeRestart(root, inputPath, operations = {}) {
  root = resolve(root);
  const context = await loadRestartContext(root, { operations });
  await protectedPath(inputPath);
  const hostCleanup = await readJson(inputPath);
  let e;
  try {
    e = await evidence(root, context, hostCleanup);
  } catch (error) {
    await saveRestartFailure(root, context, error);
    throw error;
  }
  const receipt = {
    schema: "fixed-usb-reset-origin-restart-result-v1",
    context,
    context_sha256: digest(JSON.stringify(context)),
    ...result(e),
    ...e,
    cleanup: hostCleanup,
    cleanup_input: { path: resolve(inputPath), sha256: await fileDigest(inputPath) },
    inventory: await inventory(root),
  };
  await writeNew(resolve(root, "result.json"), { receipt, sha256: digest(JSON.stringify(receipt)) });
  return result(e);
}
export async function readRestartResult(path, operations = {}) {
  path = resolve(path);
  const root = dirname(path);
  check(path === resolve(root, "result.json"), "restart_result_path");
  const saved = (await proof(path)).value;
  exactObject(saved, ["receipt", "sha256"]);
  const r = saved.receipt;
  check(r.schema === "fixed-usb-reset-origin-restart-result-v1" && saved.sha256 === digest(JSON.stringify(r)), "restart_result_integrity");
  const context = await loadRestartContext(root, { historical: true, operations });
  check(equal(context, r.context) && r.context_sha256 === digest(JSON.stringify(context)), "restart_result_context");
  await protectedPath(r.cleanup_input.path);
  check(
    (await fileDigest(r.cleanup_input.path)) === r.cleanup_input.sha256 && equal(await readJson(r.cleanup_input.path), r.cleanup),
    "restart_cleanup_changed",
  );
  const e = await evidence(root, context, r.cleanup);
  check(
    equal(r, {
      schema: "fixed-usb-reset-origin-restart-result-v1",
      context,
      context_sha256: digest(JSON.stringify(context)),
      ...result(e),
      ...e,
      cleanup: r.cleanup,
      cleanup_input: r.cleanup_input,
      inventory: (await inventory(root)).filter((row) => row.path !== "result.json"),
    }),
    "restart_result_changed",
  );
  return r;
}
