import { createHash, randomBytes } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { internalCommandSpec, type AutomationCategory } from "./contracts.generated.js";
import {
  campaignRecoveryFactsFromDocuments,
  type RecoveryFacts,
} from "./api-command-effects-recovery.js";
import { isDeviceSessionProjectionFailure, readClosedDeviceSession } from "./device-session-projection.js";
import {
  OperatorCheckpointError,
  superviseOperatorCheckpoints,
  type OperatorCheckpointSink,
} from "./api-command-effects-checkpoint.js";
import { isClosedReadinessTransition } from "./api-command-effects-readiness.js";
import type { ProcessLifetime, ProcessOutcome, ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<AutomationCategory, "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed">;
type FixtureSettlement =
  | { readonly kind: "launch_failed" }
  | { readonly kind: "outcome"; readonly outcome: ProcessOutcome };

export type ApiCommandEffectsOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly durationSeconds: number;
};

export class ApiCommandEffectsError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "ApiCommandEffectsError";
  }
}

function failure(
  category: FailureCategory,
  message: string,
  recovery: RecoveryFacts = {
    safeStopConfirmed: false,
    cleanupComplete: false,
    recoveryAttempted: false,
    secondaryRecoveryFailure: false,
  },
): ApiCommandEffectsError {
  return new ApiCommandEffectsError(category, message, {
    stage: "command_effects",
    safe_stop_confirmed: recovery.safeStopConfirmed,
    cleanup_complete: recovery.cleanupComplete,
    recovery_attempted: recovery.recoveryAttempted,
    secondary_recovery_failure: recovery.secondaryRecoveryFailure,
  });
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as JsonObject;
}

function stringField(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") {
    throw failure("evidence_invalid", `${context} ${field} is invalid`);
  }
  return candidate;
}

async function requireAbsentPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof ApiCommandEffectsError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(root, { mode: 0o700, recursive: true });
  await chmod(root, 0o700);
}

async function writePrivateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o600,
  });
  await chmod(output, 0o600);
}

async function readPrivateDocument(
  input: string,
  context: string,
): Promise<{ readonly document: string; readonly value: JsonObject }> {
  const metadata = await stat(input);
  if (!metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
    throw failure("evidence_invalid", `${context} is not a private regular file`);
  }
  const document = await readFile(input, "utf8");
  return { document, value: object(JSON.parse(document), context) };
}

async function readRequiredPrivateDocument(
  input: string,
  context: string,
): Promise<{ readonly document: string; readonly value: JsonObject }> {
  try {
    return await readPrivateDocument(input, context);
  } catch (error) {
    if (error instanceof ApiCommandEffectsError) throw error;
    throw failure("evidence_invalid", `${context} is unavailable or malformed`);
  }
}

async function readPrivateJson(input: string, context: string): Promise<JsonObject> {
  return (await readPrivateDocument(input, context)).value;
}

async function campaignRecoveryFacts(campaignRoot: string): Promise<RecoveryFacts> {
  try {
    const result = await readPrivateJson(path.join(campaignRoot, "campaign-result.json"), "campaign result");
    const network = await readPrivateJson(path.join(campaignRoot, "campaign-network.private.json"), "campaign network evidence");
    return campaignRecoveryFactsFromDocuments(result, network);
  } catch {
    return {
      safeStopConfirmed: false,
      cleanupComplete: false,
      recoveryAttempted: false,
      secondaryRecoveryFailure: false,
    };
  }
}

