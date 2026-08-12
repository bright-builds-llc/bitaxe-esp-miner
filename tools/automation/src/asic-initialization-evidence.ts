import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AsicInitializationEvidence,
  type AutomationCategory,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type AsicInitializationEvidenceOptions = {
  readonly attemptRoot: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;
type JsonObject = Readonly<Record<string, unknown>>;

const expectedAttemptFiles = [
  "campaign-diagnostics.private.json",
  "campaign-observations.private.json",
  "campaign-result.json",
  "campaign-result.sha256",
] as const;

const initializationPaths = [
  "crates/bitaxe-asic/src/bm1366/init_plan.rs",
  "crates/bitaxe-asic/src/bm1366/mining_ready.rs",
  "firmware/bitaxe/src/asic_adapter/reset.rs",
  "firmware/bitaxe/src/asic_adapter/status.rs",
  "firmware/bitaxe/src/asic_adapter/uart.rs",
  "firmware/bitaxe/src/mining_actuation.rs",
  "firmware/bitaxe/src/mining_actuation_adapter.rs",
] as const;

export class AsicInitializationEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "AsicInitializationEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): AsicInitializationEvidenceError {
  return new AsicInitializationEvidenceError(category, message, {
    stage: "sealed_initialization_projection",
    hardware_rerun_used: false,
    projection_published: false,
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

function boolean(value: JsonObject, field: string, context: string): boolean {
  const candidate = value[field];
  if (typeof candidate !== "boolean") {
    throw failure("evidence_invalid", `${context} boolean field is invalid`);
  }
  return candidate;
}

function lowerHex(value: string, length: number): boolean {
  return value.length === length && new RegExp(`^[0-9a-f]{${String(length)}}$`, "u").test(value);
}

async function jsonFile(candidate: string, context: string): Promise<{ readonly document: string; readonly value: JsonObject }> {
  const document = await readFile(candidate, "utf8");
  try {
    return { document, value: object(JSON.parse(document), context) };
  } catch (error) {
    if (error instanceof AsicInitializationEvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
}

async function verifyProtectedModes(root: string): Promise<void> {
  if (((await stat(root)).mode & 0o777) !== 0o700) {
    throw failure("evidence_invalid", "protected attempt root mode is invalid");
  }
  const entries = (await readdir(root)).sort();
  if (entries.length !== expectedAttemptFiles.length
    || entries.some((entry, index) => entry !== expectedAttemptFiles[index])) {
    throw failure("evidence_invalid", "protected attempt file set is invalid");
  }
  for (const entry of entries) {
    const metadata = await stat(path.join(root, entry));
    if (!metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
      throw failure("evidence_invalid", "protected attempt file mode is invalid");
    }
  }
}

async function childText(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
  context: string,
): Promise<string> {
  try {
    const outcome = await processPort.run(internalCommandSpec(program, [...args], (value) => value));
    if (outcome.timedOut || outcome.exitCode !== 0) {
      throw failure("evidence_invalid", `${context} did not pass`);
    }
    return outcome.stdout.trim();
  } catch (error) {
    if (error instanceof AsicInitializationEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof AsicInitializationEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function validateArchivedTask(document: string, attemptSourceCommit: string): void {
  const heading = "### task-ultra205-accepted-pool-share |";
  const start = document.indexOf(heading);
  if (start === -1) throw failure("evidence_invalid", "accepted campaign task lineage is missing");
  const maybeEnd = document.indexOf("\n### ", start + heading.length);
  const block = document.slice(start, maybeEnd === -1 ? document.length : maybeEnd);
  const normalizedBlock = block.replace(/\s+/gu, " ");
  for (const required of [
    `Clean commit \`${attemptSourceCommit.slice(0, 8)}\` \`attempt-007\``,
    "completed every preparation boundary",
    "admitted the exact package and one Ultra 205",
    "confirmed safe stop",
    "USB cleanup ready",
    "parity promotion is false",
  ]) {
    if (!normalizedBlock.includes(required)) {
      throw failure("evidence_invalid", "accepted campaign task lineage is incomplete");
    }
  }
}

function validateCampaign(result: JsonObject, diagnostics: JsonObject): void {
  const latest = object(diagnostics["latest_preparation_event"], "latest preparation event");
  const campaignFailure = object(result["campaign_failure"], "campaign failure");
  const exactFacts = [
    string(result, "schema", "campaign result") === "mining-campaign-result-v2",
    string(result, "evidence_class", "campaign result") === "protected-operational",
    string(result, "stage", "campaign result") === "live-share",
    string(result, "profile", "campaign result") === "conservative",
    integer(result, "duration_seconds", "campaign result") === 600,
    string(result, "status", "campaign result") === "accepted",
    string(result, "terminal_category", "campaign result") === "submit_response_observed",
    boolean(result, "package_admitted", "campaign result"),
    string(result, "runtime_identity", "campaign result") === "trusted",
    string(result, "runtime_attestation_status", "campaign result") === "trusted",
    string(result, "serial_outcome_detail", "campaign result") === "clean",
    integer(result, "marker_count", "campaign result") > 0,
    string(result, "submit_outcome", "campaign result") === "accepted",
    integer(result, "qualified_candidate_count", "campaign result") >= 1,
    integer(result, "active_ms", "campaign result") > 0,
    string(result, "safety", "campaign result") === "fresh",
    result["mineonboot"] === false,
    string(result, "safe_stop", "campaign result") === "confirmed",
    string(result, "usb_cleanup", "campaign result") === "ready",
    string(result, "terminal_reason", "campaign result") === "campaign_lease_consumed",
    boolean(result, "redacted", "campaign result"),
    result["parity_promotion"] === false,
    campaignFailure["phase"] === "none",
    campaignFailure["step"] === "none",
    string(diagnostics, "schema", "campaign diagnostics") === "mining-campaign-serial-diagnostics-v1",
    boolean(diagnostics, "observation_started", "campaign diagnostics"),
    integer(diagnostics, "preparation_candidate_count", "campaign diagnostics") === 18,
    integer(diagnostics, "accepted_preparation_event_count", "campaign diagnostics") === 18,
    integer(diagnostics, "preparation_invalid_encoding_count", "campaign diagnostics") === 0,
    integer(diagnostics, "preparation_invalid_json_count", "campaign diagnostics") === 0,
    integer(diagnostics, "preparation_invalid_schema_count", "campaign diagnostics") === 0,
    latest["schema"] === "mining-campaign-preparation-v1",
    latest["step"] === "retain_production_uart",
    latest["outcome"] === "completed",
  ];
  if (exactFacts.some((fact) => !fact)) {
    throw failure("evidence_invalid", "sealed campaign initialization quorum is incomplete");
  }
}

export async function projectAsicInitializationEvidence(
  workspaceRoot: string,
  options: AsicInitializationEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  validatorProgram: string,
): Promise<AsicInitializationEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const attemptRoot = assertWithinWorkspace(workspaceRoot, options.attemptRoot);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");
  await verifyProtectedModes(attemptRoot);

  const resultPath = path.join(attemptRoot, "campaign-result.json");
  const diagnosticsPath = path.join(attemptRoot, "campaign-diagnostics.private.json");
  const observationsPath = path.join(attemptRoot, "campaign-observations.private.json");
  const [{ document: resultDocument, value: result }, { document: diagnosticsDocument, value: diagnostics }, observationsDocument] = await Promise.all([
    jsonFile(resultPath, "campaign result"),
    jsonFile(diagnosticsPath, "campaign diagnostics"),
    readFile(observationsPath, "utf8"),
  ]);
  const campaignResultSha256 = sha256(resultDocument);
  const resultSeal = (await readFile(path.join(attemptRoot, "campaign-result.sha256"), "utf8")).trim();
  if (!lowerHex(resultSeal, 64) || resultSeal !== campaignResultSha256) {
    throw failure("evidence_invalid", "campaign result seal is invalid");
  }
  const diagnosticsSha256 = sha256(diagnosticsDocument);
  const observationsSha256 = sha256(observationsDocument);
  if (string(result, "diagnostics_sha256", "campaign result") !== diagnosticsSha256
    || string(result, "observations_sha256", "campaign result") !== observationsSha256) {
    throw failure("evidence_invalid", "campaign private artifact digest is invalid");
  }
  validateCampaign(result, diagnostics);

  const taskDocument = await readFile(path.join(workspaceRoot, "TASKS.archive.md"), "utf8");
  validateArchivedTask(taskDocument, options.attemptSourceCommit);
  const currentSourceCommit = await childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity");
  const referenceCommit = await childText(
    processPort,
    gitProgram,
    ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
    "reference source identity",
  );
  if (!lowerHex(currentSourceCommit, 40) || !lowerHex(referenceCommit, 40)) {
    throw failure("evidence_invalid", "current source identity is invalid");
  }
  await childText(
    processPort,
    gitProgram,
    ["cat-file", "-e", `${options.attemptSourceCommit}^{commit}`],
    "attempt source admission",
  );
  await childText(
    processPort,
    gitProgram,
    ["diff", "--quiet", options.attemptSourceCommit, currentSourceCommit, "--", ...initializationPaths],
    "initialization source compatibility",
  );
  const worktreeState = await childText(
    processPort,
    gitProgram,
    ["status", "--porcelain", "--", ...initializationPaths],
    "initialization worktree compatibility",
  );
  if (worktreeState !== "") throw failure("evidence_invalid", "initialization paths have uncommitted drift");

  const requestSha256 = sha256(JSON.stringify({
    command: "project-asic-initialization-evidence",
    attempt_root: path.relative(workspaceRoot, attemptRoot),
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: AsicInitializationEvidence = {
    schema_version: "bitaxe-asic-initialization-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    source_task_sha256: sha256(taskDocument),
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-initialization-evidence",
      request_sha256: requestSha256,
    },
    attempt: {
      campaign_result_sha256: campaignResultSha256,
      diagnostics_sha256: diagnosticsSha256,
      observations_sha256: observationsSha256,
      result_seal_valid: true,
      private_digests_valid: true,
      protected_modes_valid: true,
    },
    initialization: {
      planned_step_count: 9,
      accepted_preparation_event_count: 18,
      invalid_preparation_event_count: 0,
      terminal_preparation_step: "retain_production_uart",
      terminal_preparation_outcome: "completed",
      all_preparation_steps_completed: true,
      exactly_one_chip_detected: true,
      mining_ready_initialization_completed: true,
      production_uart_retained: true,
      live_initialized_work_observed: true,
      initialization_paths_unchanged: true,
      compatible_path_count: initializationPaths.length,
    },
    package_admitted: true,
    runtime_identity: "trusted",
    runtime_attestation_status: "trusted",
    serial_outcome_detail: "clean",
    campaign_terminal_category: "submit_response_observed",
    submit_outcome: "accepted",
    safety_status: "fresh",
    mine_on_boot_disabled: true,
    safe_stop_confirmed: true,
    lease_cleanup_confirmed: true,
    usb_cleanup_ready: true,
    hardware_rerun_used: false,
    redaction_status: "passed",
  };

  await mkdir(path.dirname(projection), { recursive: true });
  try {
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx", mode: 0o600 });
    await chmod(candidate, 0o600);
    await childText(processPort, validatorProgram, [candidate], "independent evidence validation");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
  } catch (error) {
    await unlink(candidate).catch(() => undefined);
    throw error;
  }
  return evidence;
}
