import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, lstat, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import {
  completedRestoreCommandReceipt,
  requireNoOwnedUsbProcesses,
} from "./native-usb-transition-recovery.js";
import { admitStratumV2RestoreBundle } from "./stratum-v2-restore-admission.js";
import { sourceWorkspaceRoot } from "./workspace.js";

export type DisplayRecoveryAction = "preflight" | "capture" | "start" | "finalize";
export type DisplayRecoveryArgs = {
  readonly action: DisplayRecoveryAction;
  readonly port: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly poolCredentials: string;
  readonly restoreBundle: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly plan: string;
};

const plan = "docs/parity/work-plans/20260830T161148Z-NATIVE-USB-DISPLAY-RECOVERY/PLAN.md";
const planSha256 = "cba106c78f7a12105d64f185a5989ac445afe64f3479a917ac7cc95285196427";
const task = "task-native-usb-display-recovery-205";
const root = "scratch/native-usb-display-recovery/attempt-001";
const projection = "docs/parity/evidence/native-usb-display-recovery/recovery-projection-001.json";
const manifest = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
const bundle = "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const backup = "scratch/str005-stratum-v2/attempt-004/settings-backup.private.json";
const predecessor = "scratch/native-usb-transition/recovery-002";

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function fail(category: string): never { throw new Error(category); }
function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) fail("evidence_invalid");
  return value as JsonObject;
}
async function requireMode(candidate: string, mode: number, directory = false): Promise<void> {
  const metadata = await lstat(candidate);
  if (metadata.isSymbolicLink() || (directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) fail("protected_mode");
}
async function absent(candidate: string): Promise<boolean> {
  try { await lstat(candidate); return false; }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return true;
    throw error;
  }
}
async function writePrivate(candidate: string, value: unknown): Promise<void> {
  await writeFile(candidate, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx", mode: 0o600 });
  await chmod(candidate, 0o600);
}

export function parseDisplayRecoveryArgs(action: string | undefined, values: readonly string[]): DisplayRecoveryArgs {
  if (!(["preflight", "capture", "start", "finalize"] as const).includes(action as DisplayRecoveryAction)) {
    fail("invalid_invocation");
  }
  const options = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") { options.set(key, true); continue; }
    const value = values[index + 1];
    if (key === undefined || value === undefined || !key.startsWith("--") || value.startsWith("--")) fail("invalid_invocation");
    options.set(key, value); index += 1;
  }
  const value = (key: string) => typeof options.get(key) === "string" ? options.get(key) as string : fail("invalid_invocation");
  const args = {
    action: action as DisplayRecoveryAction,
    port: value("--port"),
    packageManifest: value("--package-manifest"),
    wifiCredentials: value("--wifi-credentials"),
    poolCredentials: value("--pool-credentials"),
    restoreBundle: value("--restore-bundle"),
    privateRoot: value("--private-root"),
    projection: value("--projection"),
    plan: value("--plan"),
  };
  if (value("--board") !== "205" || args.packageManifest !== manifest
    || args.wifiCredentials !== "wifi-credentials.json" || args.restoreBundle !== bundle
    || args.privateRoot !== root || args.projection !== projection || args.plan !== plan
    || options.get("--redact-evidence") !== true) fail("invalid_invocation");
  return args;
}

export function displayRecoveryWorkspaceRoot(environment = process.env, cwd = process.cwd()): string {
  const configured = environment["BUILD_WORKSPACE_DIRECTORY"];
  return sourceWorkspaceRoot(configured === undefined ? [cwd] : [configured, cwd]);
}

