import { createHash, randomBytes } from "node:crypto";
import { lstat, readFile, stat } from "node:fs/promises";
import { execFileSync, spawnSync } from "node:child_process";
import { basename, dirname, isAbsolute, resolve } from "node:path";
export { physicalInstruction, requiresPhysicalReacquisition } from "./browser-contract.mjs";
export const PREFLIGHT_PROFILE = "bwg-worker-restoration-preflight/0.1";
export const RESULT_PROFILE = "bwg-worker-restoration-result/0.1";
export const SCENARIOS = [
  "completion",
  "pause",
  "cancel",
  "expiry",
  "disconnect",
  "reboot",
  "monotonic_uncertainty",
  "authorization_negatives",
];
export const GATE_PROFILE_COMMIT = "0b07d36942aa8ca3473771d2f72a373e66cedf58";
export const GATE_BROWSER_COMMIT = "0b07d36942aa8ca3473771d2f72a373e66cedf58";

export function exactOptions(args, names) {
  const allowed = new Set(names);
  const parsed = {};
  for (let index = 0; index < args.length; index += 2) {
    const name = args[index];
    const value = args[index + 1];
    if (!allowed.has(name) || typeof value !== "string" || value.length === 0) {
      throw new Error("invalid_options");
    }
    if (Object.hasOwn(parsed, name)) throw new Error("duplicate_option");
    parsed[name] = value;
  }
  for (const name of names) {
    if (!Object.hasOwn(parsed, name)) throw new Error("missing_option");
  }
  return parsed;
}

export function parseScenario(value) {
  if (!SCENARIOS.includes(value)) throw new Error("invalid_scenario");
  return value;
}

export function canonicalJson(value) {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  return `{${Object.keys(value).sort().map((key) =>
    `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
}

export function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

export function randomIdentifier(prefix) {
  return `${prefix}${randomBytes(24).toString("hex")}`;
}

export async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

export async function requireProtectedFile(path) {
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.isSymbolicLink() || (metadata.mode & 0o777) !== 0o600) {
    throw new Error("protected_file_invalid");
  }
}

export async function requireProtectedDirectory(path) {
  const metadata = await lstat(path);
  if (!metadata.isDirectory() || metadata.isSymbolicLink() || (metadata.mode & 0o777) !== 0o700) {
    throw new Error("protected_directory_invalid");
  }
}

export async function requireFreshDetector(path, nowMilliseconds = Date.now()) {
  await requireProtectedFile(path);
  const metadata = await stat(path);
  if (
    nowMilliseconds - metadata.mtimeMs > 12 * 60 * 60 * 1_000 ||
    metadata.mtimeMs > nowMilliseconds + 1_000
  ) {
    throw new Error("detector_stale");
  }
  const text = await readFile(path, "utf8");
  if (
    text.match(/configuration_candidate:/g)?.length !== 1 ||
    text.match(/usb_session: ready/g)?.length !== 1 ||
    text.match(/port: \/dev\//g)?.length !== 1
  ) {
    throw new Error("detector_not_admitted");
  }
  return sha256(text);
}

export function gitHead(repository) {
  return execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: repository,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  }).trim();
}

export function requireCleanRepository(repository, expectedHead) {
  const head = gitHead(repository);
  const status = execFileSync("git", ["status", "--porcelain"], {
    cwd: repository,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  });
  if (head !== expectedHead || status !== "") throw new Error("repository_not_exact");
  return head;
}

export function requireAncestor(repository, ancestor, descendant) {
  const result = spawnGit(repository, ["merge-base", "--is-ancestor", ancestor, descendant]);
  if (result !== "") throw new Error("repository_ancestry_invalid");
}

function spawnGit(repository, args) {
  return execFileSync("git", args, {
    cwd: repository,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  });
}

export function absolutePath(value) {
  if (!isAbsolute(value)) throw new Error("path_not_absolute");
  return resolve(value);
}

export function requirePathWithin(child, parent) {
  const root = resolve(parent);
  const value = resolve(child);
  if (value === root || !value.startsWith(`${root}/`)) throw new Error("path_escape");
  return value;
}

