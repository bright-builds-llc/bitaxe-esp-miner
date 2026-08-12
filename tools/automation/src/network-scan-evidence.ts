import { createHash } from "node:crypto";
import { isIP } from "node:net";
import { chmod, mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type NetworkScanEvidence,
} from "./contracts.generated.js";
import { fetchJsonFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type NetworkScanEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type FailureCategory = Extract<AutomationCategory,
  "evidence_invalid" | "hardware_blocked" | "process_failed" | "service_recovery_failed" | "timeout">;
type RecoveryFacts = {
  readonly recovery_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};
type JsonObject = Readonly<Record<string, unknown>>;
type AddressKind = "global" | "link_local" | "unique_local";

const noRecovery: RecoveryFacts = {
  recovery_complete: false,
  recovery_flash_used: false,
  secondary_recovery_failure: false,
};

export class NetworkScanEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "NetworkScanEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): NetworkScanEvidenceError {
    return new NetworkScanEvidenceError(this.category, this.message, {
      ...this.publicValue,
      ...recovery,
    });
  }
}

function failure(category: FailureCategory, message: string): NetworkScanEvidenceError {
  return new NetworkScanEvidenceError(category, message, {
    stage: "network_scan_capture",
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
    throw failure("evidence_invalid", `${context} string field is invalid`);
  }
  return candidate;
}

function integer(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 0) {
    throw failure("evidence_invalid", `${context} integer field is invalid`);
  }
  return candidate;
}

export function stationAddressKind(value: string): AddressKind {
  const separator = value.lastIndexOf("%");
  const address = separator === -1 ? value : value.slice(0, separator);
  const maybeZone = separator === -1 ? undefined : value.slice(separator + 1);
  if (isIP(address) !== 6 || (maybeZone !== undefined && !/^[1-9][0-9]*$/u.test(maybeZone))) {
    throw failure("hardware_blocked", "station address is unavailable or invalid");
  }
  const firstText = address.split(":", 1)[0];
  if (firstText === undefined || firstText === "") {
    throw failure("hardware_blocked", "station address is not a reportable unicast address");
  }
  const first = Number.parseInt(firstText, 16);
  if ((first & 0xffc0) === 0xfe80) return "link_local";
  if (maybeZone !== undefined) {
    throw failure("evidence_invalid", "only link-local station addresses may include a zone");
  }
  if ((first & 0xfe00) === 0xfc00) return "unique_local";
  if ((first & 0xff00) === 0xff00) {
    throw failure("hardware_blocked", "station address is not a reportable unicast address");
  }
  return "global";
}

function validateSystemInfo(
  value: JsonObject,
  manifest: JsonObject,
): { readonly session: string; readonly uptime: number; readonly address: string; readonly kind: AddressKind } {
  for (const [wire, manifestField] of [
    ["sourceCommit", "source_commit"],
    ["referenceCommit", "reference_commit"],
    ["appElfSha256", "app_elf_sha256"],
  ] as const) {
    if (string(value, wire, "system info") !== string(manifest, manifestField, "package manifest")) {
      throw failure("evidence_invalid", "system info does not match the exact package");
    }
  }
  if (value["wifiStatus"] !== "connected" || value["apEnabled"] !== 0) {
    throw failure("service_recovery_failed", "station service is not connected client-only");
  }
  const session = string(value, "bootSession", "system info");
  if (!/^[0-9a-f]{32}$/u.test(session)) throw failure("evidence_invalid", "boot session is invalid");
  const maybeAddress = value["ipv6"];
  if (typeof maybeAddress !== "string" || maybeAddress === "") {
    throw failure("hardware_blocked", "station address is unavailable or invalid");
  }
  const address = maybeAddress;
  return {
    session,
    uptime: integer(value, "uptimeSeconds", "system info"),
    address,
    kind: stationAddressKind(address),
  };
}

function validateScan(value: JsonObject): number {
  const networks = value["networks"];
  if (!Array.isArray(networks)) throw failure("evidence_invalid", "scan response records are invalid");
  if (networks.length === 0) throw failure("hardware_blocked", "scan returned no visible records");
  if (networks.length > 20) throw failure("evidence_invalid", "scan response exceeded its bound");
  for (const unknownRecord of networks) {
    const record = object(unknownRecord, "scan record");
    const keys = Object.keys(record).sort();
    if (keys.length !== 3 || keys[0] !== "authmode" || keys[1] !== "rssi" || keys[2] !== "ssid") {
      throw failure("evidence_invalid", "scan record shape is invalid");
    }
    if (typeof record["ssid"] !== "string") throw failure("evidence_invalid", "scan record name is invalid");
    const rssi = record["rssi"];
    if (typeof rssi !== "number" || !Number.isSafeInteger(rssi) || rssi < -128 || rssi > 127) {
      throw failure("evidence_invalid", "scan signal value is invalid");
    }
    const authMode = record["authmode"];
    if (typeof authMode !== "number" || !Number.isSafeInteger(authMode) || authMode < 0 || authMode > 9) {
      throw failure("evidence_invalid", "scan auth mode is invalid");
    }
  }
  return networks.length;
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof NetworkScanEvidenceError) throw error;
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

