import { validateNoiseDiagnosticProjection } from "./stratum-v2-noise-diagnostic-validator.js";

const [candidate, expectedSource, ordinal, extra] = process.argv.slice(2);
if (candidate === undefined || expectedSource === undefined || ordinal === undefined
  || !/^[1-9][0-9]*$/u.test(ordinal) || extra !== undefined) {
  process.exitCode = 2;
} else {
  try {
    await validateNoiseDiagnosticProjection(candidate, expectedSource, Number(ordinal));
    process.stdout.write('{"status":"accepted"}\n');
  } catch {
    process.exitCode = 1;
  }
}
