import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type SettingsPatchEvidence,
} from "./contracts.generated.js";
import {
  factoryImageDigest,
  flashChildFailureFacts,
  flashEffectEnvironment,
  inspectFlashEffect,
} from "./flash-child-diagnostics.js";
import { fetchJsonFromSameOrigin, sendSameOriginRequest, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type SettingsPatchEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<AutomationCategory, "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed">;
type SettingsPair = { readonly hostname: string; readonly rotation: number };
type RecoveryFacts = {
  readonly restoration_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};

const noRecovery: RecoveryFacts = {
  restoration_complete: false,
  recovery_flash_used: false,
  secondary_recovery_failure: false,
};

export class SettingsPatchEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "SettingsPatchEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): SettingsPatchEvidenceError {
    return new SettingsPatchEvidenceError(this.category, this.message, { ...this.publicValue, ...recovery });
  }
}

function failure(category: FailureCategory, message: string, facts: Readonly<Record<string, unknown>> = {}) {
  return new SettingsPatchEvidenceError(category, message, { stage: "settings_patch_capture", ...facts, ...noRecovery });
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
    throw failure("evidence_invalid", `${context} string field is invalid`);
  }
  return candidate;
}

function requiredRotation(value: JsonObject, context: string): number {
  const candidate = value["rotation"];
  if (typeof candidate !== "number" || ![0, 90, 180, 270].includes(candidate)) {
    throw failure("evidence_invalid", `${context} rotation field is invalid`);
  }
  return candidate;
}

function validateIdentity(value: JsonObject, manifest: JsonObject): void {
  for (const [wire, source] of [
    ["sourceCommit", "source_commit"],
    ["referenceCommit", "reference_commit"],
    ["appElfSha256", "app_elf_sha256"],
  ] as const) {
    if (requiredString(value, wire, "system info") !== requiredString(manifest, source, "package manifest")) {
      throw failure("evidence_invalid", "system info does not match the exact package");
    }
  }
}

function pair(value: JsonObject, manifest: JsonObject): SettingsPair {
  validateIdentity(value, manifest);
  return {
    hostname: requiredString(value, "hostname", "system info"),
    rotation: requiredRotation(value, "system info"),
  };
}

function samePair(left: SettingsPair, right: SettingsPair): boolean {
  return left.hostname === right.hostname && left.rotation === right.rotation;
}

function alternatePair(original: SettingsPair): SettingsPair {
  const rotations = [0, 90, 180, 270] as const;
  const index = rotations.indexOf(original.rotation as (typeof rotations)[number]);
  const rotation = rotations[(index + 1) % rotations.length];
  if (rotation === undefined) throw failure("evidence_invalid", "alternate rotation is unavailable");
  return {
    hostname: original.hostname === "bitaxe-patch-205" ? "bitaxe-patch-alt" : "bitaxe-patch-205",
    rotation,
  };
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof SettingsPatchEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
}

async function privateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
}

async function child(processPort: ProcessPort, program: string, args: readonly string[], context: string): Promise<ProcessOutcome> {
  try {
    return await processPort.run(internalCommandSpec(program, [...args], (value) => value));
  } catch {
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function readPair(origin: URL, output: string, manifest: JsonObject): Promise<SettingsPair> {
  return pair(object(await fetchJsonFromSameOrigin(origin, "/api/system/info", output), "system info"), manifest);
}

async function restore(
  origin: URL,
  privateRoot: string,
  original: SettingsPair,
  manifest: JsonObject,
  prefix: string,
): Promise<boolean> {
  await sendSameOriginRequest(origin, "/api/system", "PATCH", path.join(privateRoot, `${prefix}-restore.private.txt`), original);
  return samePair(original, await readPair(origin, path.join(privateRoot, `${prefix}-restored.private.json`), manifest));
}

async function recover(
  processPort: ProcessPort,
  flashProgram: string,
  origin: URL,
  privateRoot: string,
  original: SettingsPair,
  manifest: JsonObject,
  options: SettingsPatchEvidenceOptions,
  manifestPath: string,
  credentialsPath: string,
): Promise<RecoveryFacts> {
  try {
    if (await restore(origin, privateRoot, original, manifest, "recovery")) {
      return { ...noRecovery, restoration_complete: true };
    }
  } catch {
    // The exact-package flash below is the bounded fallback.
  }
  try {
    const outcome = await processPort.run(flashCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest: manifestPath,
      wifiCredentials: credentialsPath,
    }));
    const failed = outcome.timedOut || outcome.exitCode !== 0;
    return { restoration_complete: false, recovery_flash_used: true, secondary_recovery_failure: failed };
  } catch {
    return { restoration_complete: false, recovery_flash_used: true, secondary_recovery_failure: true };
  }
}

