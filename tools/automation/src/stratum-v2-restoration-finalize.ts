import { chmod, lstat, mkdir, readFile, readdir, rename, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { validateRestoredSettingsAndTheme } from "./self-test-campaign-restoration.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import { validateRestorableInputs } from "./stratum-v2-campaign-settings.js";
import { restoreRuntimeMatches } from "./stratum-v2-restore-admission.js";
import { sha256, type RestoreBundle } from "./stratum-v2-restore-model.js";
import { fetchRuntimeObject, monitorRuntimeOrigin } from "./stratum-v2-runtime-admission.js";

type JsonObject = Record<string, unknown>;

export type FinalizeArgs = {
  readonly board: "205";
  readonly port: string;
  readonly packageManifest: string;
  readonly restoreBundle: string;
  readonly campaignRoot: string;
  readonly wifiCredentials: string;
  readonly remediationRoot: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly plan: string;
  readonly redactEvidence: true;
};

const taskId = "task-str005-inactive-restoration-and-campaign-continuation";
const planRelative = "docs/parity/work-plans/20260826T135721Z-STR-005-INACTIVE-RESTORATION/PLAN.md";
const planSha256 = "14c7676fb26b6291a24d08d229bc38717691835978d61ae24fd8cff91736470a";
const implementationSource = "e3bd08bb86489bb6a27a295937120eb089d00b50";
const bundleSha256 = "1d5e2e3b76489c36458f63f11bf28b399ea4cd6c2d45f8dab20ef060b03e18f4";
const backupSha256 = "ac3d28d451c466f4fc6bfdc40b327c891dac9f3eba644ce62a7f2a2276790631";
const oldPlanSha256 = "946ec6b353add5e2ef08fe9047640f9271a68556021ca92e47661f6393103c1a";
const recoveryPlanSha256 = "0328084c0157831b9d85ac6369777b48ed4fc32cb5d709c4fc3570e9fc373fdf";
const expected = {
  packageManifest: "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  restoreBundle: "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  campaignRoot: "scratch/str005-stratum-v2/attempt-004",
  wifiCredentials: "wifi-credentials.json",
  remediationRoot: "scratch/str005-exact-restoration/remediation-002",
  privateRoot: "scratch/str005-restoration-finalize/finalize-001",
  projection: "docs/parity/evidence/str005-exact-restoration/restoration-projection.json",
} as const;

export class RestorationFinalizeError extends Error {
  public constructor(
    public readonly category: "invalid_invocation" | "evidence_invalid" | "hardware_blocked",
    public readonly checkpoint: string,
  ) {
    super("restoration finalization failed");
    this.name = "RestorationFinalizeError";
  }
}

function fail(category: RestorationFinalizeError["category"], checkpoint: string): never {
  throw new RestorationFinalizeError(category, checkpoint);
}

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) fail("evidence_invalid", "contract");
  return value as JsonObject;
}

export function parseRestorationFinalizeArgs(values: readonly string[]): FinalizeArgs {
  const options = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") {
      if (options.has(key)) fail("invalid_invocation", "invocation");
      options.set(key, true);
      continue;
    }
    const value = values[index + 1];
    if (key === undefined || value === undefined || !key.startsWith("--") || value.startsWith("--")
      || options.has(key)) fail("invalid_invocation", "invocation");
    options.set(key, value);
    index += 1;
  }
  const required = (key: string): string => {
    const value = options.get(key);
    if (typeof value !== "string" || value.length === 0) fail("invalid_invocation", "invocation");
    return value;
  };
  const parsed = {
    board: required("--board"), port: required("--port"),
    packageManifest: required("--package-manifest"), restoreBundle: required("--restore-bundle"),
    campaignRoot: required("--campaign-root"), wifiCredentials: required("--wifi-credentials"),
    remediationRoot: required("--remediation-root"), privateRoot: required("--private-root"),
    projection: required("--projection"), plan: required("--plan"),
    redactEvidence: options.get("--redact-evidence"),
  };
  if (parsed.board !== "205" || parsed.redactEvidence !== true || options.size !== 11
    || parsed.packageManifest !== expected.packageManifest || parsed.restoreBundle !== expected.restoreBundle
    || parsed.campaignRoot !== expected.campaignRoot || parsed.wifiCredentials !== expected.wifiCredentials
    || parsed.remediationRoot !== expected.remediationRoot || parsed.privateRoot !== expected.privateRoot
    || parsed.projection !== expected.projection || parsed.plan !== planRelative) {
    fail("invalid_invocation", "invocation");
  }
  return { ...parsed, board: "205", redactEvidence: true };
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await lstat(candidate);
  if (metadata.isSymbolicLink() || (directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) fail("evidence_invalid", "protected_mode");
}

