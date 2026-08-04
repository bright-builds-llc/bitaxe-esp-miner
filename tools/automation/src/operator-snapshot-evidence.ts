import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type OperatorSnapshotEpochEvidence,
  type OperatorSnapshotEvidence,
} from "./contracts.generated.js";
import { fetchJsonFromSameOrigin, fetchTextFromSameOrigin } from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { captureJsonWebSocketFrame, type WebSocketFactory } from "./websocket.js";
import { assertWithinWorkspace } from "./workspace.js";

export type OperatorSnapshotEvidenceOptions = {
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

const noRecovery: RecoveryFacts = {
  recovery_complete: false,
  recovery_flash_used: false,
  secondary_recovery_failure: false,
};

const deviceSessionFields = new Set([
  "schema_version", "terminal_category", "platform_category", "board_category",
  "same_physical_device", "stable_enumeration", "reenumerated", "reader_armed",
  "pre_restart_serial_delivery", "post_restart_serial_delivery", "serial_delivery",
  "request_outcome", "request_attempt_count", "service_loss_observed",
  "trusted_origin_preserved", "application_recovered", "build_identity_matches",
  "boot_session_changed", "boot_ordinal_advanced_by_one", "software_reset_observed",
  "postcondition_matches", "cleanup_complete", "usb_disappearance_count",
  "enumeration_change_count", "serial_byte_count", "http_observation_count", "duration_millis",
]);

const stableFields = [
  "ASICModel", "boardVersion", "version", "semanticVersion", "sourceCommit",
  "referenceCommit", "appElfSha256", "buildChannel", "sourceDirty", "releaseTag",
  "axeOSVersion", "idfVersion", "miningPaused", "miningActivity", "startMiningOnBoot",
] as const;

export class OperatorSnapshotEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "OperatorSnapshotEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): OperatorSnapshotEvidenceError {
    return new OperatorSnapshotEvidenceError(this.category, this.message, { ...this.publicValue, ...recovery });
  }
}

function failure(category: FailureCategory, message: string, facts: Readonly<Record<string, unknown>> = {}) {
  return new OperatorSnapshotEvidenceError(category, message, { stage: "operator_snapshot_capture", ...facts, ...noRecovery });
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

function positiveInteger(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 1) {
    throw new Error(`${context} ${field} must be a positive integer`);
  }
  return candidate;
}

