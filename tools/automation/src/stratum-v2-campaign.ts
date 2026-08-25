import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import { chmod, mkdir, readFile, readdir, rename, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { restoreSelfTestSettings } from "./self-test-campaign-restoration.js";
import { type JsonObject, prepareStratumV2Campaign } from "./stratum-v2-campaign-preflight.js";
import { fetchRuntimeObject, monitorRuntimeOrigin, prepareStratumV2RuntimeAdmission } from "./stratum-v2-runtime-admission.js";
import type { RestoreBundle } from "./stratum-v2-restore-model.js";
import { admitStratumV2RestoreBundle, restoreRuntimeMatches } from "./stratum-v2-restore-admission.js";
import { sourceWorkspaceRoot } from "./workspace.js";

export type CampaignArgs = {
  readonly board: "205";
  readonly port: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly restoreBundle: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly durationSeconds: 180;
  readonly redactEvidence: true;
};

export type ProcessResult = {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
};
export type StratumV2CampaignCheckpoint =
  | "invocation"
  | "workspace"
  | "outputs_absent"
  | "private_path_ignored"
  | "wifi_restore_input"
  | "pool_restore_input"
  | "source_identity"
  | "runtime_monitor_process"
  | "runtime_origin"
  | "runtime_settings"
  | "restoration_inputs"
  | "restore_package"
  | "pre_effect_ready"
  | "runtime_admission_ready"
  | "unclassified";

export type StratumV2CampaignFailureResult = {
  readonly schema_version: "bitaxe-stratum-v2-campaign-result-v1";
  readonly status: "failed";
  readonly category: string;
  readonly checkpoint: StratumV2CampaignCheckpoint;
  readonly projection_published: false;
};

export type StratumV2CampaignPreflightResult = {
  readonly schema_version: "bitaxe-stratum-v2-campaign-preflight-v1";
  readonly status: "ready";
  readonly checkpoint: "pre_effect_ready";
  readonly effect_started: false;
  readonly private_root_created: false;
};

export type StratumV2RuntimeAdmissionResult = {
  readonly schema_version: "bitaxe-stratum-v2-runtime-admission-v1";
  readonly status: "ready";
  readonly checkpoint: "runtime_admission_ready";
  readonly effect_started: false;
  readonly private_root_created: false;
};

const expectedPrivateRoot = "scratch/str005-stratum-v2/attempt-004";
const expectedProjection = "docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json";
const expectedRestoreBundle =
  "scratch/str005-installed-package-recovery/recovery-004/restore-bundle.private.json";
const maximumOutputBytes = 1_048_576;

export function campaignWorkspaceRoot(
  environment: NodeJS.ProcessEnv = process.env,
  currentDirectory: string = process.cwd(),
): string {
  const configured = environment["BUILD_WORKSPACE_DIRECTORY"];
  const starts = configured === undefined ? [currentDirectory] : [configured, currentDirectory];
  try {
    return sourceWorkspaceRoot(starts);
  } catch {
    throw new StratumV2CampaignError("evidence_invalid", "workspace unavailable", "workspace");
  }
}

export class StratumV2CampaignError extends Error {
  public constructor(
    public readonly category: string,
    message: string,
    public readonly checkpoint: StratumV2CampaignCheckpoint = "unclassified",
  ) {
    super(message);
    this.name = "StratumV2CampaignError";
  }
}

export function stratumV2CampaignFailureResult(error: unknown): StratumV2CampaignFailureResult {
  return {
    schema_version: "bitaxe-stratum-v2-campaign-result-v1",
    status: "failed",
    category: error instanceof StratumV2CampaignError ? error.category : "process_failed",
    checkpoint: error instanceof StratumV2CampaignError ? error.checkpoint : "unclassified",
    projection_published: false,
  };
}

function fail(
  category: string,
  message: string,
  checkpoint: StratumV2CampaignCheckpoint = "unclassified",
): never {
  throw new StratumV2CampaignError(category, message, checkpoint);
}

function object(
  value: unknown,
  context: string,
  checkpoint: StratumV2CampaignCheckpoint = "unclassified",
): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    fail("evidence_invalid", `${context} must be an object`, checkpoint);
  }
  return value as JsonObject;
}

