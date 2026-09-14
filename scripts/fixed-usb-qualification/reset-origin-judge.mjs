import { readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, fileDigest, missing, protectedPath, readJson, requireCondition as check, writeNew } from "./contract.mjs";
import { inventory, proof } from "./cadence-premining-evidence.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";
import { requireRecordedAccounting } from "./no-mining-accounting.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { inspectResetOriginObservation } from "./reset-origin-observation-review.mjs";
import { loadResetOriginContext } from "./reset-origin-context.mjs";

const integer = (value) => Number.isSafeInteger(value) && value >= 0;
function baseline(state, released = false) {
  check(
    state.status === (released ? "closed" : "ready") &&
      state.connected === !released &&
      state.serialOwnershipReleased === released &&
      !state.running &&
      !state.failure &&
      state.deviceBaselineConfirmed === true &&
      state.deviceLeaseInactive &&
      state.renewalsConfirmed === 0 &&
      state.preservation?.device_identity_match === true &&
      state.preservation.settings_match === true &&
      state.preservation.authorization_high_water_match === true &&
      state.preservation.mine_on_boot === false,
    "reset_origin_baseline",
  );
}
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
    value.schema === "worker-reset-origin-cleanup-v1" &&
      value.source === "parent-observed" &&
      value.supervisor_exit_code === 0 &&
      ["browser_closed", "supervisor_exited", "listener_absent", "owned_children_absent", "serial_holders_absent"].every(
        (key) => value[key] === true,
      ),
    "reset_origin_cleanup",
  );
}
async function evidence(root, context, hostCleanup) {
  cleanup(hostCleanup);
  const serverClaim = await proof(resolve(root, "reset-origin-server-claim.json"));
  check(
    equal(serverClaim.value, { schema: "fixed-usb-reset-origin-server-claim-v1", context_sha256: digest(JSON.stringify(context)) }),
    "reset_origin_server_claim",
  );
  for (const name of [
    "issued.json",
    "consumed.json",
    "install-consumed.json",
    "iterative.fault.json",
    "no-mining-read-only-interruption.json",
    "restart-consumed.json",
    "reset-origin-failure.json",
    "failed-inventory.json",
  ])
    await missing(resolve(root, name));
  check(!(await readdir(root)).some((name) => /^(?:flash|install|cycle)-[0-9]/u.test(name)), "reset_origin_effect_evidence_forbidden");
  const inner = context.no_mining_context,
    states = await readNoMiningStates(root, inner);
  const before = (await proof(resolve(root, "no-mining-accounting-before.json"))).value,
    after = (await proof(resolve(root, "no-mining-accounting-after.json"))).value;
  await requireRecordedAccounting(root, inner, {
    ledger_before: before.ledger,
    ledger_after: after.ledger,
    original_budget_before: before.original_budget,
    original_budget_after: after.original_budget,
    recovery_before: before.state,
    recovery_after: after.state,
  });
  for (const value of [before, after]) {
    requireIdleLedger(value.ledger, 17, 1380000);
    requireExhaustedOriginal(value.original_budget);
    baseline(value.state);
    check(equal(states[value.observed_sequence - 1]?.state, value.state), "reset_origin_accounting_state");
  }
  const startFile = await proof(resolve(root, "reset-origin-start.json")),
    endFile = await proof(resolve(root, "reset-origin-end.json")),
    start = startFile.value,
    end = endFile.value;
  check(
    start.observed_sequence === before.observed_sequence &&
      integer(end.observed_sequence) &&
      end.observed_sequence >= start.observed_sequence &&
      end.observed_sequence < after.observed_sequence &&
      after.observed_sequence < states.at(-1).sequence,
    "reset_origin_journal_order",
  );
  baseline(states[start.observed_sequence - 1]?.state);
  baseline(states[end.observed_sequence - 1]?.state);
  baseline(states.at(-1).state, true);
  const baselineId = before.state.preservation.baseline_id;
  check(
    states.every(
      ({ state }) =>
        !state.failure &&
        !state.serialFailureCategory &&
        !state.admissionFailureStage &&
        (state.qualification?.attempt === undefined || state.qualification.attempt.ordinal < 17),
    ) &&
      states
        .slice(start.observed_sequence - 1, after.observed_sequence)
        .every(
          ({ state }) =>
            state.connected &&
            !state.serialOwnershipReleased &&
            !state.running &&
            state.deviceLeaseInactive &&
            state.deviceBaselineConfirmed === true &&
            state.preservation?.baseline_id === baselineId,
        ) &&
      after.state.preservation.baseline_id === baselineId &&
      states.at(-1).state.preservation.baseline_id === baselineId,
    "reset_origin_session_changed",
  );
  return {
    ...(await inspectResetOriginObservation(root, context, start, end)),
    ledger: after.ledger,
    original_budget: after.original_budget,
    final_state: states.at(-1).state,
    before_accounting_sha256: await fileDigest(resolve(root, "no-mining-accounting-before.json")),
    after_accounting_sha256: await fileDigest(resolve(root, "no-mining-accounting-after.json")),
    start_sha256: startFile.sha256,
    end_sha256: endFile.sha256,
    final_sequence: states.at(-1).sequence,
  };
}
function result(e) {
  return {
    result: "observed_stable",
    observation_qualified: true,
    reset_origin_resolved: false,
    prior_reset_attribution: "unknown",
    restart_performed: false,
    qualification_pass: false,
    mining_authorized: false,
    cadence_admission_authorized: false,
    host_span_ms: e.summary.hostSpanMs,
    boot_advances: e.summary.bootAdvances,
    startup_advances: e.summary.healthyStartupAdvances,
    next_ordinal: e.ledger.next_ordinal,
    total_charged_ms: e.ledger.total_charged_ms,
  };
}
export async function judgeResetOrigin(root, inputPath, operations = {}) {
  root = resolve(root);
  const context = await loadResetOriginContext(root, { operations });
  await protectedPath(inputPath);
  const hostCleanup = await readJson(inputPath);
  const observed = await evidence(root, context, hostCleanup);
  const receipt = {
    schema: "fixed-usb-reset-origin-result-v1",
    context,
    context_sha256: digest(JSON.stringify(context)),
    ...result(observed),
    ...observed,
    cleanup: hostCleanup,
    cleanup_input: { path: resolve(inputPath), sha256: await fileDigest(inputPath) },
    inventory: (await inventory(root)).filter((row) => row.path !== "result.json"),
  };
  await writeNew(resolve(root, "result.json"), { receipt, sha256: digest(JSON.stringify(receipt)) });
  return result(observed);
}
export async function readResetOrigin(path, operations = {}) {
  path = resolve(path);
  const root = dirname(path);
  check(path === resolve(root, "result.json"), "reset_origin_result_path");
  const saved = await proof(path);
  exactObject(saved.value, ["receipt", "sha256"]);
  const receipt = saved.value.receipt;
  check(
    saved.value.sha256 === digest(JSON.stringify(receipt)) && receipt.schema === "fixed-usb-reset-origin-result-v1",
    "reset_origin_result_integrity",
  );
  const context = await loadResetOriginContext(root, { historical: true, operations });
  check(receipt.context_sha256 === digest(JSON.stringify(context)) && equal(context, receipt.context), "reset_origin_result_context");
  await protectedPath(receipt.cleanup_input.path);
  check(
    (await fileDigest(receipt.cleanup_input.path)) === receipt.cleanup_input.sha256 &&
      equal(await readJson(receipt.cleanup_input.path), receipt.cleanup),
    "reset_origin_cleanup_changed",
  );
  const observed = await evidence(root, context, receipt.cleanup);
  const expected = {
    schema: "fixed-usb-reset-origin-result-v1",
    context,
    context_sha256: digest(JSON.stringify(context)),
    ...result(observed),
    ...observed,
    cleanup: receipt.cleanup,
    cleanup_input: receipt.cleanup_input,
    inventory: (await inventory(root)).filter((row) => row.path !== "result.json"),
  };
  check(equal(receipt, expected), "reset_origin_result_changed");
  return receipt;
}
