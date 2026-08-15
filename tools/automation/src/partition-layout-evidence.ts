import { createHash } from "node:crypto";
import { access, chmod, lstat, mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type PartitionLayoutEvidence,
} from "./contracts.generated.js";
import { readClosedDeviceSession, isDeviceSessionProjectionFailure } from "./device-session-projection.js";
import {
  factoryImageDigest,
  flashChildFailureFacts,
  flashEffectEnvironment,
  inspectFlashEffect,
} from "./flash-child-diagnostics.js";
import { fetchJsonFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import { canonicalPartitionRows, requiredPartitionCount } from "./partition-table.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { verifySemanticEvidenceRedaction } from "./redaction.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type PartitionLayoutEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<
  AutomationCategory,
  "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed"
>;

type Artifact = {
  readonly path: string;
  readonly sha256: string;
};

export class PartitionLayoutEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "PartitionLayoutEvidenceError";
  }

  public withCompletedFlash(): PartitionLayoutEvidenceError {
    return new PartitionLayoutEvidenceError(this.category, this.message, {
      ...this.publicValue,
      flash_effect_completed: true,
    });
  }
}

function failure(
  category: FailureCategory,
  message: string,
  facts: Readonly<Record<string, unknown>> = {},
): PartitionLayoutEvidenceError {
  return new PartitionLayoutEvidenceError(category, message, {
    stage: "partition_layout_capture",
    flash_effect_completed: false,
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
  const maybeValue = value[field];
  if (typeof maybeValue !== "string" || maybeValue === "") {
    throw failure("evidence_invalid", `${context} field is invalid`);
  }
  return maybeValue;
}

function requiredOrdinal(value: JsonObject, field: string, context: string): number {
  const maybeValue = value[field];
  if (typeof maybeValue !== "number" || !Number.isSafeInteger(maybeValue) || maybeValue < 1) {
    throw failure("evidence_invalid", `${context} ordinal is invalid`);
  }
  return maybeValue;
}

function monitorBootSession(document: string): string {
  const sessions = new Set<string>();
  for (const match of document.matchAll(/\bruntime_boot_identity session=([0-9a-f]{32})\b/gu)) {
    const maybeSession = match[1];
    if (maybeSession !== undefined) sessions.add(maybeSession);
  }
  const [maybeSession] = sessions;
  if (sessions.size !== 1 || maybeSession === undefined) {
    throw failure("hardware_blocked", "monitor capture lacks one stable boot session");
  }
  return maybeSession;
}

function artifact(manifest: JsonObject, kind: string): Artifact {
  const artifacts = manifest["artifacts"];
  if (!Array.isArray(artifacts)) throw failure("evidence_invalid", "package artifacts are invalid");
  const matches = artifacts
    .map((candidate) => object(candidate, "package artifact"))
    .filter((candidate) => candidate["kind"] === kind);
  if (matches.length !== 1) throw failure("evidence_invalid", `package ${kind} artifact is invalid`);
  return {
    path: requiredString(matches[0] ?? {}, "path", "package artifact"),
    sha256: requiredString(matches[0] ?? {}, "sha256", "package artifact"),
  };
}

function resolveArtifact(manifestPath: string, artifactPath: string): string {
  return path.isAbsolute(artifactPath)
    ? artifactPath
    : path.join(path.dirname(manifestPath), artifactPath);
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof PartitionLayoutEvidenceError) throw error;
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

async function runChild(
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

function validateRuntimeIdentity(runtime: JsonObject, manifest: JsonObject): void {
  for (const [wireField, manifestField] of [
    ["sourceCommit", "source_commit"],
    ["referenceCommit", "reference_commit"],
    ["appElfSha256", "app_elf_sha256"],
  ] as const) {
    if (
      requiredString(runtime, wireField, "system info")
      !== requiredString(manifest, manifestField, "package manifest")
    ) {
      throw failure("evidence_invalid", "runtime identity does not match the exact package");
    }
  }
}

async function privateModesValid(root: string): Promise<boolean> {
  const metadata = await lstat(root);
  if (!metadata.isDirectory() || metadata.isSymbolicLink() || (metadata.mode & 0o777) !== 0o700) {
    return false;
  }
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const child = path.join(root, entry.name);
    if (entry.isDirectory()) {
      if (!await privateModesValid(child)) return false;
      continue;
    }
    const childMetadata = await lstat(child);
    if (!childMetadata.isFile() || childMetadata.isSymbolicLink() || (childMetadata.mode & 0o777) !== 0o600) {
      return false;
    }
  }
  return true;
}

export async function capturePartitionLayoutEvidence(
  workspaceRoot: string,
  options: PartitionLayoutEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  deviceSessionProgram: string,
  validatorProgram: string,
): Promise<PartitionLayoutEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await access(manifestPath);
  await access(credentialsPath);
  await createPrivateRoot(privateRoot);

  const manifestDocument = await readFile(manifestPath, "utf8");
  let manifest: JsonObject;
  let otaArtifact: Artifact;
  let partitionArtifact: Artifact;
  let factoryDigest: string;
  try {
    manifest = object(JSON.parse(manifestDocument), "package manifest");
    requiredString(manifest, "source_commit", "package manifest");
    requiredString(manifest, "reference_commit", "package manifest");
    requiredString(manifest, "app_elf_sha256", "package manifest");
    factoryDigest = factoryImageDigest(manifest);
    otaArtifact = artifact(manifest, "firmware_ota_image");
    partitionArtifact = artifact(manifest, "partition_table");
  } catch (error) {
    if (error instanceof PartitionLayoutEvidenceError) throw error;
    throw failure("evidence_invalid", "package manifest identity is invalid");
  }

  const otaImagePath = resolveArtifact(manifestPath, otaArtifact.path);
  const partitionTablePath = resolveArtifact(manifestPath, partitionArtifact.path);
  const otaImage = await readFile(otaImagePath);
  const partitionTable = await readFile(partitionTablePath, "utf8");
  if (
    sha256(otaImage) !== otaArtifact.sha256
    || sha256(partitionTable) !== partitionArtifact.sha256
    || !canonicalPartitionRows(partitionTable)
  ) {
    throw failure("evidence_invalid", "package partition artifacts are invalid");
  }

  const manifestDigest = sha256(manifestDocument);
  const effectPath = path.join(privateRoot, "flash-effect.private.json");
  const expectedEffectIdentity = {
    packageIdentityDigest: manifestDigest,
    factoryImageDigest: factoryDigest,
  };
  const baseSpec = flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: manifestPath,
    wifiCredentials: credentialsPath,
    captureTimeoutSeconds: options.captureTimeoutSeconds,
    evidenceMode: "dual",
    evidenceDir: privateRoot,
  });
  const flashSpec = internalCommandSpec(
    baseSpec.program,
    [...baseSpec.args],
    baseSpec.result,
    flashEffectEnvironment(effectPath, expectedEffectIdentity),
  );
  let flashOutcome: ProcessOutcome;
  try {
    flashOutcome = await processPort.run(flashSpec);
  } catch {
    const effect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
    throw failure("process_failed", "exact-package flash-monitor launch failed", flashChildFailureFacts(undefined, effect));
  }
  const effect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
  const effectFacts = flashChildFailureFacts(flashOutcome, effect);
  if (flashOutcome.timedOut) throw failure("timeout", "exact-package flash-monitor timed out", effectFacts);
  if (flashOutcome.exitCode !== 0) {
    throw failure("hardware_blocked", "exact-package flash-monitor did not reach readiness", effectFacts);
  }
  if (effect.flash_effect_result_status !== "valid" || effect.flash_effect_status !== "completed") {
    throw failure("evidence_invalid", "exact-package flash effect result is invalid", effectFacts);
  }

  try {
    const monitor = await readFile(path.join(privateRoot, "flash-monitor.classifier-input.log"), "utf8");
    if (!hasPassiveSafeState(monitor)) throw failure("hardware_blocked", "factory boot lacks passive safe-state evidence");
    let origin: URL;
    try {
      origin = uniqueRuntimeOrigin(monitor);
    } catch {
      throw failure("hardware_blocked", "runtime origin admission is invalid");
    }
    const baseline = object(
      await fetchJsonFromSameOrigin(origin, "/api/system/info", path.join(privateRoot, "baseline-system-info.private.json")),
      "baseline system info",
    );
    validateRuntimeIdentity(baseline, manifest);
    if (requiredString(baseline, "bootSession", "baseline system info") !== monitorBootSession(monitor)) {
      throw failure("evidence_invalid", "factory API and monitor boot sessions differ");
    }
    if (requiredString(baseline, "runningPartition", "baseline system info") !== "factory") {
      throw failure("hardware_blocked", "factory baseline partition is not active");
    }
    const hostname = requiredString(baseline, "hostname", "baseline system info");
    const intentPath = path.join(privateRoot, "device-session-ota-intent.private.json");
    const sessionRoot = path.join(privateRoot, "device-session");
    const sessionProjectionPath = path.join(privateRoot, "device-session-projection.private.json");
    await privateJson(intentPath, {
      schema_version: "bitaxe-device-transaction-intent-v1",
      goal: {
        transaction_kind: "ota_transition",
        ota: {
          schema_version: "esp-device-session-ota-intent-v1",
          board_category: "205",
          trusted_origin: origin.origin,
          baseline: {
            boot_session: requiredString(baseline, "bootSession", "baseline system info"),
            boot_ordinal: requiredOrdinal(baseline, "bootOrdinal", "baseline system info"),
            source_commit: requiredString(manifest, "source_commit", "package manifest"),
            reference_commit: requiredString(manifest, "reference_commit", "package manifest"),
            app_elf_sha256: requiredString(manifest, "app_elf_sha256", "package manifest"),
            running_partition: "factory",
          },
          expected_postcondition: {
            hostname_sha256: sha256(hostname),
            running_partition: "ota_0",
          },
          ota_image_sha256: otaArtifact.sha256,
        },
      },
    });
    await mkdir(sessionRoot, { mode: 0o700 });
    await chmod(sessionRoot, 0o700);
    const sessionOutcome = await runChild(processPort, deviceSessionProgram, [
      "transact-live", "--port", options.port,
      "--private-root", sessionRoot,
      "--intent-input", intentPath,
      "--ota-image", otaImagePath,
      "--projection-output", sessionProjectionPath,
      "--timeout-seconds", String(options.captureTimeoutSeconds),
    ], "device-session OTA");
    if (sessionOutcome.timedOut) throw failure("timeout", "device-session OTA timed out");
    let session: JsonObject;
    try {
      session = await readClosedDeviceSession(sessionProjectionPath);
    } catch (error) {
      if (isDeviceSessionProjectionFailure(error)) {
        throw failure(error.category, error.message, error.facts);
      }
      throw failure("evidence_invalid", "device-session OTA projection is invalid");
    }
    if (sessionOutcome.exitCode !== 0) {
      throw failure("hardware_blocked", "device-session OTA child failed after a ready projection");
    }
    const post = object(
      await fetchJsonFromSameOrigin(origin, "/api/system/info", path.join(privateRoot, "post-system-info.private.json")),
      "post-OTA system info",
    );
    validateRuntimeIdentity(post, manifest);
    if (requiredString(post, "runningPartition", "post-OTA system info") !== "ota_0") {
      throw failure("evidence_invalid", "post-OTA partition is not ota_0");
    }
    const baselineOrdinal = requiredOrdinal(baseline, "bootOrdinal", "baseline system info");
    if (
      requiredString(post, "bootSession", "post-OTA system info")
        === requiredString(baseline, "bootSession", "baseline system info")
      || requiredOrdinal(post, "bootOrdinal", "post-OTA system info") !== baselineOrdinal + 1
    ) {
      throw failure("evidence_invalid", "post-OTA API boot identity did not advance exactly once");
    }
    const sessionSerial = await readFile(path.join(sessionRoot, "serial.private.bin"), "utf8");
    const bootValidationComplete = /\bruntime_boot_attestation\b[^\r\n]*\bota_boot_validation=complete\b/u.test(sessionSerial);
    if (!bootValidationComplete || !hasPassiveSafeState(sessionSerial)) {
      throw failure("hardware_blocked", "post-OTA boot validation or safe-state evidence is missing");
    }
    const modesValid = await privateModesValid(privateRoot);
    if (!modesValid) throw failure("evidence_invalid", "private artifact modes are invalid");
    const evidence: PartitionLayoutEvidence = {
      schema_version: "bitaxe-partition-layout-evidence-v1",
      board: 205,
      source_commit: requiredString(manifest, "source_commit", "package manifest"),
      reference_commit: requiredString(manifest, "reference_commit", "package manifest"),
      package_manifest_sha256: manifestDigest,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-partition-layout-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: manifestDigest,
          partition_table: partitionArtifact.sha256,
          ota_image: otaArtifact.sha256,
          timeout: options.captureTimeoutSeconds,
        })),
      },
      detector_admitted: true,
      partition_layout: {
        partition_table_sha256: partitionArtifact.sha256,
        ota_image_sha256: otaArtifact.sha256,
        required_partition_count: requiredPartitionCount(),
        canonical_layout_matches: true,
        factory_baseline_observed: true,
        ota_0_recovered: true,
        ota_upload_complete: true,
        ota_boot_validation_complete: true,
      },
      ota_session: session,
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      private_modes_valid: true,
      redaction_status: "passed",
    };
    const privateCandidate = path.join(privateRoot, "final-evidence.private.json");
    await privateJson(privateCandidate, evidence);
    await verifySemanticEvidenceRedaction(privateRoot);
    const validation = await runChild(processPort, validatorProgram, [privateCandidate], "partition layout evidence validator");
    if (validation.timedOut) throw failure("timeout", "partition layout evidence validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "partition layout evidence validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof PartitionLayoutEvidenceError
      ? error
      : failure("evidence_invalid", "partition layout orchestration evidence is invalid");
    throw primary.withCompletedFlash();
  }
}
