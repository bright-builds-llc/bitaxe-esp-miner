import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type ThemeDurabilityEvidence,
} from "./contracts.generated.js";
import { isDeviceSessionProjectionFailure, readClosedDeviceSession, type JsonObject } from "./device-session-projection.js";
import {
  factoryImageDigest,
  flashChildFailureFacts,
  flashEffectEnvironment,
  inspectFlashEffect,
} from "./flash-child-diagnostics.js";
import { fetchJsonFromSameOrigin, sendSameOriginRequest } from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type ThemeDurabilityOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type ThemeFailureCategory = Extract<AutomationCategory, "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed">;
type Theme = { readonly colorScheme: string; readonly accentColors: JsonObject };
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

export class ThemeDurabilityError extends Error {
  public constructor(
    public readonly category: ThemeFailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "ThemeDurabilityError";
  }

  public withRecovery(recovery: RecoveryFacts): ThemeDurabilityError {
    return new ThemeDurabilityError(this.category, this.message, { ...this.publicValue, ...recovery });
  }
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function failure(
  category: ThemeFailureCategory,
  message: string,
  facts: Readonly<Record<string, unknown>> = {},
): ThemeDurabilityError {
  return new ThemeDurabilityError(category, message, { stage: "theme_durability", ...facts, ...noRecovery });
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
    throw failure("evidence_invalid", `${context} ${field} is invalid`);
  }
  return candidate;
}

function requiredOrdinal(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 1) {
    throw failure("evidence_invalid", `${context} ${field} is invalid`);
  }
  return candidate;
}

