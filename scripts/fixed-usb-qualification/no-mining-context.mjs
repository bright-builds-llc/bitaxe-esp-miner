import { inspectOriginalCampaign, readFrozenCampaign } from "./no-mining-accounting.mjs";
import { mkdir, readFile } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { admitTrust, BUNDLE, canonicalDirectory, cleanPushed, digest, exactObject, fileDigest, ignored,
  missing, packageSnapshot, PAGE, protectedPath, readJson, requireCondition, REQUIRED_CYCLES, writeNew } from "./contract.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export const NO_MINING_SCHEMA = "fixed-usb-no-mining-context-v1";
export async function inspectNoMiningSources(options, operations = {}) {
  const checkRepo = operations.cleanPushed ?? cleanPushed;
  checkRepo(options.firmwareRoot, options.firmwareCommit);
  checkRepo(options.gateRoot, options.gateCommit);
  const tasks = await readFile(resolve(options.firmwareRoot, "TASKS.md"), "utf8");
  let active = false, matches = 0;
  for (const line of tasks.split(/\r?\n/u)) {
    if (line.startsWith("## ")) active = line === "## Active";
    if (active && /^### task-fixed-usb-hello-resynchronization(?:\s|$)/u.test(line)) matches += 1;
  }
  requireCondition(matches === 1, "active_task_missing");
  const packaged = await packageSnapshot(options.firmwareRoot, options.manifest, options.firmwareCommit);
  const trustPath = resolve(options.firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json");
  const trust = await readJson(trustPath);
  // Validate only deployed public keys; this mode never opens a private authority.
  admitTrust(trust, trust);
  const bundle = await readFile(resolve(options.gateRoot, BUNDLE));
  requireCondition(bundle.includes(options.gateCommit), "gate_bundle_stale");
  return { ...packaged, firmware_commit: options.firmwareCommit, gate_commit: options.gateCommit,
    gate_bundle_sha256: digest(bundle), gate_page_relative_path: PAGE,
    gate_page_sha256: await fileDigest(resolve(options.gateRoot, PAGE)),
    trust_sha256: await fileDigest(trustPath),
    supervisor_client_sha256: await fileDigest(resolve(SCRIPT_ROOT, "no-mining-client.mjs")) };
}

export async function noMiningPreflight(options, operations = {}) {
  requireCondition(options.authorityDirectory === undefined && options.poolCredentials === undefined, "no_mining_credentials_forbidden");
  for (const key of ["firmwareRoot", "gateRoot"]) options[key] = await canonicalDirectory(options[key]);
  const root = resolve(options.privateRoot);
  await protectedPath(dirname(root), true);
  await missing(root);
  requireCondition(/^[a-z0-9][a-z0-9-]{0,95}$/u.test(basename(root)), "attempt_name");
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const snapshot = await inspectNoMiningSources(options, operations);
  const maybeCampaignRecord = options.originalCampaignRecord === undefined ? undefined : await inspectOriginalCampaign(options.originalCampaignRecord);
  const context = { schema: NO_MINING_SCHEMA, ...snapshot, required_no_mining_cycles: REQUIRED_CYCLES,
    mining_authorized: false, ...(maybeCampaignRecord ? { original_campaign_record: maybeCampaignRecord } : {}), firmware_root: options.firmwareRoot, gate_root: options.gateRoot, manifest: resolve(options.manifest) };
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  for (const suffix of ["stdout", "stderr"]) {
    await writeNew(resolve(dirname(root), `${basename(root)}.server.${suffix}.log`), { schema: "fixed-usb-no-mining-supervisor-log-v1" });
  }
  return { schema: "fixed-usb-no-mining-preflight-v1", ready: true, context_sha256: digest(JSON.stringify(context)),
    device_effects: false, mining_authorized: false };
}

export function validateNoMiningContext(context) {
  exactObject(context, ["schema", "manifest_sha256", "app_elf_sha256", "reference_commit", "artifacts", "update_segments",
    "firmware_commit", "gate_commit", "gate_bundle_sha256", "gate_page_relative_path", "gate_page_sha256", "trust_sha256",
    "supervisor_client_sha256", "required_no_mining_cycles", "mining_authorized", "firmware_root", "gate_root", "manifest"], ["original_campaign_record"]);
  requireCondition(context?.schema === NO_MINING_SCHEMA && context.mining_authorized === false &&
    context.required_no_mining_cycles === REQUIRED_CYCLES && context.campaign_id === undefined &&
    context.window_limits_ms === undefined && context.authority_directory === undefined && context.pool_credentials === undefined,
  "no_mining_context_integrity");
}

export async function verifyNoMiningFrozen(context, operations = {}) {
  validateNoMiningContext(context);
  if (context.original_campaign_record !== undefined) await readFrozenCampaign(context);
  const observed = await inspectNoMiningSources({ firmwareRoot: context.firmware_root, gateRoot: context.gate_root,
    firmwareCommit: context.firmware_commit, gateCommit: context.gate_commit, manifest: context.manifest }, operations);
  for (const [key, value] of Object.entries(observed)) {
    requireCondition(JSON.stringify(context[key]) === JSON.stringify(value), "frozen_source_drift");
  }
}
