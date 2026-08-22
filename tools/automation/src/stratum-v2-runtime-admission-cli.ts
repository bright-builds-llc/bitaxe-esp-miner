import {
  campaignWorkspaceRoot,
  inspectStratumV2RuntimeAdmission,
  parseStratumV2CampaignArgs,
  stratumV2CampaignFailureResult,
} from "./stratum-v2-campaign.js";

async function main(): Promise<number> {
  try {
    const args = parseStratumV2CampaignArgs(process.argv.slice(2));
    const result = await inspectStratumV2RuntimeAdmission(campaignWorkspaceRoot(), args);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return 0;
  } catch (error) {
    process.stdout.write(`${JSON.stringify(stratumV2CampaignFailureResult(error))}\n`);
    return 1;
  }
}

process.exitCode = await main();
