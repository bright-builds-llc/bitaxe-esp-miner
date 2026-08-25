import { campaignWorkspaceRoot } from "./stratum-v2-campaign.js";
import {
  parseRuntimeMonitorDiagnosticArgs,
  runRuntimeMonitorDiagnostic,
  RuntimeMonitorDiagnosticError,
} from "./stratum-v2-runtime-monitor-diagnostic.js";

try {
  const workspace = campaignWorkspaceRoot();
  const args = parseRuntimeMonitorDiagnosticArgs(process.argv.slice(2));
  await runRuntimeMonitorDiagnostic(workspace, args);
  process.stdout.write(`${JSON.stringify({
    schema_version: "bitaxe-stratum-v2-runtime-monitor-diagnostic-v1",
    status: "succeeded",
    category: "complete",
    checkpoint: "runtime_monitor_ready",
    receipt_recorded: true,
  })}\n`);
} catch (error) {
  const category = error instanceof RuntimeMonitorDiagnosticError
    ? error.category
    : "evidence_invalid";
  const checkpoint = error instanceof RuntimeMonitorDiagnosticError
    ? error.checkpoint
    : "unclassified";
  process.stdout.write(`${JSON.stringify({
    schema_version: "bitaxe-stratum-v2-runtime-monitor-diagnostic-v1",
    status: "failed",
    category,
    checkpoint,
    receipt_recorded: false,
  })}\n`);
  process.exitCode = 1;
}