export async function digestFile(path) {
  return sha256(await readFile(path));
}

export async function validatePackage(manifestPath, firmwareHead, firmwareRepository) {
  const manifest = await readJson(manifestPath);
  if (
    manifest.schema_version !== 3 ||
    manifest.source_commit !== firmwareHead ||
    manifest.build_identity?.source_dirty !== false ||
    typeof manifest.reference_commit !== "string" ||
    !/^[0-9a-f]{40}$/.test(manifest.reference_commit) ||
    typeof manifest.app_elf_sha256 !== "string" ||
    !/^[0-9a-f]{64}$/.test(manifest.app_elf_sha256) ||
    manifest.default_flash_image !== "bitaxe-ultra205.elf" ||
    manifest.image_metadata?.board !== "205" ||
    manifest.image_metadata?.asic !== "BM1366" ||
    !Array.isArray(manifest.artifacts)
  ) {
    throw new Error("package_not_exact");
  }
  const required = new Set([
    "firmware_elf",
    "firmware_ota_image",
    "www_spiffs_image",
    "factory_merged_image",
    "partition_table",
    "otadata_initial",
  ]);
  const paths = {
    firmware_elf: "bitaxe-ultra205.elf",
    firmware_ota_image: "esp-miner.bin",
    www_spiffs_image: "www.bin",
    factory_merged_image: "bitaxe-ultra205-factory.bin",
    partition_table: "firmware/bitaxe/partitions-ultra205.csv",
    otadata_initial: "otadata-initial.bin",
  };
  for (const artifact of manifest.artifacts) {
    if (
      !required.delete(artifact.kind) || artifact.path !== paths[artifact.kind] ||
      isAbsolute(artifact.path) || artifact.path.split("/").includes("..") ||
      !/^[0-9a-f]{64}$/.test(artifact.sha256)
    ) {
      throw new Error("package_artifact_invalid");
    }
    const artifactPath = artifact.kind === "partition_table"
      ? resolve(firmwareRepository, artifact.path)
      : resolve(dirname(manifestPath), artifact.path);
    const metadata = await lstat(artifactPath);
    if (!metadata.isFile() || metadata.isSymbolicLink()) {
      throw new Error("package_artifact_invalid");
    }
    if (await digestFile(artifactPath) !== artifact.sha256) {
      throw new Error("package_artifact_drift");
    }
  }
  if (required.size !== 0) throw new Error("package_artifact_missing");
  const elf = manifest.artifacts.find((artifact) => artifact.kind === "firmware_elf");
  if (elf.sha256 !== manifest.app_elf_sha256) throw new Error("package_identity_invalid");
  return { manifest, digest: await digestFile(manifestPath) };
}

export function requirePackageAdmission(manifestPath, firmwareRepository) {
  const result = spawnSync(
    "bazel",
    [
      "run", "//tools/flash:flash", "--", "flash", "--board", "205", "--dry-run",
      "--manifest", manifestPath,
    ],
    {
      cwd: firmwareRepository,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
      timeout: 5 * 60 * 1_000,
    },
  );
  if (result.status !== 0) throw new Error("package_admission_failed");
}

export function requireGateBrowserBuild(gateRepository) {
  const result = spawnSync("bun", ["run", "build:browser"], {
    cwd: gateRepository,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 5 * 60 * 1_000,
  });
  if (result.status !== 0) throw new Error("gate_browser_build_failed");
}

export async function validatePoolCredentials(path) {
  await requireProtectedFile(path);
  const value = await readJson(path);
  admitPoolCredentials(value);
  return sha256(canonicalJson({
    endpointPresent: true,
    usernamePresent: true,
    passwordPresent: true,
  }));
}

