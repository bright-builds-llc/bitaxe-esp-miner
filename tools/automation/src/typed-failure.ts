import { OperatorSnapshotEvidenceError } from "./operator-snapshot-evidence.js";
import { RuntimeHealthEvidenceError } from "./runtime-health-evidence.js";
import { SettingsDurabilityError } from "./settings-durability.js";
import { SystemInfoEvidenceError } from "./system-info-evidence.js";
import { ThemeDurabilityError } from "./theme-durability.js";

export function maybeTypedFailurePublicValue(error: unknown): Readonly<Record<string, unknown>> | undefined {
  if (error instanceof SettingsDurabilityError
    || error instanceof ThemeDurabilityError
    || error instanceof OperatorSnapshotEvidenceError
    || error instanceof RuntimeHealthEvidenceError
    || error instanceof SystemInfoEvidenceError) {
    return error.publicValue;
  }
  return undefined;
}
