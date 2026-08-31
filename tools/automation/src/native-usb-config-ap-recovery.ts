import { createHash } from "node:crypto";
import { constants } from "node:fs";
import { access, lstat, readFile, realpath } from "node:fs/promises";
import path from "node:path";

import {
  completedRestoreCommandReceipt,
  requireNoOwnedUsbProcesses,
} from "./native-usb-transition-recovery.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { admitStratumV2RestoreBundle } from "./stratum-v2-restore-admission.js";
import { validateTcpPayloadRecoveryTooling } from "./stratum-v2-tcp-recovery-tooling.js";
import { sourceWorkspaceRoot } from "./workspace.js";

export type ConfigApRecoveryAction =
  | "preflight"
  | "read-nvs"
  | "recover"
  | "resume"
  | "finalize";

export type ConfigApRecoveryArgs = {
  readonly action: ConfigApRecoveryAction;
  readonly port: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly restoreBundle: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly plan: string;
};

const immutablePlan =
  "docs/parity/work-plans/20260831T033840Z-NATIVE-USB-CONFIG-AP-RECOVERY-NVS-FIRST/PLAN.md";
const immutablePlanSha256 =
  "44f35fcef288199baab06da036adff88e815a49583aa3b230ea7fa565ff05bf6";
const task = "task-native-usb-config-ap-recovery-205";
const privateRoot = "scratch/native-usb-config-ap-recovery/attempt-001";
const projection =
  "docs/parity/evidence/native-usb-config-ap-recovery/recovery-projection-001.json";
const packageManifest = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
const restoreBundle =
  "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const predecessor = "scratch/native-usb-transition/recovery-002";
const nvsTool =
  ".embuild/espressif/esp-idf/v5.5.4/components/nvs_flash/nvs_partition_tool/nvs_tool.py";

function fail(category: string): never {
  throw new Error(category);
}

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    fail("evidence_invalid");
  }
  return value as JsonObject;
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

async function isAbsent(candidate: string): Promise<boolean> {
  try {
    await lstat(candidate);
    return false;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return true;
    throw error;
  }
}

async function requireMode(candidate: string, mode: number, directory = false): Promise<void> {
  const metadata = await lstat(candidate);
  if (
    metadata.isSymbolicLink()
    || (directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode
  ) {
    fail("protected_mode");
  }
}

async function requireContainedTool(workspace: string, relative: string): Promise<void> {
  const workspaceReal = await realpath(workspace);
  const candidate = path.join(workspace, relative);
  const metadata = await lstat(candidate);
  if (metadata.isSymbolicLink() || !metadata.isFile()) fail("nvs_tooling");
  const relativeReal = path.relative(workspaceReal, await realpath(candidate));
  if (relativeReal === "" || relativeReal.startsWith("..") || path.isAbsolute(relativeReal)) {
    fail("nvs_tooling");
  }
}

export function parseConfigApRecoveryArgs(
  action: string | undefined,
  values: readonly string[],
): ConfigApRecoveryArgs {
  const actions = ["preflight", "read-nvs", "recover", "resume", "finalize"] as const;
  if (!actions.includes(action as ConfigApRecoveryAction)) fail("invalid_invocation");
  const options = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") {
      options.set(key, true);
      continue;
    }
    const value = values[index + 1];
    if (key === undefined || value === undefined || !key.startsWith("--") || value.startsWith("--")) {
      fail("invalid_invocation");
    }
    options.set(key, value);
    index += 1;
  }
  const value = (key: string): string => {
    const maybeValue = options.get(key);
    return typeof maybeValue === "string" ? maybeValue : fail("invalid_invocation");
  };
  const args = {
    action: action as ConfigApRecoveryAction,
    port: value("--port"),
    packageManifest: value("--package-manifest"),
    wifiCredentials: value("--wifi-credentials"),
    restoreBundle: value("--restore-bundle"),
    privateRoot: value("--private-root"),
    projection: value("--projection"),
    plan: value("--plan"),
  };
  if (
    value("--board") !== "205"
    || args.packageManifest !== packageManifest
    || args.wifiCredentials !== "wifi-credentials.json"
    || args.restoreBundle !== restoreBundle
    || args.privateRoot !== privateRoot
    || args.projection !== projection
    || args.plan !== immutablePlan
    || options.get("--redact-evidence") !== true
  ) {
    fail("invalid_invocation");
  }
  return args;
}

export function configApRecoveryWorkspaceRoot(
  environment = process.env,
  cwd = process.cwd(),
): string {
  const configured = environment["BUILD_WORKSPACE_DIRECTORY"];
  return sourceWorkspaceRoot(configured === undefined ? [cwd] : [configured, cwd]);
}

