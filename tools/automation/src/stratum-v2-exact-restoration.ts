import { chmod, lstat, mkdir, readFile, readdir, rename, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { restoreSelfTestSettings } from "./self-test-campaign-restoration.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import { validateRestorableInputs } from "./stratum-v2-campaign-settings.js";
import { sha256, type RestoreBundle } from "./stratum-v2-restore-model.js";
import { restoreRuntimeMatches } from "./stratum-v2-restore-admission.js";
import { validateRestoreReadiness } from "./stratum-v2-restore-validator.js";
import { fetchRuntimeObject, monitorRuntimeOrigin } from "./stratum-v2-runtime-admission.js";
import { validateValidatorChildReceipt } from "./stratum-v2-validator-child.js";

type JsonObject = Record<string, unknown>;
export type RestorationAction = "preflight" | "start" | "resume";
export type RestorationArgs = {
  readonly action: RestorationAction;
  readonly board: "205";
  readonly port: string;
  readonly packageManifest: string;
  readonly restoreBundle: string;
  readonly recoveryProjection: string;
  readonly campaignRoot: string;
  readonly wifiCredentials: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly plan: string;
  readonly redactEvidence: true;
};

const taskId = "task-str005-inactive-restoration-and-campaign-continuation";
const planRelative = "docs/parity/work-plans/20260826T135721Z-STR-005-INACTIVE-RESTORATION/PLAN.md";
const planSha256 = "14c7676fb26b6291a24d08d229bc38717691835978d61ae24fd8cff91736470a";
const recoveryBundleSha256 = "1d5e2e3b76489c36458f63f11bf28b399ea4cd6c2d45f8dab20ef060b03e18f4";
const campaignBackupSha256 = "ac3d28d451c466f4fc6bfdc40b327c891dac9f3eba644ce62a7f2a2276790631";
const recoveryPlanRelative =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";
const currentDeviceSource = "a11b579b62cb52a53bbf6072bde209d3eb3f17e2";
const expected = {
  packageManifest: "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  restoreBundle: "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  recoveryProjection:
    "docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json",
  campaignRoot: "scratch/str005-stratum-v2/attempt-004",
  wifiCredentials: "wifi-credentials.json",
  projection: "docs/parity/evidence/str005-exact-restoration/restoration-projection-remediation-004.json",
  preflightRoot: "scratch/str005-exact-restoration/preflight-004",
  effectRoot: "scratch/str005-exact-restoration/remediation-004",
} as const;

export class ExactRestorationError extends Error {
  public constructor(
    public readonly category: "invalid_invocation" | "evidence_invalid" | "hardware_blocked" | "timeout",
    public readonly checkpoint: string,
  ) {
    super("exact restoration failed");
    this.name = "ExactRestorationError";
  }
}

function fail(category: ExactRestorationError["category"], checkpoint: string): never {
  throw new ExactRestorationError(category, checkpoint);
}

export function parseExactRestorationArgs(values: readonly string[]): RestorationArgs {
  const action = values[0];
  if (!["preflight", "start", "resume"].includes(action ?? "")) fail("invalid_invocation", "invocation");
  const options = new Map<string, string | true>();
  for (let index = 1; index < values.length; index += 1) {
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
    action: action as RestorationAction,
    board: required("--board"),
    port: required("--port"),
    packageManifest: required("--package-manifest"),
    restoreBundle: required("--restore-bundle"),
    recoveryProjection: required("--recovery-projection"),
    campaignRoot: required("--campaign-root"),
    wifiCredentials: required("--wifi-credentials"),
    privateRoot: required("--private-root"),
    projection: required("--projection"),
    plan: required("--plan"),
    redactEvidence: options.get("--redact-evidence"),
  };
  const root = parsed.action === "preflight" ? expected.preflightRoot : expected.effectRoot;
  if (parsed.board !== "205" || parsed.redactEvidence !== true || parsed.privateRoot !== root
    || parsed.packageManifest !== expected.packageManifest || parsed.restoreBundle !== expected.restoreBundle
    || parsed.recoveryProjection !== expected.recoveryProjection || parsed.campaignRoot !== expected.campaignRoot
    || parsed.wifiCredentials !== expected.wifiCredentials || parsed.projection !== expected.projection
    || parsed.plan !== planRelative || options.size !== 11) fail("invalid_invocation", "invocation");
  return { ...parsed, board: "205", redactEvidence: true };
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const value = await lstat(candidate);
  if (value.isSymbolicLink()
    || (directory ? !value.isDirectory() : !value.isFile())
    || (value.mode & 0o777) !== mode) {
    fail("evidence_invalid", "protected_mode");
  }
}

export async function createProtectedPrivateRoot(privateRoot: string): Promise<void> {
  const parent = path.dirname(privateRoot);
  try {
    await mkdir(parent, { mode: 0o700 });
    await chmod(parent, 0o700);
  } catch (error) {
    if (!(error instanceof Error && "code" in error && error.code === "EEXIST")) throw error;
    await requireMode(parent, 0o700, true);
  }
  await mkdir(privateRoot, { mode: 0o700 });
  await chmod(privateRoot, 0o700);
}

async function writePrivate(candidate: string, value: unknown): Promise<string> {
  const document = `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(candidate, document, { mode: 0o600, flag: "wx" });
  await chmod(candidate, 0o600);
  return document;
}

async function replacePrivate(candidate: string, value: unknown): Promise<void> {
  const temporary = `${candidate}.tmp`;
  await writePrivate(temporary, value);
  await rename(temporary, candidate);
  await chmod(candidate, 0o600);
}

async function git(workspace: string, args: readonly string[]): Promise<string> {
  const result = await runCampaignProcess(workspace, "git", args, 10_000);
  if (result.exitCode !== 0) fail("evidence_invalid", "source_identity");
  return result.stdout.trim();
}

async function poolInput(workspace: string): Promise<string> {
  const candidates = (await readdir(workspace))
    .filter(value => /^pool-credentials(?:-[A-Za-z0-9_-]+)?\.json$/u.test(value));
  if (candidates.length !== 1 || candidates[0] === undefined) fail("hardware_blocked", "pool_input");
  const candidate = path.join(workspace, candidates[0]);
  await requireMode(candidate, 0o600, false);
  return candidate;
}

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) fail("evidence_invalid", "contract");
  return value as JsonObject;
}

async function commonAdmission(workspace: string, args: RestorationArgs, privateRoot: string) {
  const source = await git(workspace, ["rev-parse", "HEAD"]);
  const status = await git(workspace, ["status", "--porcelain"]);
  const sync = await git(workspace, ["rev-list", "--left-right", "--count", "HEAD...@{u}"]);
  if (status !== "" || sync !== "0\t0") fail("evidence_invalid", "source_identity");
  const plan = await readFile(path.join(workspace, planRelative), "utf8");
  const tasks = await readFile(path.join(workspace, "TASKS.md"), "utf8");
  if (sha256(plan) !== planSha256 || !tasks.includes(`### ${taskId}`)) fail("evidence_invalid", "plan_binding");
  const manifest = object(JSON.parse(await readFile(path.join(workspace, args.packageManifest), "utf8")));
  if (manifest["source_commit"] !== source) fail("evidence_invalid", "package_identity");
  await requireMode(path.join(workspace, args.wifiCredentials), 0o600, false);
  await requireMode(path.join(workspace, args.campaignRoot), 0o700, true);
  await requireMode(path.join(workspace, args.campaignRoot, "settings-backup.private.json"), 0o600, false);
  const poolPath = await poolInput(workspace);
  const bundlePath = path.join(workspace, args.restoreBundle);
  const bundleDocument = await readFile(bundlePath, "utf8");
  if (sha256(bundleDocument) !== recoveryBundleSha256) fail("evidence_invalid", "bundle_binding");
  const bundle = JSON.parse(bundleDocument) as RestoreBundle;
  const recoveryPlan = await readFile(path.join(workspace, recoveryPlanRelative), "utf8");
  await validateRestoreReadiness(
    bundlePath, path.join(workspace, args.recoveryProjection),
    bundle.capture_source_commit, sha256(recoveryPlan),
  );
  await validateValidatorChildReceipt(
    path.join(path.dirname(bundlePath), "validator-child-receipt.private.json"),
    bundle.capture_source_commit, sha256(recoveryPlan),
  );
  if (args.action === "resume") {
    await requireMode(privateRoot, 0o700, true);
  } else {
    await createProtectedPrivateRoot(privateRoot);
  }
  const authorization = {
    schema_version: "bitaxe-stratum-v2-restore-authorization-v1",
    board: 205, ordinal: 4, action: args.action === "preflight" ? "preflight" : "start",
    current_source_commit: source,
    reference_commit: manifest["reference_commit"],
    bundle_sha256: sha256(bundleDocument),
    bundle_capture_source_commit: bundle.capture_source_commit,
    recovery_plan_sha256: sha256(recoveryPlan), remediation_plan_sha256: planSha256,
  };
  const authorizationPath = path.join(privateRoot, "restore-authorization.private.json");
  if (args.action === "resume") {
    await requireMode(authorizationPath, 0o600, false);
    const retainedAuthorization = object(JSON.parse(await readFile(authorizationPath, "utf8")));
    if (JSON.stringify(retainedAuthorization) !== JSON.stringify(authorization)) {
      fail("evidence_invalid", "restore_authorization");
    }
  } else {
    await writePrivate(authorizationPath, authorization);
  }
  const backupDocument = await readFile(
    path.join(workspace, args.campaignRoot, "settings-backup.private.json"), "utf8",
  );
  if (sha256(backupDocument) !== campaignBackupSha256) fail("evidence_invalid", "backup_binding");
  const backup = object(JSON.parse(backupDocument));
  await validateRestorableInputs(
    object(backup["settings"]), path.join(workspace, args.wifiCredentials), poolPath,
    (category) => fail(category === "evidence_invalid" ? "evidence_invalid" : "hardware_blocked", "settings_inputs"),
  );
  return { source, manifest, bundle, backup, poolPath, authorization };
}

async function currentRuntime(workspace: string, port: string, receipt: string, source: string) {
  const failRuntime = (category: string, _message: string, checkpoint: string): never =>
    fail(category === "evidence_invalid" ? "evidence_invalid" : "hardware_blocked", checkpoint);
  const origin = await monitorRuntimeOrigin(
    workspace, path.join(workspace, "bazel-bin/tools/flash/flash"), port,
    runCampaignProcess, failRuntime,
    { receiptPath: receipt, sourceCommit: source, planSha256 },
  );
  return { origin, info: await fetchRuntimeObject(origin, "/api/system/info", failRuntime) };
}

async function requireFreshDetector(
  workspace: string,
  privateRoot: string,
  port: string,
  receiptName: string,
): Promise<void> {
  const result = await runCampaignProcess(
    workspace,
    path.join(workspace, "bazel-bin/tools/flash/flash"),
    ["detect"],
    120_000,
  );
  await writePrivate(path.join(privateRoot, receiptName), {
    exit_code: result.exitCode,
    stdout: result.stdout,
    stderr: result.stderr,
  });
  const selectedPort = `port: ${port}`;
  if (result.exitCode !== 0
    || !result.stdout.split("\n").includes(selectedPort)
    || !result.stdout.split("\n").includes("usb_session: ready")) {
    fail("hardware_blocked", "detector");
  }
}

function safeCurrent(info: JsonObject): boolean {
  return info["sourceCommit"] === currentDeviceSource && info["startMiningOnBoot"] === false
    && ["paused", "safe_blocked"].includes(String(info["miningActivity"] ?? ""))
    && Number(info["hashRate"] ?? 0) === 0
    && Number(info["sharesAccepted"] ?? 0) === 0 && Number(info["sharesRejected"] ?? 0) === 0;
}

export async function runExactRestoration(workspace: string, args: RestorationArgs): Promise<JsonObject> {
  const privateRoot = path.join(workspace, args.privateRoot);
  if (args.action !== "resume") {
    try {
      await stat(privateRoot);
      fail("evidence_invalid", "outputs_absent");
    } catch (error) {
      if (error instanceof ExactRestorationError) throw error;
      if (!(error instanceof Error && "code" in error && error.code === "ENOENT")) throw error;
    }
  }
  if (args.action === "resume") return resumeRestoration(workspace, args, privateRoot);
  const admitted = await commonAdmission(workspace, args, privateRoot);
  await requireFreshDetector(workspace, privateRoot, args.port, "detector.private.json");
  const baseline = await currentRuntime(
    workspace, args.port, path.join(privateRoot, "baseline-monitor.private.json"), admitted.source,
  );
  if (!safeCurrent(baseline.info)) fail("hardware_blocked", "current_runtime");
  const statePath = path.join(privateRoot, "state.private.json");
  await writePrivate(statePath, { schema_version: "bitaxe-stratum-v2-restoration-state-v1", stage: "pre_effect_ready" });
  const flashArgs = [
    "restore-installed", "--board", "205", "--port", args.port,
    "--restore-bundle", args.restoreBundle,
    "--restore-authorization", path.join(args.privateRoot, "restore-authorization.private.json"),
    "--remediation-plan", args.plan, "--private-root", args.privateRoot,
    "--wifi-credentials", args.wifiCredentials, "--redact-evidence",
  ];
  if (args.action === "preflight") {
    flashArgs.push("--admission-only");
    const result = await runCampaignProcess(workspace, path.join(workspace, "bazel-bin/tools/flash/flash"), flashArgs, 120_000);
    if (result.exitCode !== 0) fail("evidence_invalid", "restore_admission");
    return { checkpoint: "restoration_pre_effect_ready", effect_started: false };
  }
  await replacePrivate(statePath, { schema_version: "bitaxe-stratum-v2-restoration-state-v1", stage: "flash_started" });
  const restored = await runCampaignProcess(
    workspace, path.join(workspace, "bazel-bin/tools/flash/flash"), flashArgs, 900_000,
  );
  await writePrivate(path.join(privateRoot, "restore-child.private.json"), {
    exit_code: restored.exitCode,
    stdout_sha256: sha256(restored.stdout),
    stderr_sha256: sha256(restored.stderr),
  });
  if (restored.exitCode !== 0) {
    await requireFreshDetector(
      workspace,
      privateRoot,
      args.port,
      "restore-cleanup.private.json",
    );
    fail("hardware_blocked", "snapshot_restore");
  }
  const original = await currentRuntime(
    workspace, args.port, path.join(privateRoot, "original-monitor.private.json"), admitted.source,
  );
  if (!restoreRuntimeMatches(admitted.bundle, original.info)) fail("hardware_blocked", "original_runtime");
  await replacePrivate(statePath, { schema_version: "bitaxe-stratum-v2-restoration-state-v1", stage: "firmware_restored" });
  return finishSettings(workspace, args, admitted, original.origin, statePath);
}

async function finishSettings(
  workspace: string, args: RestorationArgs,
  admitted: Awaited<ReturnType<typeof commonAdmission>>, origin: URL, statePath: string,
): Promise<JsonObject> {
  await restoreSelfTestSettings(
    origin, admitted.backup, path.join(workspace, args.wifiCredentials), admitted.poolPath,
  );
  await replacePrivate(statePath, {
    schema_version: "bitaxe-stratum-v2-restoration-state-v1",
    stage: "settings_restored",
  });
  const failFinal = (category: string, _message: string, checkpoint: string): never =>
    fail(category === "evidence_invalid" ? "evidence_invalid" : "hardware_blocked", checkpoint);
  const confirmed = await fetchRuntimeObject(origin, "/api/system/info", failFinal);
  await validateRestorableInputs(
    confirmed, path.join(workspace, args.wifiCredentials), admitted.poolPath,
    (category) => fail(category === "evidence_invalid" ? "evidence_invalid" : "hardware_blocked", "settings_final"),
  );
  if (!restoreRuntimeMatches(admitted.bundle, confirmed)
    || confirmed["referenceCommit"] !== admitted.bundle.installed_identity.reference_commit
    || !["paused", "safe_blocked"].includes(String(confirmed["miningActivity"] ?? ""))
    || Number(confirmed["hashRate"] ?? 0) !== 0
    || Number(confirmed["sharesAccepted"] ?? 0) !== 0
    || Number(confirmed["sharesRejected"] ?? 0) !== 0) {
    fail("hardware_blocked", "final_runtime");
  }
  await replacePrivate(statePath, { schema_version: "bitaxe-stratum-v2-restoration-state-v1", stage: "complete" });
  const projection = {
    schema_version: "bitaxe-stratum-v2-exact-restoration-v2", status: "accepted", board: 205,
    remediation_ordinal: 4, original_runtime_restored: true, settings_restored: true,
    theme_restored: true, mineonboot_false: true, mining_inactive: true,
    mining_activity_category: String(confirmed["miningActivity"]), zero_hashrate: true,
    zero_shares: true, read_only_finalization: false,
    usb_cleanup_ready: true, redaction_status: "passed",
    source_commit: admitted.source,
  };
  const candidate = path.join(workspace, `${args.projection}.candidate`);
  await mkdir(path.dirname(candidate), { recursive: true });
  await writeFile(candidate, `${JSON.stringify(projection, null, 2)}\n`, { mode: 0o600, flag: "wx" });
  const validation = await runCampaignProcess(
    workspace,
    process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath,
    [
      fileURLToPath(new URL("./stratum-v2-exact-restoration-validator-cli.js", import.meta.url)),
      candidate,
      admitted.source,
    ],
    10_000,
  );
  if (validation.exitCode !== 0) fail("evidence_invalid", "projection_validation");
  await rename(candidate, path.join(workspace, args.projection));
  return projection;
}

async function resumeRestoration(workspace: string, args: RestorationArgs, privateRoot: string) {
  await requireMode(privateRoot, 0o700, true);
  const statePath = path.join(privateRoot, "state.private.json");
  const state = object(JSON.parse(await readFile(statePath, "utf8")));
  if (state["stage"] !== "firmware_restored") fail("evidence_invalid", "resume_state");
  const admitted = await commonAdmission(workspace, args, privateRoot);
  await requireFreshDetector(workspace, privateRoot, args.port, "resume-detector.private.json");
  const original = await currentRuntime(
    workspace, args.port, path.join(privateRoot, "resume-monitor.private.json"), admitted.source,
  );
  if (!restoreRuntimeMatches(admitted.bundle, original.info)) fail("hardware_blocked", "original_runtime");
  return finishSettings(workspace, args, admitted, original.origin, statePath);
}
