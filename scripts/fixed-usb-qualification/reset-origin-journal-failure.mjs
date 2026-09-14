import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { canonicalDirectory, digest, exactObject, fileDigest, requireCondition as check } from "./contract.mjs";
import { baseline, inventory, proof } from "./cadence-premining-evidence.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { createResetOriginObservation, parseResetOriginDiagnostic } from "./reset-origin-observation.mjs";

export const RESET_ORIGIN_JOURNAL_FAILURE_SEAL = "bfa2376b1c58af97100b83755cc0964a321ef64801d3902a738a15f9a7fe2944";
export const RESET_ORIGIN_JOURNAL_REGRESSION = "fc107819ea5ca2b94114d6e485cc207df8c1067da73bde930b7c42f7ef4c0906";
export const RESET_ORIGIN_CORRECTED_JOURNAL_CLIENT = "2409804b08f6aa903d38e259baf527a7164c8329cd1bc43724f5632ce9cfa09b";
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
  "parent-journal-failure.json",
  "reset-origin-failure.json",
  "reset-origin-server-claim.json",
  "reset-origin-start.json",
  "server-stop-request.json",
  "supervisor.stderr.log",
  "supervisor.stdout.log",
  ...Array.from({ length: 83 }, (_, n) => `no-mining-state-${String(n + 1).padStart(4, "0")}.json`),
  ...Array.from({ length: 446 }, (_, n) => `diagnostic-export-${String(n).padStart(4, "0")}.json`),
]);
async function journal(root, context, seal) {
  const records = await readNoMiningStates(root, context.no_mining_context);
  check(
    records.length === 83 &&
      records.slice(0, 2).every((row) => row.state.status === "configured") &&
      records.slice(2).every((row) => row.state.status === "ready") &&
      records.every((row) => !row.state.failure && !row.state.qualification && !row.state.attempt),
    "reset_origin_journal_failure_states",
  );
  for (const row of records.slice(2)) {
    baseline(row.state, false);
    check(row.state.preservation.baseline_id === records[2].state.preservation.baseline_id, "reset_origin_journal_failure_baseline");
  }
  const saved = await proof(resolve(root, "no-mining-accounting-before.json")),
    value = saved.value;
  exactObject(value, ["schema", "context_sha256", "observed_sequence", "stage", "ledger", "original_budget", "state"]);
  check(
    saved.sha256 === seal.before_accounting_sha256 &&
      value.schema === "fixed-usb-no-mining-accounting-v1" &&
      value.context_sha256 === digest(JSON.stringify(context.no_mining_context)) &&
      value.stage === "before" &&
      value.observed_sequence === 3 &&
      equal(value.state, records[2].state),
    "reset_origin_journal_failure_accounting",
  );
  requireIdleLedger(value.ledger, 17, 1380000);
  requireExhaustedOriginal(value.original_budget);
}
async function partialCapture(root, context, seal) {
  const start = (await proof(resolve(root, "reset-origin-start.json"))).value,
    hash = digest(JSON.stringify(context));
  exactObject(start, ["schema", "context_sha256", "hostMonotonicMs", "observed_sequence", "primeObservations"]);
  check(
    start.schema === "fixed-usb-reset-origin-start-v1" &&
      start.context_sha256 === hash &&
      start.observed_sequence === 3 &&
      Number.isSafeInteger(start.hostMonotonicMs) &&
      start.hostMonotonicMs > 0,
    "reset_origin_journal_failure_start",
  );
  const reducer = createResetOriginObservation({
    firmwareCommit: context.firmware_commit,
    appElfSha256: context.app_elf_sha256,
    minimumSpanMs: 120000,
    maximumGapMs: 6000,
    startedAtHostMonotonicMs: start.hostMonotonicMs,
  });
  const key = (d) => `${d.category}:${d.stage ?? d.origin ?? ""}`,
    last = new Map();
  let sequence = 0,
    lastTime = start.hostMonotonicMs,
    maximumGap = 0;
  for (let n = 0; n < 446; n++) {
    const batch = (await proof(resolve(root, `diagnostic-export-${String(n).padStart(4, "0")}.json`))).value;
    exactObject(batch, ["schema", "context_sha256", "sequence", "hostMonotonicMs", "observations"]);
    check(
      batch.schema === "fixed-usb-reset-origin-batch-v1" &&
        batch.context_sha256 === hash &&
        batch.sequence === n &&
        Number.isSafeInteger(batch.hostMonotonicMs) &&
        Array.isArray(batch.observations) &&
        batch.observations.length <= 40,
      "reset_origin_journal_failure_batch",
    );
    const values = batch.observations.map(parseResetOriginDiagnostic);
    check(new Set(values.map(key)).size === values.length, "reset_origin_journal_failure_duplicate");
    if (n === 0) {
      check(
        equal(values, start.primeObservations) &&
          batch.hostMonotonicMs <= start.hostMonotonicMs &&
          start.hostMonotonicMs - batch.hostMonotonicMs <= 6000,
        "reset_origin_journal_failure_prime",
      );
      for (const d of values) {
        last.set(key(d), d);
        if (!["boot", "startup"].includes(d.category))
          reducer.observe({ sequence: ++sequence, hostMonotonicMs: start.hostMonotonicMs, diagnostic: d });
      }
      continue;
    }
    check(
      batch.hostMonotonicMs >= lastTime && batch.hostMonotonicMs - start.hostMonotonicMs <= 135000,
      "reset_origin_journal_failure_clock",
    );
    maximumGap = Math.max(maximumGap, batch.hostMonotonicMs - lastTime);
    lastTime = batch.hostMonotonicMs;
    for (const d of values.sort((a, b) => Number(b.category === "boot") - Number(a.category === "boot"))) {
      if (equal(last.get(key(d)), d)) continue;
      last.set(key(d), d);
      reducer.observe({ sequence: ++sequence, hostMonotonicMs: batch.hostMonotonicMs, diagnostic: d });
    }
  }
  check(
    seal.partial_summary_end_basis === "last_saved_batch_not_end_record" &&
      equal(seal.partial_observation_summary, reducer.finish({ hostMonotonicMs: lastTime })) &&
      seal.maximum_batch_gap_ms === maximumGap,
    "reset_origin_journal_failure_partial_evidence",
  );
}
async function closure(root, seal) {
  const failure = await proof(resolve(root, "parent-journal-failure.json")),
    cleanup = await proof(resolve(root, "host-cleanup.json"));
  check(
    failure.sha256 === seal.parent_failure_sha256 &&
      equal(failure.value, {
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
      }),
    "reset_origin_journal_failure_provenance",
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
    "reset_origin_journal_failure_cleanup",
  );
}
/** Revalidate the exact failed capture without inventing an end, after-ledger, or closed journal row. */
export async function readJournalFailure(root, operations, loadContext) {
  root = await canonicalDirectory(root);
  const saved = await proof(resolve(root, "failed-inventory.json")),
    seal = saved.value;
  check(
    saved.sha256 === (operations.expectedResetOriginJournalFailureSeal ?? RESET_ORIGIN_JOURNAL_FAILURE_SEAL),
    "reset_origin_journal_failure_anchor",
  );
  check(
    seal.schema === "fixed-usb-reset-origin-journal-failed-inventory-v1" &&
      seal.outcome === "unverified" &&
      seal.first_failure === "no_mining_record_rejected" &&
      seal.failed_transport_cause === "not_retained" &&
      seal.observer_cleanup_failed === true &&
      seal.host_cleanup_confirmed === true &&
      seal.parent_observed_worker_closed === true &&
      [
        "final_closed_state_journal_present",
        "end_record_present",
        "after_accounting_observed",
        "observation_pass",
        "device_recovery_claimed",
        "continuation_authorized",
      ].every((key) => seal[key] === false),
    "reset_origin_journal_failure_outcome",
  );
  check(
    (await readdir(root)).every((name) => FILES.has(name)),
    "reset_origin_journal_failure_activity",
  );
  const wrapper = await proof(resolve(root, "context.json"));
  check(
    wrapper.value.context.observation_attempt === 3 && wrapper.value.context.journal_failure_predecessor === undefined,
    "reset_origin_journal_failure_class",
  );
  const context = await loadContext(root, { historical: true, operations }),
    hash = digest(JSON.stringify(context));
  check(
    seal.context_sha256 === hash &&
      seal.artifact_snapshot_sha256 === (await fileDigest(resolve(root, "artifact-snapshot.json"))) &&
      equal(seal.inventory, await inventory(root)),
    "reset_origin_journal_failure_changed",
  );
  check(
    equal((await proof(resolve(root, "reset-origin-failure.json"))).value, {
      schema: "fixed-usb-reset-origin-failure-v1",
      context_sha256: hash,
      code: "observer_client_failed",
    }) &&
      equal((await proof(resolve(root, "reset-origin-server-claim.json"))).value, {
        schema: "fixed-usb-reset-origin-server-claim-v1",
        context_sha256: hash,
      }),
    "reset_origin_journal_failure_claim",
  );
  await closure(root, seal);
  await journal(root, context, seal);
  await partialCapture(root, context, seal);
  return { context, binding: { root, failed_inventory_sha256: saved.sha256 } };
}
