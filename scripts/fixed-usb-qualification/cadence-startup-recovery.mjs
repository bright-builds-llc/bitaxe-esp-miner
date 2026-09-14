import { readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import {
  BUNDLE,
  digest,
  exactObject,
  fileDigest,
  missing,
  protectedPath,
  QualificationError,
  readJson,
  requireCondition,
  writeNew,
} from "./contract.mjs";
import { createNoMiningSupervisor } from "./no-mining-server.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";
import { requireRecordedAccounting } from "./no-mining-accounting.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { inventory, proof } from "./cadence-premining-evidence.mjs";
import { send } from "./http.mjs";
import { forbidStartupCredentials, loadStartupRecoveryContext } from "./cadence-startup-context.mjs";
import { requireStartupInstallation } from "./cadence-startup-install.mjs";
export { startupRecoveryPreflight, loadStartupRecoveryContext } from "./cadence-startup-context.mjs";
export { consumeStartupRecoveryInstall } from "./cadence-startup-install.mjs";

/** The legacy no-mining transport runs only inside this explicitly bound recovery authority. */
export async function createStartupRecoverySupervisor(options, operations = {}) {
  forbidStartupCredentials(options);
  requireCondition(options.context === undefined, "startup_context_injection");
  const root = resolve(options.privateRoot),
    context = await loadStartupRecoveryContext(root, { operations });
  const verify = async () => {
    requireCondition(isDeepStrictEqual(context, await loadStartupRecoveryContext(root, { operations })), "startup_context_changed");
    await requireStartupInstallation(root, context);
  };
  const server = await createNoMiningSupervisor(
    { privateRoot: root, context: context.no_mining_context, bun: options.bun },
    { verifyFrozen: verify },
  );
  const [handler] = server.listeners("request");
  server.removeListener("request", handler);
  const get = ["/", "/context", "/supervisor-state", "/no-mining-client.mjs", `/${context.gate_page_relative_path}`, `/${BUNDLE}`];
  const post = ["/activate", "/original-budget-context", "/record", "/accounting"];
  server.on("request", async (request, response) => {
    try {
      const path = new URL(request.url, "http://127.0.0.1").pathname;
      if (!(request.method === "GET" ? get : request.method === "POST" ? post : []).includes(path))
        return send(response, 404, { error: "route_unavailable" });
      handler(request, response);
    } catch (error) {
      if (response.headersSent) return response.destroy();
      send(response, 400, { error: error instanceof QualificationError ? error.code : "local_operation_failed" });
    }
  });
  return server;
}
function baseline(state, released) {
  requireCondition(
    state.status === (released ? "closed" : "ready") &&
      state.connected === !released &&
      !state.running &&
      !state.failure &&
      state.renewalsConfirmed === 0 &&
      state.deviceLeaseInactive &&
      state.deviceBaselineConfirmed === true &&
      state.serialOwnershipReleased === released &&
      state.preservation?.device_identity_match === true &&
      state.preservation.settings_match === true &&
      state.preservation.authorization_high_water_match === true &&
      state.preservation.mine_on_boot === false,
    "startup_recovery_baseline",
  );
}
function validateCleanup(value) {
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
  requireCondition(
    value.schema === "worker-cadence-startup-recovery-cleanup-v1" &&
      value.source === "parent-observed" &&
      value.supervisor_exit_code === 0 &&
      ["browser_closed", "supervisor_exited", "listener_absent", "owned_children_absent", "serial_holders_absent"].every(
        (key) => value[key] === true,
      ),
    "startup_recovery_host_cleanup",
  );
}
async function recoveryEvidence(root, context, cleanup) {
  validateCleanup(cleanup);
  for (const name of [
    "issued.json",
    "consumed.json",
    "iterative.samples.jsonl",
    "iterative.fault.json",
    "cadence-idle-arm.json",
    "cadence-usb-arm.json",
    "cadence-mining-arm.json",
    "no-mining-read-only-interruption.json",
    "failed-inventory.json",
    "recovery-failure.json",
  ])
    await missing(resolve(root, name));
  requireCondition(!(await readdir(root)).some((name) => /^cycle-[1-4]\./u.test(name)), "startup_recovery_cycles_forbidden");
  const installation = await requireStartupInstallation(root, context),
    inner = context.no_mining_context;
  const records = await readNoMiningStates(root, inner);
  const before = (await proof(resolve(root, "no-mining-accounting-before.json"))).value;
  const after = (await proof(resolve(root, "no-mining-accounting-after.json"))).value;
  await requireRecordedAccounting(root, inner, {
    ledger_before: before.ledger,
    ledger_after: after.ledger,
    original_budget_before: before.original_budget,
    original_budget_after: after.original_budget,
    recovery_before: before.state,
    recovery_after: after.state,
  });
  for (const accounting of [before, after]) {
    requireIdleLedger(accounting.ledger, 17, 1380000);
    requireExhaustedOriginal(accounting.original_budget);
    baseline(accounting.state, false);
    requireCondition(isDeepStrictEqual(accounting.state, records[accounting.observed_sequence - 1]?.state), "startup_accounting_journal");
  }
  const final = records.at(-1);
  baseline(final.state, true);
  requireCondition(
    before.observed_sequence < after.observed_sequence &&
      after.observed_sequence < final.sequence &&
      before.state.preservation.baseline_id === after.state.preservation.baseline_id &&
      before.state.preservation.baseline_id === final.state.preservation.baseline_id &&
      records
        .slice(before.observed_sequence - 1, after.observed_sequence)
        .every((row) => row.state.connected && !row.state.serialOwnershipReleased),
    "startup_recovery_session",
  );
  requireCondition(
    records.every(
      ({ state }) =>
        !state.failure &&
        !state.serialFailureCategory &&
        !state.admissionFailureStage &&
        (state.qualification?.attempt === undefined || state.qualification.attempt.ordinal < 17),
    ),
    "startup_recovery_activity",
  );
  return {
    installation,
    ledger: after.ledger,
    original_budget: after.original_budget,
    accounting_before_sequence: before.observed_sequence,
    accounting_after_sequence: after.observed_sequence,
    final_sequence: final.sequence,
    final_state: final.state,
    accounting_before_sha256: await fileDigest(resolve(root, "no-mining-accounting-before.json")),
    accounting_after_sha256: await fileDigest(resolve(root, "no-mining-accounting-after.json")),
  };
}
function publicResult(receipt) {
  return {
    result: receipt.result,
    device_recovered: true,
    qualification_pass: false,
    failed_preparation_pass: false,
    mining_authorized: false,
    pre_failure_settings_continuity_proven: false,
    next_ordinal: receipt.ledger.next_ordinal,
    total_charged_ms: receipt.ledger.total_charged_ms,
    cleanup_confirmed: true,
  };
}
export async function judgeStartupRecovery(root, inputPath, operations = {}) {
  root = resolve(root);
  const context = await loadStartupRecoveryContext(root, { operations });
  await protectedPath(inputPath);
  const cleanup = await readJson(inputPath),
    evidence = await recoveryEvidence(root, context, cleanup);
  const receipt = {
    schema: "worker-cadence-startup-recovery-result-v1",
    result: "recovered",
    context,
    context_sha256: digest(JSON.stringify(context)),
    ...evidence,
    cleanup,
    cleanup_input: { path: resolve(inputPath), sha256: await fileDigest(inputPath) },
    device_recovered: true,
    qualification_pass: false,
    failed_preparation_pass: false,
    mining_authorized: false,
    pre_failure_settings_continuity_proven: false,
    within_recovery_session_continuity: true,
    inventory: (await inventory(root)).filter((row) => row.path !== "result.json"),
  };
  await writeNew(resolve(root, "result.json"), { receipt, sha256: digest(JSON.stringify(receipt)) });
  return publicResult(receipt);
}
export async function readStartupRecovery(path, operations = {}) {
  path = resolve(path);
  const root = dirname(path);
  requireCondition(path === resolve(root, "result.json"), "startup_recovery_result_path");
  const saved = await proof(path);
  exactObject(saved.value, ["receipt", "sha256"]);
  const receipt = saved.value.receipt;
  exactObject(receipt, [
    "schema",
    "result",
    "context",
    "context_sha256",
    "installation",
    "ledger",
    "original_budget",
    "accounting_before_sequence",
    "accounting_after_sequence",
    "final_sequence",
    "final_state",
    "accounting_before_sha256",
    "accounting_after_sha256",
    "cleanup",
    "cleanup_input",
    "device_recovered",
    "qualification_pass",
    "failed_preparation_pass",
    "mining_authorized",
    "pre_failure_settings_continuity_proven",
    "within_recovery_session_continuity",
    "inventory",
  ]);
  exactObject(receipt.cleanup_input, ["path", "sha256"]);
  requireCondition(
    saved.value.sha256 === digest(JSON.stringify(receipt)) &&
      receipt.schema === "worker-cadence-startup-recovery-result-v1" &&
      receipt.result === "recovered" &&
      receipt.device_recovered === true &&
      receipt.qualification_pass === false &&
      receipt.failed_preparation_pass === false &&
      receipt.mining_authorized === false &&
      receipt.pre_failure_settings_continuity_proven === false &&
      receipt.within_recovery_session_continuity === true,
    "startup_recovery_result_shape",
  );
  const context = await loadStartupRecoveryContext(root, { historical: true, operations });
  requireCondition(
    isDeepStrictEqual(context, receipt.context) && receipt.context_sha256 === digest(JSON.stringify(context)),
    "startup_recovery_result_context",
  );
  await protectedPath(receipt.cleanup_input.path);
  requireCondition(
    (await fileDigest(receipt.cleanup_input.path)) === receipt.cleanup_input.sha256 &&
      isDeepStrictEqual(await readJson(receipt.cleanup_input.path), receipt.cleanup),
    "startup_recovery_cleanup_changed",
  );
  const evidence = await recoveryEvidence(root, context, receipt.cleanup);
  requireCondition(
    Object.keys(evidence).every((key) => isDeepStrictEqual(evidence[key], receipt[key])) &&
      isDeepStrictEqual(
        receipt.inventory,
        (await inventory(root)).filter((row) => row.path !== "result.json"),
      ),
    "startup_recovery_evidence_changed",
  );
  return receipt;
}
