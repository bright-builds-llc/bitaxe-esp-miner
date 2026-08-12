import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AsicInitializationEvidence,
  type AsicWorkSendEvidence,
  type AutomationCategory,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type AsicWorkSendEvidenceOptions = {
  readonly sourceProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedSourceProjection =
  "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json";

const corePaths = [
  "crates/bitaxe-asic/src/bm1366/work.rs",
  "crates/bitaxe-asic/src/bm1366/production.rs",
  "crates/bitaxe-asic/src/bm1366/command.rs",
] as const;

const spanContracts = [
  {
    path: "firmware/bitaxe/src/production_mining_session/asic_worker.rs",
    start: "                        AsicWorkerCommand::Dispatch {",
    end: "                        AsicWorkerCommand::Poll {",
  },
  {
    path: "firmware/bitaxe/src/asic_adapter/production.rs",
    start: "            Bm1366ProductionCommand::SendProductionWork(_) => {",
    end: "            Bm1366ProductionCommand::ReadProductionResult => {",
  },
  {
    path: "firmware/bitaxe/src/asic_adapter/production.rs",
    start: "        Bm1366AdapterAction::WriteFrame(frame) => {",
    end: "        Bm1366AdapterAction::ReadExact { len, timeout_ms } => {",
  },
] as const;

export class AsicWorkSendEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "AsicWorkSendEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): AsicWorkSendEvidenceError {
  return new AsicWorkSendEvidenceError(category, message, {
    stage: "sealed_work_send_projection",
    hardware_rerun_used: false,
    projection_published: false,
  });
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function lowerHex(value: string, length: number): boolean {
  return value.length === length && new RegExp(`^[0-9a-f]{${String(length)}}$`, "u").test(value);
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
    if (error instanceof AsicWorkSendEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof AsicWorkSendEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function parseSource(document: string): AsicInitializationEvidence {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", "source initialization projection is malformed");
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", "source initialization projection must be an object");
  }
  return value as AsicInitializationEvidence;
}

function validateSourceFacts(
  source: AsicInitializationEvidence,
  attemptSourceCommit: string,
): void {
  if (source.schema_version !== "bitaxe-asic-initialization-evidence-v1"
    || source.board !== 205
    || source.attempt_source_commit !== attemptSourceCommit
    || !source.package_admitted
    || source.runtime_identity !== "trusted"
    || source.runtime_attestation_status !== "trusted"
    || source.campaign_terminal_category !== "submit_response_observed"
    || source.submit_outcome !== "accepted"
    || source.safety_status !== "fresh"
    || !source.mine_on_boot_disabled
    || !source.safe_stop_confirmed
    || !source.lease_cleanup_confirmed
    || !source.usb_cleanup_ready
    || source.hardware_rerun_used
    || source.redaction_status !== "passed"
    || !source.initialization.mining_ready_initialization_completed
    || !source.initialization.production_uart_retained
    || !source.initialization.live_initialized_work_observed) {
    throw failure("evidence_invalid", "source initialization projection quorum is incomplete");
  }
}

function extractUniqueSpan(document: string, start: string, end: string): string {
  const startIndex = document.indexOf(start);
  if (startIndex === -1 || document.indexOf(start, startIndex + start.length) !== -1) {
    throw failure("evidence_invalid", "work-send source span start is not unique");
  }
  const endIndex = document.indexOf(end, startIndex + start.length);
  if (endIndex === -1 || document.indexOf(end, endIndex + end.length) !== -1) {
    throw failure("evidence_invalid", "work-send source span end is not unique");
  }
  return document.slice(startIndex, endIndex);
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  attemptSourceCommit: string,
  currentSourceCommit: string,
): Promise<void> {
  await childText(
    processPort,
    gitProgram,
    ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", ...corePaths],
    "work-send core source compatibility",
  );

  for (const span of spanContracts) {
    const [attemptDocument, currentDocument] = await Promise.all([
      childText(
        processPort,
        gitProgram,
        ["show", `${attemptSourceCommit}:${span.path}`],
        "attempt work-send source span",
      ),
      childText(
        processPort,
        gitProgram,
        ["show", `${currentSourceCommit}:${span.path}`],
        "current work-send source span",
      ),
    ]);
    if (extractUniqueSpan(attemptDocument, span.start, span.end)
      !== extractUniqueSpan(currentDocument, span.start, span.end)) {
      throw failure("evidence_invalid", "work-send source span drifted");
    }
  }
}

export async function projectAsicWorkSendEvidence(
  workspaceRoot: string,
  options: AsicWorkSendEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  sourceValidatorProgram: string,
  validatorProgram: string,
): Promise<AsicWorkSendEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const sourceProjection = assertWithinWorkspace(workspaceRoot, options.sourceProjection);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  if (path.relative(workspaceRoot, sourceProjection) !== expectedSourceProjection) {
    throw failure("evidence_invalid", "source initialization projection path is invalid");
  }
  const candidate = `${projection}.candidate`;
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");

  const sourceDocument = await readFile(sourceProjection, "utf8");
  await childText(
    processPort,
    sourceValidatorProgram,
    [sourceProjection],
    "source initialization evidence validation",
  );
  const source = parseSource(sourceDocument);
  validateSourceFacts(source, options.attemptSourceCommit);

  const currentSourceCommit = await childText(
    processPort,
    gitProgram,
    ["rev-parse", "HEAD"],
    "current source identity",
  );
  const referenceCommit = await childText(
    processPort,
    gitProgram,
    ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
    "reference source identity",
  );
  if (!lowerHex(currentSourceCommit, 40)) {
    throw failure("evidence_invalid", "current work-send source identity is invalid");
  }
  if (!lowerHex(referenceCommit, 40)) {
    throw failure("evidence_invalid", "reference work-send source identity is invalid");
  }
  if (!lowerHex(source.current_source_commit, 40)) {
    throw failure("evidence_invalid", "source projection identity is invalid");
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
    ["merge-base", "--is-ancestor", source.current_source_commit, currentSourceCommit],
    "source projection ancestry",
  );
  await childText(
    processPort,
    gitProgram,
    ["ls-files", "--error-unmatch", expectedSourceProjection],
    "source projection tracking",
  );
  await validateSourceCompatibility(
    processPort,
    gitProgram,
    options.attemptSourceCommit,
    currentSourceCommit,
  );
  const relevantPaths = [
    expectedSourceProjection,
    ...corePaths,
    ...new Set(spanContracts.map((span) => span.path)),
  ];
  const worktreeState = await childText(
    processPort,
    gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths],
    "work-send worktree compatibility",
  );
  if (worktreeState !== "") {
    throw failure("evidence_invalid", "work-send evidence paths have uncommitted drift");
  }

  const initializationProjectionSha256 = sha256(sourceDocument);
  const requestSha256 = sha256(JSON.stringify({
    command: "project-asic-work-send-evidence",
    source_projection: expectedSourceProjection,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: AsicWorkSendEvidence = {
    schema_version: "bitaxe-asic-work-send-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-work-send-evidence",
      request_sha256: requestSha256,
    },
    source: {
      initialization_projection_sha256: initializationProjectionSha256,
      initialization_projection_current_commit: source.current_source_commit,
      initialization_projection_valid: true,
    },
    work_send: {
      payload_length_bytes: 82,
      frame_length_bytes: 88,
      job_id_step: 8,
      job_id_modulus: 128,
      typed_write_frame_action: true,
      production_ready_gate_required: true,
      live_work_observed: true,
      qualified_result_observed: true,
      accepted_submit_observed: true,
      production_uart_retained: true,
      core_paths_unchanged: true,
      compatible_core_path_count: corePaths.length,
      dispatch_spans_unchanged: true,
      uart_write_span_unchanged: true,
    },
    package_admitted: true,
    runtime_identity: "trusted",
    runtime_attestation_status: "trusted",
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
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
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
