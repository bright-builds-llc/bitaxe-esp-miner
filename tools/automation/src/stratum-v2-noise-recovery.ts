import { createHash } from "node:crypto";
import { chmod, lstat, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";

import { restoreSelfTestSettings } from "./self-test-campaign-restoration.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { fetchRuntimeObject, monitorRuntimeOrigin } from "./stratum-v2-runtime-admission.js";
import { admitStratumV2RestoreBundle, restoreRuntimeMatches } from "./stratum-v2-restore-admission.js";
import type { RestoreBundle } from "./stratum-v2-restore-model.js";
import { runManagedDiagnosticProcess } from "./stratum-v2-noise-diagnostic-process.js";
import {
  NoiseDiagnosticError,
  type NoiseDiagnosticArgs,
} from "./stratum-v2-noise-diagnostic.js";

const recoveryRoot = "scratch/str005-noise-auth/recovery-001";
const plan = "docs/parity/work-plans/20260829T143226Z-STR-005-NOISE-AUTH/PLAN.md";
const planSha256 = "9a3e5a630a52de6b8819dcb33aac64f5324df030fab50fd248fc33437b6587ea";
const restoreBundle =
  "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const packageManifest = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
const wifiCredentials = "wifi-credentials.json";
const recoveryPlan =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";
const backupRelative = "scratch/str005-stratum-v2/attempt-004/settings-backup.private.json";
const backupSha256 = "ac3d28d451c466f4fc6bfdc40b327c891dac9f3eba644ce62a7f2a2276790631";
const taskId = "task-str005-noise-auth-205";

type PreparedRecovery = {
  readonly head: string;
  readonly manifest: JsonObject;
  readonly manifestDocument: string;
  readonly wifiPath: string;
  readonly poolPath: string;
  readonly backup: JsonObject;
  readonly restoreBundle: RestoreBundle;
  readonly restoreBundlePath: string;
};

function fail(category: string, checkpoint: string): never {
  throw new NoiseDiagnosticError(category, checkpoint);
}

function object(value: unknown, checkpoint: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    fail("evidence_invalid", checkpoint);
  }
  return value as JsonObject;
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

async function requireMode(candidate: string, mode: number): Promise<void> {
  let metadata;
  try { metadata = await lstat(candidate); }
  catch { fail("evidence_invalid", "protected_inputs"); }
  if (metadata.isSymbolicLink() || !metadata.isFile() || (metadata.mode & 0o777) !== mode) {
    fail("evidence_invalid", "protected_inputs");
  }
}

async function requireAbsent(candidate: string): Promise<void> {
  try {
    await lstat(candidate);
    fail("evidence_invalid", "outputs_absent");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
    throw error;
  }
}

async function solePoolInput(workspace: string): Promise<string> {
  const candidates = (await readdir(workspace))
    .filter(name => /^pool-credentials(?:-[A-Za-z0-9_-]+)?\.json$/u.test(name))
    .map(name => path.join(workspace, name));
  if (candidates.length !== 1 || candidates[0] === undefined) {
    fail("hardware_blocked", "pool_restore_input");
  }
  await requireMode(candidates[0], 0o600);
  return candidates[0];
}

async function writePrivate(candidate: string, value: unknown): Promise<void> {
  const document = typeof value === "string" ? value : `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(candidate, document, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(candidate, 0o600);
}

async function prepareRecovery(
  workspace: string,
  args: NoiseDiagnosticArgs,
): Promise<PreparedRecovery> {
  if (args.action !== "recover" || args.privateRoot !== recoveryRoot
    || args.packageManifest !== packageManifest || args.wifiCredentials !== wifiCredentials
    || args.restoreBundle !== restoreBundle || args.plan !== plan
    || args.diagnosticOrdinal !== 1 || !args.redactEvidence) {
    fail("invalid_invocation", "invocation");
  }
  await requireAbsent(path.join(workspace, recoveryRoot));
  const ignored = await runCampaignProcess(workspace, "git", ["check-ignore", "-q", recoveryRoot], 5_000);
  if (ignored.exitCode !== 0) fail("evidence_invalid", "private_path_ignored");
  const [planDocument, tasks] = await Promise.all([
    readFile(path.join(workspace, plan), "utf8"),
    readFile(path.join(workspace, "TASKS.md"), "utf8"),
  ]);
  if (sha256(planDocument) !== planSha256 || !tasks.includes(`### ${taskId}`)) {
    fail("evidence_invalid", "source_identity");
  }
  const [headResult, status, sync] = await Promise.all([
    runCampaignProcess(workspace, "git", ["rev-parse", "HEAD"], 5_000),
    runCampaignProcess(workspace, "git", ["status", "--porcelain", "--untracked-files=all"], 5_000),
    runCampaignProcess(workspace, "git", ["rev-list", "--left-right", "--count", "HEAD...@{upstream}"], 5_000),
  ]);
  const head = headResult.stdout.trim();
  if (headResult.exitCode !== 0 || status.exitCode !== 0 || status.stdout.trim() !== ""
    || sync.exitCode !== 0 || sync.stdout.trim() !== "0\t0") {
    fail("evidence_invalid", "source_identity");
  }
  const manifestDocument = await readFile(path.join(workspace, packageManifest), "utf8");
  const manifest = object(JSON.parse(manifestDocument), "source_identity");
  if (manifest["schema_version"] !== 3 || manifest["source_commit"] !== head
    || manifest["reference_commit"] !== "c1915b0a63bfabebdb95a515cedfee05146c1d50") {
    fail("evidence_invalid", "source_identity");
  }
  const wifiPath = path.join(workspace, wifiCredentials);
  const poolPath = await solePoolInput(workspace);
  const backupPath = path.join(workspace, backupRelative);
  await requireMode(wifiPath, 0o600);
  await requireMode(backupPath, 0o600);
  const backupDocument = await readFile(backupPath, "utf8");
  if (sha256(backupDocument) !== backupSha256) fail("evidence_invalid", "restoration_inputs");
  const admitted = await admitStratumV2RestoreBundle(workspace, restoreBundle, runCampaignProcess);
  const detector = await runCampaignProcess(
    workspace,
    path.join(workspace, "bazel-bin/tools/flash/flash"),
    ["detect", "--board", "205", "--port", args.port],
    120_000,
  );
  if (detector.exitCode !== 0) fail("hardware_blocked", "device_admission");
  return {
    head,
    manifest,
    manifestDocument,
    wifiPath,
    poolPath,
    backup: object(JSON.parse(backupDocument), "restoration_inputs"),
    restoreBundle: admitted.bundle,
    restoreBundlePath: admitted.path,
  };
}

