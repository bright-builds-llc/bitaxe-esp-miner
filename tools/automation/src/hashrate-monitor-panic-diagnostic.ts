type JsonObject = Readonly<Record<string, unknown>>;

const mixedResetReasons = [
  "not_observed",
  "none",
  "power_on",
  "software_cpu",
  "watchdog",
  "panic",
  "brownout",
  "other",
] as const;
const panicSignatures = [
  "not_observed",
  "none",
  "unknown",
  "stack_overflow",
  "stack_smashing",
  "heap_corruption",
  "assertion",
  "abort",
  "rust_panic",
  "guru_meditation",
] as const;
const panicTaskFamilies = [
  "not_observed",
  "none",
  "production_mining_session",
  "production_asic",
  "axeos_live_websocket",
  "deferred_effects",
  "safety_supervisor",
  "operator_sensor",
  "fan_controller",
  "statistics",
  "wifi_reconnect",
  "http_server",
  "main",
  "other",
] as const;

type MixedResetReason = typeof mixedResetReasons[number];
type PanicSignature = typeof panicSignatures[number];
type PanicTaskFamily = typeof panicTaskFamilies[number];

export type PanicFailureDiagnostic = Readonly<{
  panic_signature: PanicSignature;
  panic_task_family: PanicTaskFamily;
  panic_signature_count: number;
}>;

export type ParsedPanicDiagnostic = Readonly<{
  mixedResetReason: MixedResetReason;
  maybeFailure: PanicFailureDiagnostic | undefined;
}>;

function requiredString(value: JsonObject, field: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate.length === 0) {
    throw new Error("campaign panic diagnostic string is invalid");
  }
  return candidate;
}

function requiredCount(value: JsonObject, field: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 0) {
    throw new Error("campaign panic diagnostic count is invalid");
  }
  return candidate;
}

export function parsePanicDiagnostic(value: JsonObject): ParsedPanicDiagnostic {
  if (requiredString(value, "schema") !== "mining-campaign-serial-diagnostics-v4") {
    throw new Error("campaign panic diagnostic schema is invalid");
  }
  const mixedResetReason = requiredString(value, "runtime_attestation_mixed_reset_reason");
  const signature = requiredString(value, "panic_signature");
  const taskFamily = requiredString(value, "panic_task_family");
  const count = requiredCount(value, "panic_signature_count");
  if (!mixedResetReasons.includes(mixedResetReason as MixedResetReason)
    || !panicSignatures.includes(signature as PanicSignature)
    || !panicTaskFamilies.includes(taskFamily as PanicTaskFamily)) {
    throw new Error("campaign panic diagnostic label is invalid");
  }
  if (signature === "not_observed" || signature === "none") {
    if (taskFamily !== signature || count !== 0 || mixedResetReason === "panic") {
      throw new Error("campaign empty panic diagnostic is inconsistent");
    }
    return { mixedResetReason: mixedResetReason as MixedResetReason, maybeFailure: undefined };
  }
  if (signature === "unknown") {
    if (taskFamily !== "none" || count !== 0 || mixedResetReason !== "panic") {
      throw new Error("campaign unknown panic diagnostic is inconsistent");
    }
  } else if (count === 0
    || (signature !== "stack_overflow" && taskFamily !== "none")
    || (signature === "stack_overflow"
      && (taskFamily === "none" || taskFamily === "not_observed"))) {
    throw new Error("campaign recognized panic diagnostic is inconsistent");
  }
  return {
    mixedResetReason: mixedResetReason as MixedResetReason,
    maybeFailure: {
      panic_signature: signature as PanicSignature,
      panic_task_family: taskFamily as PanicTaskFamily,
      panic_signature_count: count,
    },
  };
}
