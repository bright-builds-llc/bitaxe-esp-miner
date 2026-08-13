import { OperatorSnapshotEvidenceError } from "./operator-snapshot-evidence.js";
import { ApiCommandEffectsError } from "./api-command-effects.js";
import { AsicFrequencyTransitionEvidenceError } from "./asic-frequency-transition-evidence.js";
import { AsicInitializationEvidenceError } from "./asic-initialization-evidence.js";
import { AsicPowerInitializationEvidenceError } from "./asic-power-initialization-evidence.js";
import { CoreVoltageControlEvidenceError } from "./core-voltage-control-evidence.js";
import { Ina260EvidenceError } from "./ina260-evidence.js";
import { Emc2101ThermalEvidenceError } from "./emc2101-thermal-evidence.js";
import { Emc2101ThermalFaultEvidenceError } from "./emc2101-thermal-fault-evidence.js";
import { AsicResetEvidenceError } from "./asic-reset-evidence.js";
import { AsicResultParsingEvidenceError } from "./asic-result-parsing-evidence.js";
import { AsicSerialTransportEvidenceError } from "./asic-serial-transport-evidence.js";
import { AsicWorkSendEvidenceError } from "./asic-work-send-evidence.js";
import { StratumSocketEvidenceError } from "./stratum-socket-evidence.js";
import { ProtocolCoordinatorEvidenceError } from "./protocol-coordinator-evidence.js";
import { MiningCriteriaEvidenceError } from "./mining-criteria-evidence.js";
import { LogBufferEvidenceError } from "./log-buffer-evidence.js";
import { NetworkReconnectEvidenceError } from "./network-reconnect-evidence.js";
import { NetworkScanEvidenceError } from "./network-scan-evidence.js";
import { ProvisioningNetworkEvidenceError } from "./provisioning-network-evidence.js";
import { PartitionLayoutEvidenceError } from "./partition-layout-evidence.js";
import { RuntimeHealthEvidenceError } from "./runtime-health-evidence.js";
import { SettingsDurabilityError } from "./settings-durability.js";
import { SettingsPatchEvidenceError } from "./settings-patch-evidence.js";
import { SdkconfigRollbackEvidenceError } from "./sdkconfig-rollback-evidence.js";
import { SystemInfoEvidenceError } from "./system-info-evidence.js";
import { ThemeDurabilityError } from "./theme-durability.js";
import { Ultra205DefaultsEvidenceError } from "./ultra205-defaults-evidence.js";
import { UiWorkflowEvidenceError } from "./ui-workflow-evidence.js";
import type { AutomationCategory } from "./contracts.generated.js";

type TypedFailure = Error & {
  readonly category: AutomationCategory;
  readonly publicValue: Readonly<Record<string, unknown>>;
};

function maybeTypedFailure(error: unknown): TypedFailure | undefined {
  if (error instanceof ApiCommandEffectsError
    || error instanceof SettingsDurabilityError
    || error instanceof AsicInitializationEvidenceError
    || error instanceof AsicPowerInitializationEvidenceError
    || error instanceof CoreVoltageControlEvidenceError
    || error instanceof Ina260EvidenceError
    || error instanceof Emc2101ThermalEvidenceError
    || error instanceof Emc2101ThermalFaultEvidenceError
    || error instanceof AsicResetEvidenceError
    || error instanceof AsicFrequencyTransitionEvidenceError
    || error instanceof StratumSocketEvidenceError
    || error instanceof ProtocolCoordinatorEvidenceError
    || error instanceof MiningCriteriaEvidenceError
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
    || error instanceof SdkconfigRollbackEvidenceError
    || error instanceof LogBufferEvidenceError
    || error instanceof PartitionLayoutEvidenceError
    || error instanceof UiWorkflowEvidenceError) {
    return error;
  }
  return undefined;
}

export function maybeTypedFailureCategory(error: unknown): AutomationCategory | undefined {
  return maybeTypedFailure(error)?.category;
}

export function maybeTypedFailurePublicValue(error: unknown): Readonly<Record<string, unknown>> | undefined {
  return maybeTypedFailure(error)?.publicValue;
}