async function commonStageOneAdmission(
  workspace: string,
  args: ConfigApRecoveryArgs,
): Promise<void> {
  if (
    !(await isAbsent(path.join(workspace, privateRoot)))
    || !(await isAbsent(path.join(workspace, projection)))
  ) {
    fail("outputs");
  }
  const [planDocument, tasks, head, status, sync, manifestDocument] = await Promise.all([
    readFile(path.join(workspace, immutablePlan), "utf8"),
    readFile(path.join(workspace, "TASKS.md"), "utf8"),
    runCampaignProcess(workspace, "git", ["rev-parse", "HEAD"], 5_000),
    runCampaignProcess(
      workspace,
      "git",
      ["status", "--porcelain", "--untracked-files=all"],
      5_000,
    ),
    runCampaignProcess(
      workspace,
      "git",
      ["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
      5_000,
    ),
    readFile(path.join(workspace, args.packageManifest), "utf8"),
  ]);
  const source = head.stdout.trim();
  const manifestValue = object(JSON.parse(manifestDocument));
  if (
    sha256(planDocument) !== immutablePlanSha256
    || !tasks.includes(`### ${task}`)
    || status.stdout.trim() !== ""
    || sync.stdout.trim() !== "0\t0"
    || manifestValue["source_commit"] !== source
  ) {
    fail("source_identity");
  }
  await requireMode(path.join(workspace, args.wifiCredentials), 0o600);
  await requireMode(path.join(workspace, args.restoreBundle), 0o600);
  await access(path.join(workspace, args.packageManifest), constants.R_OK);
  await admitStratumV2RestoreBundle(workspace, args.restoreBundle, runCampaignProcess);
  const snapshotPath = path.join(workspace, predecessor, "restoration/snapshot-write.private.json");
  const wifiSeedPath = path.join(workspace, predecessor, "restoration/wifi-seed.private.json");
  await requireMode(snapshotPath, 0o600);
  await requireMode(wifiSeedPath, 0o600);
  const snapshot = object(JSON.parse(await readFile(snapshotPath, "utf8")));
  const wifiSeed = object(JSON.parse(await readFile(wifiSeedPath, "utf8")));
  if (
    !completedRestoreCommandReceipt(snapshot, "managed_esptool_write_flash")
    || !completedRestoreCommandReceipt(wifiSeed, "espflash_write_bin")
  ) {
    fail("restore_receipts");
  }
  await validateTcpPayloadRecoveryTooling(workspace, runCampaignProcess);
  await requireContainedTool(workspace, nvsTool);
  await requireNoOwnedUsbProcesses();
}

function readbackArgs(args: ConfigApRecoveryArgs, admissionOnly: boolean): string[] {
  const values = [
    "nvs-readback",
    "--board",
    "205",
    "--port",
    args.port,
    "--wifi-credentials",
    args.wifiCredentials,
    "--private-root",
    args.privateRoot,
    "--plan",
    args.plan,
    "--redact-evidence",
  ];
  if (admissionOnly) values.push("--admission-only");
  return values;
}

export async function runConfigApRecovery(
  workspace: string,
  args: ConfigApRecoveryArgs,
): Promise<JsonObject> {
  if (args.action === "recover" || args.action === "resume" || args.action === "finalize") {
    fail("nvs_checkpoint_required");
  }
  await commonStageOneAdmission(workspace, args);
  if (args.action === "preflight") {
    const admission = await runCampaignProcess(
      workspace,
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      readbackArgs(args, true),
      30_000,
    );
    if (admission.exitCode !== 0) fail("nvs_admission_failed");
    return {
      schema_version: "bitaxe-native-usb-config-ap-recovery-preflight-v1",
      status: "ready",
      device_effect: false,
      host_network_effect: false,
    };
  }
  const result = await runCampaignProcess(
    workspace,
    path.join(workspace, "bazel-bin/tools/flash/flash"),
    readbackArgs(args, false),
    420_000,
  );
  const statePath = path.join(workspace, privateRoot, "state.private.json");
  if (
    await isAbsent(path.join(workspace, privateRoot))
    || await isAbsent(statePath)
  ) {
    fail("nvs_read_failed");
  }
  await requireMode(path.join(workspace, privateRoot), 0o700, true);
  await requireMode(statePath, 0o600);
  const state = object(JSON.parse(await readFile(statePath, "utf8")));
  if (
    state["schema_version"] !== "bitaxe-native-usb-config-ap-recovery-state-v1"
    || state["device_write_observed"] !== false
    || state["cleanup_complete"] !== true
  ) {
    fail("nvs_evidence_invalid");
  }
  if (state["stage"] === "nvs_mismatch") fail("nvs_mismatch");
  if (result.exitCode !== 0 || state["stage"] !== "nvs_match" || state["nvs_match"] !== true) {
    fail("nvs_read_failed");
  }
  return {
    schema_version: "bitaxe-native-usb-config-ap-recovery-read-nvs-result-v1",
    status: "accepted",
    stage: "nvs_match",
    device_write: false,
    host_network_effect: false,
  };
}
