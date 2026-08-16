import { flashProgram, toolProgram } from "./cli-tools.js";
import { portFromDetectorOutput } from "./detector.js";
import { optionValue, type ParsedInvocation } from "./invocation.js";
import type { ProcessPort } from "./process.js";
import { captureStatisticsHistoryEvidence } from "./statistics-history-evidence.js";

export async function captureStatisticsHistoryEvidenceFromInvocation(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
) {
  const detectorOutput = optionValue(invocation, "--detector-output");
  const port = await portFromDetectorOutput(root, detectorOutput);
  return captureStatisticsHistoryEvidence(root, {
    privateRoot: optionValue(invocation, "--private-root"),
    packageManifest: optionValue(invocation, "--package-manifest"),
    wifiCredentials: optionValue(invocation, "--wifi-credentials"),
    detectorOutput,
    port,
    projection: optionValue(invocation, "--projection"),
    captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
  }, processPort, flashProgram(root), "git",
  toolProgram(root, "crates/bitaxe-automation-contracts/validate_statistics_history_evidence"));
}
