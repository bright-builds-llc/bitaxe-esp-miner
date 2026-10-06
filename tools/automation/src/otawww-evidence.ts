import { access, chmod, lstat, mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import http from "node:http";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type OtawwwEvidence,
} from "./contracts.generated.js";
import { isDeviceSessionProjectionFailure, readClosedDeviceSession } from "./device-session-projection.js";
import { flashChildFailureFacts, flashEffectEnvironment, inspectFlashEffect } from "./flash-child-diagnostics.js";
import { fetchJsonFromSameOrigin, fetchTextFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import { sendInterruptedUpload } from "./interrupted-upload.js";
import {
  isRecoveryPage, OtawwwInputError, parseOtawwwInputs, retainedLine, settingsDigest, sha256,
  unavailableAssetVersion, wwwFinishedLine, wwwProtocolErrorLine, wwwRoute, wwwSuccessBody, type OtawwwInputs,
} from "./otawww-evidence-model.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { verifySemanticEvidenceRedaction } from "./redaction.js";
import { passiveSafeStateLine } from "./sdkconfig-rollback-retained-log.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type OtawwwEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wwwProbeManifest: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type FailureCategory = Extract<
  AutomationCategory,
  | "package_invalid" | "process_failed" | "timeout" | "hardware_blocked" | "evidence_invalid"
  | "origin_unavailable" | "update_not_observed" | "asset_identity_mismatch" | "interruption_not_observed"
  | "recovery_not_observed" | "nvs_not_preserved"
>;
type JsonObject = Readonly<Record<string, unknown>>;
type RecoveryFacts = {
  readonly recovery_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};

const noRecovery: RecoveryFacts = { recovery_complete: false, recovery_flash_used: false, secondary_recovery_failure: false };
/** Installs boot, join Wi-Fi and repeat the 10-second runtime origin well inside this window. */
const initialCaptureSeconds = 120;
const interruptedPrefixBytes = 4_096;
/** Whole-partition erase plus a 3 MiB body over Wi-Fi; the handler blocks the HTTP task throughout. */
const uploadTimeoutMs = 600_000;
const baselineAttemptCount = 6;
/** Session facts the OTAWWW validator requires; checked per restart so a gap stops the run early. */
const requiredSessionFacts = [
  "same_physical_device", "stable_enumeration", "reader_armed", "pre_restart_serial_delivery",
  "post_restart_serial_delivery", "service_loss_observed", "trusted_origin_preserved", "application_recovered",
  "build_identity_matches", "boot_session_changed", "boot_ordinal_advanced_by_one", "software_reset_observed",
  "postcondition_matches", "cleanup_complete",
] as const;

/** Polling cadence; the handler erases all 3 MiB before its first read fails and serves nothing meanwhile. */
export type OtawwwTiming = {
  readonly interruptionPollCount: number;
  readonly interruptionPollDelayMs: number;
  readonly baselineRetryDelayMs: number;
};

const defaultTiming: OtawwwTiming = {
  interruptionPollCount: 60,
  interruptionPollDelayMs: 3_000,
  baselineRetryDelayMs: 1_000,
};

export class OtawwwEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "OtawwwEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): OtawwwEvidenceError {
    return new OtawwwEvidenceError(this.category, this.message, { ...this.publicValue, ...recovery });
  }
}

function failure(category: FailureCategory, message: string, facts: Readonly<Record<string, unknown>> = {}): OtawwwEvidenceError {
  return new OtawwwEvidenceError(category, message, { stage: "otawww_capture", ...noRecovery, ...facts });
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as JsonObject;
}

function text(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") throw failure("evidence_invalid", `${context} ${field} is invalid`);
  return candidate;
}

function ordinal(value: JsonObject, context: string): number {
  const candidate = value["bootOrdinal"];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 1) {
    throw failure("evidence_invalid", `${context} boot ordinal is invalid`);
  }
  return candidate;
}

