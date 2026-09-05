import { mkdir, readFile } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { authorityCall } from "./authority.mjs";
import { admitTrust, BUNDLE, canonicalBase64, canonicalDirectory, cleanPushed, digest, fileDigest,
  ignored, missing, nonce, packageSnapshot, PAGE, protectedPath, readJson, requireCondition,
  WINDOW_MS, writeNew } from "./contract.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export async function inspectSources(options, operations = {}) {
  const checkRepo = operations.cleanPushed ?? cleanPushed;
  checkRepo(options.firmwareRoot, options.firmwareCommit);
  checkRepo(options.gateRoot, options.gateCommit);
  const tasks = await readFile(resolve(options.firmwareRoot, "TASKS.md"), "utf8");
  for (const task of ["task-fixed-usb-serial-qualification", "task-fixed-usb-worker-live-acceptance"]) {
    requireCondition(activeTask(tasks, task), "active_task_missing");
  }
  const packaged = await packageSnapshot(options.firmwareRoot, options.manifest, options.firmwareCommit);
  const trustPath = resolve(options.firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json");
  const trust = await readJson(trustPath);
  await protectedPath(options.authorityDirectory, true);
  const call = operations.authorityCall ?? authorityCall;
  const authorityTrust = await call(options.gateRoot, options.authorityDirectory, "public-trust", undefined, options.bun);
  admitTrust(trust, authorityTrust);
  const bundle = await readFile(resolve(options.gateRoot, BUNDLE));
  requireCondition(bundle.includes(options.gateCommit), "gate_bundle_stale");
  return { ...packaged, firmware_commit: options.firmwareCommit, gate_commit: options.gateCommit,
    gate_bundle_sha256: digest(bundle), gate_page_sha256: await fileDigest(resolve(options.gateRoot, PAGE)),
    trust_sha256: await fileDigest(trustPath), authority_trust_sha256: digest(JSON.stringify(authorityTrust)),
    supervisor_client_sha256: await fileDigest(resolve(SCRIPT_ROOT, "client.mjs")) };
}

export async function preflight(options, operations = {}) {
  for (const key of ["firmwareRoot", "gateRoot", "authorityDirectory"]) options[key] = await canonicalDirectory(options[key]);
  const root = resolve(options.privateRoot);
  await protectedPath(dirname(root), true);
  await missing(root);
  requireCondition(/^[a-z0-9][a-z0-9-]{0,95}$/u.test(basename(root)), "attempt_name");
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const snapshot = await inspectSources(options, operations);
  const campaignPath = resolve(dirname(root), "campaign.json");
  let campaign;
  try { campaign = await readJson(campaignPath); await protectedPath(campaignPath); }
  catch (error) {
    if (error.code !== "ENOENT") throw error;
    campaign = { schema: "fixed-usb-campaign-v1", campaign_id: nonce() };
    await writeNew(campaignPath, campaign);
  }
  requireCondition(campaign.schema === "fixed-usb-campaign-v1" && canonicalBase64(campaign.campaign_id, 16), "campaign_identity");
  const context = { schema: "fixed-usb-qualification-context-v1", ...snapshot,
    campaign_id: campaign.campaign_id, window_limits_ms: WINDOW_MS,
    firmware_root: options.firmwareRoot, gate_root: options.gateRoot, manifest: resolve(options.manifest) };
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  for (const suffix of ["stdout", "stderr"]) {
    const path = resolve(dirname(root), `${basename(root)}.server.${suffix}.log`);
    await missing(path);
    await writeNew(path, { schema: "fixed-usb-supervisor-log-v1" });
  }
  return { schema: "fixed-usb-preflight-v1", ready: true, context_sha256: digest(JSON.stringify(context)),
    device_effects: false, credential_timer_started: false };
}

export async function loadContext(root) {
  await protectedPath(root, true);
  await protectedPath(resolve(root, "context.json"));
  const record = await readJson(resolve(root, "context.json"));
  requireCondition(record.context?.schema === "fixed-usb-qualification-context-v1" &&
    record.sha256 === digest(JSON.stringify(record.context)) && canonicalBase64(record.context.campaign_id, 16), "context_integrity");
  return record.context;
}

export async function verifyFrozen(context, authorityDirectory, bun, operations = {}) {
  const observed = await inspectSources({ firmwareRoot: context.firmware_root, gateRoot: context.gate_root,
    firmwareCommit: context.firmware_commit, gateCommit: context.gate_commit, manifest: context.manifest,
    authorityDirectory, bun }, operations);
  for (const [key, value] of Object.entries(observed)) {
    requireCondition(JSON.stringify(context[key]) === JSON.stringify(value), "frozen_source_drift");
  }
}

function activeTask(tasks, identifier) {
  let active = false, count = 0;
  for (const line of tasks.split(/\r?\n/u)) {
    if (line.startsWith("## ")) active = line === "## Active";
    if (active && line.startsWith("### ") && line.slice(4).split(/\s/u)[0] === identifier) count += 1;
  }
  return count === 1;
}
