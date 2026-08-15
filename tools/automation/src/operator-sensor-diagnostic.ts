import {
  campaignRecoveryFactsFromDocuments,
  type RecoveryFacts,
} from "./api-command-effects-recovery.js";

type JsonObject = Readonly<Record<string, unknown>>;

export type OperatorSensorDiagnostic = Readonly<{
  available: boolean;
  revision: number;
  stage: "power" | "asic_temperature" | "tachometer" | "core_voltage" | "display" | "actuation";
  outcome: "ready" | "recovered" | "driver_failed" | "budget_exhausted" | "sample_invalid" | "unavailable";
  duration_bucket: "under_100_ms" | "under_250_ms" | "under_500_ms" | "under_1000_ms" | "at_least_1000_ms";
}>;

const STAGES = new Set([
  "power", "asic_temperature", "tachometer", "core_voltage", "display", "actuation",
]);
const OUTCOMES = new Set([
  "ready", "recovered", "driver_failed", "budget_exhausted", "sample_invalid", "unavailable",
]);
const DURATION_BUCKETS = new Set([
  "under_100_ms", "under_250_ms", "under_500_ms", "under_1000_ms", "at_least_1000_ms",
]);

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("operator sensor diagnostic must be an object");
  }
  return value as JsonObject;
}

/** Validates the private boot-scoped marker and omits boot identity publicly. */
export function parseOperatorSensorDiagnostic(value: unknown): OperatorSensorDiagnostic | undefined {
  const diagnostic = object(value);
  if (diagnostic["available"] === false) {
    if (
      diagnostic["boot_session"] !== 0
      || diagnostic["revision"] !== 0
      || diagnostic["stage"] !== "none"
      || diagnostic["outcome"] !== "none"
      || diagnostic["duration_bucket"] !== "none"
    ) {
      throw new Error("unavailable operator sensor diagnostic is contradictory");
    }
    return undefined;
  }
  const bootSession = diagnostic["boot_session"];
  const revision = diagnostic["revision"];
  const stage = diagnostic["stage"];
  const outcome = diagnostic["outcome"];
  const durationBucket = diagnostic["duration_bucket"];
  if (
    diagnostic["available"] !== true
    || typeof bootSession !== "number"
    || !Number.isFinite(bootSession)
    || !Number.isInteger(bootSession)
    || bootSession < 1
    || typeof revision !== "number"
    || !Number.isSafeInteger(revision)
    || revision < 1
    || typeof stage !== "string"
    || !STAGES.has(stage)
    || typeof outcome !== "string"
    || !OUTCOMES.has(outcome)
    || typeof durationBucket !== "string"
    || !DURATION_BUCKETS.has(durationBucket)
  ) {
    throw new Error("operator sensor diagnostic is invalid");
  }
  return {
    available: true,
    revision,
    stage: stage as OperatorSensorDiagnostic["stage"],
    outcome: outcome as OperatorSensorDiagnostic["outcome"],
    duration_bucket: durationBucket as OperatorSensorDiagnostic["duration_bucket"],
  };
}

export type CampaignFailureFacts = Readonly<{
  recovery: RecoveryFacts;
  maybeOperatorSensor?: OperatorSensorDiagnostic;
}>;

export function campaignFailureFactsFromDocuments(
  result: JsonObject,
  network: JsonObject,
): CampaignFailureFacts {
  let maybeOperatorSensor: OperatorSensorDiagnostic | undefined;
  try {
    maybeOperatorSensor = parseOperatorSensorDiagnostic(result["operator_sensor"]);
  } catch {
    // A malformed optional diagnostic cannot erase independently closed recovery facts.
  }
  let recovery: RecoveryFacts;
  try {
    recovery = campaignRecoveryFactsFromDocuments(result, network);
  } catch {
    recovery = {
      safeStopConfirmed: false,
      cleanupComplete: false,
      recoveryAttempted: false,
      secondaryRecoveryFailure: false,
    };
  }
  return {
    recovery,
    ...(maybeOperatorSensor === undefined ? {} : { maybeOperatorSensor }),
  };
}
