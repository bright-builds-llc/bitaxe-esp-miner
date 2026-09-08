import { mkdir, readFile } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { authorityCall } from "./authority.mjs";
import { loadAmendment } from "./amendment.mjs";
import { verifyRetainedRuntime } from "./runtime-source.mjs";
import { admitTrust, BUNDLE, canonicalBase64, canonicalDirectory, cleanPushed, digest, fileDigest,
  ignored, missing, nonce, packageSnapshot, PAGE, protectedPath, readJson, requireCondition,
  REQUIRED_CYCLES, WINDOW_MS, writeNew } from "./contract.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export async function inspectSources(options, operations = {}) {
  const checkRepo = operations.cleanPushed ?? cleanPushed;
  checkRepo(options.firmwareRoot, options.firmwareCommit);
  checkRepo(options.gateRoot, options.gateCommit);
  await requireActiveTasks(options.firmwareRoot);
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
    gate_bundle_sha256: digest(bundle), gate_page_relative_path: PAGE,
    gate_page_sha256: await fileDigest(resolve(options.gateRoot, PAGE)),
    trust_sha256: await fileDigest(trustPath), authority_trust_sha256: digest(JSON.stringify(authorityTrust)),
    supervisor_client_sha256: await fileDigest(resolve(SCRIPT_ROOT, "client.mjs")) };
}

async function requireActiveTasks(firmwareRoot) {
  const tasks = await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8");
  for (const task of ["task-fixed-usb-serial-qualification", "task-fixed-usb-worker-live-acceptance"]) {
    requireCondition(activeTask(tasks, task), "active_task_missing");
  }
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
    required_no_mining_cycles: REQUIRED_CYCLES,
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

export async function loadContext(root, operations = {}) {
  await protectedPath(root, true);
  await protectedPath(resolve(root, "context.json"));
  const record = await readJson(resolve(root, "context.json"));
  requireCondition(record.sha256 === digest(JSON.stringify(record.context)), "context_integrity");
  if (["fixed-usb-iterative-context-v1", "fixed-usb-iterative-context-v2", "fixed-usb-iterative-context-v3", "fixed-usb-iterative-context-v4"].includes(record.context?.schema)) {
    const { validateIterativeContext } = await import("./iterative-preflight.mjs");
    await validateIterativeContext(root, record.context);
    return record.context;
  }
  requireCondition(record.context?.schema === "fixed-usb-qualification-context-v1" && canonicalBase64(record.context.campaign_id, 16) &&
    JSON.stringify(record.context.window_limits_ms) === JSON.stringify(WINDOW_MS), "context_integrity");
  await loadAmendment(root, record.context, operations);
  return record.context;
}

export async function verifyFrozen(context, authorityDirectory, bun, operations = {}, root) {
  if (context.schema === "fixed-usb-iterative-context-v4") {
    requireCondition(typeof root === "string", "retained_root_required");
    await requireActiveTasks(context.firmware_root);
    return verifyRetainedRuntime(root, context, authorityDirectory, bun, operations);
  }
  const maybeAmendment = root ? await loadAmendment(root, context, operations) : undefined;
  if (maybeAmendment) {
    await requireActiveTasks(context.firmware_root);
    const trustPath = resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json");
    requireCondition(await fileDigest(trustPath) === context.trust_sha256 &&
      await fileDigest(resolve(SCRIPT_ROOT, "client.mjs")) === context.supervisor_client_sha256, "frozen_source_drift");
    await protectedPath(authorityDirectory, true);
    const call = operations.authorityCall ?? authorityCall;
    const authorityTrust = await call(context.gate_root, authorityDirectory, "public-trust", undefined, bun);
    admitTrust(await readJson(trustPath), authorityTrust);
    requireCondition(digest(JSON.stringify(authorityTrust)) === context.authority_trust_sha256, "frozen_authority_drift");
    return { gate_root: maybeAmendment.gate_root, amendment_sha256: digest(JSON.stringify(maybeAmendment.policy)) };
  }
  const observed = await inspectSources({ firmwareRoot: context.firmware_root, gateRoot: context.gate_root,
    firmwareCommit: context.firmware_commit, gateCommit: context.gate_commit, manifest: context.manifest,
    authorityDirectory, bun }, operations);
  for (const [key, value] of Object.entries(observed)) {
    requireCondition(JSON.stringify(context[key]) === JSON.stringify(value), "frozen_source_drift");
  }
  return { gate_root: context.gate_root };
}

function activeTask(tasks, identifier) {
  let active = false, count = 0;
  for (const line of tasks.split(/\r?\n/u)) {
    if (line.startsWith("## ")) active = line === "## Active";
    if (active && line.startsWith("### ") && line.slice(4).split(/\s/u)[0] === identifier) count += 1;
  }
  return count === 1;
}