function requiredString(
  value: JsonObject,
  key: string,
  context: string,
  checkpoint: StratumV2CampaignCheckpoint = "unclassified",
): string {
  const candidate = value[key];
  if (typeof candidate !== "string" || candidate.length === 0) {
    fail("evidence_invalid", `${context} is missing identity`, checkpoint);
  }
  return candidate;
}

function sha256(value: string | Buffer): string { return createHash("sha256").update(value).digest("hex"); }

export function parseStratumV2CampaignArgs(values: readonly string[]): CampaignArgs {
  const parsed = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") {
      if (parsed.has(key)) fail("invalid_invocation", "duplicate campaign option", "invocation");
      parsed.set(key, true);
      continue;
    }
    if (key === undefined || !key.startsWith("--")) {
      fail("invalid_invocation", "campaign option is malformed", "invocation");
    }
    const optionValue = values[index + 1];
    if (optionValue === undefined || optionValue.startsWith("--") || parsed.has(key)) {
      fail("invalid_invocation", "campaign option value is missing or duplicated", "invocation");
    }
    parsed.set(key, optionValue);
    index += 1;
  }
  const allowed = new Set([
    "--board", "--port", "--package-manifest", "--wifi-credentials", "--restore-bundle", "--private-root",
    "--projection", "--duration-seconds", "--redact-evidence",
  ]);
  if ([...parsed.keys()].some(key => !allowed.has(key))) {
    fail("invalid_invocation", "campaign option is unsupported", "invocation");
  }
  const value = (key: string): string => {
    const candidate = parsed.get(key);
    if (typeof candidate !== "string" || candidate.length === 0) {
      fail("invalid_invocation", "campaign option is required", "invocation");
    }
    return candidate;
  };
  const board = value("--board");
  const duration = value("--duration-seconds");
  const privateRoot = value("--private-root");
  const projection = value("--projection");
  const restoreBundle = value("--restore-bundle");
  if (board !== "205" || duration !== "180" || parsed.get("--redact-evidence") !== true
    || privateRoot !== expectedPrivateRoot || projection !== expectedProjection
    || restoreBundle !== expectedRestoreBundle) {
    fail("invalid_invocation", "campaign contract does not match attempt-004", "invocation");
  }
  return {
    board,
    port: value("--port"),
    packageManifest: value("--package-manifest"),
    wifiCredentials: value("--wifi-credentials"),
    restoreBundle,
    privateRoot,
    projection,
    durationSeconds: 180,
    redactEvidence: true,
  };
}

export async function runCampaignProcess(
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
): Promise<ProcessResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(program, [...args], {
      cwd: workspace,
      env: process.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    let outputBytes = 0;
    let timedOut = false;
    const capture = (destination: Buffer[], chunk: Buffer) => {
      outputBytes += chunk.length;
      if (outputBytes > maximumOutputBytes) {
        child.kill("SIGTERM");
        return;
      }
      destination.push(chunk);
    };
    child.stdout.on("data", (chunk: Buffer) => capture(stdout, chunk));
    child.stderr.on("data", (chunk: Buffer) => capture(stderr, chunk));
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill("SIGTERM");
    }, timeoutMillis);
    child.once("error", reject);
    child.once("close", exitCode => {
      clearTimeout(timer);
      if (timedOut) reject(new StratumV2CampaignError("timeout", "campaign child timed out"));
      else if (outputBytes > maximumOutputBytes) {
        reject(new StratumV2CampaignError("evidence_invalid", "campaign child output exceeded bound"));
      } else {
        resolve({
          exitCode: exitCode ?? 1,
          stdout: Buffer.concat(stdout).toString("utf8"),
          stderr: Buffer.concat(stderr).toString("utf8"),
        });
      }
    });
  });
}

