import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, requireCondition as check, writeNew } from "./contract.mjs";
import { baseline, proof } from "./cadence-premining-evidence.mjs";
import { parseResetOriginDiagnostic, RESET_ORIGIN_CATEGORIES } from "./reset-origin-observation.mjs";
import { restartInnerContext } from "./reset-origin-restart-context.mjs";
import { readRestartAccounting, readRestartStates } from "./reset-origin-restart-state.mjs";

/** This one known failure is inspected as failure; no health or restoration inference is made. */
export function inspectPreinstallFailure(input, context) {
  check(
    context.restart_attempt === 2 && context.before_install_failure?.first_failure === "statistics",
    "restart_preinstall_review_not_allowed",
  );
  exactObject(input, ["schema", "observations"]);
  check(
    input.schema === "worker-diagnostic-export-v1" && Array.isArray(input.observations) && input.observations.length <= 40,
    "restart_preinstall_diagnostics",
  );
  const observations = [];
  for (const value of input.observations) {
    check(value && typeof value === "object", "restart_preinstall_diagnostics");
    if (value.category === "memory") continue;
    if (value.category === "worker_admission") {
      check(value.stage === "idle" && value.first_failure === "none", "restart_preinstall_activity");
      continue;
    }
    check(RESET_ORIGIN_CATEGORIES.includes(value.category), "restart_preinstall_unexpected_diagnostic");
    const parsed = parseResetOriginDiagnostic(value);
    check(
      !["panic", "allocation_failure", "allocation_context", "statistics_startup"].includes(parsed.category),
      "restart_preinstall_unexpected_failure",
    );
    if (parsed.category === "startup")
      check(
        parsed.state !== "failed" &&
          (parsed.first_failure === "none" ||
            (parsed.first_failure === "statistics" && parsed.stage === "runtime_ready" && parsed.state === "complete")),
        "restart_preinstall_unexpected_failure",
      );
    observations.push(parsed);
  }
  check(
    new Set(observations.map((value) => `${value.category}:${value.stage ?? value.origin ?? ""}`)).size === observations.length,
    "restart_preinstall_duplicate_diagnostic",
  );
  const known = context.before_install_failure,
    boots = observations.filter((v) => v.category === "boot");
  check(
    boots.length === 1 &&
      boots[0].boot_ordinal === known.boot_ordinal &&
      boots[0].reset_reason === known.reset_reason &&
      boots[0].uptime_ms > known.last_boot_uptime_ms,
    "restart_preinstall_boot_changed",
  );
  check(
    observations.some(
      (v) =>
        v.category === "runtime_identity" &&
        v.firmware_commit === context.before_source.firmware_commit &&
        v.app_elf_sha256 === context.before_source.app_elf_sha256,
    ),
    "restart_preinstall_identity",
  );
  check(
    observations.some(
      (v) =>
        v.category === "startup" &&
        v.stage === "runtime_ready" &&
        v.state === "complete" &&
        v.first_failure === "statistics" &&
        v.uptime_ms > known.last_startup_uptime_ms,
    ),
    "restart_preinstall_failure_missing",
  );
  check(
    observations.some((v) => v.category === "storage_http_status" && v.http_ready === "true" && v.spiffs_available === "true"),
    "restart_preinstall_storage",
  );
  return observations;
}
export async function savePreinstallFailure(root, context, input) {
  const inner = restartInnerContext(root, context, "before-install"),
    scope = resolve(root, "before-install");
  const observations = inspectPreinstallFailure(input, context),
    rows = await readRestartStates(scope, inner),
    current = rows.at(-1),
    accounting = await readRestartAccounting(scope, inner, "before");
  baseline(current.state, false);
  check(
    current.sequence >= accounting.observed_sequence &&
      current.state.preservation.baseline_id === accounting.state.preservation.baseline_id,
    "restart_preinstall_accounting_join",
  );
  await writeNew(resolve(root, "before-install-failure-review.json"), {
    schema: "fixed-usb-restart-preinstall-failure-v1",
    context_sha256: digest(JSON.stringify(context)),
    observed_sequence: current.sequence,
    observations,
  });
  return { known_failure_reviewed: true, startup_healthy: false, recovery_verified: false };
}
export async function requirePreinstallFailure(root, context, closedSequence) {
  if (context.restart_attempt === undefined) return;
  const receipt = (await proof(resolve(root, "before-install-failure-review.json"))).value;
  exactObject(receipt, ["schema", "context_sha256", "observed_sequence", "observations"]);
  check(
    receipt.schema === "fixed-usb-restart-preinstall-failure-v1" && receipt.context_sha256 === digest(JSON.stringify(context)),
    "restart_preinstall_review_binding",
  );
  const observations = inspectPreinstallFailure({ schema: "worker-diagnostic-export-v1", observations: receipt.observations }, context);
  check(equal(observations, receipt.observations), "restart_preinstall_review_projection");
  const inner = restartInnerContext(root, context, "before-install"),
    scope = resolve(root, "before-install"),
    rows = await readRestartStates(scope, inner),
    accounting = await readRestartAccounting(scope, inner, "before");
  check(
    Number.isSafeInteger(receipt.observed_sequence) &&
      receipt.observed_sequence >= accounting.observed_sequence &&
      receipt.observed_sequence < closedSequence,
    "restart_preinstall_review_order",
  );
  const row = rows[receipt.observed_sequence - 1];
  baseline(row?.state, false);
  check(row.state.preservation.baseline_id === accounting.state.preservation.baseline_id, "restart_preinstall_review_baseline");
}
