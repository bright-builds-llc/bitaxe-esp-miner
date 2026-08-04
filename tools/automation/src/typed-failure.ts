import { OperatorSnapshotEvidenceError } from "./operator-snapshot-evidence.js";
import { RuntimeHealthEvidenceError } from "./runtime-health-evidence.js";
import { SettingsDurabilityError } from "./settings-durability.js";
import { ThemeDurabilityError } from "./theme-durability.js";

export function maybeTypedFailurePublicValue(error: unknown): Readonly<Record<string, unknown>> | undefined {
  if (error instanceof SettingsDurabilityError
    || error instanceof ThemeDurabilityError
    || error instanceof OperatorSnapshotEvidenceError
    || error instanceof RuntimeHealthEvidenceError) {
    return error.publicValue;
  }
  return undefined;
}
