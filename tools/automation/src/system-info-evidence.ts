import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type SystemInfoEvidence,
} from "./contracts.generated.js";
import { fetchJsonFromSameOrigin, fetchTextFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { captureJsonWebSocketFrame, type WebSocketFactory } from "./websocket.js";
import { assertWithinWorkspace } from "./workspace.js";

export type SystemInfoEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<AutomationCategory, "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed">;
type RecoveryFacts = {
  readonly recovery_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};
type FieldRule = { readonly type: "array" | "boolean" | "number" | "object" | "string"; readonly presence: "always" | "block_found" };
type FieldContract = { readonly schema_version: string; readonly fields: Readonly<Record<string, FieldRule>> };

const noRecovery: RecoveryFacts = { recovery_complete: false, recovery_flash_used: false, secondary_recovery_failure: false };
const settingFields = new Set([
  "display", "rotation", "invertscreen", "displayTimeout", "manualFanSpeed", "minFanSpeed", "temptarget",
  "statsFrequency", "statsLimit", "overclockEnabled", "overheat_mode", "stratumURL", "stratumPort", "stratumUser",
  "stratumSuggestedDifficulty", "stratumExtranonceSubscribe", "stratumTLS", "stratumCert", "stratumDecodeCoinbase",
  "stratumProtocol", "stratumV2AuthorityPubkey", "fallbackStratumURL", "fallbackStratumPort", "fallbackStratumUser",
  "fallbackStratumSuggestedDifficulty", "fallbackStratumExtranonceSubscribe", "fallbackStratumTLS", "fallbackStratumCert",
  "fallbackStratumDecodeCoinbase",
]);

export class SystemInfoEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "SystemInfoEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): SystemInfoEvidenceError {
    return new SystemInfoEvidenceError(this.category, this.message, { ...this.publicValue, ...recovery });
  }
}

function failure(category: FailureCategory, message: string) {
  return new SystemInfoEvidenceError(category, message, { stage: "system_info_capture", ...noRecovery });
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw failure("evidence_invalid", `${context} must be an object`);
  return value as JsonObject;
}

function string(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") throw failure("evidence_invalid", `${context} field is invalid`);
  return candidate;
}

function integer(value: JsonObject, field: string, context: string, minimum = 0): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < minimum) {
    throw failure("evidence_invalid", `${context} integer field is invalid`);
  }
  return candidate;
}

function jsonType(value: unknown): FieldRule["type"] | "null" {
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  return typeof value === "object" ? "object" : typeof value as FieldRule["type"];
}

function parseContract(document: string): FieldContract {
  const candidate = object(JSON.parse(document), "field contract");
  const fields = object(candidate["fields"], "field contract fields");
  const parsed: Record<string, FieldRule> = {};
  for (const [name, unknownRule] of Object.entries(fields)) {
    const rule = object(unknownRule, "field contract rule");
    const type = rule["type"];
    const presence = rule["presence"];
    if (!new Set(["array", "boolean", "number", "object", "string"]).has(String(type))
      || !new Set(["always", "block_found"]).has(String(presence))) {
      throw failure("evidence_invalid", "field contract rule is invalid");
    }
    parsed[name] = { type: type as FieldRule["type"], presence: presence as FieldRule["presence"] };
  }
  if (candidate["schema_version"] !== "bitaxe-system-info-field-contract-v1" || Object.keys(parsed).length !== 94) {
    throw failure("evidence_invalid", "field contract identity is invalid");
  }
  return { schema_version: candidate["schema_version"], fields: parsed };
}

function validateSnapshot(snapshot: JsonObject, contract: FieldContract): void {
  const blockFound = snapshot["blockFound"];
  if (blockFound !== 0) throw failure("hardware_blocked", "system info block notification is active");
  for (const [field, rule] of Object.entries(contract.fields)) {
    const present = Object.hasOwn(snapshot, field);
    if (rule.presence === "block_found") {
      if (present) throw failure("evidence_invalid", "inactive conditional system info field is present");
      continue;
    }
    if (!present || jsonType(snapshot[field]) !== rule.type) {
      throw failure("evidence_invalid", "system info field contract does not match");
    }
  }
  for (const field of settingFields) {
    if (!Object.hasOwn(snapshot, field)) throw failure("evidence_invalid", "confirmed setting projection is incomplete");
  }
}

