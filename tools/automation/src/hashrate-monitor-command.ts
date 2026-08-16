import { captureHashrateMonitorEvidence } from "./hashrate-monitor-evidence.js";
import { flashProgram, toolProgram } from "./cli-tools.js";
import { portFromDetectorOutput } from "./detector.js";
import { optionValue, type ParsedInvocation } from "./invocation.js";
import type { ProcessPort } from "./process.js";

export async function captureHashrateMonitorEvidenceFromInvocation(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
) {
  const detectorOutput = optionValue(invocation, "--detector-output");
  const port = await portFromDetectorOutput(root, detectorOutput);
  return captureHashrateMonitorEvidence(root, {
    privateRoot: optionValue(invocation, "--private-root"),
    packageManifest: optionValue(invocation, "--package-manifest"),
    wifiCredentials: optionValue(invocation, "--wifi-credentials"),
    poolCredentials: optionValue(invocation, "--pool-credentials"),
    detectorOutput,
    port,
    projection: optionValue(invocation, "--projection"),
    durationSeconds: Number(optionValue(invocation, "--duration-seconds")),
    captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
  }, processPort, flashProgram(root), "git",
  toolProgram(root, "crates/bitaxe-automation-contracts/validate_hashrate_monitor_evidence"));
}
