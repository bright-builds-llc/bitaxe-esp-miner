import { createHash, randomBytes } from "node:crypto";
import { spawn, type ChildProcess } from "node:child_process";
import { chmod, lstat, mkdir, readFile, readdir, rename, writeFile } from "node:fs/promises";
import path from "node:path";

import { restoreSelfTestSettings } from "./self-test-campaign-restoration.js";
import { validateRestorableInputs } from "./stratum-v2-campaign-settings.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import { sameSubnetFixtureAddress } from "./stratum-v2-campaign-support.js";
import { admitStratumV2RestoreBundle, restoreRuntimeMatches } from "./stratum-v2-restore-admission.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { fetchRuntimeObject, monitorRuntimeOrigin } from "./stratum-v2-runtime-admission.js";
import type { RestoreBundle } from "./stratum-v2-restore-model.js";
import { sourceWorkspaceRoot } from "./workspace.js";
import {
  ManagedDiagnosticProcessError,
  noiseDiagnosticValidatorProgram,
  runManagedDiagnosticProcess,
  terminateManagedProcessGroup,
  type ManagedDiagnosticProcessResult,
} from "./stratum-v2-noise-diagnostic-process.js";

export { runManagedDiagnosticProcess as runNoiseDiagnosticProcess };

export type NoiseDiagnosticAction = "preflight" | "start";
export type NoiseDiagnosticArgs = {
  readonly action: NoiseDiagnosticAction;
  readonly board: "205";
  readonly port: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly restoreBundle: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly plan: string;
  readonly diagnosticOrdinal: 2;
  readonly redactEvidence: true;
};

const expectedRoot = "scratch/str005-noise-diagnostic/diagnostic-002";
const expectedProjection =
  "docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-002.json";
const expectedPlan =
  "docs/parity/work-plans/20260826T210025Z-STR-005-NOISE-DIAGNOSTIC/PLAN.md";
const expectedPlanSha256 =
  "5c5dcc8b030cd07acb60b00d8414d72bc4ad854550d70dad4b66381940629eec";
const expectedRestoreBundle =
  "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const expectedPackageManifest =
  "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
const expectedWifiCredentials = "wifi-credentials.json";
const recoveryPlan =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";
const backupRelative = "scratch/str005-stratum-v2/attempt-004/settings-backup.private.json";
const backupSha256 = "ac3d28d451c466f4fc6bfdc40b327c891dac9f3eba644ce62a7f2a2276790631";
const taskId = "task-str005-noise-handshake-diagnostic";
const maximumOutputBytes = 1_048_576;

type PreparedDiagnostic = {
  readonly manifestPath: string;
  readonly manifest: JsonObject;
  readonly manifestDocument: string;
  readonly head: string;
  readonly wifiPath: string;
  readonly poolPath: string;
  readonly backup: JsonObject;
  readonly restoreBundle: RestoreBundle;
  readonly restoreBundlePath: string;
  readonly host: string;
  readonly expectedPeer: string;
};

export class NoiseDiagnosticError extends Error {
  public constructor(
    public readonly category: string,
    public readonly checkpoint: string,
  ) {
    super(`${category}:${checkpoint}`);
    this.name = "NoiseDiagnosticError";
  }
}

function fail(category: string, _message: string, checkpoint = "unclassified"): never {
  throw new NoiseDiagnosticError(category, checkpoint);
}

function object(value: unknown, checkpoint: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    fail("evidence_invalid", "object required", checkpoint);
  }
  return value as JsonObject;
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

export function noiseDiagnosticWorkspaceRoot(
  environment: NodeJS.ProcessEnv = process.env,
  currentDirectory = process.cwd(),
): string {
  const configured = environment["BUILD_WORKSPACE_DIRECTORY"];
  return sourceWorkspaceRoot(configured === undefined
    ? [currentDirectory]
    : [configured, currentDirectory]);
}

