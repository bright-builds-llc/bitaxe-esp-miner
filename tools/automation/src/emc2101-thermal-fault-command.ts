import { flashProgram, toolProgram } from "./cli-tools.js";
import { portFromDetectorOutput } from "./detector.js";
import { captureEmc2101ThermalFaultEvidence } from "./emc2101-thermal-fault-evidence.js";
import { optionValue, type ParsedInvocation } from "./invocation.js";
import type { ProcessPort } from "./process.js";

export async function captureEmc2101ThermalFaultFromInvocation(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
) {
  const detectorOutput = optionValue(invocation, "--detector-output");
  const port = await portFromDetectorOutput(root, detectorOutput);
  return captureEmc2101ThermalFaultEvidence(root, {
    privateRoot: optionValue(invocation, "--private-root"),
    packageManifest: optionValue(invocation, "--package-manifest"),
    wifiCredentials: optionValue(invocation, "--wifi-credentials"),
    detectorOutput,
    port,
    projection: optionValue(invocation, "--projection"),
    captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
  }, processPort, flashProgram(root), "git",
  toolProgram(root, "crates/bitaxe-automation-contracts/validate_system_info_evidence"),
  toolProgram(root, "crates/bitaxe-automation-contracts/validate_emc2101_thermal_fault_evidence"));
}
