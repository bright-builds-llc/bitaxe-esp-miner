import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, lstat, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { requireNoOwnedUsbProcesses } from "./native-usb-transition-recovery.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { validateTcpPayloadRecoveryTooling } from "./stratum-v2-tcp-recovery-tooling.js";
import { sourceWorkspaceRoot } from "./workspace.js";

export type OwnerRecoveryAction = "preflight" | "observe" | "recover" | "finalize";
export type OwnerRecoveryArgs = {
  readonly action: OwnerRecoveryAction;
  readonly port: string;
  readonly packageManifest: string;
  readonly restoreBundle: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly plan: string;
};

const plan =
  "docs/parity/work-plans/20260901T161405Z-NATIVE-USB-SERIAL-OWNER-RECOVERY/PLAN.md";
const planSha256 = "8d59b142bacc2d7aab7614ee3a3f51ed015abb0b0badf9707dbe819f21db4cc2";
const task = "task-native-usb-rom-exit-discriminator-205";
const root = "scratch/native-usb-owner-recovery/attempt-001";
const projection =
  "docs/parity/evidence/native-usb-owner-recovery/owner-projection-001.json";
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

async function writePrivate(candidate: string, value: unknown): Promise<void> {
  await writeFile(candidate, `${JSON.stringify(value, null, 2)}\n`, {
    flag: "wx",
    mode: 0o600,
  });
  await chmod(candidate, 0o600);
}

