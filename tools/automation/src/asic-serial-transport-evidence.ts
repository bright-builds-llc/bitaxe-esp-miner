import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AsicResultParsingEvidence,
  type AsicSerialTransportEvidence,
  type AsicWorkSendEvidence,
  type AutomationCategory,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type AsicSerialTransportEvidenceOptions = {
  readonly workSendProjection: string;
  readonly resultParsingProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedWorkSendProjection =
  "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json";
const expectedResultParsingProjection =
  "docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json";
const uartPath = "firmware/bitaxe/src/asic_adapter/uart.rs";
const adapterPath = "firmware/bitaxe/src/asic_adapter.rs";
const productionPath = "firmware/bitaxe/src/asic_adapter/production.rs";

const uartFragments = [
  "pub const UART_INITIAL_BAUD: u32 = 115_200;",
  "pub const UART_TX_PIN: i32 = 17;",
  "pub const UART_RX_PIN: i32 = 18;",
  "pub const WAIT_TX_DONE_TIMEOUT_MS: u32 = 1_000;",
  "const UART_RX_BUFFER_BYTES: usize = UART_BUF_SIZE * 2;",
  "const READ_CHUNK_MAX: usize = 64;",
  ".data_bits(config::DataBits::DataBits8)",
  ".parity_none()",
  ".stop_bits(config::StopBits::STOP1)",
  ".flow_control(config::FlowControl::None)",
  "ensure!(written == frame.len(), \"partial BM1366 UART frame write\");",
  "let deadline = started + std::time::Duration::from_millis(u64::from(timeout_ms));",
  "let mut scratch = [0_u8; READ_CHUNK_MAX];",
  "Err(error) if is_uart_timeout_error(&error) && buf.is_empty() => 0,",
  "buf.extend_from_slice(&scratch[..read]);",
  "self.clear_rx()?;",
] as const;

export class AsicSerialTransportEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "AsicSerialTransportEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): AsicSerialTransportEvidenceError {
  return new AsicSerialTransportEvidenceError(category, message, {
    stage: "sealed_serial_transport_projection",
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
    if (error instanceof AsicSerialTransportEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof AsicSerialTransportEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function parseObject<T>(document: string, context: string): T {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", `${context} is malformed`);
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as T;
}

function validateCampaignFacts(source: AsicWorkSendEvidence | AsicResultParsingEvidence): void {
  if (source.board !== 205
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
    || source.redaction_status !== "passed") {
    throw failure("evidence_invalid", "source campaign quorum is incomplete");
  }
}

function validateSourceFacts(
  work: AsicWorkSendEvidence,
  result: AsicResultParsingEvidence,
  attemptSourceCommit: string,
  workDocument: string,
): void {
  validateCampaignFacts(work);
  validateCampaignFacts(result);
  if (work.schema_version !== "bitaxe-asic-work-send-evidence-v1"
    || result.schema_version !== "bitaxe-asic-result-parsing-evidence-v1"
    || work.attempt_source_commit !== attemptSourceCommit
    || result.attempt_source_commit !== attemptSourceCommit
    || work.reference_commit !== result.reference_commit
    || !work.work_send.live_work_observed
    || !work.work_send.qualified_result_observed
    || !work.work_send.accepted_submit_observed
    || !work.work_send.production_uart_retained
    || !result.result_parsing.live_qualified_result_observed
    || !result.result_parsing.accepted_submit_observed
    || !result.result_parsing.result_transport_module_unchanged
    || !result.source.work_send_projection_valid
    || result.source.work_send_projection_sha256 !== sha256(workDocument)
    || result.source.work_send_projection_current_commit !== work.current_source_commit) {
    throw failure("evidence_invalid", "serial-transport source quorum is incomplete");
  }
}

function extractUniqueSpan(document: string, start: string, end: string): string {
  const startIndex = document.indexOf(start);
  if (startIndex === -1 || document.indexOf(start, startIndex + start.length) !== -1) {
    throw failure("evidence_invalid", "serial-transport span start is not unique");
  }
  const endIndex = document.indexOf(end, startIndex + start.length);
  if (endIndex === -1 || document.indexOf(end, endIndex + end.length) !== -1) {
    throw failure("evidence_invalid", "serial-transport span end is not unique");
  }
  return document.slice(startIndex, endIndex);
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "serial-transport semantic fragment is not unique");
  }
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  attemptSourceCommit: string,
  currentSourceCommit: string,
): Promise<void> {
  for (const sourcePath of [uartPath, adapterPath]) {
    await childText(
      processPort,
      gitProgram,
      ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", sourcePath],
      "serial-transport module compatibility",
    );
  }
  const [uartSource, attemptProduction, currentProduction] = await Promise.all([
    childText(processPort, gitProgram, ["show", `${currentSourceCommit}:${uartPath}`], "UART source admission"),
    childText(processPort, gitProgram, ["show", `${attemptSourceCommit}:${productionPath}`], "production source admission"),
    childText(processPort, gitProgram, ["show", `${currentSourceCommit}:${productionPath}`], "production source admission"),
  ]);
  for (const fragment of uartFragments) requireUniqueFragment(uartSource, fragment);

  const spans = [
    ["            Bm1366ProductionCommand::SendProductionWork(_) => {", "            Bm1366ProductionCommand::ReadProductionResult => {"],
    ["    let maybe_frame = match uart.maybe_try_read_exact(BM1366_RESULT_FRAME_LEN, poll_timeout_ms) {", "    let Some(frame) = maybe_frame else {"],
  ] as const;
  for (const [start, end] of spans) {
    if (extractUniqueSpan(attemptProduction, start, end)
      !== extractUniqueSpan(currentProduction, start, end)) {
      throw failure("evidence_invalid", "production serial-transport span drifted");
    }
  }
}

export async function projectAsicSerialTransportEvidence(
  workspaceRoot: string,
  options: AsicSerialTransportEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  workValidatorProgram: string,
  resultValidatorProgram: string,
  validatorProgram: string,
): Promise<AsicSerialTransportEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const workProjection = assertWithinWorkspace(workspaceRoot, options.workSendProjection);
  const resultProjection = assertWithinWorkspace(workspaceRoot, options.resultParsingProjection);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  if (path.relative(workspaceRoot, workProjection) !== expectedWorkSendProjection
    || path.relative(workspaceRoot, resultProjection) !== expectedResultParsingProjection) {
    throw failure("evidence_invalid", "source projection path is invalid");
  }
  const candidate = `${projection}.candidate`;
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");

  let workDocument: string;
  let resultDocument: string;
  try {
    [workDocument, resultDocument] = await Promise.all([
      readFile(workProjection, "utf8"),
      readFile(resultProjection, "utf8"),
    ]);
  } catch {
    throw failure("evidence_invalid", "source projection is unavailable");
  }
  await childText(processPort, workValidatorProgram, [workProjection], "work-send validation");
  await childText(processPort, resultValidatorProgram, [resultProjection], "result-parsing validation");
  const work = parseObject<AsicWorkSendEvidence>(workDocument, "work-send projection");
  const result = parseObject<AsicResultParsingEvidence>(resultDocument, "result-parsing projection");
  validateSourceFacts(work, result, options.attemptSourceCommit, workDocument);

  const currentSourceCommit = await childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity");
  const referenceCommit = await childText(
    processPort,
    gitProgram,
    ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
    "reference source identity",
  );
  if (!lowerHex(currentSourceCommit, 40) || !lowerHex(referenceCommit, 40)
    || referenceCommit !== work.reference_commit) {
    throw failure("evidence_invalid", "serial-transport source identity is invalid");
  }
  await childText(processPort, gitProgram, ["cat-file", "-e", `${options.attemptSourceCommit}^{commit}`], "attempt source admission");
  for (const sourceCommit of [work.current_source_commit, result.current_source_commit]) {
    await childText(processPort, gitProgram, ["merge-base", "--is-ancestor", sourceCommit, currentSourceCommit], "source projection ancestry");
  }
  for (const sourcePath of [expectedWorkSendProjection, expectedResultParsingProjection]) {
    await childText(processPort, gitProgram, ["ls-files", "--error-unmatch", sourcePath], "source projection tracking");
  }
  await validateSourceCompatibility(processPort, gitProgram, options.attemptSourceCommit, currentSourceCommit);
  const relevantPaths = [expectedWorkSendProjection, expectedResultParsingProjection, uartPath, adapterPath, productionPath];
  if (await childText(processPort, gitProgram, ["status", "--porcelain", "--", ...relevantPaths], "serial-transport worktree compatibility") !== "") {
    throw failure("evidence_invalid", "serial-transport evidence paths have uncommitted drift");
  }

  const requestSha256 = sha256(JSON.stringify({
    command: "project-asic-serial-transport-evidence",
    work_send_projection: expectedWorkSendProjection,
    result_parsing_projection: expectedResultParsingProjection,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: AsicSerialTransportEvidence = {
    schema_version: "bitaxe-asic-serial-transport-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-serial-transport-evidence",
      request_sha256: requestSha256,
    },
    source: {
      work_send_projection_sha256: sha256(workDocument),
      work_send_projection_current_commit: work.current_source_commit,
      work_send_projection_valid: true,
      result_parsing_projection_sha256: sha256(resultDocument),
      result_parsing_projection_current_commit: result.current_source_commit,
      result_parsing_projection_valid: true,
    },
    serial_transport: {
      initial_baud: 115_200,
      tx_pin: 17,
      rx_pin: 18,
      data_bits: 8,
      stop_bits: 1,
      parity_none: true,
      flow_control_none: true,
      tx_wait_timeout_ms: 1_000,
      rx_buffer_bytes: 2_048,
      read_chunk_max_bytes: 64,
      exact_write_required: true,
      absolute_read_deadline: true,
      partial_reads_accumulated: true,
      empty_timeout_is_idle: true,
      partial_timeout_clears_rx: true,
      live_work_tx_observed: true,
      live_result_rx_observed: true,
      accepted_submit_observed: true,
      uart_module_unchanged: true,
      adapter_module_unchanged: true,
      production_tx_span_compatible: true,
      production_rx_span_compatible: true,
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
