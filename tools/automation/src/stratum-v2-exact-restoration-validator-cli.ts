import { validateExactRestorationProjection } from "./stratum-v2-exact-restoration-validator.js";

const [candidate, source] = process.argv.slice(2);
if (candidate === undefined || source === undefined) process.exitCode = 2;
else {
  try {
    await validateExactRestorationProjection(candidate, source);
    process.stdout.write("exact_restoration_projection=accepted\n");
  } catch {
    process.stdout.write("exact_restoration_projection=rejected\n");
    process.exitCode = 1;
  }
}