async function git(workspace: string, args: readonly string[]): Promise<string> {
  const result = await runCampaignProcess(workspace, "git", args, 10_000);
  if (result.exitCode !== 0) fail("evidence_invalid", "source_identity");
  return result.stdout.trim();
}

async function solePoolInput(workspace: string): Promise<string> {
  const names = (await readdir(workspace)).filter(name =>
    /^pool-credentials(?:-[A-Za-z0-9_-]+)?\.json$/u.test(name));
  if (names.length !== 1 || names[0] === undefined) fail("hardware_blocked", "pool_input");
  const candidate = path.join(workspace, names[0]);
  await requireMode(candidate, 0o600, false);
  return candidate;
}

function completedDiagnostic(value: JsonObject, executor: string): boolean {
  const diagnostic = object(value["diagnostic"]);
  return value["schema_version"] === "bitaxe-stratum-v2-restore-command-v1"
    && value["executor"] === executor && value["diagnostic_available"] === true
    && diagnostic["terminal_category"] === "ready"
    && diagnostic["device_effect_state"] === "completed"
    && diagnostic["termination"] === "exited_success" && diagnostic["attempt_count"] === 1
    && diagnostic["transfer_started"] === true && diagnostic["transfer_completed"] === true
    && diagnostic["raw_output_included"] === false;
}

async function requireDetector(workspace: string, root: string, port: string): Promise<void> {
  const result = await runCampaignProcess(
    workspace, path.join(workspace, "bazel-bin/tools/flash/flash"), ["detect"], 120_000,
  );
  await writeFile(path.join(root, "detector.private.json"), `${JSON.stringify({
    exit_code: result.exitCode,
    stdout_sha256: sha256(result.stdout),
    stderr_sha256: sha256(result.stderr),
    selected_port_present: result.stdout.split("\n").includes(`port: ${port}`),
    usb_cleanup_ready: result.stdout.split("\n").includes("usb_session: ready"),
  }, null, 2)}\n`, { mode: 0o600, flag: "wx" });
  if (result.exitCode !== 0 || !result.stdout.split("\n").includes(`port: ${port}`)
    || !result.stdout.split("\n").includes("usb_session: ready")) fail("hardware_blocked", "detector");
}

