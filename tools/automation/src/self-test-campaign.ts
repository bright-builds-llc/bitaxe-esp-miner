import { randomBytes } from "node:crypto";
import { chmod, mkdir, readFile, rename } from "node:fs/promises";
import path from "node:path";

import { internalCommandSpec, monitorCommand } from "./contracts.generated.js";
import { portFromDetectorOutput } from "./detector.js";
import { maybeOptionValue, optionValue, type ParsedInvocation } from "./invocation.js";
import type { ProcessPort } from "./process.js";
import {
  type FlashTarget,
  ordinaryFlashMonitorSpec,
  selfTestFlashMonitorDryRunSpec,
  selfTestFlashMonitorSpec,
  selfTestMonitorSpec,
} from "./self-test-campaign-flash.js";
import { createSelfTestEvidence } from "./self-test-campaign-evidence.js";
import {
  type SelfTestCampaignProfile,
  selfTestCampaignProfile,
  admitsCampaignPackage,
} from "./self-test-campaign-profile.js";
import { restorationDiagnostic, restoreSelfTestSettings } from "./self-test-campaign-restoration.js";
import {
  failure,
  type JsonObject,
  object,
  privateJson,
  replacePrivateJson,
  requireAbsent,
  requiredString,
  requireMode,
  SelfTestCampaignError,
  sha256,
} from "./self-test-campaign-support.js";
import { assertWithinWorkspace } from "./workspace.js";

const expectedReference = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const monitorSeconds = 360;

type CampaignState = {
  readonly schema_version: "bitaxe-self-test-campaign-state-v1";
  readonly lease_hex: string;
  readonly source_commit: string;
  readonly reference_commit: string;
  readonly app_elf_sha256: string;
  readonly package_manifest_sha256: string;
  readonly settings_backup_sha256: string;
  readonly pre_boot_session: string;
  readonly stage: "failure_prepared" | "cancel_ready";
};

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
  target: FlashTarget,
): Promise<string> {
  const outcome = await processPort.run(target.board === 205
    ? monitorCommand(flashProgram, { board: 205, port, captureTimeoutSeconds: monitorSeconds })
    : selfTestMonitorSpec(target, flashProgram, port));
  if (outcome.timedOut) throw failure("timeout", "passive monitor timed out", "monitor");
  if (outcome.exitCode !== 0) {
    throw failure("hardware_blocked", "passive monitor failed", "monitor");
  }
  return outcome.stdout;
}

function intent(
  profile: SelfTestCampaignProfile,
  state: Pick<CampaignState, "lease_hex" | "source_commit" | "reference_commit" | "app_elf_sha256">,
  selfTestCase: "planned_failure" | "pass",
): JsonObject {
  return {
    schema_version: "bitaxe-self-test-intent-v1",
    board: profile.board,
    attempt_ordinal: profile.attemptOrdinal,
    source_commit: state.source_commit,
    reference_commit: state.reference_commit,
    app_elf_sha256: state.app_elf_sha256,
    plan_path: profile.plan,
    plan_sha256: profile.planSha256,
    case: selfTestCase,
    lease_hex: state.lease_hex,
  };
}

async function runFlashMonitor(
  processPort: ProcessPort,
  spec: ReturnType<typeof internalCommandSpec>,
  context: string,
): Promise<void> {
  const outcome = await processPort.run(spec);
  if (outcome.timedOut) throw failure("timeout", `${context} timed out`, context);
  if (outcome.exitCode !== 0) {
    throw failure("hardware_blocked", `${context} did not complete`, context);
  }
}

async function recoverOrdinaryRuntime(
  target: FlashTarget,
  processPort: ProcessPort,
  flashProgram: string,
  port: string,
  manifestPath: string,
  wifiPath: string,
  poolPath: string,
  privateRoot: string,
  backup: JsonObject,
): Promise<boolean> {
  const recoveryRoot = path.join(privateRoot, "recovery");
  let restored = false;
  try {
    await runFlashMonitor(
      processPort,
      ordinaryFlashMonitorSpec(target, {
        flashProgram,
        port,
        manifest: manifestPath,
        wifiCredentials: wifiPath,
        evidenceDir: recoveryRoot,
      }),
      "recovery",
    );
    const recoveryLog = await readFile(
      path.join(recoveryRoot, "flash-monitor.classifier-input.log"),
      "utf8",
    );
    await restoreSettings(singleOrigin(recoveryLog), backup, wifiPath, poolPath,
      path.join(privateRoot, "restoration-recovery.private.json"));
    restored = true;
    return true;
  } catch {
    return false;
  } finally {
    await privateJson(path.join(privateRoot, "recovery-status.private.json"), {
      schema_version: "bitaxe-self-test-recovery-status-v1",
      exact_package_attempted: true,
      settings_restored: restored,
      projection_published: false,
    });
  }
}

