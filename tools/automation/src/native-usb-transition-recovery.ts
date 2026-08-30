import { chmod, lstat, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { restoreSelfTestSettings } from "./self-test-campaign-restoration.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { runManagedDiagnosticProcess } from "./stratum-v2-noise-diagnostic-process.js";
import { admitStratumV2RestoreBundle, restoreRuntimeMatches } from "./stratum-v2-restore-admission.js";
import type { RestoreBundle } from "./stratum-v2-restore-model.js";
import { fetchRuntimeObject, monitorRuntimeOrigin } from "./stratum-v2-runtime-admission.js";
import { validateTcpPayloadRecoveryTooling } from "./stratum-v2-tcp-recovery-tooling.js";
import {
  backupRelative,
  backupSha256,
  contingencyRoot,
  diagnosticRoot,
  fail,
  NativeUsbRecoveryError,
  object,
  packageManifest,
  planRelative,
  planSha256,
  primaryRoot,
  projectionRelative,
  recoveryPlan,
  restoreBundle,
  sha256,
  taskId,
  transientPreflightRoot,
  wifiCredentials,
  type NativeUsbRecoveryArgs,
} from "./native-usb-transition-recovery-contract.js";

export {
  NativeUsbRecoveryError,
  nativeUsbRecoveryWorkspaceRoot,
  parseNativeUsbRecoveryArgs,
  type NativeUsbRecoveryAction,
  type NativeUsbRecoveryArgs,
} from "./native-usb-transition-recovery-contract.js";

type PreparedRecovery = {
  readonly head: string;
  readonly manifest: JsonObject;
  readonly manifestDocument: string;
  readonly wifiPath: string;
  readonly poolPath: string;
  readonly backup: JsonObject;
  readonly bundle: RestoreBundle;
  readonly bundlePath: string;
};

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  let metadata;
  try {
    metadata = await lstat(candidate);
  } catch (error) {
    void error;
    fail("evidence_invalid", "protected_inputs");
  }
  if (metadata.isSymbolicLink()
    || (directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    fail("evidence_invalid", "protected_inputs");
  }
}

async function requireAbsent(candidate: string, checkpoint: string): Promise<void> {
  try {
    await lstat(candidate);
    fail("evidence_invalid", checkpoint);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
    throw error;
  }
}

async function writePrivate(candidate: string, value: unknown): Promise<void> {
  const document = typeof value === "string" ? value : `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(candidate, document, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(candidate, 0o600);
}

async function solePoolInput(workspace: string): Promise<string> {
  const candidates = (await readdir(workspace))
    .filter(name => /^pool-credentials(?:-[A-Za-z0-9_-]+)?\.json$/u.test(name))
    .map(name => path.join(workspace, name));
  if (candidates.length !== 1 || candidates[0] === undefined) {
    fail("evidence_invalid", "restoration_inputs");
  }
  await requireMode(candidates[0], 0o600, false);
  return candidates[0];
}

async function validateCanonicalManifest(
  workspace: string,
  manifestPath: string,
  head: string,
): Promise<{ readonly manifest: JsonObject; readonly document: string }> {
  const document = await readFile(manifestPath, "utf8");
  const manifest = object(JSON.parse(document), "manifest_identity");
  if (manifest["schema_version"] !== 3 || manifest["source_commit"] !== head
    || manifest["reference_commit"] !== "c1915b0a63bfabebdb95a515cedfee05146c1d50") {
    fail("evidence_invalid", "manifest_identity");
  }
  const artifacts = manifest["artifacts"];
  if (!Array.isArray(artifacts) || artifacts.length !== 6) {
    fail("evidence_invalid", "manifest_artifacts");
  }
  for (const raw of artifacts) {
    const artifact = object(raw, "manifest_artifacts");
    const relative = artifact["path"];
    const digest = artifact["sha256"];
    if (typeof relative !== "string" || typeof digest !== "string") {
      fail("evidence_invalid", "manifest_artifacts");
    }
    const candidate = relative === "firmware/bitaxe/partitions-ultra205.csv"
      ? path.join(workspace, relative)
      : path.join(path.dirname(manifestPath), relative);
    if (sha256(await readFile(candidate)) !== digest) {
      fail("evidence_invalid", "manifest_artifacts");
    }
  }
  return { manifest, document };
}

function processIsAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return (error as NodeJS.ErrnoException).code === "EPERM";
  }
}

async function requireNoOwnedUsbProcesses(): Promise<void> {
  const stateRoot = path.join(os.tmpdir(), `bitaxe-device-sessions-${process.getuid?.() ?? 0}`);
  let deviceRoots: string[];
  try {
    deviceRoots = await readdir(stateRoot);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
    throw error;
  }
  for (const deviceRoot of deviceRoots) {
    const journalPath = path.join(stateRoot, deviceRoot, "crash-journal.json");
    try {
      const journal = object(JSON.parse(await readFile(journalPath, "utf8")), "usb_cleanup");
      const ownerPid = journal["owner_pid"];
      const child = journal["child"];
      const childObject = child === null ? undefined : object(child, "usb_cleanup");
      const childPid = childObject?.["pid"];
      if ((typeof ownerPid === "number" && processIsAlive(ownerPid))
        || (typeof childPid === "number" && processIsAlive(childPid))) {
        fail("hardware_blocked", "usb_cleanup");
      }
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
}

async function prepareRecovery(
  workspace: string,
  args: NativeUsbRecoveryArgs,
): Promise<PreparedRecovery> {
  await requireAbsent(path.join(workspace, args.privateRoot), "outputs_absent");
  const ignored = await runCampaignProcess(
    workspace,
    "git",
    ["check-ignore", "-q", args.privateRoot],
    5_000,
  );
  if (ignored.exitCode !== 0) fail("evidence_invalid", "private_path_ignored");
  const [planDocument, tasks, headResult, status, sync] = await Promise.all([
    readFile(path.join(workspace, planRelative), "utf8"),
    readFile(path.join(workspace, "TASKS.md"), "utf8"),
    runCampaignProcess(workspace, "git", ["rev-parse", "HEAD"], 5_000),
    runCampaignProcess(workspace, "git", ["status", "--porcelain", "--untracked-files=all"], 5_000),
    runCampaignProcess(workspace, "git", ["rev-list", "--left-right", "--count", "HEAD...@{upstream}"], 5_000),
  ]);
  const head = headResult.stdout.trim();
  if (sha256(planDocument) !== planSha256 || !tasks.includes(`### ${taskId}`)
    || headResult.exitCode !== 0 || status.exitCode !== 0 || status.stdout.trim() !== ""
    || sync.exitCode !== 0 || sync.stdout.trim() !== "0\t0") {
    fail("evidence_invalid", "source_identity");
  }
  const manifestPath = path.join(workspace, packageManifest);
  const manifest = await validateCanonicalManifest(workspace, manifestPath, head);
  const wifiPath = path.join(workspace, wifiCredentials);
  const backupPath = path.join(workspace, backupRelative);
  await requireMode(wifiPath, 0o600, false);
  await requireMode(backupPath, 0o600, false);
  const poolPath = await solePoolInput(workspace);
  const backupDocument = await readFile(backupPath, "utf8");
  if (sha256(backupDocument) !== backupSha256) fail("evidence_invalid", "restoration_inputs");
  let admitted: Awaited<ReturnType<typeof admitStratumV2RestoreBundle>>;
  try {
    admitted = await admitStratumV2RestoreBundle(workspace, restoreBundle, runCampaignProcess);
  } catch (error) {
    void error;
    fail("evidence_invalid", "restore_readiness");
  }
  try {
    await validateTcpPayloadRecoveryTooling(workspace, runCampaignProcess);
  } catch (error) {
    const checkpoint = error instanceof Error ? error.message : "restore_tooling";
    fail("evidence_invalid", checkpoint);
  }
  await requireNoOwnedUsbProcesses();
  return {
    head,
    manifest: manifest.manifest,
    manifestDocument: manifest.document,
    wifiPath,
    poolPath,
    backup: object(JSON.parse(backupDocument), "restoration_inputs"),
    bundle: admitted.bundle,
    bundlePath: admitted.path,
  };
}

