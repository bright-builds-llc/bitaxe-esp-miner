import { createHash } from "node:crypto";
import { chmod, mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type NetworkReconnectEvidence,
} from "./contracts.generated.js";
import { fetchJsonFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessPort } from "./process.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type NetworkReconnectEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type FailureCategory = Extract<AutomationCategory,
  | "evidence_invalid"
  | "hardware_blocked"
  | "process_failed"
  | "reconnect_not_observed"
  | "reconnect_timing_invalid"
  | "service_recovery_failed"
  | "timeout">;
type RecoveryFacts = {
  readonly recovery_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};
type JsonObject = Readonly<Record<string, unknown>>;

const noRecovery: RecoveryFacts = {
  recovery_complete: false,
  recovery_flash_used: false,
  secondary_recovery_failure: false,
};

export class NetworkReconnectEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "NetworkReconnectEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): NetworkReconnectEvidenceError {
    return new NetworkReconnectEvidenceError(this.category, this.message, {
      ...this.publicValue,
      ...recovery,
    });
  }
}

function failure(category: FailureCategory, message: string): NetworkReconnectEvidenceError {
  return new NetworkReconnectEvidenceError(category, message, {
    stage: "network_reconnect_capture",
    ...noRecovery,
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

function string(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") {
    throw failure("evidence_invalid", `${context} field is invalid`);
  }
  return candidate;
}

function marker(document: string, expression: RegExp, category: FailureCategory): RegExpExecArray {
  const matches = [...document.matchAll(expression)];
  if (matches.length !== 1 || matches[0] === undefined) {
    throw failure(category, "required reconnect lifecycle marker was not observed exactly once");
  }
  return matches[0];
}

function millis(match: RegExpExecArray, index: number): number {
  const candidate = Number(match[index]);
  if (!Number.isSafeInteger(candidate) || candidate < 0) {
    throw failure("evidence_invalid", "reconnect lifecycle timestamp is invalid");
  }
  return candidate;
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof NetworkReconnectEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
}

async function writePrivate(output: string, contents: string): Promise<void> {
  await writeFile(output, contents, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
}

async function privateModesValid(candidate: string): Promise<boolean> {
  const metadata = await stat(candidate);
  const expected = metadata.isDirectory() ? 0o700 : 0o600;
  if ((metadata.mode & 0o777) !== expected) return false;
  if (!metadata.isDirectory()) return true;
  for (const entry of await readdir(candidate)) {
    if (!await privateModesValid(path.join(candidate, entry))) return false;
  }
  return true;
}

async function recover(
  processPort: ProcessPort,
  flashProgram: string,
  options: NetworkReconnectEvidenceOptions,
  manifest: string,
  credentials: string,
): Promise<RecoveryFacts> {
  try {
    const outcome = await processPort.run(flashCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest,
      wifiCredentials: credentials,
    }));
    const complete = !outcome.timedOut && outcome.exitCode === 0;
    return {
      recovery_complete: complete,
      recovery_flash_used: true,
      secondary_recovery_failure: !complete,
    };
  } catch {
    return {
      recovery_complete: false,
      recovery_flash_used: true,
      secondary_recovery_failure: true,
    };
  }
}

export async function captureNetworkReconnectEvidence(
  workspaceRoot: string,
  options: NetworkReconnectEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
): Promise<NetworkReconnectEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await createPrivateRoot(privateRoot);
  const manifestDocument = await readFile(manifestPath, "utf8");
  await stat(credentialsPath);
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const sourceCommit = string(manifest, "source_commit", "package manifest");
  const referenceCommit = string(manifest, "reference_commit", "package manifest");
  const appElfSha256 = string(manifest, "app_elf_sha256", "package manifest");
  let effectStarted = false;

  try {
    effectStarted = true;
    let capture;
    try {
      capture = await processPort.run(flashMonitorCommand(flashProgram, {
        board: 205,
        port: options.port,
        manifest: manifestPath,
        wifiCredentials: credentialsPath,
        networkReconnectProbe: true,
        captureTimeoutSeconds: options.captureTimeoutSeconds,
      }));
    } catch {
      throw failure("process_failed", "network reconnect child launch failed");
    }
    if (capture.timedOut) throw failure("timeout", "network reconnect child timed out");
    if (capture.exitCode !== 0) throw failure("hardware_blocked", "network reconnect child was not ready");
    const serialPath = path.join(privateRoot, "flash-monitor.private.log");
    await writePrivate(serialPath, capture.stdout);

    if (!hasPassiveSafeState(capture.stdout)) {
      throw failure("evidence_invalid", "safe-state boot evidence is missing");
    }
    for (const identity of [sourceCommit, referenceCommit, appElfSha256]) {
      if (!capture.stdout.includes(identity)) {
        throw failure("evidence_invalid", "serial build identity does not match the package");
      }
    }
    const armed = marker(
      capture.stdout,
      /wifi_reconnect_probe=armed uptime_ms=(\d+)/gu,
      "reconnect_not_observed",
    );
    const disconnected = marker(
      capture.stdout,
      /wifi_reconnect=disconnected reason=[a-z_]+ retry_ordinal=1 fallback=true retry_delay_ms=5000 uptime_ms=(\d+)/gu,
      "reconnect_not_observed",
    );
    const attempted = marker(
      capture.stdout,
      /wifi_reconnect=attempt_started retry_ordinal=1 uptime_ms=(\d+)/gu,
      "reconnect_not_observed",
    );
    const connected = marker(
      capture.stdout,
      /wifi_reconnect=connected completed_retry_ordinal=1 retry_ordinal=0 fallback=false uptime_ms=(\d+)/gu,
      "reconnect_not_observed",
    );
    const recovered = marker(
      capture.stdout,
      /wifi_reconnect_probe=recovered completed_retry_ordinal=1 uptime_ms=(\d+)/gu,
      "reconnect_not_observed",
    );
    const stable = marker(
      capture.stdout,
      /wifi_reconnect_probe=stable completed_retry_ordinal=1 stability_ms=15000 uptime_ms=(\d+)/gu,
      "reconnect_not_observed",
    );
    const armedMs = millis(armed, 1);
    const disconnectedMs = millis(disconnected, 1);
    const attemptedMs = millis(attempted, 1);
    const connectedMs = millis(connected, 1);
    const recoveredMs = millis(recovered, 1);
    const stableMs = millis(stable, 1);
    const observedRetryDelayMs = attemptedMs - disconnectedMs;
    if (!(armedMs < disconnectedMs
      && disconnectedMs < attemptedMs
      && attemptedMs <= connectedMs
      && connectedMs <= recoveredMs
      && recoveredMs - connectedMs <= 1_000
      && observedRetryDelayMs >= 5_000
      && observedRetryDelayMs <= 15_000
      && stableMs - recoveredMs >= 15_000)) {
      throw failure("reconnect_timing_invalid", "network reconnect timing contract did not hold");
    }

    let origin: URL;
    try {
      origin = uniqueRuntimeOrigin(capture.stdout);
    } catch {
      throw failure("service_recovery_failed", "recovered runtime origin was unavailable");
    }
    let systemInfo: JsonObject;
    try {
      systemInfo = object(
        await fetchJsonFromSameOrigin(origin, "/api/system/info", path.join(privateRoot, "system-info.private.json")),
        "system info",
      );
    } catch {
      throw failure("service_recovery_failed", "same-origin system service did not recover");
    }
    const sessions = new Set(
      [...capture.stdout.matchAll(/runtime_boot_attestation .*\bsession=([0-9a-f]{32})\b/gu)]
        .map((match) => match[1])
        .filter((value): value is string => value !== undefined),
    );
    const sameBootSession = sessions.size === 1 && sessions.has(string(systemInfo, "bootSession", "system info"));
    const apiPostconditionMatches = systemInfo["wifiStatus"] === "connected" && systemInfo["apEnabled"] === 0;
    const exactBuildIdentityMatches = systemInfo["sourceCommit"] === sourceCommit
      && systemInfo["referenceCommit"] === referenceCommit
      && systemInfo["appElfSha256"] === appElfSha256;
    if (!sameBootSession || !apiPostconditionMatches || !exactBuildIdentityMatches) {
      throw failure("service_recovery_failed", "recovered system service postcondition did not match");
    }

    const evidence: NetworkReconnectEvidence = {
      schema_version: "bitaxe-network-reconnect-evidence-v1",
      board: 205,
      source_commit: sourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: sha256(manifestDocument),
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-network-reconnect-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: sha256(manifestDocument),
          timeout: options.captureTimeoutSeconds,
          probe: "network-reconnect-v1",
        })),
      },
      detector_admitted: true,
      boot_observed: true,
      same_boot_session: true,
      reconnect: {
        disconnect_event_count: 1,
        fallback_enabled: true,
        first_retry_ordinal: 1,
        configured_retry_delay_ms: 5_000,
        observed_retry_delay_ms: observedRetryDelayMs,
        dhcp_recovery_observed: true,
        retry_ordinal_reset: true,
        client_only_restored: true,
        stability_window_ms: 15_000,
        stability_observed: true,
        api_postcondition_matches: true,
        exact_build_identity_matches: true,
      },
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      recovery_flash_used: false,
      private_modes_valid: true,
      redaction_status: "passed",
    };
    const candidatePath = path.join(privateRoot, "network-reconnect-evidence.private.json");
    await writePrivate(candidatePath, `${JSON.stringify(evidence, null, 2)}\n`);
    if (!await privateModesValid(privateRoot)) {
      throw failure("evidence_invalid", "private network reconnect artifact modes are invalid");
    }
    let validation;
    try {
      validation = await processPort.run(internalCommandSpec(validatorProgram, [candidatePath], (value) => value));
    } catch {
      throw failure("process_failed", "network reconnect validator launch failed");
    }
    if (validation.timedOut) throw failure("timeout", "network reconnect validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "network reconnect validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof NetworkReconnectEvidenceError
      ? error
      : failure("evidence_invalid", "network reconnect evidence processing failed");
    if (!effectStarted) throw primary;
    const recovery = await recover(processPort, flashProgram, options, manifestPath, credentialsPath);
    throw primary.withRecovery(recovery);
  }
}
