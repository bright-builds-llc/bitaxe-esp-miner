import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AsicResultParsingEvidence,
  type AsicWorkSendEvidence,
  type AutomationCategory,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type AsicResultParsingEvidenceOptions = {
  readonly sourceProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedSourceProjection =
  "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json";
const transcriptPath = "crates/bitaxe-asic/src/bm1366/transcript.rs";
const resultPath = "crates/bitaxe-asic/src/bm1366/result.rs";
const adapterPath = "firmware/bitaxe/src/asic_adapter/production.rs";
const workerPath = "firmware/bitaxe/src/production_mining_session/asic_worker.rs";
const correlationPath = "crates/bitaxe-stratum/src/v1/production_work.rs";

const exactSpanContracts = [
  {
    path: resultPath,
    start: "pub fn parse_bm1366_result_frame(",
    end: "/// Classifies strict parser failures as a soft discard",
  },
  {
    path: resultPath,
    start: "fn validate_result_frame(",
    end: "fn parse_job_result(",
  },
  {
    path: resultPath,
    start: "fn parse_job_result(",
    end: "fn parse_register_read(",
  },
  {
    path: adapterPath,
    start: "        Bm1366ProductionResult::JobNonce(result) => Ok(ProductionReadOutcome::JobNonce(result)),",
    end: "        Bm1366ProductionResult::RegisterRead(read) => {",
  },
] as const;

const exactFragmentContracts = [
  {
    path: workerPath,
    fragment: `                            Ok(ProductionReadOutcome::JobNonce(result)) => {
                                emit(AsicWorkerEvent::Result { generation, result });
                            }`,
  },
] as const;

const correlationFragments = [
  "observation.result.job_id.lookup_key()",
  "record.generation != self.generation",
  "!stored_work_context_matches_nonce_result(record, observation.result)",
  "ShareSubmission::from_nonce_result(&record.work, observation.result)",
  "CorrelationOutcome::SubmitIntent(SubmitIntent {",
] as const;

const currentResultFragments = [
  "pub const BM1366_RESULT_FRAME_LEN: usize = 11;",
  "let frame = ResultFrameBytes::try_from_slice(bytes)?;",
  "if actual_preamble != BM1366_RECEIVE_PREAMBLE {",
  "if crc5(&bytes[2..]) != 0 {",
  "if !valid_jobs.contains(job_id) {",
  "let submit_nonce = u32::from_le_bytes(nonce_bytes);",
  "if core_id >= BM1366_NORMAL_CORE_COUNT {",
  "let address_interval = valid_address_interval(address_interval)?;\n    let asic_index = (u16::from(((nonce_be >> 17)",
  "let version_bits = (u32::from(version_be)) << 13;",
  "let register = Bm1366Register::try_from(bytes[7])?;",
  "Err(fault) => Bm1366ProductionResult::Discarded(discard_reason(fault)),",
] as const;

const discardCategories = [
  "InvalidLength",
  "InvalidPreamble",
  "InvalidCrc",
  "JobLookup",
  "Core",
  "AddressInterval",
  "RegisterResponse",
  "ParserInvariant",
] as const;

export class AsicResultParsingEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "AsicResultParsingEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): AsicResultParsingEvidenceError {
  return new AsicResultParsingEvidenceError(category, message, {
    stage: "sealed_result_parsing_projection",
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
    if (error instanceof AsicResultParsingEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof AsicResultParsingEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function parseSource(document: string): AsicWorkSendEvidence {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", "source work-send projection is malformed");
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", "source work-send projection must be an object");
  }
  return value as AsicWorkSendEvidence;
}

function validateSourceFacts(source: AsicWorkSendEvidence, attemptSourceCommit: string): void {
  if (source.schema_version !== "bitaxe-asic-work-send-evidence-v1"
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
    || !source.work_send.live_work_observed
    || !source.work_send.qualified_result_observed
    || !source.work_send.accepted_submit_observed
    || !source.work_send.production_uart_retained) {
    throw failure("evidence_invalid", "source work-send projection quorum is incomplete");
  }
}

function extractUniqueSpan(document: string, start: string, end: string): string {
  const startIndex = document.indexOf(start);
  if (startIndex === -1 || document.indexOf(start, startIndex + start.length) !== -1) {
    throw failure("evidence_invalid", "result-parsing source span start is not unique");
  }
  const endIndex = document.indexOf(end, startIndex + start.length);
  if (endIndex === -1 || document.indexOf(end, endIndex + end.length) !== -1) {
    throw failure("evidence_invalid", "result-parsing source span end is not unique");
  }
  return document.slice(startIndex, endIndex);
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "result-parsing semantic fragment is not unique");
  }
}

async function sourceAt(
  processPort: ProcessPort,
  gitProgram: string,
  commit: string,
  sourcePath: string,
): Promise<string> {
  return childText(
    processPort,
    gitProgram,
    ["show", `${commit}:${sourcePath}`],
    "result-parsing source admission",
  );
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
    ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", transcriptPath],
    "result-parsing transcript compatibility",
  );

  const documentCache = new Map<string, string>();
  const documentAt = async (commit: string, sourcePath: string): Promise<string> => {
    const key = `${commit}:${sourcePath}`;
    const maybeDocument = documentCache.get(key);
    if (maybeDocument !== undefined) return maybeDocument;
    const document = await sourceAt(processPort, gitProgram, commit, sourcePath);
    documentCache.set(key, document);
    return document;
  };

  for (const span of exactSpanContracts) {
    const [attemptDocument, currentDocument] = await Promise.all([
      documentAt(attemptSourceCommit, span.path),
      documentAt(currentSourceCommit, span.path),
    ]);
    if (extractUniqueSpan(attemptDocument, span.start, span.end)
      !== extractUniqueSpan(currentDocument, span.start, span.end)) {
      throw failure("evidence_invalid", "result-parsing exact source span drifted");
    }
  }
  for (const contract of exactFragmentContracts) {
    const [attemptDocument, currentDocument] = await Promise.all([
      documentAt(attemptSourceCommit, contract.path),
      documentAt(currentSourceCommit, contract.path),
    ]);
    requireUniqueFragment(attemptDocument, contract.fragment);
    requireUniqueFragment(currentDocument, contract.fragment);
  }

  const [attemptCorrelation, currentCorrelation] = await Promise.all([
    documentAt(attemptSourceCommit, correlationPath),
    documentAt(currentSourceCommit, correlationPath),
  ]);
  const attemptCorrelationSpan = extractUniqueSpan(
    attemptCorrelation,
    "    pub fn correlate_nonce_result(",
    "    pub const fn valid_jobs(",
  );
  const currentCorrelationSpan = extractUniqueSpan(
    currentCorrelation,
    "    pub fn correlate_nonce_result(",
    "    pub const fn valid_jobs(",
  );
  for (const fragment of correlationFragments) {
    requireUniqueFragment(attemptCorrelationSpan, fragment);
    requireUniqueFragment(currentCorrelationSpan, fragment);
  }

  const currentResult = await documentAt(currentSourceCommit, resultPath);
  for (const fragment of currentResultFragments) requireUniqueFragment(currentResult, fragment);
  for (const category of discardCategories) {
    requireUniqueFragment(
      extractUniqueSpan(
        currentResult,
        "pub enum Bm1366ResultDiscardReason {",
        "impl Bm1366ResultDiscardReason {",
      ),
      category,
    );
  }
}

