import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";

export const cfg07ProductionFragments = new Map<string, readonly string[]>([
  ["tools/automation/src/invocation.ts", [
    '"capture-scoreboard-evidence": {\n    "--private-root": value({ required: true }),\n    "--package-manifest": value({ required: true }),\n    "--wifi-credentials": value({ required: true }),\n    "--pool-credentials": value({ required: true })',
  ]],
  ["tools/automation/src/scoreboard-evidence.ts", [
    '"--wifi-credentials", options.wifiCredentials,\n    "--pool-credentials", options.poolCredentials,\n    "--evidence-dir", campaignRoot,\n    "--duration-seconds", String(options.durationSeconds), "--redact-evidence"',
  ]],
  ["tools/automation/src/process.ts", [
    'if (/(?:TOKEN|PASSWORD|SECRET|CREDENTIAL|API_KEY)/iu.test(key)) continue;',
  ]],
  ["tools/flash/src/cli.rs", [
    'pub(crate) struct MiningCampaignCommand {',
    '#[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]\n    pub(crate) wifi_credentials: Utf8PathBuf,\n\n    #[arg(long = "pool-credentials", value_parser = parse_utf8_path)]\n    pub(crate) pool_credentials: Option<Utf8PathBuf>',
  ]],
  ["tools/flash/src/campaign.rs", [
    'let admission = match admit_campaign(command, environment)',
    'let nvs_seed = prepare_campaign_nvs_seed(command, admission, &port, environment)',
    'let nvs_result = environment.execute(&nvs_seed.command);',
  ]],
  ["tools/flash/src/campaign/admission.rs", [
    'MiningCampaignStage::LiveShare => {\n            command.profile == Some(MiningCampaignProfile::Conservative)\n                && command.pool_credentials.is_some()',
    'let wifi = read_wifi_credentials(&wifi_path, environment)',
    'read_pool_credentials(&pool_path, environment)',
    'set_private_file_mode(&csv_path)?;',
    'set_private_file_mode(&image_path)?;',
    '(_, None) => bail!("mining campaign pool credentials missing")',
  ]],
  ["tools/flash/src/wifi.rs", [
    'pub(crate) fn read_wifi_credentials(',
    'validate_wifi_credentials(file)',
  ]],
]);

export const cfg07EvaluatorFragments = new Map<string, readonly string[]>([
  ["tools/automation/src/cfg07-evidence.ts", ["export async function projectCfg07Evidence("]],
  ["tools/automation/src/cfg07-source-inventory.ts", []],
  ["tools/automation/src/cli.ts", ['invocation.command === "project-cfg07-evidence"']],
  ["tools/automation/src/contracts.generated.ts", [
    '| "project-cfg07-evidence"\n  | "capture-provisioning-network-evidence"',
  ]],
  ["crates/bitaxe-automation-contracts/src/cfg07_evidence.rs", ["pub struct Cfg07Evidence {"]],
  ["crates/bitaxe-automation-contracts/src/bin/validate_cfg07_evidence.rs", ["evidence.validate()?;"]],
  ["tools/automation/BUILD.bazel", ['"project_cfg07_evidence": "project-cfg07-evidence"']],
  ["crates/bitaxe-automation-contracts/BUILD.bazel", [
    'rust_binary(\n    name = "validate_cfg07_evidence",',
  ]],
]);

export const cfg07ReferenceFragments = new Map<string, readonly string[]>([
  ["reference/esp-miner/main/nvs_config.c", [
    '[NVS_CONFIG_WIFI_SSID]',
    '[NVS_CONFIG_WIFI_PASS]',
    '[NVS_CONFIG_STRATUM_URL]',
    '[NVS_CONFIG_STRATUM_USER]',
    '[NVS_CONFIG_STRATUM_PASS]',
  ]],
  ["reference/esp-miner/config-205.cvs", [
    "\nwifissid,data,string,",
    "\nwifipass,data,string,",
    "\nstratumurl,data,string,",
    "\nstratumuser,data,string,",
    "\nstratumpass,data,string,",
  ]],
]);

function verifyFragments(document: string, fragments: readonly string[], relative: string): void {
  for (const fragment of fragments) {
    if (document.split(fragment).length !== 2) {
      throw new Error(`CFG-07 source semantics are invalid for ${relative}`);
    }
  }
}

function semanticDigest(entries: ReadonlyMap<string, readonly string[]>): string {
  const digest = createHash("sha256");
  for (const [relative, fragments] of entries) {
    digest.update(relative).update("\0");
    for (const fragment of fragments) digest.update(fragment).update("\0");
  }
  return digest.digest("hex");
}

export async function cfg07CurrentInventory(workspaceRoot: string): Promise<{
  readonly digest: string;
  readonly productionSemanticDigest: string;
  readonly pathCount: number;
}> {
  const digest = createHash("sha256");
  for (const [relative, fragments] of [
    ...cfg07ProductionFragments,
    ...cfg07EvaluatorFragments,
    ...cfg07ReferenceFragments,
  ]) {
    const document = await readFile(path.join(workspaceRoot, relative));
    verifyFragments(document.toString("utf8"), fragments, relative);
    digest.update(relative).update("\0").update(document).update("\0");
  }
  return {
    digest: digest.digest("hex"),
    productionSemanticDigest: semanticDigest(cfg07ProductionFragments),
    pathCount: cfg07ProductionFragments.size
      + cfg07EvaluatorFragments.size
      + cfg07ReferenceFragments.size,
  };
}

export function cfg07AttemptProductionDigest(documents: ReadonlyMap<string, Buffer>): string {
  for (const [relative, fragments] of cfg07ProductionFragments) {
    const document = documents.get(relative);
    if (document === undefined) throw new Error("CFG-07 attempt source is incomplete");
    verifyFragments(document.toString("utf8"), fragments, relative);
  }
  return semanticDigest(cfg07ProductionFragments);
}
