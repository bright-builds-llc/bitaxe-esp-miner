import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { canonicalDirectory, digest, fileDigest, requireCondition as check } from "./contract.mjs";
import { baseline, inventory, proof } from "./cadence-premining-evidence.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";
import { requireRecordedAccounting } from "./no-mining-accounting.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { inspectResetOriginObservation } from "./reset-origin-observation-review.mjs";

export const RESET_ORIGIN_FINALIZATION_SEAL = "6c00926573200c8175809564cdc6833d5e875ff3a05e02b0aee40b8f0146a311";
export const RESET_ORIGIN_FINALIZATION_REGRESSION = "a79a8b9bc39b947a76b2269a446a46cdadaf5e5354c15caa713ff49b9c594935";
export const RESET_ORIGIN_FINALIZATION_CLIENT = "dc15002f7c9e7dd8bf09bbd75afe61a79d70408961af1f7a1657a0994b1de525";
const FILES = new Set([
  "artifact-snapshot.json",
  "context.json",
  "qualified-artifacts",
  "failed-inventory.json",
  "detector.detect.host-root.json",
  "detector.detect.observer-armed.json",
  "detector.detect.stderr.log",
  "detector.detect.stdout.log",
  "detector.device.private.json",
  "detector.observation.json",
  "host-cleanup.json",
  "no-mining-accounting-before.json",
  "no-mining-accounting-after.json",
  "parent-review-failure.json",
  "reset-origin-server-claim.json",
  "reset-origin-start.json",
  "reset-origin-end.json",
  "server-stop-request.json",
  "supervisor.stderr.log",
  "supervisor.stdout.log",
  ...Array.from({ length: 118 }, (_, n) => `no-mining-state-${String(n + 1).padStart(4, "0")}.json`),
  ...Array.from({ length: 453 }, (_, n) => `diagnostic-export-${String(n).padStart(4, "0")}.json`),
]);
async function provenance(root, seal) {
  const failure = await proof(resolve(root, "parent-review-failure.json")),
    cleanup = await proof(resolve(root, "host-cleanup.json"));
  check(
    failure.sha256 === seal.parent_failure_sha256 &&
      equal(failure.value, {
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
      }),
    "reset_origin_finalization_provenance",
  );
  check(
    cleanup.sha256 === seal.cleanup_sha256 &&
      equal(cleanup.value, {
        schema: "worker-reset-origin-cleanup-v1",
        source: "parent-observed",
        browser_closed: true,
        supervisor_exited: true,
        supervisor_exit_code: 0,
        listener_absent: true,
        owned_children_absent: true,
        serial_holders_absent: true,
      }),
    "reset_origin_finalization_cleanup",
  );
}
async function capturedFailure(root, context, seal) {
  const records = await readNoMiningStates(root, context.no_mining_context);
  check(
    records.length === 118 &&
      records.slice(0, 2).every((row) => row.state.status === "configured") &&
      records.slice(2, 115).every((row) => row.state.status === "ready") &&
      records.slice(115, 117).every((row) => row.state.status === "closing") &&
      records.at(-1).state.status === "closed" &&
      records.every(
        (row) =>
          !row.state.failure &&
          !row.state.serialFailureCategory &&
          !row.state.admissionFailureStage &&
          !row.state.qualification &&
          !row.state.attempt,
      ),
    "reset_origin_finalization_states",
  );
  for (const row of records.slice(2, 115)) baseline(row.state, false);
  baseline(records.at(-1).state, true);
  check(
    records
      .slice(2)
      .every(
        (row) =>
          row.state.preservation.baseline_id === records[2].state.preservation.baseline_id &&
          row.state.deviceBaselineConfirmed &&
          row.state.deviceLeaseInactive,
      ),
    "reset_origin_finalization_baseline",
  );
  const before = await proof(resolve(root, "no-mining-accounting-before.json")),
    after = await proof(resolve(root, "no-mining-accounting-after.json"));
  await requireRecordedAccounting(root, context.no_mining_context, {
    ledger_before: before.value.ledger,
    ledger_after: after.value.ledger,
    original_budget_before: before.value.original_budget,
    original_budget_after: after.value.original_budget,
    recovery_before: before.value.state,
    recovery_after: after.value.state,
  });
  for (const value of [before.value, after.value]) {
    requireIdleLedger(value.ledger, 17, 1380000);
    requireExhaustedOriginal(value.original_budget);
    check(equal(records[value.observed_sequence - 1]?.state, value.state), "reset_origin_finalization_accounting_state");
  }
  check(
    before.sha256 === seal.before_accounting_sha256 &&
      after.sha256 === seal.after_accounting_sha256 &&
      equal(before.value.ledger, after.value.ledger) &&
      equal(before.value.original_budget, after.value.original_budget),
    "reset_origin_finalization_accounting",
  );
  const start = await proof(resolve(root, "reset-origin-start.json")),
    end = await proof(resolve(root, "reset-origin-end.json"));
  check(
    start.sha256 === seal.start_sha256 &&
      end.sha256 === seal.end_sha256 &&
      start.value.observed_sequence === 3 &&
      before.value.observed_sequence === 3 &&
      end.value.observed_sequence === 115 &&
      after.value.observed_sequence === 115,
    "reset_origin_finalization_exact_failure",
  );
  check(
    equal(seal.observation_capture, await inspectResetOriginObservation(root, context, start.value, end.value)),
    "reset_origin_finalization_capture",
  );
}
/** Recognize the exact completed capture rejected for end/after sequence equality, without relaxing that rejection. */
export async function readFinalizationFailure(root, operations, loadContext) {
  root = await canonicalDirectory(root);
  const saved = await proof(resolve(root, "failed-inventory.json")),
    seal = saved.value;
  check(
    saved.sha256 === (operations.expectedResetOriginFinalizationSeal ?? RESET_ORIGIN_FINALIZATION_SEAL),
    "reset_origin_finalization_anchor",
  );
  check(
    seal.schema === "fixed-usb-reset-origin-finalization-failed-inventory-v1" &&
      seal.outcome === "unverified" &&
      seal.first_failure === "reset_origin_journal_order" &&
      seal.final_closed_journal === true &&
      seal.accounting_values_unchanged === true &&
      ["browser_failure_observed", "observation_qualified", "device_recovery_claimed", "continuation_authorized"].every(
        (key) => seal[key] === false,
      ),
    "reset_origin_finalization_outcome",
  );
  check(
    (await readdir(root)).every((name) => FILES.has(name)),
    "reset_origin_finalization_activity",
  );
  const wrapper = await proof(resolve(root, "context.json"));
  check(
    wrapper.value.context.observation_attempt === 4 && wrapper.value.context.finalization_predecessor === undefined,
    "reset_origin_finalization_class",
  );
  const context = await loadContext(root, { historical: true, operations }),
    hash = digest(JSON.stringify(context));
  check(
    seal.context_sha256 === hash &&
      seal.artifact_snapshot_sha256 === (await fileDigest(resolve(root, "artifact-snapshot.json"))) &&
      equal(seal.inventory, await inventory(root)),
    "reset_origin_finalization_changed",
  );
  check(
    equal((await proof(resolve(root, "reset-origin-server-claim.json"))).value, {
      schema: "fixed-usb-reset-origin-server-claim-v1",
      context_sha256: hash,
    }),
    "reset_origin_finalization_claim",
  );
  await provenance(root, seal);
  await capturedFailure(root, context, seal);
  return { context, binding: { root, failed_inventory_sha256: saved.sha256 } };
}
