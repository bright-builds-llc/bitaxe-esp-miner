import { createHash } from "node:crypto";
import { access, chmod, lstat, mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type SdkconfigRollbackEvidence,
} from "./contracts.generated.js";
import { readClosedDeviceSession, isDeviceSessionProjectionFailure } from "./device-session-projection.js";
import {
  factoryImageDigest,
  flashChildFailureFacts,
  flashEffectEnvironment,
  inspectFlashEffect,
} from "./flash-child-diagnostics.js";
import { fetchJsonFromSameOrigin, fetchTextFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import { sendInterruptedFirmwareUpload } from "./interrupted-upload.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { verifySemanticEvidenceRedaction } from "./redaction.js";
import { rollbackProbeSchema, type RollbackProbeMetadata } from "./rollback-probe.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type SdkconfigRollbackEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly rollbackProbeImage: string;
  readonly rollbackProbeMetadata: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<
  AutomationCategory,
  | "hardware_blocked"
  | "evidence_invalid"
  | "timeout"
  | "process_failed"
  | "package_invalid"
  | "interruption_not_observed"
  | "probe_boot_failed"
  | "rollback_not_observed"
>;
type RecoveryFacts = {
  readonly recovery_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};

const noRecovery: RecoveryFacts = {
  recovery_complete: false,
  recovery_flash_used: false,
  secondary_recovery_failure: false,
};
const digestPattern = /^[a-f0-9]{64}$/u;
const commitPattern = /^[a-f0-9]{40}$/u;
const interruptedPrefixBytes = 4_096;
const initialFlashCaptureTimeoutSeconds = 90;
const baselineHttpAttemptCount = 6;
const baselineHttpRetryDelayMs = 1_000;

export class SdkconfigRollbackEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "SdkconfigRollbackEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): SdkconfigRollbackEvidenceError {
    return new SdkconfigRollbackEvidenceError(this.category, this.message, {
      ...this.publicValue,
      ...recovery,
    });
  }
}

function failure(
  category: FailureCategory,
  message: string,
  facts: Readonly<Record<string, unknown>> = {},
): SdkconfigRollbackEvidenceError {
  return new SdkconfigRollbackEvidenceError(category, message, {
    stage: "sdkconfig_rollback_capture",
    ...noRecovery,
    ...facts,
  });
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as JsonObject;
}

function requiredString(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") {
    throw failure("evidence_invalid", `${context} field is invalid`);
  }
  return candidate;
}

function requiredOrdinal(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 1) {
    throw failure("evidence_invalid", `${context} ordinal is invalid`);
  }
  return candidate;
}

function validDigest(value: string, context: string): string {
  if (!digestPattern.test(value) || /^0+$/u.test(value)) {
    throw failure("package_invalid", `${context} digest is invalid`);
  }
  return value;
}

function validateRuntimeIdentity(runtime: JsonObject, source: string, reference: string, app: string): void {
  if (
    requiredString(runtime, "sourceCommit", "system info") !== source
    || requiredString(runtime, "referenceCommit", "system info") !== reference
    || requiredString(runtime, "appElfSha256", "system info") !== app
  ) {
    throw failure("evidence_invalid", "runtime identity does not match the admitted build");
  }
}

function monitorBootSession(document: string): string {
  const sessions = new Set<string>();
  for (const match of document.matchAll(/\bruntime_boot_identity session=([0-9a-f]{32})\b/gu)) {
    const candidate = match[1];
    if (candidate !== undefined) sessions.add(candidate);
  }
  const [maybeSession] = sessions;
  if (sessions.size !== 1 || maybeSession === undefined) {
    throw failure("hardware_blocked", "monitor capture lacks one stable boot session");
  }
  return maybeSession;
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof SdkconfigRollbackEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
}

async function privateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o600,
  });
  await chmod(output, 0o600);
}

async function child(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
  context: string,
): Promise<ProcessOutcome> {
  try {
    return await processPort.run(internalCommandSpec(program, [...args], (value) => value));
  } catch {
    throw failure("process_failed", `${context} launch failed`);
  }
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
    if (!childMetadata.isFile() || childMetadata.isSymbolicLink() || (childMetadata.mode & 0o777) !== 0o600) {
      return false;
    }
  }
  return true;
}