export function parseNoiseDiagnosticArgs(
  action: string | undefined,
  values: readonly string[],
): NoiseDiagnosticArgs {
  if (action !== "preflight" && action !== "start") {
    fail("invalid_invocation", "action required", "invocation");
  }
  const parsed = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") {
      if (parsed.has(key)) fail("invalid_invocation", "duplicate", "invocation");
      parsed.set(key, true);
      continue;
    }
    const value = values[index + 1];
    if (key === undefined || !key.startsWith("--") || value === undefined
      || value.startsWith("--") || parsed.has(key)) {
      fail("invalid_invocation", "malformed option", "invocation");
    }
    parsed.set(key, value);
    index += 1;
  }
  const allowed = new Set([
    "--board", "--port", "--package-manifest", "--wifi-credentials", "--restore-bundle",
    "--private-root", "--projection", "--plan", "--diagnostic-ordinal", "--redact-evidence",
  ]);
  if ([...parsed.keys()].some(key => !allowed.has(key))) {
    fail("invalid_invocation", "unsupported option", "invocation");
  }
  const value = (key: string): string => {
    const candidate = parsed.get(key);
    if (typeof candidate !== "string" || candidate.length === 0) {
      fail("invalid_invocation", "required option missing", "invocation");
    }
    return candidate;
  };
  if (value("--board") !== "205"
    || value("--package-manifest") !== expectedPackageManifest
    || value("--wifi-credentials") !== expectedWifiCredentials
    || value("--restore-bundle") !== expectedRestoreBundle
    || value("--private-root") !== expectedRoot
    || value("--projection") !== expectedProjection
    || value("--plan") !== expectedPlan
    || value("--diagnostic-ordinal") !== "2"
    || parsed.get("--redact-evidence") !== true) {
    fail("invalid_invocation", "contract mismatch", "invocation");
  }
  return {
    action,
    board: "205",
    port: value("--port"),
    packageManifest: value("--package-manifest"),
    wifiCredentials: value("--wifi-credentials"),
    restoreBundle: expectedRestoreBundle,
    privateRoot: expectedRoot,
    projection: expectedProjection,
    plan: expectedPlan,
    diagnosticOrdinal: 2,
    redactEvidence: true,
  };
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  let metadata;
  try { metadata = await lstat(candidate); }
  catch { fail("evidence_invalid", "protected input missing", "protected_inputs"); }
  if (metadata.isSymbolicLink()
    || (directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    fail("evidence_invalid", "protected input invalid", "protected_inputs");
  }
}

async function requireAbsent(candidate: string): Promise<void> {
  try {
    await lstat(candidate);
    fail("evidence_invalid", "output exists", "outputs_absent");
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
    fail("hardware_blocked", "sole pool input required", "pool_restore_input");
  }
  await requireMode(candidates[0], 0o600, false);
  return candidates[0];
}

