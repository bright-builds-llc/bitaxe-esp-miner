import { toolProgram } from "./cli-tools.js";
import { projectCoreVoltageControlEvidence } from "./core-voltage-control-evidence.js";
import { projectDisplayBehaviorEvidence } from "./display-behavior-evidence.js";
import { optionValue, type ParsedInvocation } from "./invocation.js";
import type { ProcessPort } from "./process.js";

export function projectCoreVoltageControlEvidenceFromInvocation(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
) {
  return projectCoreVoltageControlEvidence(root, {
    sourceProjection: optionValue(invocation, "--source-projection"),
    attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
    projection: optionValue(invocation, "--projection"),
  }, processPort, "git", toolProgram(root,
    "crates/bitaxe-automation-contracts/validate_asic_power_initialization_evidence"),
  toolProgram(root, "crates/bitaxe-automation-contracts/validate_core_voltage_control_evidence"));
}

export function projectDisplayBehaviorEvidenceFromInvocation(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
) {
  return projectDisplayBehaviorEvidence(root, {
    sourceDisplayUat: optionValue(invocation, "--source-display-uat"),
    sourceCommandEffects: optionValue(invocation, "--source-command-effects"),
    attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
    projection: optionValue(invocation, "--projection"),
  }, processPort, "git", toolProgram(root,
    "crates/bitaxe-automation-contracts/validate_display_behavior_evidence"));
}
