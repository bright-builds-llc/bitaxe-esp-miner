import path from "node:path";

import {
  parseStratumV2CampaignArgs,
  runStratumV2Campaign,
  StratumV2CampaignError,
} from "./stratum-v2-campaign.js";

async function main(): Promise<number> {
  try {
    const args = parseStratumV2CampaignArgs(process.argv.slice(2));
    const result = await runStratumV2Campaign(path.resolve(process.cwd()), args);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return 0;
  } catch (error) {
    const category = error instanceof StratumV2CampaignError ? error.category : "process_failed";
    process.stdout.write(`${JSON.stringify({
      schema_version: "bitaxe-stratum-v2-campaign-result-v1",
      status: "failed",
      category,
      projection_published: false,
    })}\n`);
    return 1;
  }
}

process.exitCode = await main();
