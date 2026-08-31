import { createHash } from "node:crypto";
import { chmod, lstat, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  requireNoOwnedUsbProcesses,
} from "./native-usb-transition-recovery.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { validateTcpPayloadRecoveryTooling } from "./stratum-v2-tcp-recovery-tooling.js";
import { sourceWorkspaceRoot } from "./workspace.js";

export type RomExitAction = "preflight" | "start" | "finalize";
export type RomExitArgs = {
  readonly action: RomExitAction;
  readonly port: string;
  readonly packageManifest: string;
  readonly restoreBundle: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly plan: string;
};

const plan =
  "docs/parity/work-plans/20260831T190744Z-NATIVE-USB-ROM-EXIT-DISCRIMINATOR/PLAN.md";
const planSha256 =
  "a93c88a5a0aab939c6462792bd31a5f61b60dcee45935cccc6c14466ef2b3262";
const task = "task-native-usb-rom-exit-discriminator-205";
const root = "scratch/native-usb-rom-exit/attempt-001";
const projection =
  "docs/parity/evidence/native-usb-rom-exit/rom-exit-projection-001.json";
const manifest = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
const bundle =
  "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const predecessor =
  "scratch/native-usb-config-ap-recovery/attempt-001/state.private.json";

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

async function absent(candidate: string): Promise<boolean> {
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

export function parseRomExitArgs(
  action: string | undefined,
  values: readonly string[],
): RomExitArgs {
  if (!(action === "preflight" || action === "start" || action === "finalize")) {
    fail("invalid_invocation");
  }
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
    action,
    port: value("--port"),
    packageManifest: value("--package-manifest"),
    restoreBundle: value("--restore-bundle"),
    privateRoot: value("--private-root"),
    projection: value("--projection"),
    plan: value("--plan"),
  } as RomExitArgs;
  if (
    value("--board") !== "205"
    || args.packageManifest !== manifest
    || args.restoreBundle !== bundle
    || args.privateRoot !== root
    || args.projection !== projection
    || args.plan !== plan
    || options.get("--redact-evidence") !== true
  ) {
    fail("invalid_invocation");
  }
  return args;
}

export function romExitWorkspaceRoot(
  environment = process.env,
  cwd = process.cwd(),
): string {
  const configured = environment["BUILD_WORKSPACE_DIRECTORY"];
  return sourceWorkspaceRoot(configured === undefined ? [cwd] : [configured, cwd]);
}

async function commonAdmission(
  workspace: string,
  args: RomExitArgs,
  rootAbsent: boolean,
): Promise<void> {
  if (
    (await absent(path.join(workspace, root))) !== rootAbsent
    || !(await absent(path.join(workspace, projection)))
  ) {
    fail("outputs");
  }
  if (!rootAbsent) await requireMode(path.join(workspace, root), 0o700, true);
  const [planDocument, tasks, head, status, sync, manifestDocument] = await Promise.all([
    readFile(path.join(workspace, plan), "utf8"),
    readFile(path.join(workspace, "TASKS.md"), "utf8"),
    runCampaignProcess(workspace, "git", ["rev-parse", "HEAD"], 5_000),
    runCampaignProcess(workspace, "git", ["status", "--porcelain", "--untracked-files=all"], 5_000),
    runCampaignProcess(workspace, "git", ["rev-list", "--left-right", "--count", "HEAD...@{upstream}"], 5_000),
    readFile(path.join(workspace, args.packageManifest), "utf8"),
  ]);
  const manifestValue = object(JSON.parse(manifestDocument));
  if (
    sha256(planDocument) !== planSha256
    || !tasks.includes(`### ${task}`)
    || status.stdout.trim() !== ""
    || sync.stdout.trim() !== "0\t0"
    || manifestValue["source_commit"] !== head.stdout.trim()
  ) {
    fail("source_identity");
  }
  await requireMode(path.join(workspace, args.restoreBundle), 0o600);
  await requireMode(path.join(workspace, predecessor), 0o600);
  await validateTcpPayloadRecoveryTooling(workspace, runCampaignProcess);
  await requireNoOwnedUsbProcesses();
}

