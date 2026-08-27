import {
  inspectNoiseDiagnosticPreflight,
  noiseDiagnosticFailureResult,
  noiseDiagnosticWorkspaceRoot,
  parseNoiseDiagnosticArgs,
  runNoiseDiagnostic,
} from "./stratum-v2-noise-diagnostic.js";

async function main(): Promise<number> {
  try {
    const [action, ...values] = process.argv.slice(2);
    const args = parseNoiseDiagnosticArgs(action, values);
    const workspace = noiseDiagnosticWorkspaceRoot();
    const result = action === "preflight"
      ? await inspectNoiseDiagnosticPreflight(workspace, args)
      : await runNoiseDiagnostic(workspace, args);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return 0;
  } catch (error) {
    process.stdout.write(`${JSON.stringify(noiseDiagnosticFailureResult(error))}\n`);
    return 1;
  }
}

process.exitCode = await main();