function validatePlanAndTask(profile: SelfTestCampaignProfile, plan: string, task: string): void {
  if (sha256(plan) !== profile.planSha256
    || !plan.includes("- Parity row: `SELF-001`")
    || !plan.includes(`- Active task: \`${profile.activeTask}\``)) {
    throw failure("evidence_invalid", "immutable plan binding is invalid", "contract");
  }
  const heading = `### ${profile.activeTask} |`;
  const start = task.indexOf(heading);
  const end = task.indexOf("\n### ", start + heading.length);
  const block = task.slice(start, end === -1 ? task.length : end);
  for (const required of profile.taskTerms) {
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
  const profile = selfTestCampaignProfile(
    maybeOptionValue(invocation, "--board"),
    path.relative(root, privateRoot),
  );
  if (profile === null
    || path.relative(root, projection) !== profile.projection
    || path.relative(root, planPath) !== profile.plan) {
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
  validatePlanAndTask(profile, plan, task);
  await validateSource(root, processPort, manifest);
  await requireMode(wifiCredentials, 0o600, false);
  await requireMode(poolCredentials, 0o600, false);
  const port = await portFromDetectorOutput(root, detector);
  if (!admitsCampaignPackage(manifest)) {
    throw failure("evidence_invalid", "the package's firmware variant is not admitted", "preflight");
  }
  const target: FlashTarget = { board: profile.board };

  const preMonitor = await currentMonitor(processPort, flashProgram, port, target);
  const preOrigin = singleOrigin(preMonitor);
  const settings = await fetchJson(preOrigin, "/api/system/info");
  const theme = await fetchJson(preOrigin, "/api/theme");
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
  const preparedState: CampaignState = {
    schema_version: "bitaxe-self-test-campaign-state-v1",
    lease_hex: leaseBytes.toString("hex"),
    source_commit: requiredString(manifest, "source_commit", "manifest"),
    reference_commit: requiredString(manifest, "reference_commit", "manifest"),
    app_elf_sha256: requiredString(manifest, "app_elf_sha256", "manifest"),
    package_manifest_sha256: sha256(manifestDocument),
    settings_backup_sha256: sha256(backupDocument),
    pre_boot_session: requiredString(settings, "bootSession", "settings snapshot"),
    stage: "failure_prepared",
  };
  const statePath = path.join(privateRoot, "campaign-state.private.json");
  const backup = object(JSON.parse(backupDocument), "settings backup");
  const failureIntentPath = path.join(privateRoot, "failure-intent.private.json");
  await privateJson(failureIntentPath, intent(profile, preparedState, "planned_failure"));
  await privateJson(statePath, preparedState);
  const relativeFailureIntent = path.relative(root, failureIntentPath);
  await runFlashMonitor(
    processPort,
    selfTestFlashMonitorDryRunSpec(
      target,
      flashProgram,
      port,
      manifestPath,
      wifiCredentials,
      relativeFailureIntent,
    ),
    "failure_admission",
  );
  const failureRoot = path.join(privateRoot, "failure");
  try {
    await runFlashMonitor(
      processPort,
      selfTestFlashMonitorSpec(
        target,
        flashProgram,
        port,
        manifestPath,
        wifiCredentials,
        relativeFailureIntent,
        failureRoot,
      ),
      "failure_phase",
    );
    const log = await readFile(
      path.join(failureRoot, "flash-monitor.classifier-input.log"),
      "utf8",
    );
    if (!log.includes('"stage":"measuring"')
      || !log.includes('"checkpoint":"cancel_ready"')
      || !log.includes('"safe_state":true')
      || !log.includes('"failure":"planned_evaluation_failure"')) {
      throw failure("hardware_blocked", "failure phase checkpoint is incomplete", "failure_phase");
    }
  } catch (error) {
    await recoverOrdinaryRuntime(
      target,
      processPort,
      flashProgram,
      port,
      manifestPath,
      wifiCredentials,
      poolCredentials,
      privateRoot,
      backup,
    );
    throw error;
  }
  const readyState: CampaignState = { ...preparedState, stage: "cancel_ready" };
  await replacePrivateJson(statePath, readyState);
  return {
    checkpoint: "cancel_ready",
    safe_state: true,
    action: "hold_boot_button_for_two_seconds",
    response_deadline: "none",
    projection_published: false,
  };
}

/** Restores the backup; a failure records its value-free step at `diagnosticPath` first. */
async function restoreSettings(
  origin: URL,
  backup: JsonObject,
  wifiPath: string,
  poolPath: string,
  diagnosticPath: string,
): Promise<void> {
  try {
    await restoreSelfTestSettings(origin, backup, wifiPath, poolPath);
  } catch (error) {
    // The typed failure stands when its diagnostic cannot be written, and says so.
    const written = await privateJson(diagnosticPath, {
      schema_version: "bitaxe-self-test-restoration-diagnostic-v1",
      ...restorationDiagnostic(error),
    }).then(() => true, () => false);
    throw failure(
      "hardware_blocked",
      written ? "settings restoration failed" : "settings restoration failed; its diagnostic was not written",
      "restoration",
    );
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
  const profile = selfTestCampaignProfile(
    maybeOptionValue(invocation, "--board"),
    path.relative(root, privateRoot),
  );
  if (profile === null
    || path.relative(root, projection) !== profile.projection
    || path.relative(root, planPath) !== profile.plan) {
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
  const backup = object(JSON.parse(backupDocument), "settings backup");
  const port = await portFromDetectorOutput(root, detector);
  const manifestDocument = await readFile(manifestPath, "utf8");
  if (!admitsCampaignPackage(object(JSON.parse(manifestDocument), "manifest"))) {
    throw failure("evidence_invalid", "the package's firmware variant is not admitted", "resume");
  }
  const target: FlashTarget = { board: profile.board };
  const cancellationLog = await currentMonitor(processPort, flashProgram, port, target);
  const cancelMarker = `self_test_receipt outcome=cancelled lease=${state.lease_hex}`;
  if (!cancellationLog.includes(cancelMarker)) {
    await recoverOrdinaryRuntime(
      target,
      processPort,
      flashProgram,
      port,
      manifestPath,
      wifiPath,
      poolPath,
      privateRoot,
      backup,
    );
    throw failure("hardware_blocked", "physical cancellation receipt was not observed", "cancel");
  }
  await privateJson(path.join(privateRoot, "pass-intent.private.json"), intent(profile, state, "pass"));
  const relativePassIntent = path.relative(root, path.join(privateRoot, "pass-intent.private.json"));
  try {
    await runFlashMonitor(
      processPort,
      selfTestFlashMonitorDryRunSpec(
        target,
        flashProgram,
        port,
        manifestPath,
        wifiPath,
        relativePassIntent,
      ),
      "pass_admission",
    );
  } catch (error) {
    let restored = false;
    try {
      await restoreSettings(singleOrigin(cancellationLog), backup, wifiPath, poolPath,
        path.join(privateRoot, "restoration-cancellation.private.json"));
      restored = true;
    } catch {
      restored = false;
    } finally {
      await privateJson(path.join(privateRoot, "recovery-status.private.json"), {
        schema_version: "bitaxe-self-test-recovery-status-v1",
        exact_package_attempted: false,
        settings_restored: restored,
        projection_published: false,
      });
    }
    throw error;
  }
  const passRoot = path.join(privateRoot, "pass");
  let passLog: string;
  try {
    await runFlashMonitor(
      processPort,
      selfTestFlashMonitorSpec(
        target,
        flashProgram,
        port,
        manifestPath,
        wifiPath,
        relativePassIntent,
        passRoot,
      ),
      "pass_phase",
    );
    passLog = await readFile(path.join(passRoot, "flash-monitor.classifier-input.log"), "utf8");
    const passMarker = `self_test_receipt outcome=passed lease=${state.lease_hex}`;
    const passStages = ["warming", "measuring", "evaluating", "safe_stopping", "restarting"];
    if (!passLog.includes('"outcome":"passed"')
      || !passLog.includes(passMarker)
      || passStages.some(stage => !passLog.includes(`"stage":"${stage}"`))) {
      throw failure("hardware_blocked", "passing phase or automatic restart was not observed", "pass_phase");
    }
  } catch (error) {
    await recoverOrdinaryRuntime(
      target,
      processPort,
      flashProgram,
      port,
      manifestPath,
      wifiPath,
      poolPath,
      privateRoot,
      backup,
    );
    throw error;
  }
  const origin = singleOrigin(passLog);
  try {
    await restoreSettings(origin, backup, wifiPath, poolPath,
      path.join(privateRoot, "restoration-pass.private.json"));
  } catch (error) {
    await recoverOrdinaryRuntime(
      target,
      processPort,
      flashProgram,
      port,
      manifestPath,
      wifiPath,
      poolPath,
      privateRoot,
      backup,
    );
    throw error;
  }

  const evidence = createSelfTestEvidence(
    profile,
    state,
    manifestDocument,
    passLog,
  );
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
