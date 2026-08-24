import { validateRestoreReadiness } from "./stratum-v2-restore-validator.js";

const [bundle, projection, sourceCommit, planSha256] = process.argv.slice(2);
if (bundle === undefined || projection === undefined || sourceCommit === undefined || planSha256 === undefined) {
  process.stderr.write("restore validator requires bundle projection source and plan\n");
  process.exitCode = 2;
} else {
  try {
    await validateRestoreReadiness(bundle, projection, sourceCommit, planSha256);
    process.stdout.write("restore_readiness=accepted\n");
  } catch {
    process.stdout.write("restore_readiness=rejected\n");
    process.exitCode = 1;
  }
}