function sameBuild(info: JsonObject, inputs: OtawwwInputs, context: string): boolean {
  return text(info, "sourceCommit", context) === inputs.source
    && text(info, "referenceCommit", context) === inputs.reference
    && text(info, "appElfSha256", context) === inputs.app;
}

function monitorBootSession(document: string): string {
  const sessions = new Set([...document.matchAll(/\bruntime_boot_identity session=([0-9a-f]{32})\b/gu)].map((match) => match[1]));
  const [maybeSession] = sessions;
  if (sessions.size !== 1 || maybeSession === undefined) throw failure("hardware_blocked", "monitor capture lacks one stable boot session");
  return maybeSession;
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof OtawwwEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
}

async function privateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
}

async function privateDir(dir: string): Promise<void> {
  await mkdir(dir, { mode: 0o700 });
  await chmod(dir, 0o700);
}

async function privateModesValid(root: string): Promise<boolean> {
  const metadata = await lstat(root);
  if (!metadata.isDirectory() || metadata.isSymbolicLink() || (metadata.mode & 0o777) !== 0o700) return false;
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) {
      if (!await privateModesValid(candidate)) return false;
      continue;
    }
    const childMetadata = await lstat(candidate);
    if (!childMetadata.isFile() || childMetadata.isSymbolicLink() || (childMetadata.mode & 0o777) !== 0o600) return false;
  }
  return true;
}

const delay = async (ms: number): Promise<void> => new Promise((resolve) => setTimeout(resolve, ms));

/** One same-origin binary POST with an explicit Origin header and a bounded reply. */
export async function postBinaryOnce(
  origin: URL,
  route: string,
  body: Buffer,
  timeoutMs: number,
): Promise<{ readonly status: number; readonly body: string }> {
  const target = new URL(route, origin);
  if (target.origin !== origin.origin || origin.pathname !== "/" || origin.search !== "" || origin.hash !== "") {
    throw new Error("binary upload target must stay on the admitted origin");
  }
  return new Promise((resolve, reject) => {
    const request = http.request(target, {
      method: "POST",
      headers: {
        Origin: origin.origin,
        "Content-Type": "application/octet-stream",
        "Content-Length": String(body.length),
        Connection: "close",
      },
    }, (response) => {
      const chunks: Buffer[] = [];
      let size = 0;
      response.on("data", (chunk: Buffer) => {
        size += chunk.length;
        if (size > 4_096) {
          request.destroy(new Error("upload response is too large"));
          return;
        }
        chunks.push(chunk);
      });
      response.on("end", () => resolve({ status: response.statusCode ?? 0, body: Buffer.concat(chunks).toString("utf8") }));
      response.on("error", reject);
    });
    request.setTimeout(timeoutMs, () => request.destroy(new Error("upload timed out")));
    request.on("error", reject);
    request.end(body);
  });
}

type StageContext = {
  readonly origin: URL;
  readonly privateRoot: string;
  readonly inputs: OtawwwInputs;
  readonly processPort: ProcessPort;
  readonly deviceSessionProgram: string;
  readonly options: OtawwwEvidenceOptions;
  readonly hostnameDigest: string;
  readonly timing: OtawwwTiming;
};

async function systemInfo(context: StageContext, name: string): Promise<JsonObject> {
  return object(await fetchJsonFromSameOrigin(context.origin, "/api/system/info",
    path.join(context.privateRoot, `${name}-system-info.private.json`)), `${name} system info`);
}

async function logs(context: StageContext, name: string): Promise<string> {
  return fetchTextFromSameOrigin(context.origin, "/api/system/logs", path.join(context.privateRoot, `${name}-logs.private.txt`));
}

async function servedDigest(context: StageContext, route: string, name: string): Promise<{ digest: string; body: string }> {
  const body = await fetchTextFromSameOrigin(context.origin, route, path.join(context.privateRoot, `${name}.private.txt`));
  return { digest: sha256(body), body };
}