function admitPoolCredentials(value) {
  if (
    Object.keys(value).sort().join(",") !== "poolPassword,poolPort,poolURL,poolUser" ||
    typeof value.poolURL !== "string" || value.poolURL.length > 253 ||
    !/^[A-Za-z0-9.-]+$/.test(value.poolURL) || value.poolURL.startsWith(".") ||
    value.poolURL.endsWith(".") || value.poolURL.includes("..") ||
    !Number.isInteger(value.poolPort) || value.poolPort < 1 || value.poolPort > 65_535 ||
    typeof value.poolUser !== "string" || value.poolUser.length === 0 ||
    typeof value.poolPassword !== "string" || value.poolPassword.length === 0
  ) {
    throw new Error("pool_credentials_invalid");
  }
  return value;
}

export function browserPoolCredentials(value, resolvedAddress) {
  const admitted = admitPoolCredentials(value);
  if (typeof resolvedAddress !== "string" || resolvedAddress.length === 0) {
    throw new Error("pool_endpoint_invalid");
  }
  return {
    endpoint: `stratum+tcp://${resolvedAddress}:${admitted.poolPort}/`,
    username: admitted.poolUser,
    password: admitted.poolPassword,
  };
}

export async function validatePoolReadiness(
  path,
  firmwareCommit,
  referenceCommit,
  poolCredentialsSha256,
) {
  await requireProtectedFile(path);
  const value = await readJson(path);
  const keys = [
    "attempt_ordinal", "authorize_succeeded", "bounded", "configure_succeeded",
    "consecutive_ready", "credentials_redacted", "endpoint_redacted", "max_server_bytes",
    "max_server_messages", "pool_config", "pool_credentials_sha256", "private_lan_only",
    "protocol", "ready_samples", "reference_commit", "resolved_endpoints_sha256",
    "sample_delay_seconds", "sample_timeout_seconds",
    "samples_completed", "samples_required", "schema_version", "shares_submitted",
    "source_commit", "subscribe_succeeded", "terminal_category",
  ];
  if (
    Object.keys(value).sort().join(",") !== keys.sort().join(",") ||
    value.schema_version !== "bitaxe-pool-readiness-evidence-v1" ||
    value.attempt_ordinal !== 5 ||
    value.source_commit !== firmwareCommit ||
    value.reference_commit !== referenceCommit ||
    value.pool_config !== "local-owner-supplied" ||
    value.pool_credentials_sha256 !== poolCredentialsSha256 ||
    value.private_lan_only !== true ||
    !/^[0-9a-f]{64}$/.test(value.resolved_endpoints_sha256) ||
    value.protocol !== "stratum_v1_configure_subscribe_authorize" ||
    value.samples_required !== 3 || value.samples_completed !== 3 ||
    value.ready_samples !== 3 || value.consecutive_ready !== true ||
    value.configure_succeeded !== true || value.subscribe_succeeded !== true ||
    value.authorize_succeeded !== true || value.shares_submitted !== false ||
    value.sample_timeout_seconds !== 15 || value.sample_delay_seconds !== 2 ||
    value.max_server_bytes !== 65_536 || value.max_server_messages !== 256 ||
    value.endpoint_redacted !== true || value.credentials_redacted !== true ||
    value.bounded !== true || value.terminal_category !== "ready"
  ) {
    throw new Error("pool_readiness_invalid");
  }
  return {
    digest: await digestFile(path),
    resolvedEndpointsSha256: value.resolved_endpoints_sha256,
  };
}

export async function validateAuthorityDirectory(path) {
  await requireProtectedDirectory(path);
  for (const file of [
    "update-private.json",
    "lease-private.json",
    "lease-sequence.json",
  ]) {
    await requireProtectedFile(resolve(path, file));
  }
  const trust = await lstat(resolve(path, "trust.json"));
  if (!trust.isFile() || trust.isSymbolicLink() || (trust.mode & 0o777) !== 0o644) {
    throw new Error("authority_trust_invalid");
  }
}

export async function authoritySnapshot(directory) {
  await validateAuthorityDirectory(directory);
  const staticFiles = ["update-private.json", "lease-private.json", "trust.json"];
  const staticSha256 = sha256((await Promise.all(
    staticFiles.map((file) => digestFile(resolve(directory, file))),
  )).join(""));
  return {
    staticSha256,
    sequenceSha256: await digestFile(resolve(directory, "lease-sequence.json")),
    trust: await readJson(resolve(directory, "trust.json")),
  };
}

