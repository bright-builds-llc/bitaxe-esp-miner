type JsonObject = Readonly<Record<string, unknown>>;

export type RecoveryFacts = {
  readonly safeStopConfirmed: boolean;
  readonly cleanupComplete: boolean;
  readonly recoveryAttempted: boolean;
  readonly secondaryRecoveryFailure: boolean;
};

function maybeObject(value: unknown): JsonObject | undefined {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return undefined;
  return value as JsonObject;
}

export function campaignRecoveryFactsFromDocuments(
  result: JsonObject,
  network: JsonObject,
): RecoveryFacts {
  const maybeEffects = maybeObject(network["command_effects"]);
  const recoveryAttempted = network["recovery_pause_request_count"] === 1;
  const joinedRecoverySafeStop = recoveryAttempted
    && maybeEffects?.["recovery_pause_api_confirmed"] === true
    && maybeEffects["recovery_pause_serial_confirmed"] === true
    && maybeEffects["recovery_safe_stop_confirmed"] === true
    && maybeEffects["recovery_terminal_outcome"] === "confirmed";
  const safeStopConfirmed = result["safe_stop"] === "confirmed" || joinedRecoverySafeStop;
  return {
    safeStopConfirmed,
    cleanupComplete: result["usb_cleanup"] === "ready",
    recoveryAttempted,
    secondaryRecoveryFailure: recoveryAttempted && !safeStopConfirmed,
  };
}