async function uploadComplete(context: StageContext, image: Buffer, name: string): Promise<boolean> {
  let response;
  try {
    response = await postBinaryOnce(context.origin, wwwRoute, image, uploadTimeoutMs);
  } catch {
    return false;
  }
  await privateJson(path.join(context.privateRoot, `${name}-upload-response.private.json`), response);
  if (response.status !== 200 || response.body !== wwwSuccessBody) return false;
  return retainedLine(await logs(context, `${name}-upload`), wwwFinishedLine);
}

/** One HTTP restart proven by `device-session reboot-live`; returns the closed session and new info. */
async function provenRestart(
  context: StageContext,
  before: JsonObject,
  name: string,
): Promise<{ session: JsonObject; info: JsonObject }> {
  const intentPath = path.join(context.privateRoot, `${name}-reboot-intent.private.json`);
  const sessionRoot = path.join(context.privateRoot, `${name}-session`);
  const projectionPath = path.join(context.privateRoot, `${name}-session-projection.private.json`);
  const runningPartition = text(before, "runningPartition", `${name} pre-restart info`);
  await privateJson(intentPath, {
    schema_version: "esp-device-session-reboot-intent-v1",
    board_category: "205",
    trusted_origin: context.origin.origin,
    baseline: {
      boot_session: text(before, "bootSession", `${name} pre-restart info`),
      boot_ordinal: ordinal(before, `${name} pre-restart info`),
      source_commit: context.inputs.source,
      reference_commit: context.inputs.reference,
      app_elf_sha256: context.inputs.app,
      running_partition: runningPartition,
    },
    expected_postcondition: {
      hostname_sha256: context.hostnameDigest,
      app_elf_sha256: context.inputs.app,
      running_partition: runningPartition,
    },
  });
  await privateDir(sessionRoot);
  let outcome: ProcessOutcome;
  try {
    outcome = await context.processPort.run(internalCommandSpec(context.deviceSessionProgram, [
      "reboot-live", "--port", context.options.port,
      "--intent-input", intentPath,
      "--private-root", sessionRoot,
      "--projection-output", projectionPath,
      "--timeout-seconds", String(context.options.captureTimeoutSeconds),
    ], (value) => value));
  } catch {
    throw failure("process_failed", `${name} restart launch failed`, { stage: `${name}_restart` });
  }
  if (outcome.timedOut) throw failure("timeout", `${name} restart timed out`, { stage: `${name}_restart` });
  let session: JsonObject;
  try {
    session = await readClosedDeviceSession(projectionPath);
  } catch (error) {
    if (isDeviceSessionProjectionFailure(error)) throw failure(error.category, error.message, { ...error.facts, stage: `${name}_restart` });
    throw failure("evidence_invalid", `${name} restart projection is invalid`, { stage: `${name}_restart` });
  }
  if (outcome.exitCode !== 0) throw failure("hardware_blocked", `${name} restart was not proven`, { stage: `${name}_restart` });
  const missing = requiredSessionFacts.filter((field) => session[field] !== true);
  if (missing.length > 0) {
    throw failure("hardware_blocked", `${name} restart session lacks required facts`, { stage: `${name}_restart`, missing_facts: missing });
  }
  const info = await systemInfo(context, `${name}-restart`);
  if (
    !sameBuild(info, context.inputs, `${name} restart info`)
    || ordinal(info, `${name} restart info`) !== ordinal(before, `${name} pre-restart info`) + 1
    || text(info, "bootSession", `${name} restart info`) === text(before, "bootSession", `${name} pre-restart info`)
  ) {
    throw failure("hardware_blocked", `${name} restart identity is invalid`, { stage: `${name}_restart` });
  }
  if (!retainedLine(await logs(context, `${name}-restart`), passiveSafeStateLine)) {
    throw failure("hardware_blocked", `${name} boot lacks the passive safe state`, { stage: `${name}_restart` });
  }
  return { session, info };
}

