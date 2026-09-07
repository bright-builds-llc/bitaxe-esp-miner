import { canonicalBase64, exactObject, requireCondition } from "./contract.mjs";

export const PURPOSES = ["diagnostic", "normal", "foreground_loss", "heartbeat_loss"];
export const maximumActiveMs = (purpose) => purpose === "normal" ? 180000 : 30000;
export function validateAttempt(value) {
  exactObject(value, ["schema", "id", "ordinal", "purpose", "maximumActiveMilliseconds"]);
  requireCondition(value.schema === "worker-qualification-attempt-v1" && canonicalBase64(value.id, 16) &&
    Number.isInteger(value.ordinal) && value.ordinal > 0 && value.ordinal <= 0xffffffff && PURPOSES.includes(value.purpose) &&
    value.maximumActiveMilliseconds === maximumActiveMs(value.purpose), "iterative_attempt_shape");
  return value;
}
export function validateLedger(value) {
  exactObject(value, ["schema", "next_ordinal", "total_charged_ms", "pending", "last_completed_ordinal"]);
  requireCondition(value.schema === "worker-qualification-ledger-v1" && Number.isInteger(value.next_ordinal) &&
    value.next_ordinal >= 1 && value.next_ordinal <= 0x100000000 && Number.isSafeInteger(value.total_charged_ms) &&
    value.total_charged_ms >= 0 && typeof value.pending === "boolean" && Number.isInteger(value.last_completed_ordinal) &&
    value.last_completed_ordinal >= 0 && value.last_completed_ordinal <= 0xffffffff &&
    value.last_completed_ordinal < value.next_ordinal, "iterative_ledger_shape");
  return value;
}
export function requireIdleLedger(value, ordinal, charged) {
  validateLedger(value);
  requireCondition(!value.pending && value.next_ordinal === ordinal && value.last_completed_ordinal === ordinal - 1 &&
    value.total_charged_ms === charged, "iterative_ledger_admission");
}
export function requireExhaustedOriginal(value) {
  exactObject(value, ["schema", "campaign_match", "reserved_mask", "completed_mask", "charged_ms", "pending"]);
  requireCondition(value.schema === "worker-budget-review-v1" && value.campaign_match === true && value.reserved_mask === 7 &&
    value.completed_mask === 7 && value.charged_ms === 240000 && value.pending === false, "original_budget_not_complete");
}
export function validateCooling(proof, restoration) {
  exactObject(proof, ["schema", "fan_duty_percent", "fan_rpm", "post_command_fan_proven", "asic_effects", "budget_reserved"]);
  exactObject(restoration, ["schema", "fan_duty_percent", "cooling_proven", "asic_effects", "budget_reserved"]);
  requireCondition(proof.schema === "worker-cooling-proof-v1" && proof.fan_duty_percent === 100 && Number.isInteger(proof.fan_rpm) &&
    proof.fan_rpm > 0 && proof.fan_rpm <= 65535 && proof.post_command_fan_proven === true && proof.asic_effects === false &&
    proof.budget_reserved === false, "iterative_cooling_proof");
  requireCondition(restoration.schema === "worker-cooling-baseline-v1" && restoration.fan_duty_percent === 30 &&
    restoration.cooling_proven === true && restoration.asic_effects === false && restoration.budget_reserved === false, "iterative_cooling_restore");
}
