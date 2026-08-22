import { readFile } from "node:fs/promises";

import { validateStratumV2CampaignProjection } from "./stratum-v2-campaign-validator.js";

async function main(): Promise<number> {
  const [projectionPath, source, reference, manifestSha256] = process.argv.slice(2);
  if (projectionPath === undefined || source === undefined || reference === undefined
    || manifestSha256 === undefined || process.argv.length !== 6) return 2;
  try {
    const value: unknown = JSON.parse(await readFile(projectionPath, "utf8"));
    validateStratumV2CampaignProjection(value, source, reference, manifestSha256);
    process.stdout.write("stratum_v2_campaign_validation=passed\n");
    return 0;
  } catch {
    process.stdout.write("stratum_v2_campaign_validation=failed\n");
    return 1;
  }
}

process.exitCode = await main();