async function recover(
  processPort: ProcessPort,
  flashProgram: string,
  options: NetworkScanEvidenceOptions,
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

export async function captureNetworkScanEvidence(
  workspaceRoot: string,
  options: NetworkScanEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
): Promise<NetworkScanEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await stat(manifestPath);
  await stat(credentialsPath);
  await createPrivateRoot(privateRoot);
  const manifestDocument = await readFile(manifestPath, "utf8");
  let manifest: JsonObject;
  try {
    manifest = object(JSON.parse(manifestDocument), "package manifest");
    string(manifest, "source_commit", "package manifest");
    string(manifest, "reference_commit", "package manifest");
    string(manifest, "app_elf_sha256", "package manifest");
  } catch (error) {
    if (error instanceof NetworkScanEvidenceError) throw error;
    throw failure("evidence_invalid", "package manifest is invalid");
  }

  let flashEffect = false;
  try {
    flashEffect = true;
    const capture = await child(processPort, flashProgram, flashMonitorCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest: manifestPath,
      wifiCredentials: credentialsPath,
      captureTimeoutSeconds: options.captureTimeoutSeconds,
      evidenceMode: "dual",
      evidenceDir: privateRoot,
    }).args, "exact-package flash-monitor");
    if (capture.timedOut) throw failure("timeout", "exact-package flash-monitor timed out");
    if (capture.exitCode !== 0) throw failure("hardware_blocked", "exact-package flash-monitor was not ready");
    const monitor = await readFile(path.join(privateRoot, "flash-monitor.classifier-input.log"), "utf8");
    if (!hasPassiveSafeState(monitor)) throw failure("evidence_invalid", "boot lacks passive safe-state evidence");
    let origin: URL;
    try {
      origin = uniqueRuntimeOrigin(monitor);
    } catch {
      throw failure("evidence_invalid", "runtime origin admission is invalid");
    }

    let beforeValue: JsonObject;
    try {
      beforeValue = object(await fetchJsonFromSameOrigin(
        origin,
        "/api/system/info",
        path.join(privateRoot, "system-before.private.json"),
      ), "pre-scan system info");
    } catch (error) {
      if (error instanceof NetworkScanEvidenceError) throw error;
      throw failure("service_recovery_failed", "pre-scan system service is unavailable");
    }
    const before = validateSystemInfo(beforeValue, manifest);

    const scanStarted = Date.now();
    let scanValue: JsonObject;
    try {
      scanValue = object(await fetchJsonFromSameOrigin(
        origin,
        "/api/system/wifi/scan",
        path.join(privateRoot, "scan.private.json"),
      ), "scan response");
    } catch (error) {
      if (error instanceof NetworkScanEvidenceError) throw error;
      throw failure("hardware_blocked", "live Wi-Fi scan did not complete");
    }
    const scanDurationMs = Math.max(1, Date.now() - scanStarted);
    if (scanDurationMs > 10_000) throw failure("timeout", "live Wi-Fi scan exceeded its bound");
    const recordCount = validateScan(scanValue);

    let afterValue: JsonObject;
    try {
      afterValue = object(await fetchJsonFromSameOrigin(
        origin,
        "/api/system/info",
        path.join(privateRoot, "system-after.private.json"),
      ), "post-scan system info");
    } catch (error) {
      if (error instanceof NetworkScanEvidenceError) throw error;
      throw failure("service_recovery_failed", "post-scan system service is unavailable");
    }
    const after = validateSystemInfo(afterValue, manifest);
    if (before.session !== after.session) throw failure("service_recovery_failed", "scan crossed a boot session");
    if (after.uptime < before.uptime) throw failure("evidence_invalid", "system uptime regressed across the scan");
    if (before.address !== after.address || before.kind !== after.kind) {
      throw failure("service_recovery_failed", "station address was not preserved across the scan");
    }

    const evidence: NetworkScanEvidence = {
      schema_version: "bitaxe-network-scan-evidence-v1",
      board: 205,
      source_commit: string(manifest, "source_commit", "package manifest"),
      reference_commit: string(manifest, "reference_commit", "package manifest"),
      package_manifest_sha256: sha256(manifestDocument),
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-network-scan-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: sha256(manifestDocument),
          route: "wifi-scan-v1",
          timeout: options.captureTimeoutSeconds,
        })),
      },
      detector_admitted: true,
      boot_observed: true,
      same_origin_observed: true,
      scan: {
        record_count: recordCount,
        scan_duration_ms: scanDurationMs,
        records_shape_valid: true,
        signal_values_valid: true,
        auth_modes_valid: true,
        exact_build_identity_matches: true,
        same_boot_session: true,
        before_after_connected: true,
        client_only_preserved: true,
        uptime_monotonic: true,
        address_family: "v6",
        address_kind: before.kind,
        address_stable: true,
      },
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      recovery_flash_used: false,
      private_modes_valid: true,
      redaction_status: "passed",
    };
    const candidatePath = path.join(privateRoot, "network-scan-evidence.private.json");
    await privateJson(candidatePath, evidence);
    if (!await privateModesValid(privateRoot)) {
      throw failure("evidence_invalid", "private network scan artifact modes are invalid");
    }
    const validation = await child(processPort, validatorProgram, [candidatePath], "network scan validator");
    if (validation.timedOut) throw failure("timeout", "network scan validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "network scan validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
    });
    return evidence;
  } catch (error) {
    const primary = error instanceof NetworkScanEvidenceError
      ? error
      : failure("evidence_invalid", "network scan evidence processing failed");
    if (!flashEffect) throw primary;
    throw primary.withRecovery(await recover(processPort, flashProgram, options, manifestPath, credentialsPath));
  }
}