async function commonAdmission(workspace: string, args: DisplayRecoveryArgs, rootAbsent: boolean): Promise<void> {
  if ((await absent(path.join(workspace, root))) !== rootAbsent || !(await absent(path.join(workspace, projection)))) fail("outputs");
  if (!rootAbsent) await requireMode(path.join(workspace, root), 0o700, true);
  const [planDocument, tasks, head, status, sync, manifestDocument] = await Promise.all([
    readFile(path.join(workspace, plan), "utf8"), readFile(path.join(workspace, "TASKS.md"), "utf8"),
    runCampaignProcess(workspace, "git", ["rev-parse", "HEAD"], 5_000),
    runCampaignProcess(workspace, "git", ["status", "--porcelain", "--untracked-files=all"], 5_000),
    runCampaignProcess(workspace, "git", ["rev-list", "--left-right", "--count", "HEAD...@{upstream}"], 5_000),
    readFile(path.join(workspace, manifest), "utf8"),
  ]);
  const source = head.stdout.trim();
  const manifestValue = object(JSON.parse(manifestDocument));
  if (sha256(planDocument) !== planSha256 || !tasks.includes(`### ${task}`) || status.stdout.trim() !== ""
    || sync.stdout.trim() !== "0\t0" || manifestValue["source_commit"] !== source) fail("source_identity");
  for (const candidate of [args.wifiCredentials, args.poolCredentials, bundle, backup]) {
    await requireMode(path.join(workspace, candidate), 0o600);
  }
  await admitStratumV2RestoreBundle(workspace, bundle, runCampaignProcess);
  const snapshot = object(JSON.parse(await readFile(path.join(workspace, predecessor, "restoration/snapshot-write.private.json"), "utf8")));
  const wifiSeed = object(JSON.parse(await readFile(path.join(workspace, predecessor, "restoration/wifi-seed.private.json"), "utf8")));
  if (!completedRestoreCommandReceipt(snapshot, "managed_esptool_write_flash")
    || !completedRestoreCommandReceipt(wifiSeed, "espflash_write_bin")) fail("restore_receipts");
  await requireNoOwnedUsbProcesses();
}

async function runPrompt(workspace: string, generation: number): Promise<void> {
  const intent = path.join(workspace, root, `display-origin-prompt-00${generation}.private.json`);
  const result = path.join(workspace, root, `display-origin-capture-00${generation}.private.json`);
  await writePrivate(intent, { schema_version: "bitaxe-native-usb-display-origin-prompt-v1", operation: "prompt", generation });
  const helper = path.join(workspace, "tools/automation/src/macos-display-origin-capture.swift");
  const child = spawn("/usr/bin/xcrun", ["swift", helper, "--intent", intent, "--result", result], {
    cwd: workspace, stdio: "ignore",
  });
  const exitCode = await new Promise<number | null>((resolve, reject) => {
    child.once("error", reject); child.once("close", resolve);
  });
  if (exitCode !== 0) fail("capture_failed");
  await requireMode(result, 0o600);
  const captured = object(JSON.parse(await readFile(result, "utf8")));
  if (captured["status"] !== "accepted") fail("capture_cancelled");
}