export async function projectAsicResultParsingEvidence(
  workspaceRoot: string,
  options: AsicResultParsingEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  sourceValidatorProgram: string,
  validatorProgram: string,
): Promise<AsicResultParsingEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const sourceProjection = assertWithinWorkspace(workspaceRoot, options.sourceProjection);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  if (path.relative(workspaceRoot, sourceProjection) !== expectedSourceProjection) {
    throw failure("evidence_invalid", "source work-send projection path is invalid");
  }
  const candidate = `${projection}.candidate`;
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");

  let sourceDocument: string;
  try {
    sourceDocument = await readFile(sourceProjection, "utf8");
  } catch {
    throw failure("evidence_invalid", "source work-send projection is unavailable");
  }
  await childText(processPort, sourceValidatorProgram, [sourceProjection], "source work-send validation");
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
  if (!lowerHex(currentSourceCommit, 40)
    || !lowerHex(referenceCommit, 40)
    || !lowerHex(source.current_source_commit, 40)) {
    throw failure("evidence_invalid", "result-parsing source identity is invalid");
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
  await validateSourceCompatibility(processPort, gitProgram, options.attemptSourceCommit, currentSourceCommit);
  const relevantPaths = [
    expectedSourceProjection,
    transcriptPath,
    resultPath,
    adapterPath,
    workerPath,
    correlationPath,
  ];
  const worktreeState = await childText(
    processPort,
    gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths],
    "result-parsing worktree compatibility",
  );
  if (worktreeState !== "") {
    throw failure("evidence_invalid", "result-parsing evidence paths have uncommitted drift");
  }

  const workSendProjectionSha256 = sha256(sourceDocument);
  const requestSha256 = sha256(JSON.stringify({
    command: "project-asic-result-parsing-evidence",
    source_projection: expectedSourceProjection,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: AsicResultParsingEvidence = {
    schema_version: "bitaxe-asic-result-parsing-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-result-parsing-evidence",
      request_sha256: requestSha256,
    },
    source: {
      work_send_projection_sha256: workSendProjectionSha256,
      work_send_projection_current_commit: source.current_source_commit,
      work_send_projection_valid: true,
    },
    result_parsing: {
      result_frame_length_bytes: 11,
      strict_length_validation: true,
      preamble_validation: true,
      crc_validation: true,
      job_lookup_validation: true,
      submit_nonce_little_endian: true,
      core_validation: true,
      address_interval_validation: true,
      version_bits_recovered: true,
      known_register_classification: true,
      typed_soft_discard_category_count: 8,
      soft_discard_continuation: true,
      live_qualified_result_observed: true,
      accepted_submit_observed: true,
      transcript_path_unchanged: true,
      parser_spans_unchanged: true,
      adapter_nonce_span_unchanged: true,
      worker_nonce_span_unchanged: true,
      correlation_semantics_compatible: true,
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
