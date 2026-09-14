import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { canonicalDirectory, digest, exactObject, fileDigest, requireCondition as check } from "./contract.mjs";
import { baseline, inventory, proof } from "./cadence-premining-evidence.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";

export const RESET_ORIGIN_PREPARATION_REVIEW_SEAL = "e9d55dad8bb7bc837c5bc3507745cd8aee2be69a27b246a11fb48029f3003080";
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
  "parent-failure-review.json",
  "reset-origin-failure.json",
  "reset-origin-server-claim.json",
  "server-stop-request.json",
  "supervisor.stderr.log",
  "supervisor.stdout.log",
  ...Array.from({ length: 6 }, (_, index) => `no-mining-state-${String(index + 1).padStart(4, "0")}.json`),
]);
const FAILURE = "reset_origin_preparation_receipt_review_required";
const REVIEW = {
  schema: "reset-origin-parent-failure-review-v1",
  source: "parent-observed",
  first_failure: FAILURE,
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
};
async function beforeAccounting(root, context, seal) {
  const states = await readNoMiningStates(root, context.no_mining_context);
  check(
    equal(
      states.map((row) => row.state.status),
      ["configured", "configured", "ready", "closing", "closing", "closed"],
    ) &&
      states.every(
        (row) =>
          !row.state.running && !row.state.failure && row.state.renewalsConfirmed === 0 && !row.state.qualification && !row.state.attempt,
      ),
    "reset_origin_preparation_review_activity",
  );
  baseline(states[2].state, false);
  baseline(states.at(-1).state, true);
  check(
    states
      .slice(2)
      .every(
        (row) =>
          row.state.preservation.baseline_id === states[2].state.preservation.baseline_id &&
          row.state.deviceLeaseInactive &&
          row.state.deviceBaselineConfirmed,
      ),
    "reset_origin_preparation_review_baseline",
  );
  const saved = await proof(resolve(root, "no-mining-accounting-before.json")),
    value = saved.value;
  exactObject(value, ["schema", "context_sha256", "observed_sequence", "stage", "ledger", "original_budget", "state"]);
  check(
    saved.sha256 === seal.before_accounting_sha256 &&
      value.schema === "fixed-usb-no-mining-accounting-v1" &&
      value.context_sha256 === digest(JSON.stringify(context.no_mining_context)) &&
      value.stage === "before" &&
      value.observed_sequence === 3 &&
      equal(value.state, states[2].state),
    "reset_origin_preparation_review_accounting",
  );
  requireIdleLedger(value.ledger, 17, 1380000);
  requireExhaustedOriginal(value.original_budget);
}
/** Recognize only the sealed pre-capture classifier failure; it supplies no observation or accounting-after claim. */
export async function readPreparationReview(root, operations, loadContext) {
  root = await canonicalDirectory(root);
  const saved = await proof(resolve(root, "failed-inventory.json")),
    seal = saved.value;
  check(
    saved.sha256 === (operations.expectedResetOriginPreparationReviewSeal ?? RESET_ORIGIN_PREPARATION_REVIEW_SEAL),
    "reset_origin_preparation_review_anchor",
  );
  check(
    seal.schema === "fixed-usb-reset-origin-preparation-review-failed-inventory-v1" &&
      seal.outcome === "unverified" &&
      seal.first_failure === FAILURE &&
      seal.capture_started === false &&
      seal.observation_pass === false &&
      seal.device_recovery_claimed === false &&
      seal.continuation_authorized === false &&
      seal.after_accounting_observed === false,
    "reset_origin_preparation_review_outcome",
  );
  check(
    (await readdir(root)).every((name) => FILES.has(name)),
    "reset_origin_preparation_review_activity",
  );
  const wrapper = await proof(resolve(root, "context.json"));
  check(
    wrapper.value.context.observation_attempt === 2 && wrapper.value.context.preparation_review_predecessor === undefined,
    "reset_origin_preparation_review_class",
  );
  const context = await loadContext(root, { historical: true, operations }),
    hash = digest(JSON.stringify(context));
  check(
    seal.context_sha256 === hash &&
      seal.artifact_snapshot_sha256 === (await fileDigest(resolve(root, "artifact-snapshot.json"))) &&
      equal(seal.inventory, await inventory(root)),
    "reset_origin_preparation_review_changed",
  );
  const failure = await proof(resolve(root, "parent-failure-review.json")),
    cleanup = await proof(resolve(root, "host-cleanup.json"));
  check(failure.sha256 === seal.parent_failure_sha256 && equal(failure.value, REVIEW), "reset_origin_preparation_review_failure");
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
    "reset_origin_preparation_review_cleanup",
  );
  check(
    equal((await proof(resolve(root, "reset-origin-failure.json"))).value, {
      schema: "fixed-usb-reset-origin-failure-v1",
      context_sha256: hash,
      code: FAILURE,
    }) &&
      equal((await proof(resolve(root, "reset-origin-server-claim.json"))).value, {
        schema: "fixed-usb-reset-origin-server-claim-v1",
        context_sha256: hash,
      }),
    "reset_origin_preparation_review_claim",
  );
  await beforeAccounting(root, context, seal);
  return { context, binding: { root, failed_inventory_sha256: saved.sha256 } };
}
