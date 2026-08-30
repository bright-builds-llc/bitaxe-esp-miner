import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { planSha256 } from "./native-usb-transition-recovery-contract.js";

export function validTransitionCandidate(value: JsonObject): boolean {
  const terminalCategories = new Set([
    "complete", "runtime_profile_unknown", "handoff_unsupported",
    "handoff_rejected_unsafe_state", "handoff_ready_timeout", "handoff_commit_timeout",
    "bus_reset_timeout", "same_worker_after_commit", "handoff_transition_timeout",
    "bootloader_ambiguous", "physical_identity_drift", "rom_admission_failed",
    "application_reappearance_timeout", "foreign_holder", "cleanup_failed",
    "recovery_required",
  ]);
  const digestKeys = [
    "source_commit", "reference_commit", "plan_sha256", "evaluator_sha256",
    "manifest_sha256", "app_elf_sha256",
  ] as const;
  const countKeys = [
    "absent_count", "same_worker_count", "same_serial_jtag_count",
    "same_unknown_count", "physical_mismatch_count",
  ] as const;
  const booleanKeys = [
    "ready_received", "committed_received", "bus_reset_observed", "rom_admitted",
    "application_reappeared", "device_write_observed", "restoration_complete",
    "cleanup_complete",
  ] as const;
  const stagesAreOrdered = value["committed_received"] !== true || value["ready_received"] === true;
  const busResetIsOrdered = value["bus_reset_observed"] !== true
    || value["committed_received"] === true;
  const romIsOrdered = value["rom_admitted"] !== true || value["bus_reset_observed"] === true;
  const applicationIsOrdered = value["application_reappeared"] !== true
    || value["rom_admitted"] === true;
  const completeIsExact = value["terminal_category"] !== "complete"
    || (value["ready_received"] === true
      && value["committed_received"] === true
      && value["bus_reset_observed"] === true
      && value["rom_admitted"] === true
      && value["application_reappeared"] === true
      && value["cleanup_complete"] === true);
  return value["schema_version"] === "bitaxe-native-usb-transition-projection-v1"
    && value["plan_sha256"] === planSha256
    && digestKeys.every(key => typeof value[key] === "string"
      && /^[0-9a-f]+$/u.test(value[key] as string)
      && (key === "source_commit" || key === "reference_commit"
        ? (value[key] as string).length === 40
        : (value[key] as string).length === 64))
    && countKeys.every(key => Number.isInteger(value[key])
      && Number(value[key]) >= 0 && Number(value[key]) <= 1_024)
    && booleanKeys.every(key => typeof value[key] === "boolean")
    && stagesAreOrdered && busResetIsOrdered && romIsOrdered && applicationIsOrdered
    && completeIsExact
    && value["device_write_observed"] === false
    && value["restoration_complete"] === false
    && value["redaction_status"] === "passed"
    && typeof value["terminal_category"] === "string"
    && terminalCategories.has(value["terminal_category"] as string);
}