async function preflight(workspace: string, args: NoiseDiagnosticArgs): Promise<PreparedDiagnostic> {
  const privateRoot = path.join(workspace, args.privateRoot);
  const projection = path.join(workspace, args.projection);
  await requireAbsent(privateRoot);
  await requireAbsent(projection);
  const ignored = await runCampaignProcess(workspace, "git", ["check-ignore", "-q", args.privateRoot], 5_000);
  if (ignored.exitCode !== 0) fail("evidence_invalid", "root is not ignored", "private_path_ignored");
  const planDocument = await readFile(path.join(workspace, args.plan), "utf8");
  const tasks = await readFile(path.join(workspace, "TASKS.md"), "utf8");
  if (sha256(planDocument) !== expectedPlanSha256 || !tasks.includes(`### ${taskId}`)) {
    fail("evidence_invalid", "plan binding invalid", "source_identity");
  }
  const [headResult, status, sync] = await Promise.all([
    runCampaignProcess(workspace, "git", ["rev-parse", "HEAD"], 5_000),
    runCampaignProcess(workspace, "git", ["status", "--porcelain", "--untracked-files=all"], 5_000),
    runCampaignProcess(workspace, "git", ["rev-list", "--left-right", "--count", "HEAD...@{upstream}"], 5_000),
  ]);
  const head = headResult.stdout.trim();
  if (headResult.exitCode !== 0 || status.exitCode !== 0 || status.stdout.trim() !== ""
    || sync.exitCode !== 0 || sync.stdout.trim() !== "0\t0") {
    fail("evidence_invalid", "source is not clean and pushed", "source_identity");
  }
  const manifestPath = path.join(workspace, args.packageManifest);
  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = object(JSON.parse(manifestDocument), "source_identity");
  if (manifest["schema_version"] !== 3 || manifest["source_commit"] !== head
    || manifest["reference_commit"] !== "c1915b0a63bfabebdb95a515cedfee05146c1d50"
    || typeof manifest["app_elf_sha256"] !== "string") {
    fail("evidence_invalid", "package identity mismatch", "source_identity");
  }
  const wifiPath = path.join(workspace, args.wifiCredentials);
  const poolPath = await solePoolInput(workspace);
  await requireMode(wifiPath, 0o600, false);
  const backupPath = path.join(workspace, backupRelative);
  await requireMode(backupPath, 0o600, false);
  const backupDocument = await readFile(backupPath, "utf8");
  if (sha256(backupDocument) !== backupSha256) {
    fail("evidence_invalid", "backup drift", "restoration_inputs");
  }
  const backup = object(JSON.parse(backupDocument), "restoration_inputs");
  const restore = await admitStratumV2RestoreBundle(workspace, args.restoreBundle, runCampaignProcess);
  const flashProgram = path.join(workspace, "bazel-bin/tools/flash/flash");
  const detector = await runCampaignProcess(
    workspace,
    flashProgram,
    ["detect", "--board", "205", "--port", args.port],
    120_000,
  );
  if (detector.exitCode !== 0) fail("hardware_blocked", "detector failed", "device_admission");
  const origin = await monitorRuntimeOrigin(workspace, flashProgram, args.port, runCampaignProcess, fail);
  const settings = await fetchRuntimeObject(origin, "/api/system/info", fail);
  await validateRestorableInputs(settings, wifiPath, poolPath, (category, message) => {
    fail(category, message, "restoration_inputs");
  });
  if (!restoreRuntimeMatches(restore.bundle, settings)
    || !["paused", "safe_blocked"].includes(String(settings["miningActivity"] ?? ""))
    || Number(settings["hashRate"] ?? 0) !== 0
    || Number(settings["sharesAccepted"] ?? 0) !== 0
    || Number(settings["sharesRejected"] ?? 0) !== 0) {
    fail("hardware_blocked", "current runtime is not safely restored", "runtime_admission");
  }
  let host: string;
  try { host = sameSubnetFixtureAddress(origin); }
  catch { fail("hardware_blocked", "same subnet route unavailable", "fixture_route"); }
  return {
    manifestPath,
    manifest,
    manifestDocument,
    head,
    wifiPath,
    poolPath,
    backup,
    restoreBundle: restore.bundle,
    restoreBundlePath: restore.path,
    host,
    expectedPeer: origin.hostname,
  };
}

type FixtureOwner = {
  readonly child: ChildProcess;
  readonly completion: Promise<number>;
  readonly output: Buffer[];
};

function startFixture(
  workspace: string,
  fixtureRoot: string,
  host: string,
  expectedPeer: string,
): FixtureOwner {
  const child = spawn(
    path.join(workspace, "bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture"),
    [
      "--private-root", fixtureRoot, "--listen-address", `${host}:0`,
      "--accept-timeout-seconds", "300", "--session-timeout-seconds", "120",
      "--mode", "handshake-only",
      "--expected-peer-address", expectedPeer,
    ],
    { cwd: workspace, env: process.env, detached: true, stdio: ["ignore", "pipe", "pipe"] },
  );
  const output: Buffer[] = [];
  let outputBytes = 0;
  for (const stream of [child.stdout, child.stderr]) {
    stream?.on("data", (chunk: Buffer) => {
      outputBytes += chunk.length;
      if (outputBytes <= maximumOutputBytes) output.push(chunk);
      else terminateFixture(child);
    });
  }
  const completion = new Promise<number>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", code => resolve(code ?? 1));
  });
  return { child, completion, output };
}

function terminateFixture(child: ChildProcess): void {
  terminateManagedProcessGroup(child);
}

async function waitForFixtureReady(candidate: string): Promise<JsonObject> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    try { return object(JSON.parse(await readFile(candidate, "utf8")), "fixture_ready"); }
    catch { await new Promise(resolve => setTimeout(resolve, 25)); }
  }
  fail("timeout", "fixture readiness timed out", "fixture_ready");
}