async function createPrivateRoot(privateRoot: string): Promise<void> {
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

async function privateWrite(output: string, value: string): Promise<void> {
  await writeFile(output, value, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
}

async function privateJson(output: string, value: unknown): Promise<void> {
  await privateWrite(output, `${JSON.stringify(value, null, 2)}\n`);
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

async function classifyBaseline(processPort: ProcessPort, parityProgram: string, trace: string): Promise<JsonObject> {
  const outcome = await runChild(
    processPort,
    parityProgram,
    ["verify-settings-durability", "--trace", trace, "--mode", "terminal-baseline"],
    "baseline classifier",
  );
  if (outcome.timedOut) throw failure("timeout", "baseline classification timed out");
  if (outcome.exitCode !== 0) throw failure("process_failed", "baseline classification failed");
  try {
    const parsed = object(JSON.parse(outcome.stdout), "baseline classification");
    if (parsed["status"] !== "passed") throw new Error("baseline did not pass");
    return parsed;
  } catch {
    throw failure("evidence_invalid", "baseline classification is invalid");
  }
}

function readyDeviceSession(value: unknown): JsonObject {
  const projection = object(value, "device-session projection");
  const keys = Object.keys(projection);
  if (keys.length !== deviceSessionFields.size || keys.some((key) => !deviceSessionFields.has(key))) {
    throw failure("evidence_invalid", "device-session projection fields are invalid");
  }
  const booleans = [
    "same_physical_device", "stable_enumeration", "reenumerated", "reader_armed",
    "pre_restart_serial_delivery", "post_restart_serial_delivery", "service_loss_observed",
    "trusted_origin_preserved", "application_recovered", "build_identity_matches",
    "boot_session_changed", "boot_ordinal_advanced_by_one", "software_reset_observed",
    "postcondition_matches", "cleanup_complete",
  ];
  const counts = [
    "request_attempt_count", "usb_disappearance_count", "enumeration_change_count",
    "serial_byte_count", "http_observation_count", "duration_millis",
  ];
  if (
    projection["schema_version"] !== "esp-device-session-v1"
    || projection["board_category"] !== "205"
    || !["macos", "linux", "windows", "other"].includes(String(projection["platform_category"]))
    || !["correlated", "silent", "reacquired", "failed"].includes(String(projection["serial_delivery"]))
    || booleans.some((field) => typeof projection[field] !== "boolean")
    || counts.some((field) => typeof projection[field] !== "number" || !Number.isSafeInteger(projection[field]) || Number(projection[field]) < 0)
    || typeof projection["terminal_category"] !== "string"
  ) {
    throw failure("evidence_invalid", "device-session projection values are invalid");
  }
  if (projection["terminal_category"] !== "ready") {
    throw failure("hardware_blocked", "device-session did not become ready", {
      terminal_category: projection["terminal_category"],
    });
  }
  const requiredTrue = [
    "same_physical_device", "reader_armed", "trusted_origin_preserved", "application_recovered",
    "build_identity_matches", "boot_session_changed", "boot_ordinal_advanced_by_one",
    "software_reset_observed", "postcondition_matches", "cleanup_complete",
  ];
  if (
    projection["platform_category"] !== "macos"
    || projection["request_attempt_count"] !== 1
    || !["response_received", "response_missing"].includes(String(projection["request_outcome"]))
    || requiredTrue.some((field) => projection[field] !== true)
  ) {
    throw failure("evidence_invalid", "ready device-session projection is incomplete");
  }
  return projection;
}

async function readDeviceSession(output: string): Promise<JsonObject> {
  try {
    return readyDeviceSession(JSON.parse(await readFile(output, "utf8")));
  } catch (error) {
    if (error instanceof OperatorSnapshotEvidenceError) throw error;
    throw failure("evidence_invalid", "device-session projection is missing or malformed");
  }
}

function stableProjection(snapshot: JsonObject): JsonObject {
  const projection: Record<string, unknown> = {};
  for (const field of stableFields) {
    if (!(field in snapshot)) throw new Error(`operator snapshot is missing ${field}`);
    projection[field] = snapshot[field];
  }
  if (
    projection["ASICModel"] !== "BM1366"
    || projection["boardVersion"] !== "205"
    || typeof projection["sourceDirty"] !== "boolean"
    || typeof projection["miningPaused"] !== "boolean"
    || typeof projection["startMiningOnBoot"] !== "boolean"
  ) {
    throw new Error("operator snapshot substantive field values are invalid");
  }
  for (const field of [
    "version", "semanticVersion", "sourceCommit", "referenceCommit", "appElfSha256",
    "buildChannel", "axeOSVersion", "idfVersion", "miningActivity",
  ]) {
    if (typeof projection[field] !== "string" || projection[field] === "") {
      throw new Error("operator snapshot substantive string is invalid");
    }
  }
  return projection;
}

async function captureEpoch(
  origin: URL,
  epochRoot: string,
  manifest: JsonObject,
  parityProgram: string,
  processPort: ProcessPort,
  maybeWebSocketFactory?: WebSocketFactory,
): Promise<{ readonly session: string; readonly ordinal: number; readonly evidence: OperatorSnapshotEpochEvidence }> {
  await mkdir(epochRoot, { mode: 0o700 });
  await chmod(epochRoot, 0o700);
  const api = object(await fetchJsonFromSameOrigin(origin, "/api/system/info", path.join(epochRoot, "api.private.json")), "API snapshot");
  const envelope = object(await captureJsonWebSocketFrame(
    origin,
    "/api/ws/live",
    path.join(epochRoot, "websocket.private.json"),
    maybeWebSocketFactory,
  ), "WebSocket frame");
  if (envelope["event"] !== "update") throw failure("evidence_invalid", "WebSocket frame event is invalid");
  const websocket = object(envelope["data"], "WebSocket snapshot");
  const retainedPath = path.join(epochRoot, "retained-log.private.txt");
  await fetchTextFromSameOrigin(origin, "/api/system/logs", retainedPath);
  const apiSession = string(api, "bootSession", "API snapshot");
  const websocketSession = string(websocket, "bootSession", "WebSocket snapshot");
  const apiRevision = positiveInteger(api, "operatorSnapshotRevision", "API snapshot");
  const websocketRevision = positiveInteger(websocket, "operatorSnapshotRevision", "WebSocket snapshot");
  if (!/^[0-9a-f]{32}$/u.test(apiSession) || apiSession === "0".repeat(32)) {
    throw failure("evidence_invalid", "operator snapshot boot session is invalid");
  }
  const apiStable = stableProjection(api);
  const websocketStable = stableProjection(websocket);
  const expected = {
    sourceCommit: string(manifest, "source_commit", "package manifest"),
    referenceCommit: string(manifest, "reference_commit", "package manifest"),
    appElfSha256: string(manifest, "app_elf_sha256", "package manifest"),
  };
  if (apiStable["sourceCommit"] !== expected.sourceCommit
    || apiStable["referenceCommit"] !== expected.referenceCommit
    || apiStable["appElfSha256"] !== expected.appElfSha256) {
    throw failure("evidence_invalid", "operator snapshot does not match the exact package");
  }
  if (apiSession !== websocketSession || websocketRevision < apiRevision) {
    throw failure("evidence_invalid", "operator snapshot epoch identity is incoherent");
  }
  if (JSON.stringify(apiStable) !== JSON.stringify(websocketStable)) {
    throw failure("evidence_invalid", "operator snapshot substantive projections differ");
  }
  if (apiStable["miningActivity"] === "active") {
    throw failure("evidence_invalid", "operator snapshot safe operator state is not confirmed");
  }
  const apiDocument = path.join(epochRoot, "api-validation.private.txt");
  const websocketDocument = path.join(epochRoot, "websocket-validation.private.txt");
  await privateWrite(apiDocument, `system_info_json: ${JSON.stringify(api)}\noperator_snapshot_boot_session: ${apiSession}\noperator_snapshot_revision: ${String(apiRevision)}\n`);
  await privateWrite(websocketDocument, `live_websocket_json: ${JSON.stringify(websocket)}\noperator_snapshot_boot_session: ${websocketSession}\noperator_snapshot_revision: ${String(websocketRevision)}\n`);
  const validation = await runChild(processPort, parityProgram, [
    "validate-operator-snapshot",
    "--api-document", apiDocument,
    "--websocket-document", websocketDocument,
    "--retained-log", retainedPath,
  ], "operator snapshot validator");
  if (validation.timedOut) throw failure("timeout", "operator snapshot validation timed out");
  if (validation.exitCode !== 0) throw failure("evidence_invalid", "operator snapshot retained join is invalid");
  return {
    session: apiSession,
    ordinal: positiveInteger(api, "bootOrdinal", "API snapshot"),
    evidence: {
      boot_session_sha256: sha256(apiSession),
      http_snapshot_observed: true,
      websocket_snapshot_observed: true,
      same_boot_session: true,
      http_revision: apiRevision,
      websocket_revision: websocketRevision,
      websocket_revision_not_earlier: true,
      retained_log_marker_matches_http: true,
      retained_log_marker_matches_websocket: true,
      substantive_fields_present: true,
      stable_fields_match: true,
      safe_operator_state_confirmed: true,
      substantive_projection_sha256: sha256(JSON.stringify(apiStable)),
    },
  };
}

async function recover(
  processPort: ProcessPort,
  flashProgram: string,
  options: OperatorSnapshotEvidenceOptions,
  manifestPath: string,
  credentialsPath: string,
): Promise<RecoveryFacts> {
  try {
    const outcome = await processPort.run(flashCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest: manifestPath,
      wifiCredentials: credentialsPath,
    }));
    const complete = !outcome.timedOut && outcome.exitCode === 0;
    return { recovery_complete: complete, recovery_flash_used: true, secondary_recovery_failure: !complete };
  } catch {
    return { recovery_complete: false, recovery_flash_used: true, secondary_recovery_failure: true };
  }
}

export async function captureOperatorSnapshotEvidence(
  workspaceRoot: string,
  options: OperatorSnapshotEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  parityProgram: string,
  deviceSessionProgram: string,
  validatorProgram: string,
  maybeWebSocketFactory?: WebSocketFactory,
): Promise<OperatorSnapshotEvidence> {
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
    for (const field of ["source_commit", "reference_commit", "app_elf_sha256"]) {
      const digest = string(manifest, field, "package manifest");
      const expectedLength = field === "app_elf_sha256" ? 64 : 40;
      if (digest.length !== expectedLength || !/^[0-9a-f]+$/u.test(digest)) {
        throw new Error("package identity digest is invalid");
      }
    }
  } catch {
    throw failure("evidence_invalid", "package manifest identity is invalid");
  }
  const manifestDigest = sha256(manifestDocument);
  let exactPackageEffect = false;
  try {
    const initialRoot = path.join(privateRoot, "initial");
    await mkdir(initialRoot, { mode: 0o700 });
    exactPackageEffect = true;
    const initial = await runChild(processPort, flashProgram, flashMonitorCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest: manifestPath,
      wifiCredentials: credentialsPath,
      captureTimeoutSeconds: options.captureTimeoutSeconds,
      evidenceMode: "dual",
      evidenceDir: initialRoot,
    }).args, "exact-package flash-monitor");
    if (initial.timedOut) throw failure("timeout", "exact-package flash-monitor timed out");
    if (initial.exitCode !== 0) throw failure("process_failed", "exact-package flash-monitor failed");
    const trace = path.join(initialRoot, "flash-monitor.classifier-input.log");
    const baselineClassification = await classifyBaseline(processPort, parityProgram, trace);
    const origin = new URL(string(baselineClassification, "device_url", "baseline classification"));
    const baseline = await captureEpoch(
      origin,
      path.join(privateRoot, "baseline-epoch"),
      manifest,
      parityProgram,
      processPort,
      maybeWebSocketFactory,
    );
    if (baseline.session !== string(baselineClassification, "session", "baseline classification")
      || baseline.ordinal !== positiveInteger(baselineClassification, "boot_ordinal", "baseline classification")) {
      throw failure("evidence_invalid", "baseline API and serial identities differ");
    }
    const baselineApi = object(JSON.parse(await readFile(path.join(privateRoot, "baseline-epoch", "api.private.json"), "utf8")), "baseline API");
    const hostname = string(baselineApi, "hostname", "baseline API");
    const intent = path.join(privateRoot, "device-session-intent.private.json");
    const sessionRoot = path.join(privateRoot, "device-session");
    const sessionProjectionPath = path.join(privateRoot, "device-session-projection.private.json");
    await mkdir(sessionRoot, { mode: 0o700 });
    await privateJson(intent, {
      schema_version: "esp-device-session-reboot-intent-v1",
      board_category: "205",
      trusted_origin: origin.origin,
      baseline: {
        boot_session: baseline.session,
        boot_ordinal: baseline.ordinal,
        source_commit: string(manifest, "source_commit", "package manifest"),
        reference_commit: string(manifest, "reference_commit", "package manifest"),
        app_elf_sha256: string(manifest, "app_elf_sha256", "package manifest"),
      },
      expected_postcondition: { hostname_sha256: sha256(hostname) },
    });
    const sessionOutcome = await runChild(processPort, deviceSessionProgram, [
      "reboot-live", "--port", options.port, "--intent-input", intent,
      "--private-root", sessionRoot, "--projection-output", sessionProjectionPath,
      "--timeout-seconds", String(options.captureTimeoutSeconds),
    ], "device-session");
    if (sessionOutcome.timedOut) throw failure("timeout", "device-session timed out");
    const sessionProjection = await readDeviceSession(sessionProjectionPath);
    if (sessionOutcome.exitCode !== 0) throw failure("hardware_blocked", "device-session child failed after ready projection");
    const postRestart = await captureEpoch(
      origin,
      path.join(privateRoot, "post-restart-epoch"),
      manifest,
      parityProgram,
      processPort,
      maybeWebSocketFactory,
    );
    if (postRestart.session === baseline.session || postRestart.ordinal !== baseline.ordinal + 1) {
      throw failure("evidence_invalid", "post-restart epoch did not advance exactly once");
    }
    const evidence: OperatorSnapshotEvidence = {
      schema_version: "bitaxe-operator-snapshot-evidence-v1",
      board: 205,
      source_commit: string(manifest, "source_commit", "package manifest"),
      reference_commit: string(manifest, "reference_commit", "package manifest"),
      package_manifest_sha256: manifestDigest,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-operator-snapshot-evidence",
        request_sha256: sha256(JSON.stringify({ manifest: manifestDigest, timeout: options.captureTimeoutSeconds })),
      },
      baseline_epoch: baseline.evidence,
      post_restart_epoch: postRestart.evidence,
      distinct_boot_sessions: true,
      restart_session: sessionProjection,
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      redaction_status: "passed",
    };
    const privateCandidate = path.join(privateRoot, "final-evidence.private.json");
    await privateJson(privateCandidate, evidence);
    const validation = await runChild(processPort, validatorProgram, [privateCandidate], "operator snapshot evidence validator");
    if (validation.timedOut) throw failure("timeout", "operator snapshot evidence validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "operator snapshot evidence validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof OperatorSnapshotEvidenceError
      ? error
      : failure("process_failed", "operator snapshot orchestration failed");
    if (!exactPackageEffect) throw primary;
    throw primary.withRecovery(await recover(processPort, flashProgram, options, manifestPath, credentialsPath));
  }
}
