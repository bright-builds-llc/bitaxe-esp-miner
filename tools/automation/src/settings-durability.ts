import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { flashCommand, flashMonitorCommand, internalCommandSpec, type AutomationCategory } from "./contracts.generated.js";
import { isDeviceSessionProjectionFailure, readClosedDeviceSession } from "./device-session-projection.js";
import { fetchJsonFromSameOrigin, sendSameOriginRequest, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessPort, ProcessOutcome } from "./process.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type SettingsDurabilityOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type JsonObject = Readonly<Record<string, unknown>>;
type SettingsFailureCategory = Extract<AutomationCategory, "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed">;

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

export class SettingsDurabilityError extends Error {
  public constructor(
    public readonly category: SettingsFailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "SettingsDurabilityError";
  }

  public withRecovery(recovery: RecoveryFacts): SettingsDurabilityError {
    return new SettingsDurabilityError(this.category, this.message, { ...this.publicValue, ...recovery });
  }
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function jsonObject(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error(`${context} must be an object`);
  return value as JsonObject;
}

function requiredString(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") throw new Error(`${context} ${field} must be a non-empty string`);
  return candidate;
}

function requiredOrdinal(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 1) {
    throw new Error(`${context} ${field} must be a positive integer`);
  }
  return candidate;
}

async function requireAbsentPrivateRoot(privateRoot: string): Promise<void> {
  try {
    await stat(privateRoot);
    throw new Error("private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof Error && error.message === "private attempt root must be absent before launch") throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(privateRoot, { mode: 0o700, recursive: true });
  await chmod(privateRoot, 0o700);
}

async function writePrivateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
}

async function classifyBaseline(processPort: ProcessPort, classifierProgram: string, trace: string): Promise<JsonObject> {
  const args = ["verify-settings-durability", "--trace", trace, "--mode", "baseline"];
  const outcome = await runChild(processPort, classifierProgram, args, "baseline classification");
  if (outcome.timedOut) throw failure("timeout", "baseline classification timed out");
  if (outcome.exitCode !== 0) throw failure("process_failed", "baseline settings classification failed");
  try {
    const value = jsonObject(JSON.parse(outcome.stdout), "baseline classification");
    if (value["status"] !== "passed") throw new Error("baseline settings evidence did not pass");
    return value;
  } catch {
    throw failure("evidence_invalid", "baseline settings evidence is invalid");
  }
}

async function hostname(origin: URL, output: string): Promise<string> {
  const value = jsonObject(await fetchJsonFromSameOrigin(origin, "/api/system/info", output), "system info");
  return requiredString(value, "hostname", "system info");
}

function failure(
  category: SettingsFailureCategory,
  message: string,
  facts: Readonly<Record<string, unknown>> = {},
): SettingsDurabilityError {
  return new SettingsDurabilityError(category, message, {
    stage: "restart_session",
    ...facts,
    ...noRecovery,
  });
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

async function recoverHostname(
  processPort: ProcessPort,
  flashProgram: string,
  origin: URL,
  privateRoot: string,
  originalHostname: string,
  options: SettingsDurabilityOptions,
  manifestPath: string,
  credentialsPath: string,
): Promise<RecoveryFacts> {
  try {
    await sendSameOriginRequest(
      origin,
      "/api/system",
      "PATCH",
      path.join(privateRoot, "recovery-restore.private.txt"),
      { hostname: originalHostname },
    );
    const restored = await hostname(origin, path.join(privateRoot, "recovery-readback.private.json"));
    if (restored === originalHostname) return { ...noRecovery, restoration_complete: true };
  } catch {
    // The exact-package flash below is the bounded safe-state fallback.
  }
  try {
    const recovery = await processPort.run(flashCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest: manifestPath,
      wifiCredentials: credentialsPath,
    }));
    return {
      restoration_complete: false,
      recovery_flash_used: true,
      secondary_recovery_failure: recovery.timedOut || recovery.exitCode !== 0,
    };
  } catch {
    return {
      restoration_complete: false,
      recovery_flash_used: true,
      secondary_recovery_failure: true,
    };
  }
}