function runtimeHealth(snapshot: JsonObject): JsonObject {
  return object(snapshot["runtimeHealth"], "runtime health");
}

function retainedRecord(session: string, revision: number, value: JsonObject): string {
  return [
    `runtime_health boot_session=${session}`,
    `operator_snapshot_revision=${String(revision)}`,
    `self_test=${String(value["selfTestState"])}`,
    `supervisor=${String(value["supervisorAvailability"])}`,
    `checkpoint_category=${String(value["checkpointCategory"])}`,
    `checkpoint_sequence=${String(value["checkpointSequence"])}`,
    `checkpoint_age_millis=${String(value["checkpointAgeMillis"])}`,
    `checkpoint_health=${String(value["checkpointHealth"])}`,
    `task_watchdog_participation=${String(value["taskWatchdogParticipation"])}`,
    `task_watchdog_reason=${String(value["taskWatchdogReason"])}`,
    `task_watchdog_feed_sequence=${String(value["taskWatchdogFeedSequence"])}`,
    `task_watchdog_feed_age_millis=${String(value["taskWatchdogFeedAgeMillis"])}`,
    `task_watchdog_owner_phase=${String(value["taskWatchdogOwnerPhase"])}`,
    `task_watchdog_wait_state=${String(value["taskWatchdogWaitState"])}`,
    "redacted=true",
  ].join(" ");
}

function validateRuntimeIdentity(snapshot: JsonObject, manifest: JsonObject): { readonly session: string; readonly revision: number } {
  for (const [wire, manifestField] of [
    ["sourceCommit", "source_commit"], ["referenceCommit", "reference_commit"], ["appElfSha256", "app_elf_sha256"],
  ] as const) {
    if (string(snapshot, wire, "runtime snapshot") !== string(manifest, manifestField, "package manifest")) {
      throw failure("evidence_invalid", "system info snapshot does not match the exact package");
    }
  }
  const session = string(snapshot, "bootSession", "runtime snapshot");
  if (!/^[0-9a-f]{32}$/u.test(session)) throw failure("evidence_invalid", "boot session is invalid");
  return { session, revision: integer(snapshot, "operatorSnapshotRevision", "runtime snapshot", 1) };
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof SystemInfoEvidenceError) throw error;
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

async function recover(
  processPort: ProcessPort,
  flashProgram: string,
  options: SystemInfoEvidenceOptions,
  manifest: string,
  credentials: string,
): Promise<RecoveryFacts> {
  try {
    const outcome = await processPort.run(flashCommand(flashProgram, {
      board: 205, port: options.port, manifest, wifiCredentials: credentials,
    }));
    const complete = !outcome.timedOut && outcome.exitCode === 0;
    return { recovery_complete: complete, recovery_flash_used: true, secondary_recovery_failure: !complete };
  } catch {
    return { recovery_complete: false, recovery_flash_used: true, secondary_recovery_failure: true };
  }
}