export async function runNoiseAuthRecovery(
  workspace: string,
  args: NoiseDiagnosticArgs,
): Promise<JsonObject> {
  const prepared = await prepareRecovery(workspace, args);
  const privateRoot = path.join(workspace, recoveryRoot);
  await mkdir(privateRoot, { mode: 0o700 });
  await chmod(privateRoot, 0o700);
  const restoreRootRelative = path.join(recoveryRoot, "restoration");
  await mkdir(path.join(workspace, restoreRootRelative), { mode: 0o700 });
  await chmod(path.join(workspace, restoreRootRelative), 0o700);
  const bundleDocument = await readFile(prepared.restoreBundlePath, "utf8");
  const recoveryPlanDocument = await readFile(path.join(workspace, recoveryPlan), "utf8");
  const authorizationRelative = path.join(
    restoreRootRelative,
    "restore-authorization.private.json",
  );
  await writePrivate(path.join(workspace, authorizationRelative), {
    schema_version: "bitaxe-stratum-v2-restore-authorization-v1",
    board: 205,
    ordinal: 1,
    action: "noise_auth_recovery",
    current_source_commit: prepared.head,
    reference_commit: prepared.manifest["reference_commit"],
    bundle_sha256: sha256(bundleDocument),
    bundle_capture_source_commit: prepared.restoreBundle.capture_source_commit,
    recovery_plan_sha256: sha256(recoveryPlanDocument),
    remediation_plan_sha256: planSha256,
  });
  const flashProgram = path.join(workspace, "bazel-bin/tools/flash/flash");
  const restored = await runManagedDiagnosticProcess(workspace, flashProgram, [
    "restore-installed", "--board", "205", "--port", args.port,
    "--restore-bundle", restoreBundle,
    "--restore-authorization", authorizationRelative,
    "--remediation-plan", plan,
    "--private-root", restoreRootRelative,
    "--wifi-credentials", wifiCredentials,
    "--redact-evidence",
  ], 900_000, "restoration_child");
  await writePrivate(path.join(privateRoot, "restore.stdout.private.log"), restored.stdout);
  await writePrivate(path.join(privateRoot, "restore.stderr.private.log"), restored.stderr);
  if (restored.exitCode !== 0) fail("hardware_blocked", "restoration");
  const origin = await monitorRuntimeOrigin(workspace, flashProgram, args.port, runCampaignProcess, fail);
  await restoreSelfTestSettings(origin, prepared.backup, prepared.wifiPath, prepared.poolPath);
  const confirmed = await fetchRuntimeObject(origin, "/api/system/info", fail);
  const identityExact = restoreRuntimeMatches(prepared.restoreBundle, confirmed);
  const miningInactive = ["paused", "safe_blocked"].includes(String(confirmed["miningActivity"] ?? ""));
  const zeroWork = Number(confirmed["hashRate"] ?? 0) === 0
    && Number(confirmed["sharesAccepted"] ?? 0) === 0
    && Number(confirmed["sharesRejected"] ?? 0) === 0;
  if (!identityExact || confirmed["startMiningOnBoot"] !== false || !miningInactive || !zeroWork) {
    fail("hardware_blocked", "runtime_confirmation");
  }
  const detector = await runCampaignProcess(
    workspace,
    flashProgram,
    ["detect", "--board", "205", "--port", args.port],
    120_000,
  );
  if (detector.exitCode !== 0) fail("hardware_blocked", "final_device_admission");
  const result = {
    schema_version: "bitaxe-stratum-v2-noise-auth-recovery-private-v1",
    source_commit: prepared.head,
    package_manifest_sha256: sha256(prepared.manifestDocument),
    restored_identity: true,
    settings_exact: true,
    mineonboot_disabled: true,
    mining_inactive: true,
    zero_work: true,
    cleanup_complete: true,
  };
  await writePrivate(path.join(privateRoot, "recovery-result.private.json"), result);
  return {
    ...result,
    schema_version: "bitaxe-stratum-v2-noise-auth-recovery-result-v1",
    status: "accepted",
    category: "complete",
  };
}
