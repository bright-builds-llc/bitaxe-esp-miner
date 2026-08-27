import { validateNoiseDiagnosticProjection } from "./stratum-v2-noise-diagnostic-validator.js";

const [candidate, expectedSource, extra] = process.argv.slice(2);
if (candidate === undefined || expectedSource === undefined || extra !== undefined) {
  process.exitCode = 2;
} else {
  try {
    await validateNoiseDiagnosticProjection(candidate, expectedSource);
    process.stdout.write('{"status":"accepted"}\n');
  } catch {
    process.exitCode = 1;
  }
}
