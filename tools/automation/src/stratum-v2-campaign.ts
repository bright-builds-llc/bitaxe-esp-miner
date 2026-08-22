import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import {
  chmod,
  mkdir,
  readFile,
  readdir,
  rename,
  stat,
  writeFile,
} from "node:fs/promises";
import path from "node:path";

import { restoreSelfTestSettings } from "./self-test-campaign-restoration.js";

type JsonObject = Record<string, unknown>;
type CampaignArgs = {
  readonly board: "205";
  readonly port: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly durationSeconds: 180;
  readonly redactEvidence: true;
};

type ProcessResult = {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
};

const expectedPrivateRoot = "scratch/str005-stratum-v2/attempt-001";
const expectedProjection =
  "docs/parity/evidence/str005-stratum-v2/stratum-v2-projection.json";
const maximumOutputBytes = 1_048_576;

export class StratumV2CampaignError extends Error {
  public constructor(public readonly category: string, message: string) {
    super(message);
    this.name = "StratumV2CampaignError";
  }
}

function fail(category: string, message: string): never {
  throw new StratumV2CampaignError(category, message);
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    fail("evidence_invalid", `${context} must be an object`);
  }
  return value as JsonObject;
}

function requiredString(value: JsonObject, key: string, context: string): string {
  const candidate = value[key];
  if (typeof candidate !== "string" || candidate.length === 0) {
    fail("evidence_invalid", `${context} is missing identity`);
  }
  return candidate;
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

export function parseStratumV2CampaignArgs(values: readonly string[]): CampaignArgs {
  const parsed = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") {
      if (parsed.has(key)) fail("invalid_invocation", "duplicate campaign option");
      parsed.set(key, true);
      continue;
    }
    if (key === undefined || !key.startsWith("--")) {
      fail("invalid_invocation", "campaign option is malformed");
    }
    const optionValue = values[index + 1];
    if (optionValue === undefined || optionValue.startsWith("--") || parsed.has(key)) {
      fail("invalid_invocation", "campaign option value is missing or duplicated");
    }
    parsed.set(key, optionValue);
    index += 1;
  }
  const allowed = new Set([
    "--board", "--port", "--package-manifest", "--wifi-credentials", "--private-root",
    "--projection", "--duration-seconds", "--redact-evidence",
  ]);
  if ([...parsed.keys()].some(key => !allowed.has(key))) {
    fail("invalid_invocation", "campaign option is unsupported");
  }
  const value = (key: string): string => {
    const candidate = parsed.get(key);
    if (typeof candidate !== "string" || candidate.length === 0) {
      fail("invalid_invocation", "campaign option is required");
    }
    return candidate;
  };
  const board = value("--board");
  const duration = value("--duration-seconds");
  const privateRoot = value("--private-root");
  const projection = value("--projection");
  if (board !== "205" || duration !== "180" || parsed.get("--redact-evidence") !== true
    || privateRoot !== expectedPrivateRoot || projection !== expectedProjection) {
    fail("invalid_invocation", "campaign contract does not match attempt-001");
  }
  return {
    board,
    port: value("--port"),
    packageManifest: value("--package-manifest"),
    wifiCredentials: value("--wifi-credentials"),
    privateRoot,
    projection,
    durationSeconds: 180,
    redactEvidence: true,
  };
}

