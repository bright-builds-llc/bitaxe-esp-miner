import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type RuntimeHealthEvidence,
} from "./contracts.generated.js";
import { fetchJsonFromSameOrigin, fetchTextFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { captureJsonWebSocketFrame, type WebSocketFactory } from "./websocket.js";
import { assertWithinWorkspace } from "./workspace.js";

export type RuntimeHealthEvidenceOptions = {
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

const noRecovery: RecoveryFacts = { recovery_complete: false, recovery_flash_used: false, secondary_recovery_failure: false };

export class RuntimeHealthEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "RuntimeHealthEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): RuntimeHealthEvidenceError {
    return new RuntimeHealthEvidenceError(this.category, this.message, { ...this.publicValue, ...recovery });
  }
}

function failure(category: FailureCategory, message: string) {
  return new RuntimeHealthEvidenceError(category, message, { stage: "runtime_health_capture", ...noRecovery });
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error(`${context} must be an object`);
  return value as JsonObject;
}

function string(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") throw new Error(`${context} ${field} must be a non-empty string`);
  return candidate;
}

function integer(value: JsonObject, field: string, context: string, minimum = 0): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < minimum) {
    throw new Error(`${context} ${field} must be an integer`);
  }
  return candidate;
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw new Error("private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof Error && error.message === "private attempt root must be absent before launch") throw error;
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

function health(snapshot: JsonObject, context: string): JsonObject {
  return object(snapshot["runtimeHealth"], `${context} runtime health`);
}

function retainedRecord(session: string, revision: number, value: JsonObject): string {
  return [
    `runtime_health boot_session=${session}`,
    `operator_snapshot_revision=${String(revision)}`,
    `self_test=${string(value, "selfTestState", "runtime health")}`,
    `supervisor=${string(value, "supervisorAvailability", "runtime health")}`,
    `checkpoint_category=${string(value, "checkpointCategory", "runtime health")}`,
    `checkpoint_sequence=${String(integer(value, "checkpointSequence", "runtime health", 1))}`,
    `checkpoint_age_millis=${String(integer(value, "checkpointAgeMillis", "runtime health"))}`,
    `checkpoint_health=${string(value, "checkpointHealth", "runtime health")}`,
    `task_watchdog_participation=${string(value, "taskWatchdogParticipation", "runtime health")}`,
    `task_watchdog_reason=${string(value, "taskWatchdogReason", "runtime health")}`,
    `task_watchdog_feed_sequence=${String(integer(value, "taskWatchdogFeedSequence", "runtime health", 1))}`,
    `task_watchdog_feed_age_millis=${String(integer(value, "taskWatchdogFeedAgeMillis", "runtime health"))}`,
    `task_watchdog_owner_phase=${string(value, "taskWatchdogOwnerPhase", "runtime health")}`,
    `task_watchdog_wait_state=${string(value, "taskWatchdogWaitState", "runtime health")}`,
    "redacted=true",
  ].join(" ");
}

function validateHealth(value: JsonObject): { readonly checkpointSequence: number; readonly feedSequence: number } {
  const checkpointSequence = integer(value, "checkpointSequence", "runtime health", 1);
  const checkpointAge = integer(value, "checkpointAgeMillis", "runtime health");
  const feedSequence = integer(value, "taskWatchdogFeedSequence", "runtime health", 1);
  const feedAge = integer(value, "taskWatchdogFeedAgeMillis", "runtime health");
  const ownerPhase = string(value, "taskWatchdogOwnerPhase", "runtime health");
  const waitState = string(value, "taskWatchdogWaitState", "runtime health");
  if (
    string(value, "selfTestState", "runtime health") !== "unavailable"
    || string(value, "supervisorAvailability", "runtime health") !== "available"
    || string(value, "checkpointHealth", "runtime health") !== "healthy"
    || string(value, "taskWatchdogParticipation", "runtime health") !== "participating"
    || string(value, "taskWatchdogReason", "runtime health") !== "feed_fresh"
    || ![
      "subscribing", "loop_start", "waiting_inbox", "handling_inbox",
      "handling_observation", "handling_readiness", "publishing_campaign_status",
      "servicing_hashrate", "shutdown",
    ].includes(ownerPhase)
    || !["not_waiting", "within_deadline", "deadline_overrun"].includes(waitState)
    || checkpointAge > 1_500
    || feedAge > 2_000
  ) {
    throw failure("hardware_blocked", "runtime health is not substantively healthy");
  }
  string(value, "checkpointCategory", "runtime health");
  return { checkpointSequence, feedSequence };
}

async function recover(
  processPort: ProcessPort,
  flashProgram: string,
  options: RuntimeHealthEvidenceOptions,
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

export async function captureRuntimeHealthEvidence(
  workspaceRoot: string,
  options: RuntimeHealthEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
  maybeWebSocketFactory?: WebSocketFactory,
): Promise<RuntimeHealthEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await access(manifestPath);
  await access(credentialsPath);
  await createPrivateRoot(privateRoot);
  const manifestDocument = await readFile(manifestPath, "utf8");
  let manifest: JsonObject;
  try {
    manifest = object(JSON.parse(manifestDocument), "package manifest");
    string(manifest, "source_commit", "package manifest");
    string(manifest, "reference_commit", "package manifest");
    string(manifest, "app_elf_sha256", "package manifest");
  } catch {
    throw failure("evidence_invalid", "package manifest identity is invalid");
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
    const origin = uniqueRuntimeOrigin(monitor);
    const api = object(await fetchJsonFromSameOrigin(origin, "/api/system/info", path.join(privateRoot, "api.private.json")), "API snapshot");
    const envelope = object(await captureJsonWebSocketFrame(origin, "/api/ws/live", path.join(privateRoot, "websocket.private.json"), maybeWebSocketFactory), "WebSocket frame");
    if (envelope["event"] !== "update") throw failure("evidence_invalid", "WebSocket event is invalid");
    const websocket = object(envelope["data"], "WebSocket snapshot");
    const retained = await fetchTextFromSameOrigin(origin, "/api/system/logs", path.join(privateRoot, "retained-log.private.txt"));
    const session = string(api, "bootSession", "API snapshot");
    const websocketSession = string(websocket, "bootSession", "WebSocket snapshot");
    const apiRevision = integer(api, "operatorSnapshotRevision", "API snapshot", 1);
    const websocketRevision = integer(websocket, "operatorSnapshotRevision", "WebSocket snapshot", 1);
    if (!/^[0-9a-f]{32}$/u.test(session) || session !== websocketSession || websocketRevision < apiRevision) {
      throw failure("evidence_invalid", "runtime health snapshot identity is incoherent");
    }
    for (const snapshot of [api, websocket]) {
      if (string(snapshot, "sourceCommit", "runtime snapshot") !== string(manifest, "source_commit", "package manifest")
        || string(snapshot, "referenceCommit", "runtime snapshot") !== string(manifest, "reference_commit", "package manifest")
        || string(snapshot, "appElfSha256", "runtime snapshot") !== string(manifest, "app_elf_sha256", "package manifest")) {
        throw failure("evidence_invalid", "runtime health snapshot does not match the exact package");
      }
    }
    const apiHealth = health(api, "API snapshot");
    const websocketHealth = health(websocket, "WebSocket snapshot");
    const apiSequence = validateHealth(apiHealth);
    const websocketSequence = validateHealth(websocketHealth);
    if (websocketSequence.checkpointSequence < apiSequence.checkpointSequence
      || websocketSequence.feedSequence < apiSequence.feedSequence
      || string(websocketHealth, "checkpointCategory", "runtime health")
        !== string(apiHealth, "checkpointCategory", "runtime health")) {
      throw failure("evidence_invalid", "runtime health sequences regressed");
    }
    const retainedLines = new Set(retained.split(/\r?\n/u));
    if (!retainedLines.has(retainedRecord(session, apiRevision, apiHealth))
      || !retainedLines.has(retainedRecord(session, websocketRevision, websocketHealth))) {
      throw failure("evidence_invalid", "retained runtime health tuple is missing");
    }
    const evidence: RuntimeHealthEvidence = {
      schema_version: "bitaxe-runtime-health-evidence-v1",
      board: 205,
      source_commit: string(manifest, "source_commit", "package manifest"),
      reference_commit: string(manifest, "reference_commit", "package manifest"),
      package_manifest_sha256: sha256(manifestDocument),
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-runtime-health-evidence",
        request_sha256: sha256(JSON.stringify({ manifest: sha256(manifestDocument), timeout: options.captureTimeoutSeconds })),
      },
      detector_admitted: true,
      boot_observed: true,
      same_origin_observed: true,
      runtime_health: {
        boot_session_sha256: sha256(session),
        http_revision: apiRevision,
        websocket_revision: websocketRevision,
        same_boot_session: true,
        websocket_revision_not_earlier: true,
        self_test_state: "unavailable",
        supervisor_availability: "available",
        checkpoint_category: string(apiHealth, "checkpointCategory", "runtime health"),
        http_checkpoint_sequence: apiSequence.checkpointSequence,
        websocket_checkpoint_sequence: websocketSequence.checkpointSequence,
        checkpoint_sequence_not_regressed: true,
        checkpoint_health: "healthy",
        checkpoint_age_bounded: true,
        task_watchdog_participation: "participating",
        task_watchdog_reason: "feed_fresh",
        http_task_watchdog_feed_sequence: apiSequence.feedSequence,
        websocket_task_watchdog_feed_sequence: websocketSequence.feedSequence,
        task_watchdog_feed_sequence_not_regressed: true,
        task_watchdog_feed_age_bounded: true,
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
    const validation = await child(processPort, validatorProgram, [candidate], "runtime health evidence validator");
    if (validation.timedOut) throw failure("timeout", "runtime health validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "runtime health validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof RuntimeHealthEvidenceError
      ? error
      : failure("process_failed", "runtime health orchestration failed");
    if (!flashEffect) throw primary;
    throw primary.withRecovery(await recover(processPort, flashProgram, options, manifestPath, credentialsPath));
  }
}
