import { OperatorSnapshotEvidenceError } from "./operator-snapshot-evidence.js";
import { AsicFrequencyTransitionEvidenceError } from "./asic-frequency-transition-evidence.js";
import { AsicInitializationEvidenceError } from "./asic-initialization-evidence.js";
import { AsicResultParsingEvidenceError } from "./asic-result-parsing-evidence.js";
import { AsicSerialTransportEvidenceError } from "./asic-serial-transport-evidence.js";
import { AsicWorkSendEvidenceError } from "./asic-work-send-evidence.js";
import { NetworkReconnectEvidenceError } from "./network-reconnect-evidence.js";
import { NetworkScanEvidenceError } from "./network-scan-evidence.js";
import { ProvisioningNetworkEvidenceError } from "./provisioning-network-evidence.js";
import { RuntimeHealthEvidenceError } from "./runtime-health-evidence.js";
import { SettingsDurabilityError } from "./settings-durability.js";
import { SettingsPatchEvidenceError } from "./settings-patch-evidence.js";
import { SdkconfigRollbackEvidenceError } from "./sdkconfig-rollback-evidence.js";
import { SystemInfoEvidenceError } from "./system-info-evidence.js";
import { ThemeDurabilityError } from "./theme-durability.js";
import { Ultra205DefaultsEvidenceError } from "./ultra205-defaults-evidence.js";

export function maybeTypedFailurePublicValue(error: unknown): Readonly<Record<string, unknown>> | undefined {
  if (error instanceof SettingsDurabilityError
    || error instanceof AsicInitializationEvidenceError
    || error instanceof AsicFrequencyTransitionEvidenceError
    || error instanceof AsicWorkSendEvidenceError
    || error instanceof AsicResultParsingEvidenceError
    || error instanceof AsicSerialTransportEvidenceError
    || error instanceof NetworkReconnectEvidenceError
    || error instanceof NetworkScanEvidenceError
    || error instanceof ProvisioningNetworkEvidenceError
    || error instanceof ThemeDurabilityError
    || error instanceof OperatorSnapshotEvidenceError
    || error instanceof RuntimeHealthEvidenceError
    || error instanceof SystemInfoEvidenceError
    || error instanceof Ultra205DefaultsEvidenceError
    || error instanceof SettingsPatchEvidenceError
    || error instanceof SdkconfigRollbackEvidenceError) {
    return error.publicValue;
  }
  return undefined;
}