async function runChild(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
  lifetime: ProcessLifetime,
  context: string,
): Promise<ProcessOutcome> {
  try {
    return await processPort.run(
      internalCommandSpec(program, [...args], (value) => value),
      lifetime,
    );
  } catch {
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function localFixtureHost(processPort: ProcessPort): Promise<string> {
  const route = await runChild(processPort, "/sbin/route", ["-n", "get", "default"], 5_000, "route discovery");
  if (route.timedOut || route.exitCode !== 0) throw failure("hardware_blocked", "local fixture route unavailable");
  const matches = [...route.stdout.matchAll(/^\s*interface:\s*([A-Za-z0-9._-]+)\s*$/gmu)];
  const maybeInterface = matches.length === 1 ? matches[0]?.[1] : undefined;
  if (maybeInterface === undefined) throw failure("hardware_blocked", "local fixture interface is ambiguous");
  const address = await runChild(
    processPort,
    "/usr/sbin/ipconfig",
    ["getifaddr", maybeInterface],
    5_000,
    "address discovery",
  );
  const host = address.stdout.trim();
  if (address.timedOut || address.exitCode !== 0 || !/^\d{1,3}(?:\.\d{1,3}){3}$/u.test(host)) {
    throw failure("hardware_blocked", "local fixture address unavailable");
  }
  return host;
}

async function waitForPrivateJson(input: string, timeoutMillis: number): Promise<JsonObject> {
  const deadline = Date.now() + timeoutMillis;
  while (Date.now() < deadline) {
    try {
      return await readPrivateJson(input, "fixture readiness");
    } catch (error) {
      if (error instanceof ApiCommandEffectsError && error.message.includes("private regular file")) {
        await new Promise((resolve) => setTimeout(resolve, 100));
        continue;
      }
      if ((error as NodeJS.ErrnoException).code === "ENOENT") {
        await new Promise((resolve) => setTimeout(resolve, 100));
        continue;
      }
      throw error;
    }
  }
  throw failure("timeout", "local fixture readiness timed out");
}

function launchFixture(
  processPort: ProcessPort,
  fixtureProgram: string,
  args: readonly string[],
): Promise<FixtureSettlement> {
  try {
    return processPort.run(
      internalCommandSpec(fixtureProgram, [...args], (value) => value, { BAZEL_BINDIR: "." }),
      "operator-gated",
    ).then(
      (outcome) => ({ kind: "outcome", outcome }),
      () => ({ kind: "launch_failed" }),
    );
  } catch {
    return Promise.resolve({ kind: "launch_failed" });
  }
}

function fixtureFailure(settlement: FixtureSettlement): ApiCommandEffectsError {
  if (settlement.kind === "outcome" && settlement.outcome.timedOut) {
    return failure("timeout", "local fixture process timed out");
  }
  return failure("process_failed", "local fixture process failed before readiness");
}

async function writeFixtureDiagnostic(
  output: string,
  settlement: FixtureSettlement,
  durationMillis: number,
): Promise<void> {
  const maybeOutcome = settlement.kind === "outcome" ? settlement.outcome : undefined;
  const terminalCategory = settlement.kind === "launch_failed"
    ? "launch_failed"
    : maybeOutcome?.timedOut === true
      ? "timeout"
      : maybeOutcome?.exitCode === 0
        ? "complete"
        : "nonzero_exit";
  await writePrivateJson(output, {
    schema_version: "api-command-effects-fixture-process-v1",
    terminal_category: terminalCategory,
    exit_code: maybeOutcome?.exitCode ?? null,
    timed_out: maybeOutcome?.timedOut ?? false,
    duration_millis: durationMillis,
    stdout_byte_count: Buffer.byteLength(maybeOutcome?.stdout ?? "", "utf8"),
    stderr_byte_count: Buffer.byteLength(maybeOutcome?.stderr ?? "", "utf8"),
    stdout_sha256: sha256(maybeOutcome?.stdout ?? ""),
    stderr_sha256: sha256(maybeOutcome?.stderr ?? ""),
    raw_output_persisted: false,
  });
}

function validatedCommandEffects(network: JsonObject): JsonObject {
  if (network["status"] !== "accepted") throw failure("hardware_blocked", "command effects network result was not accepted");
  const effects = object(network["command_effects"], "command effects evidence");
  const requiredTrue = [
    "genuine_block_notification_observed", "positive_block_count_observed",
    "pause_confirmed", "resume_confirmed", "identify_operator_ready_confirmed",
    "identify_rendered_confirmed",
    "identify_cleared_confirmed", "dismiss_confirmed", "block_count_preserved",
    "active_before_pause", "active_after_resume", "same_boot_and_package",
    "safety_valid", "terminal_http_valid", "terminal_pool_persisted",
  ];
  if (
    effects["schema"] !== "mining-campaign-command-effects-v3"
    || effects["identify_terminal_outcome"] !== "none"
    || requiredTrue.some((field) => effects[field] !== true)
    || effects["pause_request_count"] !== 1
    || effects["resume_request_count"] !== 1
    || effects["identify_request_count"] !== 2
    || effects["dismiss_request_count"] !== 1
  ) {
    throw failure("evidence_invalid", "command effects evidence quorum is incomplete");
  }
  return effects;
}

function validReadyFlashDiagnostic(value: unknown): boolean {
  const diagnostic = object(value, "flash command diagnostic");
  const stdoutBytes = diagnostic["stdout_bytes"];
  const stderrBytes = diagnostic["stderr_bytes"];
  const stdoutSha256 = diagnostic["stdout_sha256"];
  const stderrSha256 = diagnostic["stderr_sha256"];
  return diagnostic["schema_version"] === "esp-usb-command-diagnostic-v1"
    && diagnostic["terminal_category"] === "ready"
    && diagnostic["device_effect_state"] === "completed"
    && diagnostic["termination"] === "exited_success"
    && diagnostic["attempt_count"] === 1
    && diagnostic["connection_signature"] === "not_applicable"
    && typeof stdoutBytes === "number"
    && Number.isSafeInteger(stdoutBytes)
    && stdoutBytes >= 0
    && typeof stderrBytes === "number"
    && Number.isSafeInteger(stderrBytes)
    && stderrBytes >= 0
    && typeof stdoutSha256 === "string"
    && /^[0-9a-f]{64}$/u.test(stdoutSha256)
    && typeof stderrSha256 === "string"
    && /^[0-9a-f]{64}$/u.test(stderrSha256)
    && diagnostic["transfer_started"] === true
    && diagnostic["transfer_completed"] === true
    && diagnostic["raw_output_included"] === false;
}

function validateFlashDiagnostics(
  result: JsonObject,
  flashDiagnostics: JsonObject,
  flashDiagnosticsDocument: string,
): void {
  const flashDiagnosticsSha256 = result["flash_diagnostics_sha256"];
  if (
    typeof flashDiagnosticsSha256 !== "string"
    || !/^[0-9a-f]{64}$/u.test(flashDiagnosticsSha256)
    || flashDiagnosticsSha256 !== sha256(flashDiagnosticsDocument)
    || flashDiagnostics["schema"] !== "mining-campaign-flash-diagnostics-v1"
    || !validReadyFlashDiagnostic(flashDiagnostics["factory"])
    || !validReadyFlashDiagnostic(flashDiagnostics["nvs"])
    || flashDiagnostics["raw_output_included"] !== false
  ) {
    throw failure("evidence_invalid", "campaign flash diagnostics are incomplete");
  }
}

function validateCampaign(
  result: JsonObject,
  network: JsonObject,
  flashDiagnostics: JsonObject,
  flashDiagnosticsDocument: string,
): JsonObject {
  validateFlashDiagnostics(result, flashDiagnostics, flashDiagnosticsDocument);
  if (!isClosedReadinessTransition(result["readiness_transition"])) {
    throw failure("evidence_invalid", "readiness transition is incomplete");
  }
  const qualifiedCandidateCount = result["qualified_candidate_count"];
  if (
    result["schema"] !== "mining-campaign-result-v8"
    || result["stage"] !== "command-effects"
    || result["status"] !== "accepted"
    || result["terminal_category"] !== "command_effects_complete"
    || result["runtime_identity"] !== "trusted"
    || result["protocol_gate"] !== "ready"
    || result["safe_stop"] !== "confirmed"
    || result["usb_cleanup"] !== "ready"
    || typeof qualifiedCandidateCount !== "number"
    || !Number.isSafeInteger(qualifiedCandidateCount)
    || qualifiedCandidateCount < 1
    || result["redacted"] !== true
  ) {
    throw failure("hardware_blocked", "command effects campaign result was not accepted");
  }
  return validatedCommandEffects(network);
}

function validateFixture(report: JsonObject): JsonObject {
  const counts = object(report["method_counts"], "fixture method counts");
  const requiredCounts = [
    counts["mining.configure"], counts["mining.subscribe"], counts["mining.authorize"],
    counts["mining.submit"], report["notify_sent_count"], report["accepted_submit_count"],
  ];
  if (
    report["fixture"] !== "api-command-effects-v1"
    || report["configure_observed"] !== true
    || report["subscribe_observed"] !== true
    || report["authorize_observed"] !== true
    || report["submit_observed"] !== true
    || report["raw_messages_committed"] !== false
    || report["credential_contents_read"] !== false
    || report["compact_network_target"] !== "207fffff"
    || requiredCounts.some((count) => typeof count !== "number" || !Number.isSafeInteger(count) || count < 1)
    || typeof report["source_work_fingerprint"] !== "string"
    || !/^[0-9a-f]{64}$/u.test(report["source_work_fingerprint"])
  ) {
    throw failure("evidence_invalid", "local fixture report is invalid");
  }
  return {
    fixture: "api-command-effects-v1",
    configure_count: counts["mining.configure"],
    subscribe_count: counts["mining.subscribe"],
    authorize_count: counts["mining.authorize"],
    submit_count: counts["mining.submit"],
    notify_sent_count: report["notify_sent_count"],
    accepted_submit_count: report["accepted_submit_count"],
    source_work_sha256: report["source_work_fingerprint"],
    compact_network_target: "207fffff",
    raw_messages_committed: false,
    credentials_exposed: false,
  };
}

export async function captureApiCommandEffects(
  workspaceRoot: string,
  options: ApiCommandEffectsOptions,
  processPort: ProcessPort,
  fixtureProgram: string,
  flashProgram: string,
  deviceSessionProgram: string,
  checkpointSink: OperatorCheckpointSink,
): Promise<unknown> {
  if (options.durationSeconds !== 600) throw failure("evidence_invalid", "command effects duration must be 600 seconds");
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await access(manifestPath);
  await access(credentialsPath);
  await requireAbsentPrivateRoot(privateRoot);

  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const sourceCommit = stringField(manifest, "source_commit", "package manifest");
  const referenceCommit = stringField(manifest, "reference_commit", "package manifest");
  stringField(manifest, "app_elf_sha256", "package manifest");
  const manifestDigest = sha256(manifestDocument);
  const fixtureHost = await localFixtureHost(processPort);
  const fixtureReady = path.join(privateRoot, "fixture-ready.private.json");
  const fixtureReport = path.join(privateRoot, "fixture-report.private.json");
  const fixtureStop = path.join(privateRoot, "fixture.stop.private");
  const fixtureDiagnostic = path.join(privateRoot, "fixture-process.private.json");
  const poolCredentials = path.join(privateRoot, "pool-credentials.private.json");
  const fixtureStartedAt = Date.now();
  const fixturePromise = launchFixture(processPort, fixtureProgram, [
    "--host", fixtureHost,
    "--port", "0",
    "--fixture", "api-command-effects-v1",
    "--session-label", "command-effects",
    "--ready-json", fixtureReady,
    "--report-json", fixtureReport,
    "--lifetime", "operator-gated",
    "--stop-file", fixtureStop,
  ]);

  let maybeCampaignOutcome: ProcessOutcome | undefined;
  let maybeFixtureSettlement: FixtureSettlement | undefined;
  let maybePrimaryError: unknown;
  let primaryFailed = false;
  let diagnosticWriteFailed = false;
  try {
    const first = await Promise.race([
      waitForPrivateJson(fixtureReady, 10_000).then((ready) => ({ kind: "ready", ready }) as const),
      fixturePromise,
    ]);
    if (first.kind !== "ready") {
      maybeFixtureSettlement = first;
      throw fixtureFailure(first);
    }
    const ready = first.ready;
    if (ready["status"] !== "ready" || ready["fixture"] !== "api-command-effects-v1") {
      throw failure("evidence_invalid", "local fixture readiness is invalid");
    }
    const fixturePort = ready["bound_port"];
    if (typeof fixturePort !== "number" || !Number.isSafeInteger(fixturePort) || fixturePort < 1 || fixturePort > 65535) {
      throw failure("evidence_invalid", "local fixture port is invalid");
    }
    await writePrivateJson(poolCredentials, {
      poolURL: fixtureHost,
      poolPort: fixturePort,
      poolUser: "api009.fixture",
      poolPassword: randomBytes(24).toString("hex"),
    });
    const campaignRoot = path.join(privateRoot, "campaign");
    const supervised = await superviseOperatorCheckpoints(runChild(processPort, flashProgram, [
      "mining-campaign",
      "--stage", "command-effects",
      "--profile", "conservative",
      "--board", "205",
      "--port", options.port,
      "--manifest", manifestPath,
      "--wifi-credentials", credentialsPath,
      "--pool-credentials", poolCredentials,
      "--evidence-dir", campaignRoot,
      "--duration-seconds", String(options.durationSeconds),
      "--redact-evidence",
    ], "operator-gated", "command effects campaign"), campaignRoot, checkpointSink);
    maybeCampaignOutcome = supervised.outcome;
    if (maybeCampaignOutcome.timedOut) {
      throw failure(
        "timeout",
        "command effects campaign timed out",
        await campaignRecoveryFacts(campaignRoot),
      );
    }
    if (maybeCampaignOutcome.exitCode !== 0) {
      throw failure(
        "hardware_blocked",
        "command effects campaign failed",
        await campaignRecoveryFacts(campaignRoot),
      );
    }
    if (supervised.maybeCheckpointError instanceof OperatorCheckpointError) {
      throw failure("evidence_invalid", "operator checkpoint handoff is invalid");
    }
  } catch (error) {
    primaryFailed = true;
    maybePrimaryError = error;
  } finally {
    await writeFile(fixtureStop, "stop\n", { encoding: "utf8", flag: "wx", mode: 0o600 }).catch(() => undefined);
    maybeFixtureSettlement ??= await fixturePromise;
    try {
      await writeFixtureDiagnostic(
        fixtureDiagnostic,
        maybeFixtureSettlement,
        Date.now() - fixtureStartedAt,
      );
    } catch {
      diagnosticWriteFailed = true;
    }
  }

  if (primaryFailed) throw maybePrimaryError;
  if (maybeFixtureSettlement === undefined) throw failure("process_failed", "local fixture process did not complete");
  if (maybeFixtureSettlement.kind !== "outcome") throw fixtureFailure(maybeFixtureSettlement);
  const fixtureOutcome = maybeFixtureSettlement.outcome;
  if (fixtureOutcome.timedOut) throw failure("timeout", "local fixture process timed out");
  if (fixtureOutcome.exitCode !== 0) {
    throw failure("process_failed", "local fixture process failed");
  }
  if (diagnosticWriteFailed) throw failure("evidence_invalid", "local fixture process diagnostic is unavailable");
  const campaignOutcome = maybeCampaignOutcome;
  if (campaignOutcome === undefined) throw failure("process_failed", "command effects campaign did not complete");
  const campaignRoot = path.join(privateRoot, "campaign");
  const campaignResult = await readPrivateJson(path.join(campaignRoot, "campaign-result.json"), "campaign result");
  const network = await readPrivateJson(path.join(campaignRoot, "campaign-network.private.json"), "campaign network evidence");
  const flashDiagnosticsPath = path.join(campaignRoot, "campaign-flash.private.json");
  const flashDiagnostics = await readRequiredPrivateDocument(flashDiagnosticsPath, "campaign flash diagnostics");
  const effects = validateCampaign(campaignResult, network, flashDiagnostics.value, flashDiagnostics.document);
  const fixture = validateFixture(await readPrivateJson(fixtureReport, "fixture report"));

  const intentPath = path.join(campaignRoot, "command-effects-reboot-intent.private.json");
  await readPrivateJson(intentPath, "device-session intent");
  const sessionRoot = path.join(privateRoot, "device-session");
  const sessionProjectionPath = path.join(privateRoot, "device-session-projection.private.json");
  await mkdir(sessionRoot, { mode: 0o700 });
  await chmod(sessionRoot, 0o700);
  const sessionOutcome = await runChild(processPort, deviceSessionProgram, [
    "reboot-live",
    "--port", options.port,
    "--intent-input", intentPath,
    "--private-root", sessionRoot,
    "--projection-output", sessionProjectionPath,
    "--timeout-seconds", "360",
  ], 390_000, "device-session");
  if (sessionOutcome.timedOut) throw failure("timeout", "device-session timed out");
  let restartSession: JsonObject;
  try {
    restartSession = await readClosedDeviceSession(sessionProjectionPath);
  } catch (error) {
    if (isDeviceSessionProjectionFailure(error)) {
      throw failure(error.category, error.message);
    }
    throw failure("evidence_invalid", "device-session projection is invalid");
  }
  if (sessionOutcome.exitCode !== 0) throw failure("hardware_blocked", "device-session child failed");

  const evidence = {
    schema_version: "bitaxe-api-command-effects-evidence-v1",
    board: 205,
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    package_manifest_sha256: manifestDigest,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "api-command-effects-campaign",
      request_sha256: sha256(JSON.stringify({ manifest: manifestDigest, duration_seconds: options.durationSeconds })),
    },
    command_effects: effects,
    stratum_fixture: fixture,
    restart_session: restartSession,
    safe_stop_confirmed: true,
    cleanup_complete: true,
    recovery_attempted: false,
    secondary_recovery_failure: false,
    mining_state: "disabled",
    hardware_control_state: "disabled",
    redaction_status: "passed",
  } as const;
  await mkdir(path.dirname(projectionPath), { recursive: true });
  await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
  });
  return evidence;
}
