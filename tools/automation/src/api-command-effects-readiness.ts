type JsonObject = Readonly<Record<string, unknown>>;

const exactFields = [
  "wakeup", "previous_blocker", "current_blocker", "session_phase", "campaign_state",
  "hardware_state", "safety_sample", "observation_epoch", "pending_observation_recovered",
] as const;
const wakeups = new Set([
  "deadline", "network_changed", "settings_changed", "observations_changed",
  "operator_intent_changed", "shutdown_requested",
]);
const blockers = new Set([
  "none", "operator_paused", "network_unavailable", "stratum_v1_unsupported",
  "safety_prerequisites_stale", "campaign_lease_unavailable", "campaign_lease_consumed",
  "production_asic_unavailable", "production_asic_version_mask_unavailable",
  "production_asic_dispatch_unavailable", "production_asic_poll_unavailable",
  "production_asic_queue_full", "production_asic_worker_unavailable", "actuation_unqualified",
  "pool_configuration_unavailable", "pools_exhausted", "job_transition_protocol_inconsistent",
]);
const epochs = new Set(["initial", "advanced", "unchanged", "unavailable"]);
const sessionPhases = new Set([
  "waiting_for_readiness", "connecting_primary", "running_primary", "connecting_fallback",
  "running_fallback", "recovery_paused", "safe_stopping", "shutdown",
]);
const campaignStates = new Set([
  "unavailable", "preparing", "armed", "active", "safe_stopping", "consumed",
]);
const hardwareStates = new Set([
  "unprepared", "preparing", "ready", "safe_stopping", "stopped",
]);

export function isClosedReadinessTransition(value: unknown): boolean {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
  const transition = value as JsonObject;
  const wakeup = transition["wakeup"];
  const previousBlocker = transition["previous_blocker"];
  const currentBlocker = transition["current_blocker"];
  const sessionPhase = transition["session_phase"];
  const campaignState = transition["campaign_state"];
  const hardwareState = transition["hardware_state"];
  const observationEpoch = transition["observation_epoch"];
  return Object.keys(transition).length === exactFields.length
    && exactFields.every((field) => field in transition)
    && typeof wakeup === "string" && wakeups.has(wakeup)
    && typeof previousBlocker === "string" && blockers.has(previousBlocker)
    && typeof currentBlocker === "string" && blockers.has(currentBlocker)
    && typeof sessionPhase === "string" && sessionPhases.has(sessionPhase)
    && typeof campaignState === "string" && campaignStates.has(campaignState)
    && typeof hardwareState === "string" && hardwareStates.has(hardwareState)
    && transition["safety_sample"] === "fresh"
    && typeof observationEpoch === "string" && epochs.has(observationEpoch)
    && typeof transition["pending_observation_recovered"] === "boolean";
}