async function writeAuthorization(
  workspace: string,
  rootRelative: string,
  ordinal: 2 | 3,
  prepared: PreparedRecovery,
): Promise<string> {
  const root = path.join(workspace, rootRelative);
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
  const bundleDocument = await readFile(prepared.bundlePath, "utf8");
  const recoveryPlanDocument = await readFile(path.join(workspace, recoveryPlan), "utf8");
  const authorizationRelative = path.join(rootRelative, "restore-authorization.private.json");
  await writePrivate(path.join(workspace, authorizationRelative), {
    schema_version: "bitaxe-stratum-v2-restore-authorization-v1",
    board: 205,
    ordinal,
    action: "native_usb_recovery",
    current_source_commit: prepared.head,
    reference_commit: prepared.manifest["reference_commit"],
    bundle_sha256: sha256(bundleDocument),
    bundle_capture_source_commit: prepared.bundle.capture_source_commit,
    recovery_plan_sha256: sha256(recoveryPlanDocument),
    remediation_plan_sha256: planSha256,
  });
  return authorizationRelative;
}

async function admitRecovery(
  workspace: string,
  args: NativeUsbRecoveryArgs,
  prepared: PreparedRecovery,
): Promise<void> {
  const transientRoot = path.join(workspace, transientPreflightRoot);
  await requireAbsent(transientRoot, "transient_preflight_root");
  try {
    const authorization = await writeAuthorization(workspace, transientPreflightRoot, 2, prepared);
    const outcome = await runCampaignProcess(
      workspace,
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      [
        "restore-installed", "--board", "205", "--port", args.port,
        "--restore-bundle", restoreBundle,
        "--restore-authorization", authorization,
        "--remediation-plan", planRelative,
        "--private-root", transientPreflightRoot,
        "--wifi-credentials", wifiCredentials,
        "--admission-only", "--redact-evidence",
      ],
      120_000,
    );
    if (outcome.exitCode !== 0) fail("evidence_invalid", "restore_admission");
  } finally {
    await rm(transientRoot, { recursive: true, force: true });
  }
}