async function runProcess(
  program: string,
  args: readonly string[],
  timeoutMillis: number,
): Promise<ProcessResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(program, [...args], {
      cwd: process.cwd(),
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

function singleOrigin(monitor: string): URL {
  const candidates = [...monitor.matchAll(/\bhttps?:\/\/[A-Za-z0-9.-]+(?::[0-9]+)?\b/gu)]
    .map(match => match[0])
    .filter((value, index, all) => all.indexOf(value) === index);
  if (candidates.length !== 1 || candidates[0] === undefined) {
    fail("hardware_blocked", "monitor did not provide one current origin");
  }
  return new URL(candidates[0]);
}

async function fetchObject(origin: URL, route: string): Promise<JsonObject> {
  const response = await fetch(new URL(route, origin));
  if (!response.ok) fail("hardware_blocked", "same-origin read failed");
  return object(await response.json(), "same-origin response");
}

async function monitorOrigin(flashProgram: string, port: string): Promise<URL> {
  const outcome = await runProcess(flashProgram, [
    "monitor", "--board", "205", "--port", port, "--capture-timeout-seconds", "15",
  ], 30_000);
  if (outcome.exitCode !== 0) fail("hardware_blocked", "passive monitor failed");
  return singleOrigin(outcome.stdout);
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await stat(candidate);
  if ((directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    fail("evidence_invalid", "protected input mode is invalid");
  }
}

async function requireAbsent(candidate: string): Promise<void> {
  try {
    await stat(candidate);
    fail("evidence_invalid", "fresh campaign output already exists");
  } catch (error) {
    if (error instanceof StratumV2CampaignError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

async function writePrivateJson(candidate: string, value: unknown): Promise<string> {
  const document = `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(candidate, document, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(candidate, 0o600);
  return document;
}

async function poolRestoreInput(root: string): Promise<string> {
  const candidates = (await readdir(root))
    .filter(name => /^pool-credentials(?:-[A-Za-z0-9_-]+)?\.json$/u.test(name))
    .map(name => path.join(root, name));
  if (candidates.length !== 1 || candidates[0] === undefined) {
    fail("hardware_blocked", "exactly one ignored restoration pool input is required");
  }
  await requireMode(candidates[0], 0o600, false);
  return candidates[0];
}

async function validateRestorableInputs(
  settings: JsonObject,
  wifiPath: string,
  poolPath: string,
): Promise<void> {
  const wifi = object(JSON.parse(await readFile(wifiPath, "utf8")), "Wi-Fi input");
  const pool = object(JSON.parse(await readFile(poolPath, "utf8")), "pool input");
  if (settings["startMiningOnBoot"] !== false
    || settings["ssid"] !== wifi["ssid"]
    || settings["stratumURL"] !== pool["poolURL"]
    || settings["stratumPort"] !== pool["poolPort"]
    || settings["stratumUser"] !== pool["poolUser"]
    || settings["useFallbackStratum"] === true
    || (typeof settings["fallbackStratumURL"] === "string"
      && settings["fallbackStratumURL"].length > 0)) {
    fail("hardware_blocked", "local inputs cannot construct exact restoration");
  }
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

async function restorePackageFromManifest(
  manifestPath: string,
  manifest: JsonObject,
): Promise<RestorePackage> {
  const artifacts = manifest["artifacts"];
  if (!Array.isArray(artifacts)) fail("evidence_invalid", "package artifacts are unavailable");
  const factory = artifacts
    .map(value => object(value, "artifact"))
    .find(value => value["kind"] === "factory_merged_image");
  if (factory === undefined) fail("evidence_invalid", "factory package artifact is unavailable");
  const factoryPath = path.resolve(path.dirname(manifestPath), requiredString(factory, "path", "factory"));
  if (!(await stat(factoryPath)).isFile()) fail("evidence_invalid", "factory package bytes are unavailable");
  return {
    manifestPath,
    sourceCommit: requiredString(manifest, "source_commit", "manifest"),
    appElfSha256: requiredString(manifest, "app_elf_sha256", "manifest"),
    factorySha256: requiredString(factory, "sha256", "factory"),
  };
}

async function localIpv4(): Promise<string> {
  const route = await runProcess("/sbin/route", ["-n", "get", "default"], 5_000);
  const interfaceMatch = /^\s*interface:\s*([A-Za-z0-9]+)\s*$/mu.exec(route.stdout);
  if (route.exitCode !== 0 || interfaceMatch?.[1] === undefined) {
    fail("hardware_blocked", "local interface is unavailable");
  }
  const address = await runProcess("/usr/sbin/ipconfig", ["getifaddr", interfaceMatch[1]], 5_000);
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

function startFixture(program: string, fixtureRoot: string) {
  const child = spawn(program, [
    "--private-root", fixtureRoot,
    "--accept-timeout-seconds", "120",
    "--session-timeout-seconds", "180",
  ], { cwd: process.cwd(), env: process.env, stdio: ["ignore", "pipe", "pipe"] });
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
  flashProgram: string,
  args: CampaignArgs,
  restore: RestorePackage,
  backup: JsonObject,
  poolPath: string,
  changedPackage: boolean,
): Promise<JsonObject> {
  if (changedPackage) {
    const flash = await runProcess(flashProgram, [
      "flash", "--board", "205", "--port", args.port, "--manifest", restore.manifestPath,
      "--wifi-credentials", args.wifiCredentials, "--redact-evidence",
    ], 180_000);
    if (flash.exitCode !== 0) fail("hardware_blocked", "prior package restoration failed");
  }
  const origin = await monitorOrigin(flashProgram, args.port);
  await restoreSelfTestSettings(origin, backup, args.wifiCredentials, poolPath);
  const confirmed = await fetchObject(origin, "/api/system/info");
  if (confirmed["sourceCommit"] !== restore.sourceCommit
    || confirmed["appElfSha256"] !== restore.appElfSha256
    || confirmed["startMiningOnBoot"] !== false) {
    fail("hardware_blocked", "final package or settings restoration mismatch");
  }
  return confirmed;
}

export async function runStratumV2Campaign(
  workspace: string,
  args: CampaignArgs,
): Promise<JsonObject> {
  const privateRoot = path.resolve(workspace, args.privateRoot);
  const projection = path.resolve(workspace, args.projection);
  const manifestPath = path.resolve(workspace, args.packageManifest);
  const wifiPath = path.resolve(workspace, args.wifiCredentials);
  await requireAbsent(privateRoot);
  await requireAbsent(projection);
  const ignored = await runProcess("git", ["check-ignore", "-q", args.privateRoot], 5_000);
  if (ignored.exitCode !== 0) fail("evidence_invalid", "private campaign root is not ignored");
  const privateParent = path.dirname(privateRoot);
  await mkdir(privateParent, { recursive: true, mode: 0o700 });
  await chmod(privateParent, 0o700);
  await requireMode(wifiPath, 0o600, false);
  const poolPath = await poolRestoreInput(workspace);
  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const head = (await runProcess("git", ["rev-parse", "HEAD"], 5_000)).stdout.trim();
  const status = await runProcess("git", ["status", "--porcelain"], 5_000);
  if (status.exitCode !== 0 || status.stdout.length !== 0
    || requiredString(manifest, "source_commit", "manifest") !== head) {
    fail("evidence_invalid", "campaign source or package is not exact clean HEAD");
  }

  const preOrigin = await monitorOrigin(path.join(workspace, "bazel-bin/tools/flash/flash"), args.port);
  const settings = await fetchObject(preOrigin, "/api/system/info");
  const theme = await fetchObject(preOrigin, "/api/theme");
  await validateRestorableInputs(settings, wifiPath, poolPath);
  const currentAppElf = requiredString(settings, "appElfSha256", "current settings");
  const restore = manifest["app_elf_sha256"] === currentAppElf
    ? await restorePackageFromManifest(manifestPath, manifest)
    : await selectRestorePackage(workspace, currentAppElf);
  if (restore.sourceCommit !== requiredString(settings, "sourceCommit", "current settings")) {
    fail("hardware_blocked", "prior package source identity is unavailable");
  }

  await mkdir(privateRoot, { recursive: false, mode: 0o700 });
  await chmod(privateRoot, 0o700);
  const backup: JsonObject = { settings, theme };
  const backupDocument = await writePrivateJson(path.join(privateRoot, "settings-backup.private.json"), backup);
  const fixtureRoot = path.join(privateRoot, "fixture");
  const fixture = startFixture(
    path.join(workspace, "bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture"),
    fixtureRoot,
  );
  let fixtureExit = 1;
  let effectStarted = false;
  let restorationComplete = false;
  const changedPackage = restore.appElfSha256
    !== requiredString(manifest, "app_elf_sha256", "campaign manifest");
  try {
    const ready = await waitForReady(path.join(fixtureRoot, "ready.json"));
    const host = await localIpv4();
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
    const campaign = await runProcess(path.join(workspace, "bazel-bin/tools/flash/flash"), [
      "mining-campaign", "--stage", "stratum-v2", "--profile", "conservative",
      "--board", "205", "--port", args.port, "--manifest", manifestPath,
      "--wifi-credentials", wifiPath, "--pool-credentials", fixturePoolPath,
      "--evidence-dir", path.join(args.privateRoot, "campaign"),
      "--duration-seconds", "180", "--redact-evidence",
    ], 420_000);
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
      path.join(workspace, "bazel-bin/tools/flash/flash"),
      args,
      restore,
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
    const validation = await runProcess(
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
          path.join(workspace, "bazel-bin/tools/flash/flash"),
          args,
          restore,
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
