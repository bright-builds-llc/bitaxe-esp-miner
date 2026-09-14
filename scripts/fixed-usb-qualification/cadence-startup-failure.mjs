import { readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { digest, fileDigest, missing, requireCondition } from "./contract.mjs";
import { inventory, proof } from "./cadence-premining-evidence.mjs";
import { validateCadenceContext } from "./cadence-preflight.mjs";
import { readPrevious } from "./iterative-preflight.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";

export const STARTUP_FAILURE_SHA256 = "d65b506e49d271198d16381c30dcf02596d9f7baf2e72198fba322f7a64a2de8";
const PRODUCER = "0e23b0bc4298bcc08d9e130b1d8246831ccb52f9260d72f84c8f62b7f408dffd";

/** This classification recognizes one sealed failure; it supplies no fresh device facts. */
export async function readStartupFailure(root, operations = {}) {
  root = resolve(root);
  const saved = await proof(resolve(root, "failed-inventory.json"));
  requireCondition(saved.sha256 === (operations.expectedStartupFailureSha256 ?? STARTUP_FAILURE_SHA256), "startup_failure_anchor");
  const failure = saved.value,
    stored = await proof(resolve(root, "context.json")),
    context = stored.value.context;
  requireCondition(
    failure.schema === "cpu0-cadence-initial-startup-failed-inventory-v1" &&
      failure.auditor_sha256 === PRODUCER &&
      failure.outcome === "unverified_initial_startup_failure" &&
      failure.qualification_pass === false &&
      failure.continuation_authority === false &&
      failure.device_restoration_confirmed === false &&
      failure.fresh_device_ledger_observed === false &&
      stored.value.sha256 === digest(JSON.stringify(context)) &&
      failure.context_sha256 === stored.value.sha256,
    "startup_failure_shape",
  );
  await validateCadenceContext(root, context, { historical: true, operations });
  requireCondition(
    context.qualification_attempt.ordinal === 17 && context.expected_charged_ms === 1380000 && context.preparation_attempt === undefined,
    "startup_failure_ordinal",
  );
  const previous = await (operations.readPrevious ?? readPrevious)(context.previous_receipt);
  requireCondition(
    dirname(dirname(context.previous_receipt)) === dirname(root) &&
      previous.next_ordinal === 17 &&
      previous.total_charged_ms === 1380000 &&
      failure.previous_result_sha256 === (await fileDigest(context.previous_receipt)),
    "startup_failure_predecessor",
  );
  const snapshot = await (operations.verifyArtifactSnapshot ?? verifyArtifactSnapshot)(root, context);
  requireCondition(snapshot.receipt_sha256 === failure.artifact_snapshot_sha256, "startup_failure_snapshot");
  for (const name of [
    "iterative.samples.jsonl",
    "issued.json",
    "consumed.json",
    "result.json",
    "cadence-observer.jsonl",
    "cadence-observer-result.json",
    "cadence-idle-arm.json",
    "cadence-usb-arm.json",
    "cadence-mining-arm.json",
    "cadence-idle.json",
    "cadence-usb.json",
    "cadence-mining.json",
    "cadence-probes.jsonl",
    "iterative.fault.json",
    "sample-seal-intent.json",
    "sealed.samples.jsonl",
  ])
    await missing(resolve(root, name));
  requireCondition(!(await readdir(root)).some((name) => /^cycle-[1-4]\./u.test(name)), "startup_failure_activity");
  const flashed = await proof(resolve(root, "flash-0/flash-command-evidence.json")),
    f = flashed.value,
    a = f.fixed_serial_assessment;
  requireCondition(
    flashed.sha256 === failure.initial_flash_sha256 &&
      f.flash_status === "completed" &&
      f.monitor_evidence_status === "untrusted" &&
      f.trusted_output === false &&
      f.firmware_commit === context.firmware_commit &&
      f.nvs_seed_status === "not_provided" &&
      a?.startup_complete === false &&
      a.startup_failed === true &&
      a.stable_boot === false &&
      a.retained_failure_history === true,
    "startup_failure_installation",
  );
  const cleanup = await proof(resolve(root, "early-host-cleanup.json")),
    c = cleanup.value;
  requireCondition(
    cleanup.sha256 === failure.early_host_cleanup_sha256 &&
      c.schema === "worker-cadence-unused-host-cleanup-v1" &&
      c.source === "parent-observed" &&
      c.browser_opened === false &&
      c.supervisor_exit_code === 0 &&
      c.flash_wrapper_exit_code === 1 &&
      c.flash_observer_exit_code === 0 &&
      [
        "host_cleanup_checked",
        "supervisor_absent",
        "listener_absent",
        "owned_children_absent",
        "serial_holders_absent",
        "journal_absent",
        "observer_activity_absent",
        "issuance_artifacts_absent",
      ].every((key) => c[key] === true) &&
      c.device_restoration_claimed === false &&
      c.fresh_ledger_claimed === false &&
      c.startup_qualification_pass === false,
    "startup_failure_cleanup",
  );
  requireCondition(isDeepStrictEqual(failure.inventory, await inventory(root)), "startup_failure_inventory_changed");
  return { root, context, previous, failed_inventory_sha256: saved.sha256 };
}