async function writePrivateJson(candidate: string, value: unknown): Promise<string> {
  const document = `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(candidate, document, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(candidate, 0o600);
  return document;
}

type RestorePackage = {
  readonly manifestPath: string;
  readonly sourceCommit: string;
  readonly appElfSha256: string;
  readonly factorySha256: string;
};

async function walkManifests(root: string, output: string[], budget: { remaining: number }): Promise<void> {
  if (budget.remaining <= 0) fail("evidence_invalid", "restore-package inventory exceeded bound");
  budget.remaining -= 1;
  const entries = await readdir(root, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isSymbolicLink()) continue;
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) await walkManifests(candidate, output, budget);
    else if (entry.isFile() && /(?:package-manifest|bitaxe-ultra205-package)\.json$/u.test(entry.name)) {
      output.push(candidate);
    }
  }
}

export async function selectRestorePackage(
  workspace: string,
  appElfSha256: string,
): Promise<RestorePackage> {
  const manifestPaths: string[] = [];
  await walkManifests(path.join(workspace, "scratch"), manifestPaths, { remaining: 10_000 });
  const candidates: RestorePackage[] = [];
  for (const manifestPath of manifestPaths.sort()) {
    try {
      const manifest = object(JSON.parse(await readFile(manifestPath, "utf8")), "manifest");
      if (manifest["app_elf_sha256"] !== appElfSha256) continue;
      const artifacts = manifest["artifacts"];
      if (!Array.isArray(artifacts)) continue;
      const factory = artifacts
        .map(value => object(value, "artifact"))
        .find(value => value["kind"] === "factory_merged_image");
      if (factory === undefined) continue;
      const factoryPath = path.resolve(path.dirname(manifestPath), requiredString(factory, "path", "factory"));
      if (!(await stat(factoryPath)).isFile()) continue;
      candidates.push({
        manifestPath,
        sourceCommit: requiredString(manifest, "source_commit", "manifest"),
        appElfSha256,
        factorySha256: requiredString(factory, "sha256", "factory"),
      });
    } catch {
      continue;
    }
  }
  const identities = new Set(candidates.map(candidate =>
    `${candidate.sourceCommit}:${candidate.appElfSha256}:${candidate.factorySha256}`));
  if (candidates.length === 0 || identities.size !== 1 || candidates[0] === undefined) {
    fail("hardware_blocked", "exact prior package is unavailable or ambiguous");
  }
  return candidates[0];
}

async function localIpv4(workspace: string): Promise<string> {
  const route = await runCampaignProcess(
    workspace,
    "/sbin/route",
    ["-n", "get", "default"],
    5_000,
  );
  const interfaceMatch = /^\s*interface:\s*([A-Za-z0-9]+)\s*$/mu.exec(route.stdout);
  if (route.exitCode !== 0 || interfaceMatch?.[1] === undefined) {
    fail("hardware_blocked", "local interface is unavailable");
  }
  const address = await runCampaignProcess(
    workspace,
    "/usr/sbin/ipconfig",
    ["getifaddr", interfaceMatch[1]],
    5_000,
  );
  const value = address.stdout.trim();
  if (address.exitCode !== 0 || !/^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$/u.test(value)) {
    fail("hardware_blocked", "local IPv4 address is unavailable");
  }
  return value;
}

async function waitForReady(candidate: string): Promise<JsonObject> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    try {
      return object(JSON.parse(await readFile(candidate, "utf8")), "fixture readiness");
    } catch {
      await new Promise(resolve => setTimeout(resolve, 25));
    }
  }
  fail("timeout", "fixture readiness deadline elapsed");
}

function startFixture(workspace: string, program: string, fixtureRoot: string) {
  const child = spawn(program, [
    "--private-root", fixtureRoot,
    "--accept-timeout-seconds", "120",
    "--session-timeout-seconds", "180",
  ], { cwd: workspace, env: process.env, stdio: ["ignore", "pipe", "pipe"] });
  const output: Buffer[] = [];
  let bytes = 0;
  for (const stream of [child.stdout, child.stderr]) {
    stream.on("data", (chunk: Buffer) => {
      bytes += chunk.length;
      if (bytes <= maximumOutputBytes) output.push(chunk);
      else child.kill("SIGTERM");
    });
  }
  const completion = new Promise<number>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", code => resolve(code ?? 1));
  });
  return {
    child,
    output,
    wait: () => completion,
  };
}

async function restorePackageAndSettings(
  workspace: string,
  flashProgram: string,
  args: CampaignArgs,
  restoreBundlePath: string,
  restoreBundle: RestoreBundle,
  backup: JsonObject,
  poolPath: string,
  changedPackage: boolean,
): Promise<JsonObject> {
  if (changedPackage) {
    const flash = await runCampaignProcess(workspace, flashProgram, [
      "restore-installed", "--board", "205", "--port", args.port,
      "--restore-bundle", restoreBundlePath,
      "--wifi-credentials", args.wifiCredentials, "--redact-evidence",
    ], 180_000);
    if (flash.exitCode !== 0) fail("hardware_blocked", "prior package restoration failed");
  }
  const origin = await monitorRuntimeOrigin(
    workspace,
    flashProgram,
    args.port,
    runCampaignProcess,
    fail,
  );
  await restoreSelfTestSettings(origin, backup, args.wifiCredentials, poolPath);
  const confirmed = await fetchRuntimeObject(origin, "/api/system/info", fail);
  if (!restoreRuntimeMatches(restoreBundle, confirmed)) {
    fail("hardware_blocked", "final package or settings restoration mismatch");
  }
  return confirmed;
}

export async function inspectStratumV2CampaignPreflight(
  workspace: string,
  args: CampaignArgs,
): Promise<StratumV2CampaignPreflightResult> {
  await prepareStratumV2Campaign(workspace, args, {
    runProcess: runCampaignProcess,
    fail,
  });
  await admitStratumV2RestoreBundle(workspace, args.restoreBundle, runCampaignProcess);
  return {
    schema_version: "bitaxe-stratum-v2-campaign-preflight-v1",
    status: "ready",
    checkpoint: "pre_effect_ready",
    effect_started: false,
    private_root_created: false,
  };
}

async function runtimeAdmission(
  workspace: string,
  args: CampaignArgs,
): ReturnType<typeof prepareStratumV2RuntimeAdmission> {
  const restore = await admitStratumV2RestoreBundle(
    workspace,
    args.restoreBundle,
    runCampaignProcess,
  );
  return prepareStratumV2RuntimeAdmission(workspace, args, {
    fail,
    preparePreflight: () => prepareStratumV2Campaign(
      workspace,
      args,
      { runProcess: runCampaignProcess, fail },
    ),
    runProcess: runCampaignProcess,
    restoreBundle: restore.bundle,
    restoreBundlePath: restore.path,
  });
}

export async function inspectStratumV2RuntimeAdmission(
  workspace: string,
  args: CampaignArgs,
): Promise<StratumV2RuntimeAdmissionResult> {
  await runtimeAdmission(workspace, args);
  return {
    schema_version: "bitaxe-stratum-v2-runtime-admission-v1",
    status: "ready",
    checkpoint: "runtime_admission_ready",
    effect_started: false,
    private_root_created: false,
  };
}

export async function runStratumV2Campaign(
  workspace: string,
  args: CampaignArgs,
): Promise<JsonObject> {
  const admission = await runtimeAdmission(workspace, args);
  const {
    privateRoot,
    projection,
    manifestPath,
    wifiPath,
    poolPath,
    manifestDocument,
    manifest,
    head,
    settings,
    theme,
    restoreBundle,
    restoreBundlePath,
    changedPackage,
  } = admission;
  const privateParent = path.dirname(privateRoot);
  await mkdir(privateParent, { recursive: true, mode: 0o700 });
  await chmod(privateParent, 0o700);
  await mkdir(privateRoot, { recursive: false, mode: 0o700 });
  await chmod(privateRoot, 0o700);
  const backup: JsonObject = { settings, theme };
  const backupDocument = await writePrivateJson(path.join(privateRoot, "settings-backup.private.json"), backup);
  const fixtureRoot = path.join(privateRoot, "fixture");
  const fixture = startFixture(
    workspace,
    path.join(workspace, "bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture"),
    fixtureRoot,
  );
  let fixtureExit = 1;
  let effectStarted = false;
  let restorationComplete = false;
  try {
    const ready = await waitForReady(path.join(fixtureRoot, "ready.json"));
    const host = await localIpv4(workspace);
    const fixturePoolPath = path.join(privateRoot, "fixture-pool.private.json");
    await writePrivateJson(fixturePoolPath, {
      poolURL: host,
      poolPort: ready["port"],
      poolUser: "bitaxe-fixture",
      poolPassword: "",
      stratumProtocol: "SV2",
      stratumV2ChannelType: "standard",
      stratumV2AuthorityPubkey: ready["authority_public_key"],
    });
    effectStarted = true;
    const campaign = await runCampaignProcess(
      workspace,
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      [
      "mining-campaign", "--stage", "stratum-v2", "--profile", "conservative",
      "--board", "205", "--port", args.port, "--manifest", manifestPath,
      "--wifi-credentials", wifiPath, "--pool-credentials", fixturePoolPath,
      "--evidence-dir", path.join(args.privateRoot, "campaign"),
      "--duration-seconds", "180", "--redact-evidence",
      ],
      420_000,
    );
    await writePrivateJson(path.join(privateRoot, "campaign-child.private.json"), {
      exit_code: campaign.exitCode,
      stdout_sha256: sha256(campaign.stdout),
      stderr_sha256: sha256(campaign.stderr),
    });
    if (campaign.exitCode !== 0) fail("hardware_blocked", "private V2 campaign failed");
    fixtureExit = await fixture.wait();
    if (fixtureExit !== 0) fail("hardware_blocked", "private V2 fixture failed");
    const fixtureResult = object(
      JSON.parse(await readFile(path.join(fixtureRoot, "result.json"), "utf8")),
      "fixture result",
    );
    if (fixtureResult["status"] !== "accepted" || fixtureResult["share_target_valid"] !== true) {
      fail("evidence_invalid", "fixture result did not validate share");
    }
    await restorePackageAndSettings(
      workspace,
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      args,
      restoreBundlePath,
      restoreBundle,
      backup,
      poolPath,
      changedPackage,
    );
    restorationComplete = true;
    const projectionValue = {
      schema_version: "bitaxe-stratum-v2-campaign-projection-v1",
      status: "accepted",
      board: 205,
      source_commit: head,
      reference_commit: requiredString(manifest, "reference_commit", "manifest"),
      package_manifest_sha256: sha256(manifestDocument),
      settings_backup_sha256: sha256(backupDocument),
      fixture_accepted: true,
      share_target_valid: true,
      safe_stop_complete: true,
      settings_restored: true,
      package_restored: true,
      mineonboot_false: true,
      usb_cleanup_ready: true,
      redaction_status: "passed",
      exact_non_claims: [
        "external_production_pool", "mixed_protocol_live_fallback", "other_boards",
        "unbounded_mining", "ota", "release_readiness",
      ],
    };
    const candidate = `${projection}.candidate`;
    await writeFile(candidate, `${JSON.stringify(projectionValue, null, 2)}\n`, {
      encoding: "utf8", flag: "wx", mode: 0o600,
    });
    const validation = await runCampaignProcess(
      workspace,
      path.join(
        workspace,
        "bazel-bin/tools/automation/stratum_v2_campaign_validator_/stratum_v2_campaign_validator",
      ),
      [
        candidate,
        head,
        requiredString(manifest, "reference_commit", "manifest"),
        sha256(manifestDocument),
      ],
      10_000,
    );
    if (validation.exitCode !== 0) fail("evidence_invalid", "independent projection validation failed");
    await rename(candidate, projection);
    return projectionValue;
  } catch (error) {
    if (fixture.child.exitCode === null) fixture.child.kill("SIGTERM");
    if (effectStarted && !restorationComplete) {
      let restored = false;
      try {
        await restorePackageAndSettings(
          workspace,
          path.join(workspace, "bazel-bin/tools/flash/flash"),
          args,
          restoreBundlePath,
          restoreBundle,
          backup,
          poolPath,
          changedPackage,
        );
        restored = true;
      } catch {
        restored = false;
      }
      await writePrivateJson(path.join(privateRoot, "recovery.private.json"), {
        attempted: true,
        restored,
      });
    }
    throw error;
  } finally {
    if (fixtureExit !== 0 && fixture.child.exitCode === null) fixture.child.kill("SIGTERM");
    if (fixture.output.reduce((total, chunk) => total + chunk.length, 0) > maximumOutputBytes) {
      await writePrivateJson(path.join(privateRoot, "fixture-output.private.json"), {
        status: "over_limit",
      });
    }
  }
}
