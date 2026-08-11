import { fetchTextFromSameOrigin } from "./http.js";

export const retainedFirmwareOtaProtocolErrorLine = "firmware_ota_status=Protocol Error";
export const rollbackProbePendingLine = "ota_boot_validation=rollback_probe_pending";
export const passiveSafeStateLine =
  "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled";

export function retainedLogHasExactLine(logs: string, expected: string): boolean {
  return logs.split(/\r?\n/u).some((line) => line === expected);
}

export function retainedFirmwareOtaProtocolAbortObserved(logs: string): boolean {
  return retainedLogHasExactLine(logs, retainedFirmwareOtaProtocolErrorLine);
}

export async function retainedBootLogStatus(
  origin: URL,
  output: string,
  requiredLines: readonly string[],
): Promise<"ready" | "unavailable" | "missing"> {
  let logs: string;
  try {
    logs = await fetchTextFromSameOrigin(origin, "/api/system/logs", output);
  } catch {
    return "unavailable";
  }
  return requiredLines.every((line) => retainedLogHasExactLine(logs, line)) ? "ready" : "missing";
}