export async function preflightNativeUsbRecovery(
  workspace: string,
  args: NativeUsbRecoveryArgs,
): Promise<JsonObject> {
  const prepared = await prepareRecovery(workspace, args);
  await admitRecovery(workspace, args, prepared);
  await requireAbsent(path.join(workspace, transientPreflightRoot), "preflight_cleanup");
  return {
    schema_version: "bitaxe-native-usb-recovery-preflight-v1",
    status: "ready",
    source_identity: true,
    restore_readiness: true,
    restore_tooling: true,
    restore_admission: true,
    private_root_created: false,
    device_effect: false,
  };
}

export async function startNativeUsbRecovery(
  workspace: string,
  args: NativeUsbRecoveryArgs,
): Promise<JsonObject> {
  const prepared = await prepareRecovery(workspace, args);
  await admitRecovery(workspace, args, prepared);
  const privateRoot = path.join(workspace, args.privateRoot);
  await mkdir(privateRoot, { mode: 0o700 });
  await chmod(privateRoot, 0o700);
  const restorationRelative = path.join(args.privateRoot, "restoration");
  const authorization = await writeAuthorization(
    workspace,
    restorationRelative,
    args.recoveryOrdinal,
    prepared,
  );
  const flashProgram = path.join(workspace, "bazel-bin/tools/flash/flash");
  const restored = await runManagedDiagnosticProcess(workspace, flashProgram, [
    "restore-installed", "--board", "205", "--port", args.port,
    "--restore-bundle", restoreBundle,
    "--restore-authorization", authorization,
    "--remediation-plan", planRelative,
    "--private-root", restorationRelative,
    "--wifi-credentials", wifiCredentials,
    "--redact-evidence",
  ], 900_000, "native_usb_recovery");
  await writePrivate(path.join(privateRoot, "restore.stdout.private.log"), restored.stdout);
  await writePrivate(path.join(privateRoot, "restore.stderr.private.log"), restored.stderr);
  if (restored.exitCode !== 0) fail("hardware_blocked", "restoration");
  const origin = await monitorRuntimeOrigin(workspace, flashProgram, args.port, runCampaignProcess, fail);
  await restoreSelfTestSettings(origin, prepared.backup, prepared.wifiPath, prepared.poolPath);
  const confirmed = await fetchRuntimeObject(origin, "/api/system/info", fail);
  const identityExact = restoreRuntimeMatches(prepared.bundle, confirmed);
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
  await requireNoOwnedUsbProcesses();
  const result = {
    schema_version: "bitaxe-native-usb-recovery-result-v1",
    ordinal: args.recoveryOrdinal,
    source_commit: prepared.head,
    package_manifest_sha256: sha256(prepared.manifestDocument),
    restored_identity: true,
    settings_exact: true,
    mineonboot_disabled: true,
    mining_inactive: true,
    zero_work: true,
    cleanup_complete: true,
    owned_processes_remaining: 0,
    status: "accepted",
    category: "complete",
  } as const;
  await writePrivate(path.join(privateRoot, "recovery-result.private.json"), result);
  return result;
}