export function preflightDigest(document) {
  const copy = structuredClone(document);
  delete copy.preflightDigestSha256;
  return sha256(canonicalJson(copy));
}

export function validatePreflight(document) {
  const keys = [
    "allowedInterfaces", "appElfSha256", "attemptId", "authorityDirectory",
    "authoritySequenceSha256", "authorityStaticSha256", "detectorOutput", "detectorSha256",
    "firmwareCommit", "firmwareRepository", "forbiddenInterfaces", "gateBundleSha256",
    "gateCommit", "gateProfileCommit", "gateRepository", "gateTrustSha256", "packageManifest",
    "packageManifestSha256", "poolCredentials", "poolCredentialsSha256", "poolReadiness",
    "poolReadinessSha256", "poolResolvedEndpointsSha256", "poolShapeSha256",
    "preflightDigestSha256", "profile", "projection",
    "recoveryRoot", "referenceCommit", "remediationPlan", "remediationPlanSha256",
    "restoreAuthorization", "restoreAuthorizationSha256", "restoreBundle", "restoreBundleSha256",
    "scenario", "wifiCredentials", "wifiCredentialsSha256",
  ];
  if (
    Object.keys(document).sort().join(",") !== keys.sort().join(",") ||
    document.profile !== PREFLIGHT_PROFILE ||
    !/^bwg007-attempt-[0-9]{3}$/.test(document.attemptId) ||
    !SCENARIOS.includes(document.scenario) ||
    !/^[0-9a-f]{40}$/.test(document.firmwareCommit) ||
    !/^[0-9a-f]{40}$/.test(document.referenceCommit) ||
    !/^[0-9a-f]{64}$/.test(document.appElfSha256) ||
    document.gateCommit !== GATE_BROWSER_COMMIT ||
    typeof document.firmwareRepository !== "string" ||
    !isAbsolute(document.firmwareRepository) ||
    [
      document.packageManifest,
      document.gateRepository,
      document.authorityDirectory,
      document.poolCredentials,
      document.poolReadiness,
      document.restoreBundle,
      document.restoreAuthorization,
      document.recoveryRoot,
      document.remediationPlan,
      document.wifiCredentials,
      document.detectorOutput,
    ].some((path) => typeof path !== "string" || !isAbsolute(path)) ||
    document.gateProfileCommit !== GATE_PROFILE_COMMIT ||
    !/^[0-9a-f]{64}$/.test(document.packageManifestSha256) ||
    !/^[0-9a-f]{64}$/.test(document.detectorSha256) ||
    !/^[0-9a-f]{64}$/.test(document.gateBundleSha256) ||
    !/^[0-9a-f]{64}$/.test(document.gateTrustSha256) ||
    !/^[0-9a-f]{64}$/.test(document.poolShapeSha256) ||
    !/^[0-9a-f]{64}$/.test(document.poolCredentialsSha256) ||
    !/^[0-9a-f]{64}$/.test(document.poolReadinessSha256) ||
    !/^[0-9a-f]{64}$/.test(document.poolResolvedEndpointsSha256) ||
    !/^[0-9a-f]{64}$/.test(document.authorityStaticSha256) ||
    !/^[0-9a-f]{64}$/.test(document.authoritySequenceSha256) ||
    !/^[0-9a-f]{64}$/.test(document.restoreBundleSha256) ||
    !/^[0-9a-f]{64}$/.test(document.restoreAuthorizationSha256) ||
    !/^[0-9a-f]{64}$/.test(document.remediationPlanSha256) ||
    !/^[0-9a-f]{64}$/.test(document.wifiCredentialsSha256) ||
    !Array.isArray(document.allowedInterfaces) ||
    document.allowedInterfaces.join(",") !== "usb,barrel_power" ||
    !Array.isArray(document.forbiddenInterfaces) ||
    document.forbiddenInterfaces.join(",") !== "uart,pins,probes,erasure,ad_hoc_writes" ||
    typeof document.projection !== "string" || !isAbsolute(document.projection) ||
    document.preflightDigestSha256 !== preflightDigest(document)
  ) {
    throw new Error("preflight_invalid");
  }
  return document;
}