function parseInputs(
  manifestDocument: string,
  probeDocument: string,
  probeImage: Buffer,
  sdkconfig: string,
): {
  probe: RollbackProbeMetadata;
  source: string;
  reference: string;
  normalApp: string;
  factoryDigest: string;
} {
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const source = requiredString(manifest, "source_commit", "package manifest");
  const reference = requiredString(manifest, "reference_commit", "package manifest");
  const normalApp = validDigest(requiredString(manifest, "app_elf_sha256", "package manifest"), "package app");
  const buildIdentity = object(manifest["build_identity"], "package build identity");
  const buildLabel = requiredString(buildIdentity, "label", "package build identity");
  if (!commitPattern.test(source) || !commitPattern.test(reference) || buildIdentity["source_dirty"] !== false) {
    throw failure("package_invalid", "normal package provenance is not clean and exact");
  }
  const probeValue = object(JSON.parse(probeDocument), "rollback probe metadata");
  const probe = probeValue as unknown as RollbackProbeMetadata;
  if (
    probe.schema_version !== rollbackProbeSchema
    || probe.source_commit !== source
    || probe.reference_commit !== reference
    || probe.source_dirty !== false
    || probe.build_label !== buildLabel
    || probe.rollback_probe !== true
    || probe.app_elf_sha256 === normalApp
    || !digestPattern.test(probe.app_elf_sha256)
    || probe.ota_image_sha256 !== sha256(probeImage)
    || probe.ota_image_bytes !== probeImage.length
  ) {
    throw failure("package_invalid", "rollback probe identity is not clean, isolated, and exact");
  }
  const lines = new Set(sdkconfig.split(/\r?\n/u));
  const rollbackEnabled = lines.has("CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y")
    && lines.has("CONFIG_APP_ROLLBACK_ENABLE=y");
  const antiRollbackDisabled = lines.has("# CONFIG_BOOTLOADER_APP_ANTI_ROLLBACK is not set")
    && lines.has("# CONFIG_APP_ANTI_ROLLBACK is not set");
  const buildBound = lines.has(`CONFIG_APP_PROJECT_VER="${buildLabel}"`)
    && lines.has("CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64");
  let factoryDigest: string;
  try {
    factoryDigest = factoryImageDigest(manifest as Record<string, unknown>);
  } catch {
    throw failure("package_invalid", "normal factory image identity is invalid");
  }
  if (!rollbackEnabled || !antiRollbackDisabled || !buildBound) {
    throw failure("package_invalid", "generated SDK config lacks the admitted rollback policy");
  }
  return { probe, source, reference, normalApp, factoryDigest };
}

async function readSession(output: string): Promise<JsonObject> {
  try {
    return await readClosedDeviceSession(output);
  } catch (error) {
    if (isDeviceSessionProjectionFailure(error)) {
      throw failure(error.category, error.message, error.facts);
    }
    throw failure("evidence_invalid", "device-session projection is invalid");
  }
}

async function recover(
  processPort: ProcessPort,
  flashProgram: string,
  options: SdkconfigRollbackEvidenceOptions,
  manifest: string,
  credentials: string,
): Promise<RecoveryFacts> {
  try {
    const outcome = await processPort.run(flashCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest,
      wifiCredentials: credentials,
      evidenceDir: path.join(options.privateRoot, "recovery"),
    }));
    const complete = !outcome.timedOut && outcome.exitCode === 0;
    return { recovery_complete: complete, recovery_flash_used: true, secondary_recovery_failure: !complete };
  } catch {
    return { recovery_complete: false, recovery_flash_used: true, secondary_recovery_failure: true };
  }
}

async function interruptedUploadConfirmed(
  origin: URL,
  privateRoot: string,
  baseline: JsonObject,
  image: Buffer,
  source: string,
  reference: string,
  app: string,
): Promise<boolean> {
  await sendInterruptedFirmwareUpload(origin, image, interruptedPrefixBytes);
  for (let attempt = 1; attempt <= 10; attempt += 1) {
    await new Promise((resolve) => setTimeout(resolve, 500));
    const current = object(await fetchJsonFromSameOrigin(
      origin,
      "/api/system/info",
      path.join(privateRoot, `interruption-system-info-${String(attempt)}.private.json`),
    ), "post-interruption system info");
    validateRuntimeIdentity(current, source, reference, app);
    if (
      requiredString(current, "bootSession", "post-interruption system info")
        !== requiredString(baseline, "bootSession", "baseline system info")
      || requiredOrdinal(current, "bootOrdinal", "post-interruption system info")
        !== requiredOrdinal(baseline, "bootOrdinal", "baseline system info")
      || requiredString(current, "runningPartition", "post-interruption system info") !== "factory"
    ) {
      throw failure("evidence_invalid", "interrupted upload changed the baseline application");
    }
    const logs = await fetchTextFromSameOrigin(
      origin,
      "/api/system/logs",
      path.join(privateRoot, `interruption-log-${String(attempt)}.private.txt`),
    );
    if (/\bfirmware_ota_update=protocol_error\b/u.test(logs)) return true;
  }
  return false;
}

