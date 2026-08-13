import { toolProgram } from "./cli-tools.js";
import { optionValue, type ParsedInvocation } from "./invocation.js";
import type { ProcessPort } from "./process.js";
import type { UiWorkflowEvidence } from "./ui-workflow-contracts.generated.js";
import { projectUiWorkflowEvidence } from "./ui-workflow-evidence.js";

/** Projects UI workflow evidence from the command's closed invocation surface. */
export async function projectUiWorkflowEvidenceFromInvocation(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
): Promise<UiWorkflowEvidence> {
  return projectUiWorkflowEvidence(root, {
    privateRoot: optionValue(invocation, "--private-root"),
    packageManifest: optionValue(invocation, "--package-manifest"),
    operatorSnapshotProjection: optionValue(invocation, "--operator-snapshot-projection"),
    browserAttestation: optionValue(invocation, "--browser-attestation"),
    projection: optionValue(invocation, "--projection"),
  }, processPort, "git", {
    operatorSnapshot: toolProgram(root,
      "crates/bitaxe-automation-contracts/validate_operator_snapshot_evidence"),
    settings: toolProgram(root,
      "crates/bitaxe-automation-contracts/validate_settings_patch_evidence"),
    log: toolProgram(root,
      "crates/bitaxe-automation-contracts/validate_log_buffer_evidence"),
    partition: toolProgram(root,
      "crates/bitaxe-automation-contracts/validate_partition_layout_evidence"),
    rollback: toolProgram(root,
      "crates/bitaxe-automation-contracts/validate_sdkconfig_rollback_evidence"),
    evidence: toolProgram(root,
      "crates/bitaxe-automation-contracts/validate_ui_workflow_evidence"),
  });
}
