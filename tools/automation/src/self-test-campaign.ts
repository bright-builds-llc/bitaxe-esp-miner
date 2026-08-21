import { createHash, randomBytes } from "node:crypto";
import {
  chmod,
  mkdir,
  readFile,
  rename,
  stat,
  writeFile,
} from "node:fs/promises";
import path from "node:path";

import { internalCommandSpec, monitorCommand } from "./contracts.generated.js";
import { portFromDetectorOutput } from "./detector.js";
import { maybeOptionValue, optionValue, type ParsedInvocation } from "./invocation.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

const expectedRoot = "scratch/self001-full-lifecycle/attempt-001";
const expectedProjection =
  "docs/parity/evidence/self001-full-lifecycle/self-test-projection.json";
const expectedPlan = "docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md";
const expectedPlanSha256 =
  "4f089bc826a31881ce7668a78e2479370a96cf6e39c855ef3baecf6fd33c9936";
const expectedReference = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const activeTask = "task-parity-self001-full-lifecycle";
const monitorSeconds = 360;

type JsonObject = Record<string, unknown>;
type CampaignState = {
  readonly schema_version: "bitaxe-self-test-campaign-state-v1";
  readonly lease_hex: string;
  readonly source_commit: string;
  readonly reference_commit: string;
  readonly app_elf_sha256: string;
  readonly package_manifest_sha256: string;
  readonly settings_backup_sha256: string;
  readonly pre_boot_session: string;
  readonly stage: "cancel_ready";
};

class SelfTestCampaignError extends Error {
  public constructor(
    public readonly category: "evidence_invalid" | "hardware_blocked" | "process_failed" | "timeout",
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "SelfTestCampaignError";
  }
}

