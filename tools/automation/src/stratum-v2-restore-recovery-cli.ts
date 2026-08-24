import { campaignWorkspaceRoot } from "./stratum-v2-campaign.js";
import {
  recoverInstalledFirmware,
  parseRestoreRecoveryArgs,
  RestoreRecoveryError,
} from "./stratum-v2-restore-recovery.js";

try {
  const workspace = campaignWorkspaceRoot();
  const args = parseRestoreRecoveryArgs(process.argv.slice(2));
  const validator = `${workspace}/bazel-bin/tools/automation/stratum_v2_restore_validator_/stratum_v2_restore_validator`;
  const result = await recoverInstalledFirmware(workspace, args, validator);
  process.stdout.write(`${JSON.stringify(result)}\n`);
} catch (error) {
  const category = error instanceof RestoreRecoveryError ? error.category : "process_failed";
  const checkpoint = error instanceof RestoreRecoveryError ? error.checkpoint : "unclassified";
  process.stdout.write(`${JSON.stringify({
    schema_version: "bitaxe-stratum-v2-restore-result-v1",
    status: "failed",
    category,
    checkpoint,
    projection_published: false,
  })}\n`);
  process.exitCode = 1;
}