export async function finalizeRestoration(workspace: string, args: FinalizeArgs): Promise<JsonObject> {
  const source = await git(workspace, ["rev-parse", "HEAD"]);
  if (await git(workspace, ["status", "--porcelain"]) !== ""
    || await git(workspace, ["rev-list", "--left-right", "--count", "HEAD...@{u}"]) !== "0\t0") {
    fail("evidence_invalid", "source_identity");
  }
  const plan = await readFile(path.join(workspace, args.plan), "utf8");
  const tasks = await readFile(path.join(workspace, "TASKS.md"), "utf8");
  if (sha256(plan) !== planSha256 || !tasks.includes(`### ${taskId}`)) fail("evidence_invalid", "plan_binding");
  const manifest = object(JSON.parse(await readFile(path.join(workspace, args.packageManifest), "utf8")));
  if (manifest["source_commit"] !== source) fail("evidence_invalid", "package_identity");
  const root = path.join(workspace, args.privateRoot);
  try { await stat(root); fail("evidence_invalid", "outputs_absent"); }
  catch (error) {
    if (error instanceof RestorationFinalizeError) throw error;
    if (!(error instanceof Error && "code" in error && error.code === "ENOENT")) throw error;
  }
  await mkdir(path.dirname(root), { recursive: true, mode: 0o700 });
  await chmod(path.dirname(root), 0o700);
  await mkdir(root, { mode: 0o700 });
  await chmod(root, 0o700);

  const remediation = path.join(workspace, args.remediationRoot);
  await requireMode(remediation, 0o700, true);
  const names = ["state.private.json", "restore-authorization.private.json", "restore-child.private.json",
    "snapshot-write.private.json", "wifi-seed.private.json"] as const;
  for (const name of names) await requireMode(path.join(remediation, name), 0o600, false);
  const state = object(JSON.parse(await readFile(path.join(remediation, names[0]), "utf8")));
  const authorization = object(JSON.parse(await readFile(path.join(remediation, names[1]), "utf8")));
  const child = object(JSON.parse(await readFile(path.join(remediation, names[2]), "utf8")));
  const snapshot = object(JSON.parse(await readFile(path.join(remediation, names[3]), "utf8")));
  const wifiSeed = object(JSON.parse(await readFile(path.join(remediation, names[4]), "utf8")));
  if (state["stage"] !== "settings_restored" || child["exit_code"] !== 0
    || authorization["action"] !== "start" || authorization["ordinal"] !== 2
    || authorization["current_source_commit"] !== implementationSource
    || authorization["bundle_sha256"] !== bundleSha256
    || authorization["recovery_plan_sha256"] !== recoveryPlanSha256
    || authorization["remediation_plan_sha256"] !== oldPlanSha256
    || !completedDiagnostic(snapshot, "managed_esptool_write_flash")
    || !completedDiagnostic(wifiSeed, "espflash_write_bin")) fail("evidence_invalid", "remediation_receipts");

  const bundleDocument = await readFile(path.join(workspace, args.restoreBundle), "utf8");
  if (sha256(bundleDocument) !== bundleSha256) fail("evidence_invalid", "bundle_binding");
  const bundle = JSON.parse(bundleDocument) as RestoreBundle;
  const backupPath = path.join(workspace, args.campaignRoot, "settings-backup.private.json");
  await requireMode(backupPath, 0o600, false);
  const backupDocument = await readFile(backupPath, "utf8");
  if (sha256(backupDocument) !== backupSha256) fail("evidence_invalid", "backup_binding");
  const backup = object(JSON.parse(backupDocument));
  const wifiPath = path.join(workspace, args.wifiCredentials);
  await requireMode(wifiPath, 0o600, false);
  const poolPath = await solePoolInput(workspace);
  await requireDetector(workspace, root, args.port);
  const failRuntime = (category: string, _message: string, checkpoint: string): never =>
    fail(category === "evidence_invalid" ? "evidence_invalid" : "hardware_blocked", checkpoint);
  const origin = await monitorRuntimeOrigin(
    workspace, path.join(workspace, "bazel-bin/tools/flash/flash"), args.port,
    runCampaignProcess, failRuntime,
    { receiptPath: path.join(root, "monitor.private.json"), sourceCommit: source, planSha256 },
  );
  const confirmed = await fetchRuntimeObject(origin, "/api/system/info", failRuntime);
  const theme = await fetchRuntimeObject(origin, "/api/theme", failRuntime);
  const wifi = object(JSON.parse(await readFile(wifiPath, "utf8")));
  const pool = object(JSON.parse(await readFile(poolPath, "utf8")));
  validateRestoredSettingsAndTheme(confirmed, theme, backup, wifi, pool);
  await validateRestorableInputs(confirmed, wifiPath, poolPath, () => fail("hardware_blocked", "settings"));
  if (!restoreRuntimeMatches(bundle, confirmed) || confirmed["miningActivity"] !== "paused"
    || Number(confirmed["hashRate"] ?? 0) !== 0 || Number(confirmed["sharesAccepted"] ?? 0) !== 0
    || Number(confirmed["sharesRejected"] ?? 0) !== 0) fail("hardware_blocked", "runtime");

  const projection = {
    schema_version: "bitaxe-stratum-v2-exact-restoration-v2", status: "accepted", board: 205,
    remediation_ordinal: 2, original_runtime_restored: true, settings_restored: true,
    theme_restored: true, mineonboot_false: true, mining_inactive: true,
    mining_activity_category: "paused", zero_hashrate: true, zero_shares: true,
    read_only_finalization: true, usb_cleanup_ready: true, redaction_status: "passed",
    source_commit: source,
  };
  const candidate = path.join(workspace, `${args.projection}.candidate`);
  await mkdir(path.dirname(candidate), { recursive: true });
  await writeFile(candidate, `${JSON.stringify(projection, null, 2)}\n`, { mode: 0o600, flag: "wx" });
  const validation = await runCampaignProcess(workspace, process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath, [
    fileURLToPath(new URL("./stratum-v2-exact-restoration-validator-cli.js", import.meta.url)), candidate, source,
  ], 10_000);
  if (validation.exitCode !== 0) fail("evidence_invalid", "projection_validation");
  await rename(candidate, path.join(workspace, args.projection));
  return projection;
}
