import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type LogBufferEvidence,
} from "./contracts.generated.js";
import {
  factoryImageDigest,
  flashChildFailureFacts,
  flashEffectEnvironment,
  inspectFlashEffect,
} from "./flash-child-diagnostics.js";
import {
  fetchJsonFromSameOrigin,
  fetchTextResponseFromSameOrigin,
  type SameOriginTextResponse,
  uniqueRuntimeOrigin,
} from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { verifySemanticEvidenceRedaction } from "./redaction.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import {
  captureTextWebSocketFrame,
  WebSocketProtocolError,
  type WebSocketFactory,
} from "./websocket.js";
import { assertWithinWorkspace } from "./workspace.js";

export type LogBufferEvidenceOptions = {
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

const rawLogMarker = "axeos_websocket_logs=connected\n";
const expectedContentType = "text/plain";
const expectedContentDisposition = "attachment; filename=\"bitaxe-logs.txt\"";

export class LogBufferEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "LogBufferEvidenceError";
  }

  public withCompletedFlash(): LogBufferEvidenceError {
    return new LogBufferEvidenceError(this.category, this.message, {
      ...this.publicValue,
      flash_effect_completed: true,
    });
  }
}

function failure(
  category: FailureCategory,
  message: string,
  facts: Readonly<Record<string, unknown>> = {},
): LogBufferEvidenceError {
  return new LogBufferEvidenceError(category, message, {
    stage: "log_buffer_capture",
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

function monitorBootSession(document: string): string {
  const sessions = new Set<string>();
  for (const match of document.matchAll(/\bruntime_boot_identity session=([0-9a-f]{32})\b/gu)) {
    const maybeSession = match[1];
    if (maybeSession !== undefined) sessions.add(maybeSession);
  }
  if (sessions.size !== 1) {
    throw failure("hardware_blocked", "monitor capture lacks one stable boot session");
  }
  const [session] = sessions;
  if (session === undefined) {
    throw failure("hardware_blocked", "monitor capture boot session is missing");
  }
  return session;
}

function validateRuntimeIdentity(
  systemInfo: JsonObject,
  manifest: JsonObject,
  expectedBootSession: string,
): void {
  for (const [wireField, manifestField] of [
    ["sourceCommit", "source_commit"],
    ["referenceCommit", "reference_commit"],
    ["appElfSha256", "app_elf_sha256"],
  ] as const) {
    if (
      requiredString(systemInfo, wireField, "system info")
      !== requiredString(manifest, manifestField, "package manifest")
    ) {
      throw failure("evidence_invalid", "runtime identity does not match the exact package");
    }
  }
  if (requiredString(systemInfo, "bootSession", "system info") !== expectedBootSession) {
    throw failure("evidence_invalid", "runtime boot session does not match the admitted monitor session");
  }
}

function markerCount(document: string): number {
  return document.split(/\r?\n/gu).filter((line) => line === rawLogMarker.trimEnd()).length;
}

function headersMatch(response: SameOriginTextResponse): boolean {
  return response.contentType === expectedContentType
    && response.contentDisposition === expectedContentDisposition;
}

function validateCorrelation(
  baseline: SameOriginTextResponse,
  final: SameOriginTextResponse,
  frame: string,
): { readonly baselineMarkerCount: number; readonly finalMarkerCount: number } {
  if (!headersMatch(baseline) || !headersMatch(final)) {
    throw failure("evidence_invalid", "retained log download headers do not match");
  }
  if (frame !== rawLogMarker) {
    throw failure("evidence_invalid", "raw log WebSocket frame does not match the exact marker");
  }
  if (!final.body.startsWith(baseline.body)) {
    throw failure("evidence_invalid", "final retained log does not preserve the baseline prefix");
  }
  const baselineMarkerCount = markerCount(baseline.body);
  const finalMarkerCount = markerCount(final.body);
  if (finalMarkerCount !== baselineMarkerCount + 1) {
    throw failure("evidence_invalid", "retained log does not contain exactly one new marker");
  }
  return { baselineMarkerCount, finalMarkerCount };
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof LogBufferEvidenceError) throw error;
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

async function systemInfo(origin: URL, output: string): Promise<JsonObject> {
  try {
    return object(await fetchJsonFromSameOrigin(origin, "/api/system/info", output), "system info");
  } catch (error) {
    if (error instanceof LogBufferEvidenceError) throw error;
    throw failure("hardware_blocked", "same-origin system info request failed");
  }
}

async function retainedLog(origin: URL, output: string): Promise<SameOriginTextResponse> {
  try {
    return await fetchTextResponseFromSameOrigin(origin, "/api/system/logs", output);
  } catch {
    throw failure("hardware_blocked", "same-origin retained log request failed");
  }
}

async function rawLogFrame(
  origin: URL,
  output: string,
  maybeWebSocketFactory: WebSocketFactory | undefined,
): Promise<string> {
  try {
    return await captureTextWebSocketFrame(origin, "/api/ws", output, maybeWebSocketFactory);
  } catch (error) {
    if (error instanceof WebSocketProtocolError) {
      throw failure("evidence_invalid", "raw log WebSocket frame is invalid");
    }
    throw failure("hardware_blocked", "same-origin raw log WebSocket request failed");
  }
}

export async function captureLogBufferEvidence(
  workspaceRoot: string,
  options: LogBufferEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
  maybeWebSocketFactory?: WebSocketFactory,
): Promise<LogBufferEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await access(manifestPath);
  await access(credentialsPath);
  await createPrivateRoot(privateRoot);

  const manifestDocument = await readFile(manifestPath, "utf8");
  let manifest: JsonObject;
  let factoryDigest: string;
  try {
    manifest = object(JSON.parse(manifestDocument), "package manifest");
    requiredString(manifest, "source_commit", "package manifest");
    requiredString(manifest, "reference_commit", "package manifest");
    requiredString(manifest, "app_elf_sha256", "package manifest");
    factoryDigest = factoryImageDigest(manifest);
  } catch (error) {
    if (error instanceof LogBufferEvidenceError) throw error;
    throw failure("evidence_invalid", "package manifest identity is invalid");
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
    throw failure(
      "process_failed",
      "exact-package flash-monitor launch failed",
      flashChildFailureFacts(undefined, effect),
    );
  }
  const effect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
  const effectFacts = flashChildFailureFacts(flashOutcome, effect);
  if (flashOutcome.timedOut) {
    throw failure("timeout", "exact-package flash-monitor timed out", effectFacts);
  }
  if (flashOutcome.exitCode !== 0) {
    throw failure("hardware_blocked", "exact-package flash-monitor did not reach readiness", effectFacts);
  }
  if (effect.flash_effect_result_status !== "valid" || effect.flash_effect_status !== "completed") {
    throw failure("evidence_invalid", "exact-package flash effect result is invalid", effectFacts);
  }

  try {
    const monitor = await readFile(path.join(privateRoot, "flash-monitor.classifier-input.log"), "utf8");
    if (!hasPassiveSafeState(monitor)) {
      throw failure("hardware_blocked", "boot lacks passive safe-state evidence");
    }
    const bootSession = monitorBootSession(monitor);
    let origin: URL;
    try {
      origin = uniqueRuntimeOrigin(monitor);
    } catch {
      throw failure("hardware_blocked", "runtime origin admission is invalid");
    }
    const runtime = await systemInfo(origin, path.join(privateRoot, "system-info.private.json"));
    validateRuntimeIdentity(runtime, manifest, bootSession);
    const baseline = await retainedLog(origin, path.join(privateRoot, "baseline-log.private.txt"));
    const frame = await rawLogFrame(
      origin,
      path.join(privateRoot, "raw-log-frame.private.txt"),
      maybeWebSocketFactory,
    );
    const final = await retainedLog(origin, path.join(privateRoot, "final-log.private.txt"));
    const counts = validateCorrelation(baseline, final, frame);
    const evidence: LogBufferEvidence = {
      schema_version: "bitaxe-log-buffer-evidence-v1",
      board: 205,
      source_commit: requiredString(manifest, "source_commit", "package manifest"),
      reference_commit: requiredString(manifest, "reference_commit", "package manifest"),
      package_manifest_sha256: manifestDigest,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-log-buffer-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: manifestDigest,
          routes: ["/api/system/info", "/api/system/logs", "/api/ws"],
          timeout: options.captureTimeoutSeconds,
        })),
      },
      detector_admitted: true,
      boot_observed: true,
      same_origin_observed: true,
      log_buffer: {
        boot_session_sha256: sha256(bootSession),
        baseline_body_sha256: sha256(baseline.body),
        final_body_sha256: sha256(final.body),
        raw_frame_sha256: sha256(frame),
        baseline_bytes: Buffer.byteLength(baseline.body, "utf8"),
        final_bytes: Buffer.byteLength(final.body, "utf8"),
        raw_frame_bytes: Buffer.byteLength(frame, "utf8"),
        baseline_marker_count: counts.baselineMarkerCount,
        final_marker_count: counts.finalMarkerCount,
        new_marker_count: 1,
        both_download_headers_match: true,
        baseline_is_exact_prefix: true,
        raw_frame_plain_text: true,
        raw_frame_marker_matches: true,
        retained_marker_matches_frame: true,
      },
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      redaction_status: "passed",
    };
    const privateCandidate = path.join(privateRoot, "final-evidence.private.json");
    await privateJson(privateCandidate, evidence);
    await verifySemanticEvidenceRedaction(privateRoot);
    const validation = await child(
      processPort,
      validatorProgram,
      [privateCandidate],
      "log buffer evidence validator",
    );
    if (validation.timedOut) {
      throw failure("timeout", "log buffer evidence validation timed out");
    }
    if (validation.exitCode !== 0) {
      throw failure("evidence_invalid", "log buffer evidence validation failed");
    }
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
    });
    return evidence;
  } catch (error) {
    const primary = error instanceof LogBufferEvidenceError
      ? error
      : failure("evidence_invalid", "log buffer orchestration evidence is invalid");
    throw primary.withCompletedFlash();
  }
}