export async function captureSystemInfoEvidence(
  workspaceRoot: string,
  options: SystemInfoEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
  maybeWebSocketFactory?: WebSocketFactory,
): Promise<SystemInfoEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  const contractPath = path.join(workspaceRoot, "crates/bitaxe-api/fixtures/api/system-info-contract-v1.json");
  await access(manifestPath);
  await access(credentialsPath);
  await createPrivateRoot(privateRoot);
  const manifestDocument = await readFile(manifestPath, "utf8");
  const contractDocument = await readFile(contractPath, "utf8");
  let manifest: JsonObject;
  let contract: FieldContract;
  try {
    manifest = object(JSON.parse(manifestDocument), "package manifest");
    string(manifest, "source_commit", "package manifest");
    string(manifest, "reference_commit", "package manifest");
    string(manifest, "app_elf_sha256", "package manifest");
    contract = parseContract(contractDocument);
  } catch (error) {
    if (error instanceof SystemInfoEvidenceError) throw error;
    throw failure("evidence_invalid", "system info input identity is invalid");
  }
  let flashEffect = false;
  try {
    flashEffect = true;
    const outcome = await child(processPort, flashProgram, flashMonitorCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest: manifestPath,
      wifiCredentials: credentialsPath,
      captureTimeoutSeconds: options.captureTimeoutSeconds,
      evidenceMode: "dual",
      evidenceDir: privateRoot,
    }).args, "exact-package flash-monitor");
    if (outcome.timedOut) throw failure("timeout", "exact-package flash-monitor timed out");
    if (outcome.exitCode !== 0) throw failure("process_failed", "exact-package flash-monitor failed");
    const monitor = await readFile(path.join(privateRoot, "flash-monitor.classifier-input.log"), "utf8");
    if (!hasPassiveSafeState(monitor)) throw failure("evidence_invalid", "boot lacks passive safe-state evidence");
    let origin: URL;
    try {
      origin = uniqueRuntimeOrigin(monitor);
    } catch {
      throw failure("evidence_invalid", "runtime origin admission is invalid");
    }
    const api = object(await fetchJsonFromSameOrigin(origin, "/api/system/info", path.join(privateRoot, "api.private.json")), "API snapshot");
    const envelope = object(await captureJsonWebSocketFrame(origin, "/api/ws/live", path.join(privateRoot, "websocket.private.json"), maybeWebSocketFactory), "WebSocket frame");
    if (envelope["event"] !== "update") throw failure("evidence_invalid", "WebSocket event is invalid");
    const websocket = object(envelope["data"], "WebSocket snapshot");
    const retained = await fetchTextFromSameOrigin(origin, "/api/system/logs", path.join(privateRoot, "retained-log.private.txt"));
    validateSnapshot(api, contract);
    validateSnapshot(websocket, contract);
    const apiIdentity = validateRuntimeIdentity(api, manifest);
    const websocketIdentity = validateRuntimeIdentity(websocket, manifest);
    if (apiIdentity.session !== websocketIdentity.session || websocketIdentity.revision < apiIdentity.revision) {
      throw failure("evidence_invalid", "system info snapshot identity is incoherent");
    }
    const retainedLines = new Set(retained.split(/\r?\n/u));
    if (!retainedLines.has(retainedRecord(apiIdentity.session, apiIdentity.revision, runtimeHealth(api)))
      || !retainedLines.has(retainedRecord(websocketIdentity.session, websocketIdentity.revision, runtimeHealth(websocket)))) {
      throw failure("evidence_invalid", "retained system info tuple is missing");
    }
    const conditionalFieldCount = Object.values(contract.fields).filter((rule) => rule.presence === "block_found").length;
    const evidence: SystemInfoEvidence = {
      schema_version: "bitaxe-system-info-evidence-v1",
      board: 205,
      source_commit: string(manifest, "source_commit", "package manifest"),
      reference_commit: string(manifest, "reference_commit", "package manifest"),
      package_manifest_sha256: sha256(manifestDocument),
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-system-info-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: sha256(manifestDocument), contract: sha256(contractDocument), timeout: options.captureTimeoutSeconds,
        })),
      },
      detector_admitted: true,
      boot_observed: true,
      same_origin_observed: true,
      system_info: {
        boot_session_sha256: sha256(apiIdentity.session),
        http_revision: apiIdentity.revision,
        websocket_revision: websocketIdentity.revision,
        same_boot_session: true,
        websocket_revision_not_earlier: true,
        field_contract_schema: contract.schema_version,
        field_contract_sha256: sha256(contractDocument),
        required_field_count: Object.keys(contract.fields).length,
        unconditional_field_count: Object.keys(contract.fields).length - conditionalFieldCount,
        conditional_field_count: conditionalFieldCount,
        http_unconditional_fields_complete: true,
        websocket_unconditional_fields_complete: true,
        http_field_types_match: true,
        websocket_field_types_match: true,
        inactive_block_fields_absent: true,
        confirmed_setting_fields_present: true,
        retained_http_tuple_matches: true,
        retained_websocket_tuple_matches: true,
      },
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      redaction_status: "passed",
    };
    const candidate = path.join(privateRoot, "final-evidence.private.json");
    await privateJson(candidate, evidence);
    const validation = await child(processPort, validatorProgram, [candidate], "system info evidence validator");
    if (validation.timedOut) throw failure("timeout", "system info validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "system info validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof SystemInfoEvidenceError
      ? error
      : failure("evidence_invalid", "system info orchestration evidence is invalid");
    if (!flashEffect) throw primary;
    throw primary.withRecovery(await recover(processPort, flashProgram, options, manifestPath, credentialsPath));
  }
}