function canonicalJson(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (typeof value === "object" && value !== null) {
    const entries = Object.entries(value).sort(([left], [right]) => left.localeCompare(right));
    return `{${entries.map(([key, child]) => `${JSON.stringify(key)}:${canonicalJson(child)}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function parseTheme(value: unknown): Theme {
  const root = object(value, "theme response");
  const keys = Object.keys(root);
  if (keys.length !== 2 || !keys.includes("colorScheme") || !keys.includes("accentColors")) {
    throw failure("evidence_invalid", "theme response fields are invalid");
  }
  return {
    colorScheme: requiredString(root, "colorScheme", "theme response"),
    accentColors: object(root["accentColors"], "theme accent colors"),
  };
}

function sameTheme(left: Theme, right: Theme): boolean {
  return canonicalJson(left) === canonicalJson(right);
}

function alternateTheme(original: Theme): Theme {
  return {
    colorScheme: original.colorScheme === "bitaxe-parity-light" ? "bitaxe-parity-dark" : "bitaxe-parity-light",
    accentColors: { "--primary-color": "#0b5fff", "--primary-color-text": "#ffffff" },
  };
}

async function requireAbsentPrivateRoot(privateRoot: string): Promise<void> {
  try {
    await stat(privateRoot);
    throw failure("process_failed", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof ThemeDurabilityError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(privateRoot, { mode: 0o700, recursive: true });
  await chmod(privateRoot, 0o700);
}

async function writePrivateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, { encoding: "utf8", flag: "wx", mode: 0o600 });
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

async function baseline(processPort: ProcessPort, classifierProgram: string, trace: string): Promise<JsonObject> {
  const outcome = await runChild(processPort, classifierProgram, [
    "verify-settings-durability", "--trace", trace, "--mode", "terminal-baseline",
  ], "baseline classification");
  if (outcome.timedOut) throw failure("timeout", "baseline classification timed out");
  if (outcome.exitCode !== 0) throw failure("process_failed", "baseline classification failed");
  try {
    const value = object(JSON.parse(outcome.stdout), "baseline classification");
    if (value["status"] !== "passed") throw new Error("baseline failed");
    return value;
  } catch (error) {
    if (error instanceof ThemeDurabilityError) throw error;
    throw failure("evidence_invalid", "baseline classification is invalid");
  }
}

async function getTheme(origin: URL, output: string): Promise<Theme> {
  return parseTheme(await fetchJsonFromSameOrigin(origin, "/api/theme", output));
}

async function getHostname(origin: URL, output: string): Promise<string> {
  const value = object(await fetchJsonFromSameOrigin(origin, "/api/system/info", output), "system info");
  return requiredString(value, "hostname", "system info");
}

async function restoreTheme(origin: URL, privateRoot: string, original: Theme): Promise<boolean> {
  await sendSameOriginRequest(origin, "/api/theme", "POST", path.join(privateRoot, "restore.private.txt"), original);
  return sameTheme(original, await getTheme(origin, path.join(privateRoot, "restored.private.json")));
}

async function recoverTheme(
  processPort: ProcessPort,
  flashProgram: string,
  origin: URL,
  privateRoot: string,
  original: Theme,
  options: ThemeDurabilityOptions,
  manifestPath: string,
  credentialsPath: string,
): Promise<RecoveryFacts> {
  try {
    if (await restoreTheme(origin, privateRoot, original)) return { ...noRecovery, restoration_complete: true };
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
    return { restoration_complete: false, recovery_flash_used: true, secondary_recovery_failure: true };
  }
}

export async function captureThemeDurability(
  workspaceRoot: string,
  options: ThemeDurabilityOptions,
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
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const sourceCommit = requiredString(manifest, "source_commit", "package manifest");
  const referenceCommit = requiredString(manifest, "reference_commit", "package manifest");
  const appElfSha256 = requiredString(manifest, "app_elf_sha256", "package manifest");
  const manifestDigest = sha256(manifestDocument);
  let factoryDigest: string;
  try {
    factoryDigest = factoryImageDigest(manifest);
  } catch {
    throw failure("evidence_invalid", "package manifest factory image is invalid");
  }
  const initialRoot = path.join(privateRoot, "initial");
  await mkdir(initialRoot, { mode: 0o700 });
  const effectPath = path.join(initialRoot, "flash-effect.private.json");
  const expectedEffectIdentity = {
    packageIdentityDigest: manifestDigest,
    factoryImageDigest: factoryDigest,
  };
  const initialBaseSpec = flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: manifestPath,
    wifiCredentials: credentialsPath,
    captureTimeoutSeconds: options.captureTimeoutSeconds,
    evidenceMode: "dual",
    evidenceDir: initialRoot,
  });
  const initialSpec = internalCommandSpec(
    initialBaseSpec.program,
    [...initialBaseSpec.args],
    initialBaseSpec.result,
    flashEffectEnvironment(effectPath, expectedEffectIdentity),
  );
  let initial: ProcessOutcome;
  try {
    initial = await processPort.run(initialSpec);
  } catch {
    const effect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
    throw failure("process_failed", "exact-package flash-monitor launch failed", flashChildFailureFacts(undefined, effect));
  }
  const initialEffect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
  const initialFacts = flashChildFailureFacts(initial, initialEffect);
  if (initial.timedOut) throw failure("timeout", "exact-package flash-monitor timed out", initialFacts);
  if (initial.exitCode !== 0) throw failure("process_failed", "exact-package flash-monitor failed", initialFacts);
  if (initialEffect.flash_effect_result_status !== "valid" || initialEffect.flash_effect_status !== "completed") {
    throw failure("evidence_invalid", "exact-package flash effect result is invalid", initialFacts);
  }
  const tracePath = path.join(initialRoot, "flash-monitor.classifier-input.log");
  const baselineValue = await baseline(processPort, classifierProgram, tracePath);
  const session = requiredString(baselineValue, "session", "baseline classification");
  const ordinal = requiredOrdinal(baselineValue, "boot_ordinal", "baseline classification");
  const origin = new URL(requiredString(baselineValue, "device_url", "baseline classification"));
  const hostname = await getHostname(origin, path.join(privateRoot, "system-info.private.json"));
  const original = await getTheme(origin, path.join(privateRoot, "original.private.json"));
  const alternate = alternateTheme(original);
  let themeChanged = false;

  try {
    await sendSameOriginRequest(origin, "/api/theme", "POST", path.join(privateRoot, "patch.private.txt"), alternate);
    themeChanged = true;
    if (!sameTheme(alternate, await getTheme(origin, path.join(privateRoot, "immediate.private.json")))) {
      throw failure("hardware_blocked", "immediate theme readback mismatch");
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
      expected_postcondition: { hostname_sha256: sha256(hostname) },
    });
    const sessionOutcome = await runChild(processPort, deviceSessionProgram, [
      "reboot-live", "--port", options.port, "--intent-input", intentPath,
      "--private-root", sessionRoot, "--projection-output", sessionProjectionPath,
      "--timeout-seconds", String(options.captureTimeoutSeconds),
    ], "device-session");
    if (sessionOutcome.timedOut) throw failure("timeout", "device-session timed out");
    let sessionProjection: JsonObject;
    try {
      sessionProjection = await readClosedDeviceSession(sessionProjectionPath);
    } catch (error) {
      if (isDeviceSessionProjectionFailure(error)) throw failure(error.category, error.message, error.facts);
      throw failure("evidence_invalid", "device-session projection is invalid");
    }
    if (sessionOutcome.exitCode !== 0) {
      throw failure("hardware_blocked", "device-session child failed after a ready projection");
    }
    if (!sameTheme(alternate, await getTheme(origin, path.join(privateRoot, "post-restart.private.json")))) {
      throw failure("hardware_blocked", "post-restart theme readback mismatch");
    }
    if (!await restoreTheme(origin, privateRoot, original)) {
      throw failure("hardware_blocked", "restored theme readback mismatch");
    }
    themeChanged = false;
    const evidence: ThemeDurabilityEvidence = {
      schema_version: "bitaxe-theme-durability-evidence-v1",
      board: 205,
      source_commit: sourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: manifestDigest,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "verify-theme-durability",
        request_sha256: sha256(JSON.stringify({ manifest: manifestDigest, timeout: options.captureTimeoutSeconds })),
      },
      restart_session: sessionProjection,
      theme_get_observed: true,
      theme_post_readback: true,
      normal_restart_observed: true,
      post_restart_persistence: true,
      restoration_complete: true,
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      redaction_status: "passed",
    };
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof ThemeDurabilityError
      ? error
      : failure("process_failed", "theme durability orchestration failed");
    if (!themeChanged) throw primary;
    const recovery = await recoverTheme(
      processPort, flashProgram, origin, privateRoot, original, options, manifestPath, credentialsPath,
    );
    throw primary.withRecovery(recovery);
  }
}