export async function captureSettingsDurability(
  workspaceRoot: string,
  options: SettingsDurabilityOptions,
  processPort: ProcessPort,
  flashProgram: string,
  classifierProgram: string,
  deviceSessionProgram: string,
): Promise<unknown> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await access(manifestPath);
  await access(credentialsPath);
  await requireAbsentPrivateRoot(privateRoot);
  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = jsonObject(JSON.parse(manifestDocument), "package manifest");
  const sourceCommit = requiredString(manifest, "source_commit", "package manifest");
  const referenceCommit = requiredString(manifest, "reference_commit", "package manifest");
  const appElfSha256 = requiredString(manifest, "app_elf_sha256", "package manifest");
  const manifestDigest = sha256(manifestDocument);
  const initialRoot = path.join(privateRoot, "initial");
  await mkdir(initialRoot, { mode: 0o700 });
  const initial = await processPort.run(flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: manifestPath,
    wifiCredentials: credentialsPath,
    captureTimeoutSeconds: options.captureTimeoutSeconds,
    evidenceMode: "dual",
    evidenceDir: initialRoot,
  }));
  if (initial.timedOut) throw failure("timeout", "exact-package flash-monitor timed out");
  if (initial.exitCode !== 0) throw failure("process_failed", "exact-package flash-monitor failed");
  const initialTrace = path.join(initialRoot, "flash-monitor.classifier-input.log");
  const initialDocument = await readFile(initialTrace, "utf8");
  if (!hasPassiveSafeState(initialDocument)) throw failure("evidence_invalid", "initial boot lacks safe-state evidence");
  const baseline = await classifyBaseline(processPort, classifierProgram, initialTrace);
  const session = requiredString(baseline, "session", "baseline classification");
  const ordinal = requiredOrdinal(baseline, "boot_ordinal", "baseline classification");
  const origin = uniqueRuntimeOrigin(initialDocument);
  const originalHostname = await hostname(origin, path.join(privateRoot, "original.private.json"));
  const testHostname = originalHostname === "bitaxe-parity-205" ? "bitaxe-parity-alt" : "bitaxe-parity-205";
  let hostnameChanged = false;
  try {
    await sendSameOriginRequest(
      origin,
      "/api/system",
      "PATCH",
      path.join(privateRoot, "patch.private.txt"),
      { hostname: testHostname },
    );
    hostnameChanged = true;
    if (await hostname(origin, path.join(privateRoot, "immediate.private.json")) !== testHostname) {
      throw failure("hardware_blocked", "immediate hostname readback mismatch");
    }
    const intentPath = path.join(privateRoot, "device-session-intent.private.json");
    const sessionRoot = path.join(privateRoot, "device-session");
    const sessionProjectionPath = path.join(privateRoot, "device-session-projection.private.json");
    await mkdir(sessionRoot, { mode: 0o700 });
    await chmod(sessionRoot, 0o700);
    await writePrivateJson(intentPath, {
      schema_version: "esp-device-session-reboot-intent-v1",
      board_category: "205",
      trusted_origin: origin.origin,
      baseline: {
        boot_session: session,
        boot_ordinal: ordinal,
        source_commit: sourceCommit,
        reference_commit: referenceCommit,
        app_elf_sha256: appElfSha256,
      },
      expected_postcondition: { hostname_sha256: sha256(testHostname) },
    });
    const sessionOutcome = await runChild(processPort, deviceSessionProgram, [
      "reboot-live",
      "--port", options.port,
      "--intent-input", intentPath,
      "--private-root", sessionRoot,
      "--projection-output", sessionProjectionPath,
      "--timeout-seconds", String(options.captureTimeoutSeconds),
    ], "device-session");
    if (sessionOutcome.timedOut) throw failure("timeout", "device-session timed out");
    let sessionProjection: JsonObject;
    try {
      sessionProjection = await readClosedDeviceSession(sessionProjectionPath);
    } catch (error) {
      if (isDeviceSessionProjectionFailure(error)) {
        throw failure(error.category, error.message, error.facts);
      }
      throw failure("evidence_invalid", "device-session projection is invalid");
    }
    if (sessionOutcome.exitCode !== 0) {
      throw failure("hardware_blocked", "device-session child failed after a ready projection");
    }
    await sendSameOriginRequest(
      origin,
      "/api/system",
      "PATCH",
      path.join(privateRoot, "restore.private.txt"),
      { hostname: originalHostname },
    );
    if (await hostname(origin, path.join(privateRoot, "restored.private.json")) !== originalHostname) {
      throw failure("hardware_blocked", "restored hostname readback mismatch");
    }
    hostnameChanged = false;
    const evidence = {
      schema_version: "bitaxe-settings-durability-evidence-v2",
      board: 205,
      source_commit: sourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: manifestDigest,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "verify-settings-durability",
        request_sha256: sha256(JSON.stringify({ manifest: manifestDigest, timeout: options.captureTimeoutSeconds })),
      },
      restart_session: sessionProjection,
      boot_observed: true,
      hostname_patch_readback: true,
      normal_restart_observed: true,
      post_restart_persistence: true,
      restoration_complete: true,
      mining_state: "disabled",
      hardware_control_state: "disabled",
      redaction_status: "passed",
    } as const;
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof SettingsDurabilityError
      ? error
      : failure("process_failed", "settings durability orchestration failed");
    if (!hostnameChanged) throw primary;
    const recovery = await recoverHostname(
      processPort,
      flashProgram,
      origin,
      privateRoot,
      originalHostname,
      options,
      manifestPath,
      credentialsPath,
    );
    throw primary.withRecovery(recovery);
  }
}