export function parseOwnerRecoveryArgs(
  action: string | undefined,
  values: readonly string[],
): OwnerRecoveryArgs {
  if (!(action === "preflight" || action === "observe" || action === "recover"
    || action === "finalize")) {
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
    if (key === undefined || value === undefined || !key.startsWith("--")
      || value.startsWith("--")) {
      fail("invalid_invocation");
    }
    options.set(key, value);
    index += 1;
  }
  const value = (key: string): string => {
    const maybeValue = options.get(key);
    return typeof maybeValue === "string" ? maybeValue : fail("invalid_invocation");
  };
  const args: OwnerRecoveryArgs = {
    action,
    port: value("--port"),
    packageManifest: value("--package-manifest"),
    restoreBundle: value("--restore-bundle"),
    privateRoot: value("--private-root"),
    projection: value("--projection"),
    plan: value("--plan"),
  };
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

export function ownerRecoveryWorkspaceRoot(
  environment = process.env,
  cwd = process.cwd(),
): string {
  const configured = environment["BUILD_WORKSPACE_DIRECTORY"];
  return sourceWorkspaceRoot(configured === undefined ? [cwd] : [configured, cwd]);
}

async function commonAdmission(
  workspace: string,
  args: OwnerRecoveryArgs,
  rootAbsent: boolean,
): Promise<void> {
  if ((await absent(path.join(workspace, root))) !== rootAbsent
    || !(await absent(path.join(workspace, projection)))) {
    fail("outputs");
  }
  if (!rootAbsent) await requireMode(path.join(workspace, root), 0o700, true);
  const [planDocument, tasks, head, status, sync, manifestDocument] = await Promise.all([
    readFile(path.join(workspace, plan), "utf8"),
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
  const predecessorValue = object(JSON.parse(await readFile(
    path.join(workspace, predecessor),
    "utf8",
  )));
  if (predecessorValue["stage"] !== "nvs_match"
    || predecessorValue["nvs_match"] !== true
    || predecessorValue["device_write_observed"] !== false) {
    fail("predecessor_state");
  }
  await validateTcpPayloadRecoveryTooling(workspace, runCampaignProcess);
  await requireNoOwnedUsbProcesses();
}

async function readState(workspace: string): Promise<JsonObject> {
  const recovery = path.join(workspace, root, "recovery-result.private.json");
  const candidate = (await absent(recovery))
    ? path.join(workspace, root, "observation-state.private.json")
    : recovery;
  await requireMode(candidate, 0o600);
  return object(JSON.parse(await readFile(candidate, "utf8")));
}

async function runManualPrompt(workspace: string): Promise<string> {
  const intent = path.join(workspace, root, "manual-bootstrap-intent.private.json");
  const result = path.join(workspace, root, "manual-bootstrap.private.json");
  await writePrivate(intent, {
    schema_version: "bitaxe-native-usb-owner-recovery-prompt-v1",
    operation: "prompt",
  });
  const helper = path.join(
    workspace,
    "tools/automation/src/macos-native-usb-owner-recovery.swift",
  );
  const child = spawn(
    "/usr/bin/xcrun",
    ["swift", helper, "--intent", intent, "--result", result],
    { cwd: workspace, stdio: "ignore" },
  );
  const exitCode = await new Promise<number | null>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", resolve);
  });
  if (exitCode !== 0) fail("manual_prompt_failed");
  await requireMode(result, 0o600);
  const checkpoint = object(JSON.parse(await readFile(result, "utf8")));
  if (checkpoint["status"] !== "accepted") fail("manual_prompt_cancelled");
  return path.relative(workspace, result);
}

export function projectOwnerRecovery(
  machine: JsonObject,
  evaluatorSha256: string,
): JsonObject {
  const digestKeys = [
    "source_commit",
    "reference_commit",
    "plan_sha256",
    "manifest_sha256",
    "restore_bundle_sha256",
  ];
  const countKeys = [
    "passive_observation_count",
    "rom_probe_count",
    "manual_prompt_count",
    "rom_admission_count",
    "force_bit_read_count",
    "application_exit_count",
  ];
  if (
    machine["schema_version"] !== "bitaxe-native-usb-owner-recovery-private-v1"
    || machine["stage"] !== "complete"
    || machine["execution_owner"] !== "application"
    || machine["physical_identity_match"] !== true
    || machine["device_write_observed"] !== false
    || machine["host_network_effect"] !== false
    || machine["cleanup_complete"] !== true
    || machine["terminal_category"] !== "complete"
    || machine["redaction_status"] !== "passed"
    || digestKeys.some(key => typeof machine[key] !== "string")
    || countKeys.some(key => !Number.isInteger(machine[key])
      || Number(machine[key]) < 0 || Number(machine[key]) > 1)
    || !/^[0-9a-f]{64}$/u.test(evaluatorSha256)
    || !["worker_runtime", "serial_jtag_runtime"].includes(String(machine["initial_transport"]))
    || !["none", "already_rom", "manual_boot_reset"].includes(String(machine["rom_entry_path"]))
    || !["not_read", "set", "clear"].includes(String(machine["force_download_bit_category"]))
    || !["none", "managed_esptool_hard_reset"].includes(String(machine["reset_adapter"]))
    || typeof machine["enumeration_changed"] !== "boolean"
  ) {
    fail("finalization");
  }
  const keys = [
    ...digestKeys,
    "initial_transport",
    "passive_marker_status",
    "execution_owner",
    "rom_entry_path",
    "force_download_bit_category",
    "reset_adapter",
    ...countKeys,
    "enumeration_changed",
    "physical_identity_match",
    "device_write_observed",
    "host_network_effect",
    "cleanup_complete",
    "terminal_category",
    "redaction_status",
  ];
  const projected: JsonObject = {
    schema_version: "bitaxe-native-usb-owner-recovery-projection-v1",
    evaluator_sha256: evaluatorSha256,
  };
  for (const key of keys) projected[key] = machine[key];
  return projected;
}

export function ownerRecoveryCanRecover(stage: unknown, recoveryResultAbsent: boolean): boolean {
  return recoveryResultAbsent && (stage === "rom_admitted" || stage === "manual_required");
}

export async function runOwnerRecovery(
  workspace: string,
  args: OwnerRecoveryArgs,
): Promise<JsonObject> {
  if (args.action === "preflight") {
    await commonAdmission(workspace, args, true);
    return {
      schema_version: "bitaxe-native-usb-owner-recovery-preflight-v1",
      status: "ready",
      device_effect: false,
    };
  }
  if (args.action === "observe") {
    await commonAdmission(workspace, args, true);
    const result = await runCampaignProcess(
      workspace,
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      [
        "owner-recovery",
        "--board", "205",
        "--port", args.port,
        "--action", "observe",
        "--package-manifest", args.packageManifest,
        "--restore-bundle", args.restoreBundle,
        "--private-root", args.privateRoot,
        "--plan", args.plan,
        "--redact-evidence",
      ],
      120_000,
    );
    if (result.exitCode !== 0) fail("observation_failed");
    await requireMode(path.join(workspace, root), 0o700, true);
    const state = await readState(workspace);
    return {
      schema_version: "bitaxe-native-usb-owner-recovery-observe-result-v1",
      status: "accepted",
      stage: state["stage"],
      terminal_category: state["terminal_category"],
    };
  }
  await commonAdmission(workspace, args, false);
  if (args.action === "recover") {
    const observation = object(JSON.parse(await readFile(
      path.join(workspace, root, "observation-state.private.json"),
      "utf8",
    )));
    if (!ownerRecoveryCanRecover(
      observation["stage"],
      await absent(path.join(workspace, root, "recovery-result.private.json")),
    )) {
      fail("recovery_ineligible");
    }
    const manualCheckpoint = observation["stage"] === "manual_required"
      ? await runManualPrompt(workspace)
      : undefined;
    const command = [
      "owner-recovery",
      "--board", "205",
      "--port", args.port,
      "--action", "recover",
      "--package-manifest", args.packageManifest,
      "--restore-bundle", args.restoreBundle,
      "--private-root", args.privateRoot,
      "--plan", args.plan,
      "--redact-evidence",
    ];
    if (manualCheckpoint !== undefined) {
      command.push("--manual-checkpoint", manualCheckpoint);
    }
    const result = await runCampaignProcess(
      workspace,
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      command,
      180_000,
    );
    if (result.exitCode !== 0) fail("recovery_failed");
    const state = await readState(workspace);
    return {
      schema_version: "bitaxe-native-usb-owner-recovery-recover-result-v1",
      status: "accepted",
      stage: state["stage"],
      terminal_category: state["terminal_category"],
    };
  }
  const machine = await readState(workspace);
  const evaluatorSources = await Promise.all([
    "tools/device-session/src/usb_ownership.rs",
    "tools/flash/src/commands.rs",
    "tools/flash/src/environment/owner_recovery.rs",
    "tools/flash/src/owner_recovery.rs",
    "tools/automation/src/macos-native-usb-owner-recovery.swift",
    "tools/automation/src/native-usb-owner-recovery-cli.ts",
    "tools/automation/src/native-usb-owner-recovery.ts",
  ].map(async candidate => ({
    path: candidate,
    source: await readFile(path.join(workspace, candidate), "utf8"),
  })));
  const publicValue = projectOwnerRecovery(machine, sha256(JSON.stringify(evaluatorSources)));
  const target = path.join(workspace, projection);
  await mkdir(path.dirname(target), { recursive: true });
  await writeFile(target, `${JSON.stringify(publicValue, null, 2)}\n`, {
    flag: "wx",
    mode: 0o644,
  });
  await chmod(target, 0o644);
  return {
    schema_version: "bitaxe-native-usb-owner-recovery-finalize-result-v1",
    status: "accepted",
    terminal_category: publicValue["terminal_category"],
  };
}