function exactRecoveryResult(value: JsonObject, ordinal: number): boolean {
  return value["schema_version"] === "bitaxe-native-usb-recovery-result-v1"
    && value["ordinal"] === ordinal
    && value["restored_identity"] === true
    && value["settings_exact"] === true
    && value["mineonboot_disabled"] === true
    && value["mining_inactive"] === true
    && value["zero_work"] === true
    && value["cleanup_complete"] === true
    && value["owned_processes_remaining"] === 0
    && value["status"] === "accepted";
}

export function validTransitionCandidate(value: JsonObject): boolean {
  const terminalCategories = new Set([
    "complete", "runtime_profile_unknown", "handoff_unsupported",
    "handoff_rejected_unsafe_state", "handoff_ready_timeout", "handoff_commit_timeout",
    "bus_reset_timeout", "same_worker_after_commit", "handoff_transition_timeout",
    "bootloader_ambiguous", "physical_identity_drift", "rom_admission_failed",
    "application_reappearance_timeout", "foreign_holder", "cleanup_failed",
    "recovery_required",
  ]);
  const digestKeys = [
    "source_commit", "reference_commit", "plan_sha256", "evaluator_sha256",
    "manifest_sha256", "app_elf_sha256",
  ] as const;
  const countKeys = [
    "absent_count", "same_worker_count", "same_serial_jtag_count",
    "same_unknown_count", "physical_mismatch_count",
  ] as const;
  const booleanKeys = [
    "ready_received", "committed_received", "bus_reset_observed", "rom_admitted",
    "application_reappeared", "device_write_observed", "restoration_complete",
    "cleanup_complete",
  ] as const;
  const stagesAreOrdered = value["committed_received"] !== true || value["ready_received"] === true;
  const busResetIsOrdered = value["bus_reset_observed"] !== true
    || value["committed_received"] === true;
  const romIsOrdered = value["rom_admitted"] !== true || value["bus_reset_observed"] === true;
  const applicationIsOrdered = value["application_reappeared"] !== true
    || value["rom_admitted"] === true;
  const completeIsExact = value["terminal_category"] !== "complete"
    || (value["ready_received"] === true
      && value["committed_received"] === true
      && value["bus_reset_observed"] === true
      && value["rom_admitted"] === true
      && value["application_reappeared"] === true
      && value["cleanup_complete"] === true);
  return value["schema_version"] === "bitaxe-native-usb-transition-projection-v1"
    && value["plan_sha256"] === planSha256
    && digestKeys.every(key => typeof value[key] === "string"
      && /^[0-9a-f]+$/u.test(value[key] as string)
      && (key === "source_commit" || key === "reference_commit"
        ? (value[key] as string).length === 40
        : (value[key] as string).length === 64))
    && countKeys.every(key => Number.isInteger(value[key])
      && Number(value[key]) >= 0 && Number(value[key]) <= 1_024)
    && booleanKeys.every(key => typeof value[key] === "boolean")
    && stagesAreOrdered && busResetIsOrdered && romIsOrdered && applicationIsOrdered
    && completeIsExact
    && value["device_write_observed"] === false
    && value["restoration_complete"] === false
    && value["redaction_status"] === "passed"
    && typeof value["terminal_category"] === "string"
    && terminalCategories.has(value["terminal_category"] as string);
}