export function validatePreflightScope(document, preflightPath) {
  const firmwareRepository = resolve(document.firmwareRepository);
  const privateParent = resolve(firmwareRepository, "scratch/bwg-worker-restoration");
  const attemptRoot = resolve(privateParent, document.attemptId);
  const projectionRoot = resolve(
    firmwareRepository,
    "docs/parity/evidence/bwg-worker-restoration",
  );
  if (
    resolve(preflightPath) !== resolve(attemptRoot, "preflight.private.json") ||
    resolve(document.recoveryRoot) !== resolve(attemptRoot, "recovery") ||
    resolve(document.restoreAuthorization) !==
      resolve(attemptRoot, "recovery/restore-authorization.private.json") ||
    resolve(document.packageManifest) !==
      resolve(firmwareRepository, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json") ||
    resolve(document.remediationPlan) !== resolve(
      firmwareRepository,
      "docs/adr/0019-supervise-bwg-restoration-through-a-protected-browser-campaign.md",
    ) ||
    dirname(resolve(document.projection)) !== projectionRoot ||
    basename(document.projection) !== `${document.attemptId}-${document.scenario}.json`
  ) {
    throw new Error("preflight_scope_invalid");
  }
  return attemptRoot;
}

export function privateSibling(path, name) {
  return resolve(dirname(path), name);
}

export function detectorPort(text) {
  const ports = text.split(/\r?\n/).flatMap((line) => {
    const value = line.trim().match(/^port: (\/dev\/[A-Za-z0-9._-]+)$/)?.[1];
    return value ? [value] : [];
  });
  if (ports.length !== 1) throw new Error("detector_port_invalid");
  return ports[0];
}

export function safeEvent(value) {
  const allowed = new Set([
    "event",
    "mode",
    "reason",
    "scenario",
    "state",
    "restorationStatus",
    "restorationReason",
    "outcome",
    "category",
    "cleanupCategory",
    "cleanup",
    "count",
  ]);
  if (
    typeof value !== "object" || value === null || Array.isArray(value) ||
    Object.keys(value).some((key) => !allowed.has(key)) ||
    Object.values(value).some((item) => item !== undefined && typeof item !== "string")
  ) {
    throw new Error("event_invalid");
  }
  const events = new Set([
    "runtime_identity_admitted",
    "worker_admitted",
    "capability_admitted",
    "lease_started",
    "baseline_confirmed",
    "transport_disconnected",
    "disconnect_handled",
    "physical_checkpoint_required",
    "reacquisition_retryable",
    "authorization_expired_rejected",
    "cross_context_rejected",
    "replay_rejected",
    "complete",
    "failed",
  ]);
  if (value.event !== undefined && !events.has(value.event)) throw new Error("event_invalid");
  if (value.state !== undefined && !["baseline", "mining"].includes(value.state)) {
    throw new Error("event_invalid");
  }
  if (value.restorationStatus !== undefined &&
      !["not_required", "pending", "confirmed"].includes(value.restorationStatus)) {
    throw new Error("event_invalid");
  }
  if (value.restorationReason !== undefined &&
      !expectedTerminalReasonsForAnyScenario().has(value.restorationReason)) {
    throw new Error("event_invalid");
  }
  if (value.reason !== undefined && !expectedTerminalReasonsForAnyScenario().has(value.reason)) {
    throw new Error("event_invalid");
  }
  if (value.mode !== undefined && !["initial", "recovered"].includes(value.mode)) {
    throw new Error("event_invalid");
  }
  const categories = new Set([
    "admission_failed",
    "scenario_failed",
    "cleanup_failed",
    "reacquisition_failed",
    "browser_closed_active",
  ]);
  if (value.category !== undefined && !categories.has(value.category)) {
    throw new Error("event_invalid");
  }
  if (value.cleanupCategory !== undefined && value.cleanupCategory !== "cleanup_failed") {
    throw new Error("event_invalid");
  }
  if (value.cleanup !== undefined && value.cleanup !== "confirmed") {
    throw new Error("event_invalid");
  }
  if (value.count !== undefined && value.count !== "1") throw new Error("event_invalid");
  return value;
}

export function redactedProjection(input) {
  return {
    profile: RESULT_PROFILE,
    attemptId: input.attemptId,
    scenario: input.scenario,
    outcome: "complete",
    terminalReason: input.terminalReason,
    firmwareCommit: input.firmwareCommit,
    gateCommit: input.gateCommit,
    gateProfileCommit: input.gateProfileCommit,
    packageManifestSha256: input.packageManifestSha256,
    appElfSha256: input.appElfSha256,
    gateBundleSha256: input.gateBundleSha256,
    restoreBundleSha256: input.restoreBundleSha256,
    runtimeAttestationSha256: input.runtimeAttestationSha256,
    eventsSha256: input.eventsSha256,
    campaignEventCredentialsAbsent: true,
    baselineConfirmed: true,
    cleanupConfirmed: true,
  };
}

export function expectedTerminalReasons(scenario) {
  const reasons = {
    completion: ["challenge_satisfied"],
    pause: ["paused"],
    cancel: ["cancelled"],
    expiry: ["lease_expired"],
    disconnect: ["connectivity_lost"],
    reboot: ["reboot"],
    monotonic_uncertainty: ["monotonic_reset"],
    authorization_negatives: ["control_failed"],
  };
  return reasons[scenario] ?? [];
}

function expectedTerminalReasonsForAnyScenario() {
  return new Set(SCENARIOS.flatMap(expectedTerminalReasons));
}

export function validateCompletion(scenario, value, events) {
  const body = safeEvent(value);
  if (
    Object.keys(body).sort().join(",") !==
      "cleanup,outcome,restorationReason,restorationStatus,state" ||
    body.outcome !== "complete" || body.state !== "baseline" ||
    body.restorationStatus !== "confirmed" || body.cleanup !== "confirmed" ||
    !expectedTerminalReasons(scenario).includes(body.restorationReason)
  ) {
    throw new Error("completion_invalid");
  }
  const admittedEvents = events.map((event) => safeEvent(event));
  const names = admittedEvents.map((event) => event.event).filter(Boolean);
  for (const required of [
    "runtime_identity_admitted",
    "worker_admitted",
    "capability_admitted",
    "lease_started",
    "baseline_confirmed",
  ]) {
    if (names.filter((name) => name === required).length !== 1) {
      throw new Error("trace_incomplete");
    }
  }
  const requiredOrder = [
    "runtime_identity_admitted", "worker_admitted", "capability_admitted", "lease_started",
    "baseline_confirmed",
  ];
  let previousIndex = -1;
  for (const name of requiredOrder) {
    const index = names.indexOf(name);
    if (index <= previousIndex) throw new Error("trace_order_invalid");
    previousIndex = index;
  }
  const worker = admittedEvents.find((event) => event.event === "worker_admitted");
  const lease = admittedEvents.find((event) => event.event === "lease_started");
  const baseline = admittedEvents.find((event) => event.event === "baseline_confirmed");
  if (
    worker.mode !== "initial" || worker.count !== "1" ||
    lease.state !== "mining" || lease.restorationStatus !== "pending" ||
    baseline.state !== "baseline" || baseline.restorationStatus !== "confirmed" ||
    baseline.restorationReason !== body.restorationReason
  ) {
    throw new Error("trace_fields_invalid");
  }
  if (["disconnect", "reboot", "monotonic_uncertainty"].includes(scenario) &&
      (names.filter((name) => name === "transport_disconnected").length !== 1 ||
       names.filter((name) => name === "disconnect_handled").length !== 1)) {
    throw new Error("trace_incomplete");
  }
  if (scenario === "authorization_negatives") {
    for (const required of [
      "authorization_expired_rejected",
      "cross_context_rejected",
      "replay_rejected",
    ]) {
      if (!names.includes(required)) throw new Error("trace_incomplete");
    }
  }
  return body.restorationReason;
}
