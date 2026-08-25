import { campaignWorkspaceRoot } from "./stratum-v2-campaign.js";
import {
  ExactRestorationError,
  parseExactRestorationArgs,
  runExactRestoration,
} from "./stratum-v2-exact-restoration.js";

try {
  const result = await runExactRestoration(
    campaignWorkspaceRoot(), parseExactRestorationArgs(process.argv.slice(2)),
  );
  process.stdout.write(`${JSON.stringify({ status: "succeeded", ...result })}\n`);
} catch (error) {
  process.stdout.write(`${JSON.stringify({
    status: "failed",
    category: error instanceof ExactRestorationError ? error.category : "evidence_invalid",
    checkpoint: error instanceof ExactRestorationError ? error.checkpoint : "unclassified",
    projection_published: false,
  })}\n`);
  process.exitCode = 1;
}
