import {
  noiseDiagnosticFailureResult,
  noiseDiagnosticWorkspaceRoot,
  parseNoiseDiagnosticArgs,
  requireReadOnlyNoiseAction,
} from "./stratum-v2-noise-diagnostic.js";
import { finalizeNoiseAuthDiagnostic } from "./stratum-v2-noise-finalize.js";

async function main(): Promise<number> {
  try {
    const [action, ...values] = process.argv.slice(2);
    requireReadOnlyNoiseAction(action);
    const args = parseNoiseDiagnosticArgs(action, values);
    const workspace = noiseDiagnosticWorkspaceRoot();
    const result = await finalizeNoiseAuthDiagnostic(workspace, args);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return 0;
  } catch (error) {
    process.stdout.write(`${JSON.stringify(noiseDiagnosticFailureResult(error))}\n`);
    return 1;
  }
}

process.exitCode = await main();