async function interruptionObserved(context: StageContext, before: JsonObject, image: Buffer): Promise<boolean> {
  await sendInterruptedUpload(context.origin, wwwRoute, image, interruptedPrefixBytes);
  for (let attempt = 1; attempt <= context.timing.interruptionPollCount; attempt += 1) {
    await delay(context.timing.interruptionPollDelayMs);
    let current: JsonObject;
    let retained: string;
    try {
      current = await systemInfo(context, `interruption-${String(attempt)}`);
      retained = await logs(context, `interruption-${String(attempt)}`);
    } catch {
      continue; // The handler holds the only HTTP task while it erases.
    }
    if (
      !sameBuild(current, context.inputs, "post-interruption info")
      || text(current, "bootSession", "post-interruption info") !== text(before, "bootSession", "pre-interruption info")
    ) {
      throw failure("interruption_not_observed", "interrupted upload changed the running application");
    }
    if (retainedLine(retained, wwwProtocolErrorLine)) return true;
  }
  return false;
}

async function recover(context: { processPort: ProcessPort; flashProgram: string; options: OtawwwEvidenceOptions; manifest: string }): Promise<RecoveryFacts> {
  try {
    const outcome = await context.processPort.run(flashCommand(context.flashProgram, {
      board: 205,
      port: context.options.port,
      manifest: context.manifest,
      evidenceDir: path.join(context.options.privateRoot, "recovery"),
    }));
    const complete = !outcome.timedOut && outcome.exitCode === 0;
    return { recovery_complete: complete, recovery_flash_used: true, secondary_recovery_failure: !complete };
  } catch {
    return { recovery_complete: false, recovery_flash_used: true, secondary_recovery_failure: true };
  }
}