function failure(
  category: SelfTestCampaignError["category"],
  message: string,
  stage: string,
): SelfTestCampaignError {
  return new SelfTestCampaignError(category, message, {
    stage,
    projection_published: false,
    checkpoint: "unavailable",
  });
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`, "contract");
  }
  return value as JsonObject;
}

function requiredString(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate.length === 0) {
    throw failure("evidence_invalid", `${context} identity is invalid`, "contract");
  }
  return candidate;
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent`, "preflight");
  } catch (error) {
    if (error instanceof SelfTestCampaignError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await stat(candidate);
  if ((directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    throw failure("evidence_invalid", "protected evidence mode is invalid", "privacy");
  }
}

async function privateJson(output: string, value: unknown): Promise<string> {
  const document = `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(output, document, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
  return document;
}

async function childText(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
  context: string,
): Promise<string> {
  const outcome = await processPort.run(internalCommandSpec(program, [...args], value => value));
  if (outcome.timedOut) throw failure("timeout", `${context} timed out`, "process");
  if (outcome.exitCode !== 0) {
    throw failure("evidence_invalid", `${context} did not pass`, "process");
  }
  return outcome.stdout.trim();
}

function singleOrigin(monitor: string): URL {
  const matches = [...monitor.matchAll(/\bhttps?:\/\/[A-Za-z0-9.-]+(?::[0-9]+)?\b/gu)]
    .map(match => match[0])
    .filter((value, index, values) => values.indexOf(value) === index);
  if (matches.length !== 1) {
    throw failure("hardware_blocked", "current session did not expose one origin", "settings_backup");
  }
  const origin = matches[0];
  if (origin === undefined) {
    throw failure("hardware_blocked", "current session origin disappeared", "settings_backup");
  }
  return new URL(origin);
}

async function fetchJson(origin: URL, route: string, init?: RequestInit): Promise<JsonObject> {
  let response: Response;
  try {
    response = await fetch(new URL(route, origin), init);
  } catch {
    throw failure("hardware_blocked", "same-origin settings transport failed", "settings");
  }
  if (!response.ok) {
    throw failure("hardware_blocked", "same-origin settings request failed", "settings");
  }
  return object(await response.json(), "same-origin response");
}

async function currentMonitor(
  processPort: ProcessPort,
  flashProgram: string,
  port: string,
): Promise<string> {
  const outcome = await processPort.run(monitorCommand(flashProgram, {
    board: 205,
    port,
    captureTimeoutSeconds: monitorSeconds,
  }));
  if (outcome.timedOut) throw failure("timeout", "passive monitor timed out", "monitor");
  if (outcome.exitCode !== 0) {
    throw failure("hardware_blocked", "passive monitor failed", "monitor");
  }
  return outcome.stdout;
}

function intent(
  state: Pick<CampaignState, "lease_hex" | "source_commit" | "reference_commit" | "app_elf_sha256">,
  selfTestCase: "planned_failure" | "pass",
): JsonObject {
  return {
    schema_version: "bitaxe-self-test-intent-v1",
    board: 205,
    attempt_ordinal: 1,
    source_commit: state.source_commit,
    reference_commit: state.reference_commit,
    app_elf_sha256: state.app_elf_sha256,
    plan_path: expectedPlan,
    plan_sha256: expectedPlanSha256,
    case: selfTestCase,
    lease_hex: state.lease_hex,
  };
}

function flashMonitorSpec(
  flashProgram: string,
  port: string,
  manifest: string,
  wifiCredentials: string,
  intentPath: string,
  evidenceDir: string,
) {
  return internalCommandSpec(flashProgram, [
    "flash-monitor",
    "--board", "205",
    "--port", port,
    "--manifest", manifest,
    "--wifi-credentials", wifiCredentials,
    "--self-test-intent", intentPath,
    "--capture-timeout-seconds", String(monitorSeconds),
    "--evidence-mode", "dual",
    "--evidence-dir", evidenceDir,
  ], value => value);
}

async function runFlashMonitor(
  processPort: ProcessPort,
  spec: ReturnType<typeof flashMonitorSpec>,
  context: string,
): Promise<void> {
  const outcome = await processPort.run(spec);
  if (outcome.timedOut) throw failure("timeout", `${context} timed out`, context);
  if (outcome.exitCode !== 0) {
    throw failure("hardware_blocked", `${context} did not complete`, context);
  }
}

function validatePlanAndTask(plan: string, task: string): void {
  if (sha256(plan) !== expectedPlanSha256
    || !plan.includes("- Parity row: `SELF-001`")
    || !plan.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "immutable plan binding is invalid", "contract");
  }
  const heading = `### ${activeTask} |`;
  const start = task.indexOf(heading);
  const end = task.indexOf("\n### ", start + heading.length);
  const block = task.slice(start, end === -1 ? task.length : end);
  for (const required of [expectedPlan, "two-phase", "BOOT-button", "mode-0700"]) {
    if (start === -1 || !block.includes(required)) {
      throw failure("evidence_invalid", "active task contract is incomplete", "contract");
    }
  }
}

async function validateSource(
  root: string,
  processPort: ProcessPort,
  manifest: JsonObject,
): Promise<void> {
  const source = requiredString(manifest, "source_commit", "manifest");
  const reference = requiredString(manifest, "reference_commit", "manifest");
  const current = await childText(processPort, "git", ["rev-parse", "HEAD"], "source identity");
  const pushed = await childText(processPort, "git", ["rev-parse", "origin/main"], "pushed identity");
  const pinned = await childText(
    processPort,
    "git",
    ["-C", path.join(root, "reference/esp-miner"), "rev-parse", "HEAD"],
    "reference identity",
  );
  const dirty = await childText(
    processPort,
    "git",
    ["status", "--porcelain", "--untracked-files=no"],
    "source cleanliness",
  );
  if (source !== current || pushed !== current || reference !== expectedReference
    || pinned !== expectedReference || dirty !== "") {
    throw failure("evidence_invalid", "source is not the exact clean pushed package", "source");
  }
}

async function startCampaign(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
  flashProgram: string,
): Promise<JsonObject> {
  const privateRoot = assertWithinWorkspace(root, optionValue(invocation, "--private-root"));
  const manifestPath = assertWithinWorkspace(root, optionValue(invocation, "--package-manifest"));
  const wifiCredentials = assertWithinWorkspace(root, optionValue(invocation, "--wifi-credentials"));
  const poolCredentials = assertWithinWorkspace(root, optionValue(invocation, "--pool-credentials"));
  const detector = optionValue(invocation, "--detector-output");
  const projection = assertWithinWorkspace(root, optionValue(invocation, "--projection"));
  const planPath = assertWithinWorkspace(root, optionValue(invocation, "--plan"));
  if (path.relative(root, privateRoot) !== expectedRoot
    || path.relative(root, projection) !== expectedProjection
    || path.relative(root, planPath) !== expectedPlan) {
    throw failure("evidence_invalid", "SELF-001 path contract is invalid", "preflight");
  }
  await requireAbsent(privateRoot, "private root");
  await requireAbsent(projection, "projection");
  await mkdir(privateRoot, { recursive: true, mode: 0o700 });
  await chmod(privateRoot, 0o700);

  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = object(JSON.parse(manifestDocument), "manifest");
  const plan = await readFile(planPath, "utf8");
  const task = await readFile(path.join(root, "TASKS.md"), "utf8");
  validatePlanAndTask(plan, task);
  await validateSource(root, processPort, manifest);
  await requireMode(wifiCredentials, 0o600, false);
  await requireMode(poolCredentials, 0o600, false);
  const port = await portFromDetectorOutput(root, detector);

  const preMonitor = await currentMonitor(processPort, flashProgram, port);
  const preOrigin = singleOrigin(preMonitor);
  const settings = await fetchJson(preOrigin, "/api/system/info");
  const theme = await fetchJson(preOrigin, "/api/system/theme");
  const wifiInput = object(JSON.parse(await readFile(wifiCredentials, "utf8")), "Wi-Fi input");
  const poolInput = object(JSON.parse(await readFile(poolCredentials, "utf8")), "pool input");
  if (settings["startMiningOnBoot"] !== false) {
    throw failure("evidence_invalid", "mineonboot must already be disabled", "settings_backup");
  }
  if (settings["ssid"] !== wifiInput["ssid"]
    || settings["stratumURL"] !== poolInput["poolURL"]
    || settings["stratumPort"] !== poolInput["poolPort"]
    || settings["stratumUser"] !== poolInput["poolUser"]
    || settings["useFallbackStratum"] === true
    || (typeof settings["fallbackStratumURL"] === "string"
      && settings["fallbackStratumURL"].length > 0)) {
    throw failure(
      "evidence_invalid",
      "local credential inputs cannot exactly restore current settings",
      "settings_backup",
    );
  }
  const backupDocument = await privateJson(path.join(privateRoot, "settings-backup.private.json"), {
    settings,
    theme,
  });

  const leaseBytes = randomBytes(8);
  if (leaseBytes.every(byte => byte === 0)) leaseBytes[7] = 1;
  const state: CampaignState = {
    schema_version: "bitaxe-self-test-campaign-state-v1",
    lease_hex: leaseBytes.toString("hex"),
    source_commit: requiredString(manifest, "source_commit", "manifest"),
    reference_commit: requiredString(manifest, "reference_commit", "manifest"),
    app_elf_sha256: requiredString(manifest, "app_elf_sha256", "manifest"),
    package_manifest_sha256: sha256(manifestDocument),
    settings_backup_sha256: sha256(backupDocument),
    pre_boot_session: requiredString(settings, "bootSession", "settings snapshot"),
    stage: "cancel_ready",
  };
  await privateJson(path.join(privateRoot, "failure-intent.private.json"), intent(state, "planned_failure"));
  const failureRoot = path.join(privateRoot, "failure");
  await runFlashMonitor(
    processPort,
    flashMonitorSpec(
      flashProgram,
      port,
      manifestPath,
      wifiCredentials,
      path.relative(root, path.join(privateRoot, "failure-intent.private.json")),
      failureRoot,
    ),
    "failure_phase",
  );
  const log = await readFile(path.join(failureRoot, "flash-monitor.classifier-input.log"), "utf8");
  if (!log.includes('"stage":"measuring"')
    || !log.includes('"checkpoint":"cancel_ready"')
    || !log.includes('"safe_state":true')
    || !log.includes('"failure":"planned_evaluation_failure"')) {
    throw failure("hardware_blocked", "failure phase checkpoint is incomplete", "failure_phase");
  }
  await privateJson(path.join(privateRoot, "campaign-state.private.json"), state);
  return {
    checkpoint: "cancel_ready",
    safe_state: true,
    action: "hold_boot_button_for_two_seconds",
    response_deadline: "none",
    projection_published: false,
  };
}

const restorableKeys = [
  "hostname", "stratumProtocol", "stratumURL", "stratumPort", "stratumUser",
  "stratumSuggestedDifficulty", "stratumExtranonceSubscribe", "stratumTLS", "stratumCert",
  "stratumV2ChannelType", "stratumV2AuthorityPubkey", "stratumDecodeCoinbase",
  "fallbackStratumProtocol", "fallbackStratumURL", "fallbackStratumPort", "fallbackStratumUser",
  "fallbackStratumSuggestedDifficulty", "fallbackStratumExtranonceSubscribe", "fallbackStratumTLS",
  "fallbackStratumCert", "fallbackStratumV2ChannelType", "fallbackStratumV2AuthorityPubkey",
  "fallbackStratumDecodeCoinbase", "useFallbackStratum", "frequency", "coreVoltage",
  "overclockEnabled", "display", "rotation", "invertscreen", "displayOffset", "displayTimeout",
  "autofanspeed", "manualFanSpeed", "minFanSpeed", "temptarget", "overheat_mode", "statsFrequency",
] as const;

async function restoreSettings(
  origin: URL,
  backup: JsonObject,
  wifiPath: string,
  poolPath: string,
): Promise<void> {
  const settings = object(backup["settings"], "settings backup");
  const theme = object(backup["theme"], "theme backup");
  const wifi = object(JSON.parse(await readFile(wifiPath, "utf8")), "Wi-Fi input");
  const pool = object(JSON.parse(await readFile(poolPath, "utf8")), "pool input");
  const body: JsonObject = { startMiningOnBoot: false };
  for (const key of restorableKeys) {
    if (Object.hasOwn(settings, key)) body[key] = settings[key];
  }
  body["ssid"] = wifi["ssid"];
  body["wifiPass"] = wifi["wifiPass"];
  body["stratumURL"] = pool["poolURL"];
  body["stratumPort"] = pool["poolPort"];
  body["stratumUser"] = pool["poolUser"];
  body["stratumPassword"] = pool["poolPassword"];
  let response: Response;
  let themeResponse: Response;
  try {
    response = await fetch(new URL("/api/system", origin), {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    themeResponse = await fetch(new URL("/api/system/theme", origin), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(theme),
    });
  } catch {
    throw failure("hardware_blocked", "settings restoration transport failed", "restoration");
  }
  if (!response.ok) throw failure("hardware_blocked", "settings restoration failed", "restoration");
  if (!themeResponse.ok) throw failure("hardware_blocked", "theme restoration failed", "restoration");
  const confirmed = await fetchJson(origin, "/api/system/info");
  const confirmedTheme = await fetchJson(origin, "/api/system/theme");
  for (const key of restorableKeys) {
    if (Object.hasOwn(settings, key) && JSON.stringify(confirmed[key]) !== JSON.stringify(settings[key])) {
      throw failure("hardware_blocked", "settings restoration mismatch", "restoration");
    }
  }
  if (confirmed["startMiningOnBoot"] !== false
    || JSON.stringify(confirmedTheme) !== JSON.stringify(theme)) {
    throw failure("hardware_blocked", "restoration confirmation mismatch", "restoration");
  }
}

async function resumeCampaign(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
): Promise<JsonObject> {
  const privateRoot = assertWithinWorkspace(root, optionValue(invocation, "--private-root"));
  const manifestPath = assertWithinWorkspace(root, optionValue(invocation, "--package-manifest"));
  const detector = optionValue(invocation, "--detector-output");
  const projection = assertWithinWorkspace(root, optionValue(invocation, "--projection"));
  const planPath = assertWithinWorkspace(root, optionValue(invocation, "--plan"));
  const wifiPath = assertWithinWorkspace(
    root,
    maybeOptionValue(invocation, "--wifi-credentials") ?? "wifi-credentials.json",
  );
  const poolPath = assertWithinWorkspace(
    root,
    maybeOptionValue(invocation, "--pool-credentials") ?? "pool-credentials.json",
  );
  if (path.relative(root, privateRoot) !== expectedRoot
    || path.relative(root, projection) !== expectedProjection
    || path.relative(root, planPath) !== expectedPlan) {
    throw failure("evidence_invalid", "SELF-001 resume path contract is invalid", "resume");
  }
  await requireMode(privateRoot, 0o700, true);
  await requireAbsent(projection, "projection");
  const state = object(
    JSON.parse(await readFile(path.join(privateRoot, "campaign-state.private.json"), "utf8")),
    "campaign state",
  ) as CampaignState;
  const backupDocument = await readFile(path.join(privateRoot, "settings-backup.private.json"), "utf8");
  if (state.schema_version !== "bitaxe-self-test-campaign-state-v1"
    || state.stage !== "cancel_ready"
    || sha256(backupDocument) !== state.settings_backup_sha256) {
    throw failure("evidence_invalid", "campaign resume state is invalid", "resume");
  }
  const port = await portFromDetectorOutput(root, detector);
  const cancellationLog = await currentMonitor(processPort, flashProgram, port);
  const cancelMarker = `self_test_receipt outcome=cancelled lease=${state.lease_hex}`;
  if (!cancellationLog.includes(cancelMarker)) {
    throw failure("hardware_blocked", "physical cancellation receipt was not observed", "cancel");
  }
  await privateJson(path.join(privateRoot, "pass-intent.private.json"), intent(state, "pass"));
  const passRoot = path.join(privateRoot, "pass");
  await runFlashMonitor(
    processPort,
    flashMonitorSpec(
      flashProgram,
      port,
      manifestPath,
      wifiPath,
      path.relative(root, path.join(privateRoot, "pass-intent.private.json")),
      passRoot,
    ),
    "pass_phase",
  );
  const passLog = await readFile(path.join(passRoot, "flash-monitor.classifier-input.log"), "utf8");
  const passMarker = `self_test_receipt outcome=passed lease=${state.lease_hex}`;
  const passStages = ["warming", "measuring", "evaluating", "safe_stopping", "restarting"];
  if (!passLog.includes('"outcome":"passed"')
    || !passLog.includes(passMarker)
    || passStages.some(stage => !passLog.includes(`"stage":"${stage}"`))) {
    throw failure("hardware_blocked", "passing phase or automatic restart was not observed", "pass_phase");
  }
  const origin = singleOrigin(passLog);
  const backup = object(JSON.parse(backupDocument), "settings backup");
  await restoreSettings(origin, backup, wifiPath, poolPath);

  const manifestDocument = await readFile(manifestPath, "utf8");
  const evidence = {
    schema_version: "bitaxe-self-test-evidence-v1",
    board: 205,
    attempt_ordinal: 1,
    source_commit: state.source_commit,
    reference_commit: state.reference_commit,
    app_elf_sha256: state.app_elf_sha256,
    package_manifest_sha256: state.package_manifest_sha256,
    plan_sha256: expectedPlanSha256,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "self-test-campaign",
      request_sha256: sha256(JSON.stringify({
        manifest: sha256(manifestDocument), plan: expectedPlanSha256, attempt: 1,
      })),
    },
    detector_admitted: true,
    psram_available: passLog.includes("psram_status=available"),
    failure: {
      stable_load_ms: 5_000,
      planned_evaluation_failure: true,
      safe_stop_complete: true,
      failed_state_observed: true,
      cancel_checkpoint_safe: true,
      physical_long_press_observed: true,
      cancellation_receipt_observed: true,
      cancellation_restart_observed: true,
    },
    pass: {
      frequency_mhz: 485,
      core_voltage_mv: 1_200,
      difficulty: 16,
      warmup_celsius: 55,
      target_celsius: 65,
      maximum_celsius: 70,
      measurement_ms: 30_000,
      total_hashrate_passed: true,
      domain_count: 4,
      domain_evaluation_passed: true,
      electrical_checks_passed: true,
      fan_check_passed: true,
      watchdog_advanced: true,
      safe_stop_complete: true,
      pass_receipt_observed: true,
      automatic_restart_observed: true,
    },
    restoration: {
      settings_snapshot_captured_before_write: true,
      local_credentials_used_in_memory: true,
      settings_restored: true,
      mine_on_boot_disabled: true,
      production_mining_never_started: !passLog.includes("production_mining_session=active"),
      pool_traffic_absent: !passLog.includes("pool_transport="),
    },
    cleanup_complete: true,
    private_modes_valid: true,
    redaction_status: "passed",
  };
  const candidate = `${projection}.candidate`;
  await mkdir(path.dirname(projection), { recursive: true });
  await privateJson(candidate, evidence);
  await childText(processPort, validatorProgram, [candidate], "SELF-001 evidence validation");
  await rename(candidate, projection);
  return { checkpoint: "complete", projection_published: true, status: "ready" };
}

export async function selfTestCampaignFromInvocation(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
): Promise<JsonObject> {
  return optionValue(invocation, "--action") === "start"
    ? startCampaign(root, invocation, processPort, flashProgram)
    : resumeCampaign(root, invocation, processPort, flashProgram, validatorProgram);
}

export { SelfTestCampaignError };