export async function runDisplayRecovery(workspace: string, args: DisplayRecoveryArgs): Promise<JsonObject> {
  if (args.action === "preflight") {
    await commonAdmission(workspace, args, true);
    return { schema_version: "bitaxe-native-usb-display-recovery-preflight-v1", status: "ready", device_effect: false };
  }
  if (args.action === "capture") {
    const rootIsAbsent = await absent(path.join(workspace, root));
    await commonAdmission(workspace, args, rootIsAbsent);
    const generation = rootIsAbsent ? 1 : 2;
    if (rootIsAbsent) { await mkdir(path.join(workspace, root), { mode: 0o700 }); await chmod(path.join(workspace, root), 0o700); }
    else {
      await requireMode(path.join(workspace, root, "origin-unreachable.private.json"), 0o600);
      if (!(await absent(path.join(workspace, root, "display-origin-capture-002.private.json")))) fail("capture_consumed");
    }
    await runPrompt(workspace, generation);
    return { schema_version: "bitaxe-native-usb-display-recovery-capture-result-v1", status: "accepted", generation };
  }
  await commonAdmission(workspace, args, false);
  if (args.action === "start") {
    const captureTwo = path.join(workspace, root, "display-origin-capture-002.private.json");
    const capture = (await absent(captureTwo)) ? path.join(root, "display-origin-capture-001.private.json") : path.join(root, "display-origin-capture-002.private.json");
    const result = await runCampaignProcess(workspace, path.join(workspace, "bazel-bin/tools/flash/flash"), [
      "display-recovery-start", "--board", "205", "--port", args.port,
      "--package-manifest", manifest, "--restore-bundle", bundle, "--settings-backup", backup,
      "--wifi-credentials", args.wifiCredentials, "--pool-credentials", args.poolCredentials,
      "--capture-input", capture, "--private-root", root, "--plan", plan, "--redact-evidence",
    ], 180_000);
    if (result.exitCode !== 0) fail("hardware_blocked");
    const detector = await runCampaignProcess(workspace, path.join(workspace, "bazel-bin/tools/flash/flash"), ["detect", "--board", "205", "--port", args.port], 120_000);
    await writePrivate(path.join(workspace, root, "final-detector.private.json"), { exit_code: detector.exitCode, stdout_sha256: sha256(detector.stdout), stderr_sha256: sha256(detector.stderr) });
    if (detector.exitCode !== 0) fail("final_detector");
    return { schema_version: "bitaxe-native-usb-display-recovery-start-result-v1", status: "accepted" };
  }
  const machine = object(JSON.parse(await readFile(path.join(workspace, root, "machine-result.private.json"), "utf8")));
  const detector = object(JSON.parse(await readFile(path.join(workspace, root, "final-detector.private.json"), "utf8")));
  const publicValue = projectDisplayRecovery(machine, detector);
  const evaluatorDocuments = await Promise.all([
    "tools/automation/src/macos-display-origin-capture.swift",
    "tools/automation/src/native-usb-display-recovery-cli.ts",
    "tools/automation/src/native-usb-display-recovery.ts",
  ].map(async candidate => ({ path: candidate, source: await readFile(path.join(workspace, candidate), "utf8") })));
  publicValue["evaluator_sha256"] = sha256(JSON.stringify({
    machine_evaluator_sha256: machine["evaluator_sha256"], evaluator_documents: evaluatorDocuments,
  }));
  await mkdir(path.dirname(path.join(workspace, projection)), { recursive: true });
  await writeFile(path.join(workspace, projection), `${JSON.stringify(publicValue, null, 2)}\n`, { flag: "wx", mode: 0o644 });
  await chmod(path.join(workspace, projection), 0o644);
  return { schema_version: "bitaxe-native-usb-display-recovery-finalize-result-v1", status: "accepted" };
}

export function projectDisplayRecovery(machine: JsonObject, detector: JsonObject): JsonObject {
  const requiredTrue = ["display_origin_supplied", "private_ipv4", "usb_mac_bound", "recovery_identity_exact", "settings_exact", "theme_exact", "mineonboot_disabled", "mining_inactive", "zero_work", "stable_physical_identity", "cleanup_complete"];
  if (machine["schema_version"] !== "bitaxe-native-usb-display-recovery-machine-v1"
    || machine["terminal_category"] !== "complete" || machine["redaction_status"] !== "passed"
    || requiredTrue.some(key => machine[key] !== true) || detector["exit_code"] !== 0) fail("finalization");
  const digestKeys = ["source_commit", "reference_commit", "plan_sha256", "evaluator_sha256", "package_manifest_sha256", "restore_bundle_sha256", "capture_sha256", "usb_receipt_sha256"];
  if (digestKeys.some(key => typeof machine[key] !== "string"
    || !/^[0-9a-f]+$/u.test(machine[key] as string)
    || (key === "source_commit" || key === "reference_commit" ? (machine[key] as string).length !== 40 : (machine[key] as string).length !== 64))) fail("finalization");
  for (const key of ["settings_request_count", "theme_request_count", "reconciliation_read_count"]) {
    if (!Number.isInteger(machine[key]) || Number(machine[key]) < 0 || Number(machine[key]) > 64) fail("finalization");
  }
  const publicKeys = ["schema_version", "source_commit", "reference_commit", "plan_sha256", "evaluator_sha256", "package_manifest_sha256", "restore_bundle_sha256", "capture_sha256", "usb_receipt_sha256", ...requiredTrue, "settings_request_count", "theme_request_count", "reconciliation_read_count", "terminal_category", "redaction_status"];
  const publicValue: JsonObject = {};
  for (const key of publicKeys) publicValue[key] = machine[key];
  publicValue["schema_version"] = "bitaxe-native-usb-display-recovery-projection-v1";
  return publicValue;
}