export async function captureSettingsPatchEvidence(
  workspaceRoot: string,
  options: SettingsPatchEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
): Promise<SettingsPatchEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await access(manifestPath);
  await access(credentialsPath);
  await createPrivateRoot(privateRoot);
  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  requiredString(manifest, "source_commit", "package manifest");
  requiredString(manifest, "reference_commit", "package manifest");
  requiredString(manifest, "app_elf_sha256", "package manifest");
  let factoryDigest: string;
  try {
    factoryDigest = factoryImageDigest(manifest);
  } catch {
    throw failure("evidence_invalid", "package manifest factory image is invalid");
  }

  const effectPath = path.join(privateRoot, "flash-effect.private.json");
  const expectedEffectIdentity = { packageIdentityDigest: sha256(manifestDocument), factoryImageDigest: factoryDigest };
  const baseSpec = flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: manifestPath,
    wifiCredentials: credentialsPath,
    captureTimeoutSeconds: options.captureTimeoutSeconds,
    evidenceMode: "dual",
    evidenceDir: privateRoot,
  });
  const spec = internalCommandSpec(
    baseSpec.program,
    [...baseSpec.args],
    baseSpec.result,
    flashEffectEnvironment(effectPath, expectedEffectIdentity),
  );
  let initial: ProcessOutcome;
  try {
    initial = await processPort.run(spec);
  } catch {
    const effect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
    throw failure("process_failed", "exact-package flash-monitor launch failed", flashChildFailureFacts(undefined, effect));
  }
  const effect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
  const effectFacts = flashChildFailureFacts(initial, effect);
  if (initial.timedOut) throw failure("timeout", "exact-package flash-monitor timed out", effectFacts);
  if (initial.exitCode !== 0) throw failure("process_failed", "exact-package flash-monitor failed", effectFacts);
  if (effect.flash_effect_result_status !== "valid" || effect.flash_effect_status !== "completed") {
    throw failure("evidence_invalid", "exact-package flash effect result is invalid", effectFacts);
  }

  const monitor = await readFile(path.join(privateRoot, "flash-monitor.classifier-input.log"), "utf8");
  if (!hasPassiveSafeState(monitor)) throw failure("evidence_invalid", "boot lacks passive safe-state evidence");
  let origin: URL;
  try {
    origin = uniqueRuntimeOrigin(monitor);
  } catch {
    throw failure("evidence_invalid", "runtime origin admission is invalid");
  }
  const original = await readPair(origin, path.join(privateRoot, "original.private.json"), manifest);
  const candidate = alternatePair(original);
  let mutated = false;
  try {
    await sendSameOriginRequest(origin, "/api/system", "PATCH", path.join(privateRoot, "mutation.private.txt"), candidate);
    mutated = true;
    const immediate = await readPair(origin, path.join(privateRoot, "immediate.private.json"), manifest);
    if (!samePair(candidate, immediate)) throw failure("hardware_blocked", "combined mutation readback mismatch");
    if (!await restore(origin, privateRoot, original, manifest, "final")) {
      throw failure("hardware_blocked", "combined restoration readback mismatch");
    }
    mutated = false;
    const evidence: SettingsPatchEvidence = {
      schema_version: "bitaxe-settings-patch-evidence-v1",
      board: 205,
      source_commit: requiredString(manifest, "source_commit", "package manifest"),
      reference_commit: requiredString(manifest, "reference_commit", "package manifest"),
      package_manifest_sha256: sha256(manifestDocument),
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-settings-patch-evidence",
        request_sha256: sha256(JSON.stringify({ manifest: sha256(manifestDocument), fields: ["hostname", "rotation"], timeout: options.captureTimeoutSeconds })),
      },
      detector_admitted: true,
      boot_observed: true,
      same_origin_observed: true,
      settings_patch: {
        hostname_baseline_sha256: sha256(original.hostname),
        hostname_candidate_sha256: sha256(candidate.hostname),
        rotation_baseline_sha256: sha256(String(original.rotation)),
        rotation_candidate_sha256: sha256(String(candidate.rotation)),
        mutation_request_field_count: 2,
        mutation_request_atomic: true,
        immediate_combined_readback: true,
        restoration_request_field_count: 2,
        restoration_request_atomic: true,
        restoration_complete: true,
      },
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      redaction_status: "passed",
    };
    const privateCandidate = path.join(privateRoot, "final-evidence.private.json");
    await privateJson(privateCandidate, evidence);
    const validation = await child(processPort, validatorProgram, [privateCandidate], "settings PATCH evidence validator");
    if (validation.timedOut) throw failure("timeout", "settings PATCH evidence validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "settings PATCH evidence validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof SettingsPatchEvidenceError
      ? error
      : failure("evidence_invalid", "settings PATCH orchestration evidence is invalid");
    if (!mutated) throw primary;
    throw primary.withRecovery(await recover(
      processPort,
      flashProgram,
      origin,
      privateRoot,
      original,
      manifest,
      options,
      manifestPath,
      credentialsPath,
    ));
  }
}