export async function finalizeNativeUsbRecovery(
  workspace: string,
  args: NativeUsbRecoveryArgs,
): Promise<JsonObject> {
  if (args.recoveryOrdinal !== 3) fail("invalid_invocation", "finalize_ordinal");
  await requireNoOwnedUsbProcesses();
  await requireMode(path.join(workspace, primaryRoot), 0o700, true);
  await requireMode(path.join(workspace, contingencyRoot), 0o700, true);
  await requireMode(path.join(workspace, diagnosticRoot), 0o700, true);
  await requireMode(
    path.join(workspace, primaryRoot, "recovery-result.private.json"),
    0o600,
    false,
  );
  await requireMode(
    path.join(workspace, contingencyRoot, "recovery-result.private.json"),
    0o600,
    false,
  );
  await requireMode(
    path.join(workspace, diagnosticRoot, "transition-result.private.json"),
    0o600,
    false,
  );
  const primary = object(JSON.parse(await readFile(
    path.join(workspace, primaryRoot, "recovery-result.private.json"),
    "utf8",
  )), "primary_recovery");
  const finalRecovery = object(JSON.parse(await readFile(
    path.join(workspace, contingencyRoot, "recovery-result.private.json"),
    "utf8",
  )), "final_recovery");
  const transition = object(JSON.parse(await readFile(
    path.join(workspace, diagnosticRoot, "transition-result.private.json"),
    "utf8",
  )), "transition_result");
  if (!exactRecoveryResult(primary, 2) || !exactRecoveryResult(finalRecovery, 3)
    || !validTransitionCandidate(transition)
    || primary["source_commit"] !== finalRecovery["source_commit"]
    || primary["source_commit"] !== transition["source_commit"]) {
    fail("evidence_invalid", "finalization_join");
  }
  const allowed = [
    "schema_version", "source_commit", "reference_commit", "plan_sha256",
    "evaluator_sha256", "manifest_sha256", "app_elf_sha256", "ready_received",
    "committed_received", "bus_reset_observed", "absent_count", "same_worker_count",
    "same_serial_jtag_count", "same_unknown_count", "physical_mismatch_count",
    "rom_admitted", "application_reappeared", "device_write_observed", "cleanup_complete",
    "redaction_status", "terminal_category",
  ] as const;
  const projection: JsonObject = {};
  for (const key of allowed) projection[key] = transition[key];
  const evaluatorPaths = [
    "tools/automation/src/native-usb-transition-recovery-cli.ts",
    "tools/automation/src/native-usb-transition-recovery-contract.ts",
    "tools/automation/src/native-usb-transition-recovery.ts",
    "tools/automation/src/self-test-campaign-restoration.ts",
    "tools/automation/src/stratum-v2-campaign.ts",
    "tools/automation/src/stratum-v2-noise-diagnostic-process.ts",
    "tools/automation/src/stratum-v2-restore-admission.ts",
    "tools/automation/src/stratum-v2-restore-model.ts",
    "tools/automation/src/stratum-v2-restore-validator.ts",
    "tools/automation/src/stratum-v2-runtime-admission.ts",
    "tools/automation/src/stratum-v2-tcp-recovery-tooling.ts",
    "tools/automation/src/stratum-v2-validator-child.ts",
    "tools/flash/src/restore_installed.rs",
    "tools/flash/src/restore_installed/contract.rs",
  ] as const;
  const evaluatorDocuments = await Promise.all(evaluatorPaths.map(async candidate => ({
    path: candidate,
    source: await readFile(path.join(workspace, candidate), "utf8"),
  })));
  projection["evaluator_sha256"] = sha256(JSON.stringify({
    transition_evaluator_sha256: transition["evaluator_sha256"],
    evaluator_documents: evaluatorDocuments,
  }));
  projection["restoration_complete"] = true;
  projection["cleanup_complete"] = true;
  const projectionPath = path.join(workspace, projectionRelative);
  await requireAbsent(projectionPath, "projection_exists");
  await mkdir(path.dirname(projectionPath), { recursive: true });
  await writeFile(projectionPath, `${JSON.stringify(projection, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o644,
  });
  await chmod(projectionPath, 0o644);
  return {
    schema_version: "bitaxe-native-usb-transition-finalization-v1",
    status: "accepted",
    restoration_complete: true,
    cleanup_complete: true,
    projection_published: true,
  };
}

export function nativeUsbRecoveryFailure(error: unknown): JsonObject {
  if (error instanceof NativeUsbRecoveryError) {
    return {
      schema_version: "bitaxe-native-usb-recovery-failure-v1",
      status: "failed",
      category: error.category,
      checkpoint: error.checkpoint,
    };
  }
  return {
    schema_version: "bitaxe-native-usb-recovery-failure-v1",
    status: "failed",
    category: "evidence_invalid",
    checkpoint: "unexpected_failure",
  };
}