export async function captureOtawwwEvidence(
  workspaceRoot: string,
  options: OtawwwEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  deviceSessionProgram: string,
  validatorProgram: string,
  timing: OtawwwTiming = defaultTiming,
): Promise<OtawwwEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const probeManifestPath = assertWithinWorkspace(workspaceRoot, options.wwwProbeManifest);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  const packageWwwPath = path.join(path.dirname(manifestPath), "www.bin");
  const probeWwwPath = path.join(path.dirname(probeManifestPath), "www-probe.bin");
  try {
    await Promise.all([access(manifestPath), access(probeManifestPath), access(packageWwwPath), access(probeWwwPath)]);
  } catch {
    throw failure("package_invalid", "OTAWWW package inputs are unavailable");
  }
  try {
    await stat(projectionPath);
    throw failure("evidence_invalid", "projection must be absent before launch");
  } catch (error) {
    if (error instanceof OtawwwEvidenceError) throw error;
  }
  await createPrivateRoot(privateRoot);
  const [manifestDocument, probeDocument, packageWww, probeWww] = await Promise.all([
    readFile(manifestPath, "utf8"), readFile(probeManifestPath, "utf8"), readFile(packageWwwPath), readFile(probeWwwPath),
  ]);
  let inputs: OtawwwInputs;
  try {
    inputs = parseOtawwwInputs(manifestDocument, probeDocument, packageWww, probeWww);
  } catch (error) {
    throw failure("package_invalid", error instanceof OtawwwInputError ? error.message : "OTAWWW package inputs are malformed");
  }
  const manifestDigest = sha256(manifestDocument);
  const effectPath = path.join(privateRoot, "flash-effect.private.json");
  const expectedEffect = { packageIdentityDigest: manifestDigest, factoryImageDigest: inputs.factoryDigest };
  const baseFlash = flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: manifestPath,
    captureTimeoutSeconds: initialCaptureSeconds,
    evidenceMode: "dual",
    evidenceDir: privateRoot,
  });
  const flashSpec = internalCommandSpec(baseFlash.program, [...baseFlash.args], baseFlash.result,
    flashEffectEnvironment(effectPath, expectedEffect));
  let deviceEffectStarted = false;
  let packageAssetsInstalled = false;
  try {
    let flashOutcome: ProcessOutcome;
    try {
      flashOutcome = await processPort.run(flashSpec);
    } catch {
      const effect = await inspectFlashEffect(effectPath, expectedEffect);
      deviceEffectStarted = effect.flash_effect_status !== "unavailable" && effect.flash_effect_status !== "failed_no_device_effect";
      throw failure("process_failed", "exact-package flash-monitor launch failed", flashChildFailureFacts(undefined, effect));
    }
    const effect = await inspectFlashEffect(effectPath, expectedEffect);
    deviceEffectStarted = effect.flash_effect_status !== "unavailable" && effect.flash_effect_status !== "failed_no_device_effect";
    const flashFacts = flashChildFailureFacts(flashOutcome, effect);
    if (flashOutcome.timedOut) throw failure("timeout", "exact-package flash-monitor timed out", flashFacts);
    if (flashOutcome.exitCode !== 0) throw failure("hardware_blocked", "exact-package flash-monitor failed", flashFacts);
    if (effect.flash_effect_result_status !== "valid" || effect.flash_effect_status !== "completed") {
      throw failure("evidence_invalid", "exact-package flash effect result is invalid", flashFacts);
    }
    packageAssetsInstalled = true;
    const monitor = await readFile(path.join(privateRoot, "flash-monitor.classifier-input.log"), "utf8");
    if (!hasPassiveSafeState(monitor)) throw failure("hardware_blocked", "installed boot lacks passive safe-state evidence");
    let origin: URL;
    try {
      origin = uniqueRuntimeOrigin(monitor);
    } catch {
      throw failure("origin_unavailable", "the install session reported no unique station origin");
    }
    const baseline = await readBaseline(origin, privateRoot, timing.baselineRetryDelayMs);
    if (!sameBuild(baseline, inputs, "baseline info") || text(baseline, "bootSession", "baseline info") !== monitorBootSession(monitor)) {
      throw failure("evidence_invalid", "baseline identity does not match the installed package");
    }
    if (baseline["axeOSVersion"] !== inputs.buildLabel) throw failure("asset_identity_mismatch", "installed web assets do not report the package label");
    const hostnameDigest = sha256(text(baseline, "hostname", "baseline info"));
    const baselineSettings = settingsDigest(baseline);
    const context: StageContext = {
      origin, privateRoot, inputs, processPort, deviceSessionProgram, options, hostnameDigest, timing,
    };

    packageAssetsInstalled = false;
    if (!await uploadComplete(context, probeWww, "probe")) throw failure("update_not_observed", "probe OTAWWW did not complete");
    const probeRestart = await provenRestart(context, baseline, "probe");
    const probeVersion = await servedDigest(context, "/version.txt", "probe-version");
    const probeIndex = await servedDigest(context, "/index.html", "probe-index");
    if (
      probeRestart.info["axeOSVersion"] !== inputs.probe.probe_label
      || probeVersion.digest !== inputs.probe.probe_version_txt_sha256
      || probeIndex.digest !== inputs.probe.index_html_sha256
    ) {
      throw failure("asset_identity_mismatch", "served assets do not match the probe image");
    }

    if (!await interruptionObserved(context, probeRestart.info, packageWww)) {
      throw failure("interruption_not_observed", "interrupted OTAWWW protocol error was not retained");
    }
    const interruptedInfo = await systemInfo(context, "interrupted");
    const interruptedRestart = await provenRestart(context, interruptedInfo, "interrupted");
    const fallbackVersion = await servedDigest(context, "/version.txt", "interrupted-version");
    const recoveryPage = await servedDigest(context, "/recovery", "interrupted-recovery");
    if (
      interruptedRestart.info["axeOSVersion"] !== unavailableAssetVersion
      || !isRecoveryPage(fallbackVersion.body)
      || !isRecoveryPage(recoveryPage.body)
    ) {
      throw failure("interruption_not_observed", "interrupted partition still served web assets after restart");
    }

    if (!await uploadComplete(context, packageWww, "recovery")) throw failure("recovery_not_observed", "recovery OTAWWW did not complete");
    const recoveryRestart = await provenRestart(context, interruptedRestart.info, "recovery");
    const recoveryVersion = await servedDigest(context, "/version.txt", "recovery-version");
    const recoveryIndex = await servedDigest(context, "/index.html", "recovery-index");
    if (
      recoveryRestart.info["axeOSVersion"] !== inputs.buildLabel
      || recoveryVersion.digest !== inputs.probe.package_version_txt_sha256
      || recoveryIndex.digest !== inputs.probe.index_html_sha256
    ) {
      throw failure("recovery_not_observed", "recovered assets do not match the package image");
    }
    packageAssetsInstalled = true;
    const finalInfo = recoveryRestart.info;
    if (sha256(text(finalInfo, "hostname", "final info")) !== hostnameDigest || settingsDigest(finalInfo) !== baselineSettings) {
      throw failure("nvs_not_preserved", "stored settings changed across the OTAWWW run");
    }
    if (!await privateModesValid(privateRoot)) throw failure("evidence_invalid", "private artifact modes are invalid");

    const evidence: OtawwwEvidence = {
      schema_version: "bitaxe-otawww-evidence-v1",
      board: 205,
      source_commit: inputs.source,
      reference_commit: inputs.reference,
      package_manifest_sha256: manifestDigest,
      www_probe_metadata_sha256: sha256(probeDocument),
      package_www_sha256: inputs.probe.package_www_sha256,
      probe_www_sha256: inputs.probe.probe_www_sha256,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-otawww-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: manifestDigest,
          probe: sha256(probeDocument),
          timeout: options.captureTimeoutSeconds,
        })),
      },
      detector_admitted: true,
      otawww: {
        update_request_count: 3,
        probe_update_response_complete: true,
        probe_finished_status_retained: true,
        probe_version_reported: true,
        probe_version_txt_digest_matches: true,
        probe_index_html_digest_matches: true,
        interrupted_upload_attempt_count: 1,
        interrupted_upload_prefix_bytes: interruptedPrefixBytes,
        interruption_protocol_error_retained: true,
        interruption_boot_session_unchanged: true,
        interrupted_assets_unavailable_after_restart: true,
        recovery_page_served_after_interruption: true,
        recovery_update_response_complete: true,
        recovery_version_reported: true,
        recovery_version_txt_digest_matches: true,
        recovery_index_html_digest_matches: true,
        build_identity_unchanged: true,
        hostname_unchanged: true,
        settings_digest_unchanged: true,
      },
      probe_restart_session: probeRestart.session,
      interrupted_restart_session: interruptedRestart.session,
      recovery_restart_session: recoveryRestart.session,
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      recovery_flash_used: false,
      private_modes_valid: true,
      redaction_status: "passed",
    };
    const candidate = path.join(privateRoot, "final-evidence.private.json");
    await privateJson(candidate, evidence);
    await verifySemanticEvidenceRedaction(privateRoot);
    let validation: ProcessOutcome;
    try {
      validation = await processPort.run(internalCommandSpec(validatorProgram, [candidate], (value) => value));
    } catch {
      throw failure("process_failed", "OTAWWW validator launch failed");
    }
    if (validation.timedOut || validation.exitCode !== 0) throw failure("evidence_invalid", "OTAWWW validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof OtawwwEvidenceError ? error : failure("evidence_invalid", "OTAWWW orchestration evidence is invalid");
    if (!deviceEffectStarted || packageAssetsInstalled) throw primary;
    throw primary.withRecovery(await recover({ processPort, flashProgram, options, manifest: manifestPath }));
  }
}

async function readBaseline(origin: URL, privateRoot: string, retryDelayMs: number): Promise<JsonObject> {
  for (let attempt = 1; attempt <= baselineAttemptCount; attempt += 1) {
    try {
      return object(await fetchJsonFromSameOrigin(origin, "/api/system/info",
        path.join(privateRoot, `baseline-${String(attempt)}-system-info.private.json`)), "baseline system info");
    } catch (error) {
      if (error instanceof OtawwwEvidenceError) throw error;
      await delay(retryDelayMs);
    }
  }
  throw failure("origin_unavailable", "baseline HTTP readiness was not established at the reported origin");
}