async function writePrivate(candidate: string, value: unknown): Promise<string> {
  const document = `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(candidate, document, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(candidate, 0o600);
  return document;
}

function stagesFromMonitor(output: string): JsonObject {
  const stages: JsonObject = {
    tcp_connected: false,
    act_one_created: false,
    act_one_sent: false,
    act_two_received: false,
    time_sampled: false,
    authenticated: false,
  };
  for (const match of output.matchAll(/stratum_v2_noise_diagnostic=(\{[^\r\n]+\})/gu)) {
    try {
      const marker = object(JSON.parse(match[1] ?? ""), "diagnostic_marker");
      const stage = marker["stage"];
      if (typeof stage === "string" && Object.hasOwn(stages, stage)) stages[stage] = true;
    } catch { continue; }
  }
  return stages;
}

function terminalFromMonitor(output: string): JsonObject {
  const matches = [...output.matchAll(/stratum_v2_noise_terminal=(\{[^\r\n]+\})/gu)];
  const last = matches.at(-1)?.[1];
  if (last === undefined) return { category: "terminal_missing", accepted: false };
  try { return object(JSON.parse(last), "diagnostic_terminal"); }
  catch { return { category: "terminal_malformed", accepted: false }; }
}

async function exactRestore(
  workspace: string,
  args: NoiseDiagnosticArgs,
  prepared: PreparedDiagnostic,
): Promise<JsonObject> {
  const restoreRootRelative = path.join(args.privateRoot, "restoration");
  const restoreRoot = path.join(workspace, restoreRootRelative);
  await mkdir(restoreRoot, { mode: 0o700 });
  await chmod(restoreRoot, 0o700);
  const bundleDocument = await readFile(prepared.restoreBundlePath, "utf8");
  const recoveryPlanDocument = await readFile(path.join(workspace, recoveryPlan), "utf8");
  const authorizationRelative = path.join(restoreRootRelative, "restore-authorization.private.json");
  await writePrivate(path.join(workspace, authorizationRelative), {
    schema_version: "bitaxe-stratum-v2-restore-authorization-v1",
    board: 205,
    ordinal: 2,
    action: "diagnostic_restore",
    current_source_commit: prepared.head,
    reference_commit: prepared.manifest["reference_commit"],
    bundle_sha256: sha256(bundleDocument),
    bundle_capture_source_commit: prepared.restoreBundle.capture_source_commit,
    recovery_plan_sha256: sha256(recoveryPlanDocument),
    remediation_plan_sha256: expectedPlanSha256,
  });
  const flashProgram = path.join(workspace, "bazel-bin/tools/flash/flash");
  const restored = await runManagedDiagnosticProcess(workspace, flashProgram, [
    "restore-installed", "--board", "205", "--port", args.port,
    "--restore-bundle", args.restoreBundle,
    "--restore-authorization", authorizationRelative,
    "--remediation-plan", args.plan,
    "--private-root", restoreRootRelative,
    "--wifi-credentials", args.wifiCredentials,
    "--redact-evidence",
  ], 900_000, "restoration_child");
  if (restored.exitCode !== 0) fail("hardware_blocked", "firmware restore failed", "restoration");
  const origin = await monitorRuntimeOrigin(workspace, flashProgram, args.port, runCampaignProcess, fail);
  await restoreSelfTestSettings(origin, prepared.backup, prepared.wifiPath, prepared.poolPath);
  const confirmed = await fetchRuntimeObject(origin, "/api/system/info", fail);
  if (!restoreRuntimeMatches(prepared.restoreBundle, confirmed)
    || !["paused", "safe_blocked"].includes(String(confirmed["miningActivity"] ?? ""))
    || Number(confirmed["hashRate"] ?? 0) !== 0
    || Number(confirmed["sharesAccepted"] ?? 0) !== 0
    || Number(confirmed["sharesRejected"] ?? 0) !== 0) {
    fail("hardware_blocked", "final restoration mismatch", "restoration");
  }
  return confirmed;
}

async function publishProjection(
  workspace: string,
  args: NoiseDiagnosticArgs,
  projection: JsonObject,
  expectedSource: string,
): Promise<void> {
  const privateCandidate = path.join(workspace, args.privateRoot, "projection-candidate.private.json");
  await writePrivate(privateCandidate, projection);
  const validator = noiseDiagnosticValidatorProgram(workspace);
  const validated = await runCampaignProcess(
    workspace,
    validator,
    [privateCandidate, expectedSource, "2"],
    30_000,
  );
  if (validated.exitCode !== 0) fail("evidence_invalid", "projection rejected", "projection");
  const publicPath = path.join(workspace, args.projection);
  await mkdir(path.dirname(publicPath), { recursive: true });
  const temporary = `${publicPath}.tmp`;
  await writeFile(temporary, `${JSON.stringify(projection, null, 2)}\n`, { flag: "wx" });
  await rename(temporary, publicPath);
}

export async function inspectNoiseDiagnosticPreflight(
  workspace: string,
  args: NoiseDiagnosticArgs,
): Promise<JsonObject> {
  await preflight(workspace, args);
  return {
    schema_version: "bitaxe-stratum-v2-noise-diagnostic-preflight-v1",
    status: "ready",
    checkpoint: "pre_effect_ready",
    effect_started: false,
    private_root_created: false,
  };
}

export async function runNoiseDiagnostic(
  workspace: string,
  args: NoiseDiagnosticArgs,
): Promise<JsonObject> {
  const prepared = await preflight(workspace, args);
  const privateRoot = path.join(workspace, args.privateRoot);
  await mkdir(path.dirname(privateRoot), { recursive: true, mode: 0o700 });
  await chmod(path.dirname(privateRoot), 0o700);
  await mkdir(privateRoot, { mode: 0o700 });
  await chmod(privateRoot, 0o700);
  await writePrivate(path.join(privateRoot, "settings-backup.private.json"), prepared.backup);
  const fixtureRoot = path.join(privateRoot, "fixture");
  const fixture = startFixture(
    workspace,
    fixtureRoot,
    prepared.host,
    prepared.expectedPeer,
  );
  let terminal: JsonObject = { category: "process_failed", accepted: false };
  let stages: JsonObject = {};
  let fixtureTerminal: JsonObject = { progress: {} };
  let diagnosticChild: ManagedDiagnosticProcessResult = { exitCode: 1, stdout: "", stderr: "" };
  let earliestCategory = "process_failed";
  let effectStarted = false;
  let maybeDiagnosticError: unknown;
  let fixtureCleanupComplete = false;
  let fixtureExit: number | null = null;
  try {
    const ready = await waitForFixtureReady(path.join(fixtureRoot, "ready.json"));
    const poolRelative = path.join(args.privateRoot, "fixture-pool.private.json");
    await writePrivate(path.join(workspace, poolRelative), {
      poolURL: prepared.host,
      poolPort: ready["port"],
      poolUser: "bitaxe-fixture",
      poolPassword: "",
      stratumProtocol: "SV2",
      stratumV2ChannelType: "standard",
      stratumV2AuthorityPubkey: ready["authority_public_key"],
    });
    const lease = randomBytes(8).toString("hex").replace(/^0{16}$/u, "0000000000000001");
    const intentRelative = path.join(args.privateRoot, "intent.private.json");
    await writePrivate(path.join(workspace, intentRelative), {
      schema_version: "bitaxe-stratum-v2-noise-diagnostic-intent-v1",
      board: 205,
      diagnostic_ordinal: 2,
      source_commit: prepared.head,
      reference_commit: prepared.manifest["reference_commit"],
      app_elf_sha256: prepared.manifest["app_elf_sha256"],
      plan_path: args.plan,
      plan_sha256: expectedPlanSha256,
      lease_hex: lease,
    });
    effectStarted = true;
    diagnosticChild = await runManagedDiagnosticProcess(
      workspace,
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      [
        "noise-diagnostic", "--board", "205", "--port", args.port,
        "--manifest", args.packageManifest,
        "--wifi-credentials", args.wifiCredentials,
        "--pool-credentials", poolRelative,
        "--intent", intentRelative,
        "--capture-timeout-seconds", "120",
        "--redact-evidence",
      ],
      420_000,
      "diagnostic_child",
    );
    await writePrivate(path.join(privateRoot, "diagnostic-child.private.json"), {
      exit_code: diagnosticChild.exitCode,
      stdout_sha256: sha256(diagnosticChild.stdout),
      stderr_sha256: sha256(diagnosticChild.stderr),
    });
    stages = stagesFromMonitor(diagnosticChild.stdout);
    terminal = terminalFromMonitor(diagnosticChild.stdout);
    earliestCategory = String(terminal["category"] ?? "terminal_missing");
    fixtureExit = await fixture.completion;
    fixtureTerminal = object(
      JSON.parse(await readFile(path.join(fixtureRoot, "terminal.json"), "utf8")),
      "fixture_terminal",
    );
    if (diagnosticChild.exitCode !== 0 && earliestCategory === "terminal_missing") {
      earliestCategory = "diagnostic_process";
    } else if (fixtureExit !== 0 && terminal["accepted"] === true) {
      earliestCategory = "fixture_process";
    }
  } catch (error) {
    maybeDiagnosticError = error;
    earliestCategory = error instanceof NoiseDiagnosticError
      || error instanceof ManagedDiagnosticProcessError
      ? error.category
      : "process_failed";
  } finally {
    terminateFixture(fixture.child);
    const cleanupResult = await Promise.race([
      fixture.completion.catch(() => 1),
      new Promise<number>(resolve => setTimeout(() => resolve(-1), 5_000)),
    ]);
    fixtureCleanupComplete = cleanupResult !== -1;
    fixtureExit ??= cleanupResult === -1 ? null : cleanupResult;
  }
  try {
    fixtureTerminal = object(
      JSON.parse(await readFile(path.join(fixtureRoot, "terminal.json"), "utf8")),
      "fixture_terminal",
    );
  } catch {
    // The closed fixture-process category remains authoritative when no terminal exists.
  }
  await writePrivate(path.join(privateRoot, "fixture-child.private.json"), {
    exit_code: fixtureExit,
    cleanup_complete: fixtureCleanupComplete,
    output_sha256: sha256(Buffer.concat(fixture.output)),
  });
  if (!effectStarted && maybeDiagnosticError !== undefined) throw maybeDiagnosticError;
  const finalRuntime = await exactRestore(workspace, args, prepared);
  if (!fixtureCleanupComplete) {
    fail("hardware_blocked", "fixture cleanup failed", "cleanup");
  }
  const fixtureProgress = object(fixtureTerminal["progress"] ?? {}, "fixture_terminal");
  const accepted = terminal["accepted"] === true
    && fixtureTerminal["status"] === "accepted"
    && fixtureTerminal["terminal_category"] === "accepted"
    && diagnosticChild.exitCode === 0;
  const projection: JsonObject = {
    schema_version: "bitaxe-stratum-v2-noise-diagnostic-projection-v1",
    status: accepted ? "accepted" : "failed",
    board: 205,
    diagnostic_ordinal: 2,
    source_commit: prepared.head,
    reference_commit: prepared.manifest["reference_commit"],
    app_elf_sha256: prepared.manifest["app_elf_sha256"],
    plan_sha256: expectedPlanSha256,
    package_manifest_sha256: sha256(prepared.manifestDocument),
    terminal_category: accepted ? "accepted" : earliestCategory,
    stages,
    fixture: fixtureProgress,
    campaign_started: false,
    mining_started: false,
    asic_touched: false,
    fan_touched: false,
    voltage_touched: false,
    restoration: {
      identity_exact: restoreRuntimeMatches(prepared.restoreBundle, finalRuntime),
      settings_exact: true,
      mineonboot_disabled: finalRuntime["startMiningOnBoot"] === false,
      mining_inactive: ["paused", "safe_blocked"].includes(String(finalRuntime["miningActivity"] ?? "")),
      zero_work: Number(finalRuntime["hashRate"] ?? 0) === 0,
      usb_cleanup_complete: fixtureCleanupComplete,
      owned_processes_remaining: 0,
    },
    redaction_complete: true,
  };
  await publishProjection(workspace, args, projection, prepared.head);
  return projection;
}

export function noiseDiagnosticFailureResult(error: unknown): JsonObject {
  return {
    schema_version: "bitaxe-stratum-v2-noise-diagnostic-result-v1",
    status: "failed",
    category: error instanceof NoiseDiagnosticError
      || error instanceof ManagedDiagnosticProcessError
      ? error.category
      : "process_failed",
    checkpoint: error instanceof NoiseDiagnosticError
      || error instanceof ManagedDiagnosticProcessError
      ? error.checkpoint
      : "unclassified",
    projection_published: false,
  };
}
