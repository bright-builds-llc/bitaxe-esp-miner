import {
  inspectNoiseDiagnosticPreflight,
  noiseDiagnosticFailureResult,
  noiseDiagnosticWorkspaceRoot,
  parseNoiseDiagnosticArgs,
  runNoiseDiagnostic,
} from "./stratum-v2-noise-diagnostic.js";
import { finalizeNoiseAuthDiagnostic } from "./stratum-v2-noise-finalize.js";
import { runNoiseAuthRecovery } from "./stratum-v2-noise-recovery.js";

async function main(): Promise<number> {
  try {
    const [action, ...values] = process.argv.slice(2);
    const args = parseNoiseDiagnosticArgs(action, values);
    const workspace = noiseDiagnosticWorkspaceRoot();
    const result = action === "finalize"
      ? await finalizeNoiseAuthDiagnostic(workspace, args)
      : action === "preflight"
        ? await inspectNoiseDiagnosticPreflight(workspace, args)
        : action === "recover"
          ? await runNoiseAuthRecovery(workspace, args)
          : await runNoiseDiagnostic(workspace, args);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return 0;
  } catch (error) {
    process.stdout.write(`${JSON.stringify(noiseDiagnosticFailureResult(error))}\n`);
    return 1;
  }
}

process.exitCode = await main();