export function projectRomExit(machine: JsonObject, evaluatorSha256: string): JsonObject {
  const digestKeys = [
    "source_commit",
    "reference_commit",
    "plan_sha256",
    "manifest_sha256",
    "restore_bundle_sha256",
  ];
  if (
    machine["schema_version"] !== "bitaxe-native-usb-rom-exit-private-v1"
    || machine["force_download_bit_set"] !== true
    || machine["reset_adapter"] !== "managed_esptool_hard_reset"
    || !["application", "unknown"].includes(String(machine["execution_owner"]))
    || machine["nvs_read_repeated"] !== false
    || machine["device_write_observed"] !== false
    || machine["host_network_effect"] !== false
    || machine["cleanup_complete"] !== true
    || machine["redaction_status"] !== "passed"
    || digestKeys.some(key => typeof machine[key] !== "string")
    || !/^[0-9a-f]{64}$/u.test(evaluatorSha256)
    || !["worker_runtime", "serial_jtag_runtime"].includes(String(machine["transport_profile"]))
    || typeof machine["enumeration_changed"] !== "boolean"
    || (machine["execution_owner"] === "application"
      ? machine["terminal_category"] !== "complete"
      : machine["terminal_category"] !== "execution_owner_unknown")
  ) {
    fail("finalization");
  }
  const keys = [
    ...digestKeys,
    "force_download_bit_set",
    "reset_adapter",
    "transport_profile",
    "execution_owner",
    "application_marker_status",
    "enumeration_changed",
    "nvs_read_repeated",
    "device_write_observed",
    "host_network_effect",
    "cleanup_complete",
    "terminal_category",
    "redaction_status",
  ];
  const projected: JsonObject = {
    schema_version: "bitaxe-native-usb-rom-exit-projection-v1",
    evaluator_sha256: evaluatorSha256,
  };
  for (const key of keys) projected[key] = machine[key];
  return projected;
}

export async function runRomExit(
  workspace: string,
  args: RomExitArgs,
): Promise<JsonObject> {
  if (args.action === "preflight") {
    await commonAdmission(workspace, args, true);
    return {
      schema_version: "bitaxe-native-usb-rom-exit-preflight-v1",
      status: "ready",
      device_effect: false,
    };
  }
  if (args.action === "start") {
    await commonAdmission(workspace, args, true);
    const result = await runCampaignProcess(
      workspace,
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      [
        "rom-exit-diagnostic",
        "--board", "205",
        "--port", args.port,
        "--package-manifest", args.packageManifest,
        "--restore-bundle", args.restoreBundle,
        "--private-root", args.privateRoot,
        "--plan", args.plan,
        "--observation-seconds", "30",
        "--redact-evidence",
      ],
      180_000,
    );
    if (result.exitCode !== 0) fail("rom_exit_failed");
    await requireMode(path.join(workspace, root), 0o700, true);
    await requireMode(path.join(workspace, root, "machine-result.private.json"), 0o600);
    const machine = object(JSON.parse(await readFile(
      path.join(workspace, root, "machine-result.private.json"),
      "utf8",
    )));
    return {
      schema_version: "bitaxe-native-usb-rom-exit-start-result-v1",
      status: "accepted",
      terminal_category: machine["terminal_category"],
    };
  }
  await commonAdmission(workspace, args, false);
  const machine = object(JSON.parse(await readFile(
    path.join(workspace, root, "machine-result.private.json"),
    "utf8",
  )));
  const evaluatorSources = await Promise.all([
    "crates/bitaxe-api/src/usb_boot_profile.rs",
    "firmware/bitaxe/src/boot_evidence/usb_profile.rs",
    "tools/device-session/src/usb_ownership/execution.rs",
    "tools/device-session/src/usb_ownership/verification.rs",
    "tools/flash/src/rom_exit.rs",
    "tools/automation/src/native-usb-rom-exit-cli.ts",
    "tools/automation/src/native-usb-rom-exit.ts",
  ].map(async candidate => ({
    path: candidate,
    source: await readFile(path.join(workspace, candidate), "utf8"),
  })));
  const evaluatorSha256 = sha256(JSON.stringify(evaluatorSources));
  const publicValue = projectRomExit(machine, evaluatorSha256);
  const target = path.join(workspace, projection);
  await mkdir(path.dirname(target), { recursive: true });
  await writeFile(target, `${JSON.stringify(publicValue, null, 2)}\n`, { flag: "wx", mode: 0o644 });
  await chmod(target, 0o644);
  return {
    schema_version: "bitaxe-native-usb-rom-exit-finalize-result-v1",
    status: "accepted",
    terminal_category: publicValue["terminal_category"],
  };
}