async function readBaselineWhenReady(origin: URL, privateRoot: string): Promise<JsonObject> {
  for (let attempt = 1; attempt <= baselineHttpAttemptCount; attempt += 1) {
    try {
      return object(await fetchJsonFromSameOrigin(
        origin,
        "/api/system/info",
        path.join(privateRoot, "baseline-system-info.private.json"),
      ), "baseline system info");
    } catch (error) {
      if (error instanceof SdkconfigRollbackEvidenceError) throw error;
      if (attempt === baselineHttpAttemptCount) {
        throw failure("hardware_blocked", "baseline HTTP readiness was not established");
      }
      await new Promise((resolve) => setTimeout(resolve, baselineHttpRetryDelayMs));
    }
  }
  throw failure("hardware_blocked", "baseline HTTP readiness was not established");
}

export async function captureSdkconfigRollbackEvidence(
  workspaceRoot: string,
  options: SdkconfigRollbackEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  deviceSessionProgram: string,
  validatorProgram: string,
): Promise<SdkconfigRollbackEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const probeImagePath = assertWithinWorkspace(workspaceRoot, options.rollbackProbeImage);
  const probeMetadataPath = assertWithinWorkspace(workspaceRoot, options.rollbackProbeMetadata);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  const sdkconfigPath = path.join(path.dirname(manifestPath), "bitaxe-firmware.sdkconfig");
  try {
    await Promise.all([
      access(manifestPath), access(probeImagePath), access(probeMetadataPath), access(credentialsPath), access(sdkconfigPath),
    ]);
  } catch {
    throw failure("package_invalid", "rollback package inputs are unavailable");
  }
  await createPrivateRoot(privateRoot);
  const [manifestDocument, probeDocument, probeImage, sdkconfig] = await Promise.all([
    readFile(manifestPath, "utf8"),
    readFile(probeMetadataPath, "utf8"),
    readFile(probeImagePath),
    readFile(sdkconfigPath, "utf8"),
  ]);
  let parsed;
  try {
    parsed = parseInputs(manifestDocument, probeDocument, probeImage, sdkconfig);
  } catch (error) {
    if (error instanceof SdkconfigRollbackEvidenceError) throw error;
    throw failure("package_invalid", "rollback package inputs are malformed");
  }
  const { probe, source, reference, normalApp, factoryDigest } = parsed;
  const manifestDigest = sha256(manifestDocument);
  const probeMetadataDigest = sha256(probeDocument);
  const effectPath = path.join(privateRoot, "flash-effect.private.json");
  const expectedEffect = { packageIdentityDigest: manifestDigest, factoryImageDigest: factoryDigest };
  const baseFlash = flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: manifestPath,
    wifiCredentials: credentialsPath,
    captureTimeoutSeconds: initialFlashCaptureTimeoutSeconds,
    evidenceMode: "dual",
    evidenceDir: privateRoot,
  });
  const flashSpec = internalCommandSpec(
    baseFlash.program,
    [...baseFlash.args],
    baseFlash.result,
    flashEffectEnvironment(effectPath, expectedEffect),
  );
  let deviceEffectStarted = false;
  let normalRestored = false;
  try {
    let flashOutcome: ProcessOutcome;
    try {
      flashOutcome = await processPort.run(flashSpec);
    } catch {
      const effect = await inspectFlashEffect(effectPath, expectedEffect);
      deviceEffectStarted = effect.flash_effect_status !== "unavailable"
        && effect.flash_effect_status !== "failed_no_device_effect";
      throw failure("process_failed", "exact-package flash-monitor launch failed", flashChildFailureFacts(undefined, effect));
    }
    const effect = await inspectFlashEffect(effectPath, expectedEffect);
    deviceEffectStarted = effect.flash_effect_status !== "unavailable"
      && effect.flash_effect_status !== "failed_no_device_effect";
    const flashFacts = flashChildFailureFacts(flashOutcome, effect);
    if (flashOutcome.timedOut) throw failure("timeout", "exact-package flash-monitor timed out", flashFacts);
    if (flashOutcome.exitCode !== 0) throw failure("hardware_blocked", "exact-package flash-monitor failed", flashFacts);
    if (effect.flash_effect_result_status !== "valid" || effect.flash_effect_status !== "completed") {
      throw failure("evidence_invalid", "exact-package flash effect result is invalid", flashFacts);
    }
    const monitor = await readFile(path.join(privateRoot, "flash-monitor.classifier-input.log"), "utf8");
    if (!hasPassiveSafeState(monitor)) throw failure("hardware_blocked", "factory boot lacks passive safe-state evidence");
    let origin: URL;
    try {
      origin = uniqueRuntimeOrigin(monitor);
    } catch {
      throw failure("hardware_blocked", "runtime origin admission is invalid");
    }
    const baseline = await readBaselineWhenReady(origin, privateRoot);
    validateRuntimeIdentity(baseline, source, reference, normalApp);
    if (
      requiredString(baseline, "bootSession", "baseline system info") !== monitorBootSession(monitor)
      || requiredString(baseline, "runningPartition", "baseline system info") !== "factory"
    ) {
      throw failure("evidence_invalid", "factory baseline identity is invalid");
    }
    const hostnameDigest = sha256(requiredString(baseline, "hostname", "baseline system info"));
    if (!await interruptedUploadConfirmed(origin, privateRoot, baseline, probeImage, source, reference, normalApp)) {
      normalRestored = true;
      throw failure("interruption_not_observed", "interrupted OTA protocol abort was not retained");
    }
    normalRestored = true;

    const probeIntentPath = path.join(privateRoot, "probe-ota-intent.private.json");
    const probeSessionRoot = path.join(privateRoot, "probe-session");
    const probeProjectionPath = path.join(privateRoot, "probe-session-projection.private.json");
    await privateJson(probeIntentPath, {
      schema_version: "esp-device-session-ota-intent-v1",
      board_category: "205",
      trusted_origin: origin.origin,
      baseline: {
        boot_session: requiredString(baseline, "bootSession", "baseline system info"),
        boot_ordinal: requiredOrdinal(baseline, "bootOrdinal", "baseline system info"),
        source_commit: source,
        reference_commit: reference,
        app_elf_sha256: normalApp,
        running_partition: "factory",
      },
      expected_postcondition: {
        hostname_sha256: hostnameDigest,
        app_elf_sha256: probe.app_elf_sha256,
        running_partition: "ota_0",
      },
      ota_image_sha256: probe.ota_image_sha256,
    });
    await mkdir(probeSessionRoot, { mode: 0o700 });
    await chmod(probeSessionRoot, 0o700);
    normalRestored = false;
    const probeOutcome = await child(processPort, deviceSessionProgram, [
      "ota-live", "--port", options.port,
      "--private-root", probeSessionRoot,
      "--intent-input", probeIntentPath,
      "--ota-image", probeImagePath,
      "--projection-output", probeProjectionPath,
      "--timeout-seconds", String(options.captureTimeoutSeconds),
    ], "rollback probe OTA");
    if (probeOutcome.timedOut) throw failure("timeout", "rollback probe OTA timed out");
    const probeSession = await readSession(probeProjectionPath);
    if (probeOutcome.exitCode !== 0) throw failure("probe_boot_failed", "rollback probe OTA child failed");
    const probeInfo = object(await fetchJsonFromSameOrigin(
      origin,
      "/api/system/info",
      path.join(privateRoot, "probe-system-info.private.json"),
    ), "probe system info");
    validateRuntimeIdentity(probeInfo, source, reference, probe.app_elf_sha256);
    if (
      requiredString(probeInfo, "runningPartition", "probe system info") !== "ota_0"
      || requiredOrdinal(probeInfo, "bootOrdinal", "probe system info")
        !== requiredOrdinal(baseline, "bootOrdinal", "baseline system info") + 1
      || requiredString(probeInfo, "bootSession", "probe system info")
        === requiredString(baseline, "bootSession", "baseline system info")
    ) {
      throw failure("probe_boot_failed", "rollback probe boot identity is invalid");
    }
    const probeSerial = await readFile(path.join(probeSessionRoot, "serial.private.bin"), "utf8");
    if (!/\bota_boot_validation=rollback_probe_pending\b/u.test(probeSerial) || !hasPassiveSafeState(probeSerial)) {
      throw failure("probe_boot_failed", "rollback probe pending marker or safe state is missing");
    }

    const rollbackIntentPath = path.join(privateRoot, "rollback-reboot-intent.private.json");
    const rollbackSessionRoot = path.join(privateRoot, "rollback-session");
    const rollbackProjectionPath = path.join(privateRoot, "rollback-session-projection.private.json");
    await privateJson(rollbackIntentPath, {
      schema_version: "esp-device-session-reboot-intent-v1",
      board_category: "205",
      trusted_origin: origin.origin,
      baseline: {
        boot_session: requiredString(probeInfo, "bootSession", "probe system info"),
        boot_ordinal: requiredOrdinal(probeInfo, "bootOrdinal", "probe system info"),
        source_commit: source,
        reference_commit: reference,
        app_elf_sha256: probe.app_elf_sha256,
        running_partition: "ota_0",
      },
      expected_postcondition: {
        hostname_sha256: hostnameDigest,
        app_elf_sha256: normalApp,
        running_partition: "factory",
      },
    });
    await mkdir(rollbackSessionRoot, { mode: 0o700 });
    await chmod(rollbackSessionRoot, 0o700);
    const rollbackOutcome = await child(processPort, deviceSessionProgram, [
      "reboot-live", "--port", options.port,
      "--intent-input", rollbackIntentPath,
      "--private-root", rollbackSessionRoot,
      "--projection-output", rollbackProjectionPath,
      "--timeout-seconds", String(options.captureTimeoutSeconds),
    ], "rollback reboot");
    if (rollbackOutcome.timedOut) throw failure("timeout", "rollback reboot timed out");
    const rollbackSession = await readSession(rollbackProjectionPath);
    if (rollbackOutcome.exitCode !== 0) throw failure("rollback_not_observed", "rollback reboot child failed");
    const finalInfo = object(await fetchJsonFromSameOrigin(
      origin,
      "/api/system/info",
      path.join(privateRoot, "final-system-info.private.json"),
    ), "final system info");
    validateRuntimeIdentity(finalInfo, source, reference, normalApp);
    if (
      requiredString(finalInfo, "runningPartition", "final system info") !== "factory"
      || requiredOrdinal(finalInfo, "bootOrdinal", "final system info")
        !== requiredOrdinal(probeInfo, "bootOrdinal", "probe system info") + 1
      || requiredString(finalInfo, "bootSession", "final system info")
        === requiredString(probeInfo, "bootSession", "probe system info")
    ) {
      throw failure("rollback_not_observed", "native rollback did not restore the normal factory build");
    }
    const rollbackSerial = await readFile(path.join(rollbackSessionRoot, "serial.private.bin"), "utf8");
    if (!hasPassiveSafeState(rollbackSerial)) throw failure("rollback_not_observed", "rollback boot safe state is missing");
    normalRestored = true;
    if (!await privateModesValid(privateRoot)) throw failure("evidence_invalid", "private artifact modes are invalid");
    const evidence: SdkconfigRollbackEvidence = {
      schema_version: "bitaxe-sdkconfig-rollback-evidence-v1",
      board: 205,
      source_commit: source,
      reference_commit: reference,
      package_manifest_sha256: manifestDigest,
      rollback_probe_image_sha256: probe.ota_image_sha256,
      rollback_probe_metadata_sha256: probeMetadataDigest,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-sdkconfig-rollback-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: manifestDigest,
          probe_image: probe.ota_image_sha256,
          probe_metadata: probeMetadataDigest,
          sdkconfig: sha256(sdkconfig),
          timeout: options.captureTimeoutSeconds,
        })),
      },
      detector_admitted: true,
      rollback: {
        sdkconfig_sha256: sha256(sdkconfig),
        rollback_enabled: true,
        anti_rollback_disabled: true,
        rollback_probe_isolated: true,
        interrupted_upload_attempt_count: 1,
        interrupted_upload_prefix_bytes: interruptedPrefixBytes,
        interruption_protocol_abort_observed: true,
        baseline_boot_session_unchanged: true,
        baseline_boot_ordinal_unchanged: true,
        baseline_build_unchanged: true,
        probe_pending_validation_observed: true,
        probe_running_partition_ota_0: true,
        rollback_running_partition_factory: true,
        final_normal_build_restored: true,
      },
      probe_boot_session: probeSession as SdkconfigRollbackEvidence["probe_boot_session"],
      rollback_session: rollbackSession as SdkconfigRollbackEvidence["rollback_session"],
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      normal_package_restored: true,
      recovery_flash_used: false,
      private_modes_valid: true,
      redaction_status: "passed",
    };
    const privateCandidate = path.join(privateRoot, "final-evidence.private.json");
    await privateJson(privateCandidate, evidence);
    await verifySemanticEvidenceRedaction(privateRoot);
    const validation = await child(processPort, validatorProgram, [privateCandidate], "SDK config rollback validator");
    if (validation.timedOut) throw failure("timeout", "SDK config rollback validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "SDK config rollback validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof SdkconfigRollbackEvidenceError
      ? error
      : failure("evidence_invalid", "SDK config rollback orchestration evidence is invalid");
    if (!deviceEffectStarted || normalRestored) throw primary;
    throw primary.withRecovery(await recover(processPort, flashProgram, options, manifestPath, credentialsPath));
  }
}
