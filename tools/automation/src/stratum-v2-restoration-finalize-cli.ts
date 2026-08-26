import { campaignWorkspaceRoot } from "./stratum-v2-campaign.js";
import {
  finalizeRestoration,
  parseRestorationFinalizeArgs,
  RestorationFinalizeError,
} from "./stratum-v2-restoration-finalize.js";

try {
  const value = await finalizeRestoration(
    campaignWorkspaceRoot(), parseRestorationFinalizeArgs(process.argv.slice(2)),
  );
  process.stdout.write(`${JSON.stringify({ status: "succeeded", ...value })}\n`);
} catch (error) {
  process.stdout.write(`${JSON.stringify({
    status: "failed",
    category: error instanceof RestorationFinalizeError ? error.category : "evidence_invalid",
    checkpoint: error instanceof RestorationFinalizeError ? error.checkpoint : "unclassified",
    projection_published: false,
  })}\n`);
  process.exitCode = 1;
}
