import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import type {
  AsicInitializationEvidence,
  AutomationCategory,
  StratumSocketEvidence,
} from "./contracts.generated.js";
import { internalCommandSpec } from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type StratumSocketEvidenceOptions = {
  readonly sourceProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedSourceProjection =
  "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json";
const expectedSourceProjectionSha256 =
  "eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4";
const transportPath = "firmware/bitaxe/src/production_mining_session/transport.rs";
const ownerPath = "firmware/bitaxe/src/production_mining_session.rs";
const lifecyclePath = "crates/bitaxe-stratum/src/v1/production_session/orchestration.rs";

const transportFragments = [
  "const COMMAND_CAPACITY: usize = 8;",
  "const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);",
  "const READ_TIMEOUT: Duration = Duration::from_millis(50);",
  "const WRITE_TIMEOUT: Duration = Duration::from_secs(2);",
  "const READ_BUFFER_BYTES: usize = 2 * 1024;",
  "stream.set_nodelay(true)?;",
  "if connection.transport_epoch != transport_epoch {",
] as const;
const compatibleSpans = new Map<string, readonly (readonly [string, string])[]>([
  [ownerPath, [
    ["OwnerInboxMessage::Transport(event) => match event {", "OwnerInboxMessage::Asic(event) => match event {"],
    ["ProductionSessionEffect::ConnectPool {", "effect @ (ProductionSessionEffect::ApplyVersionMask"],
    ["ProductionSessionEffect::ClosePoolConnection {", "ProductionSessionEffect::SafeStopHardware {"],
  ]],
  [lifecyclePath, [
    ["PendingRequestKind::Submit => {", "PendingRequestKind::Runtime(kind) => {"],
  ]],
]);

export class StratumSocketEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "StratumSocketEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): StratumSocketEvidenceError {
  return new StratumSocketEvidenceError(category, message, {
    stage: "sealed_stratum_socket_projection",
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
    if (error instanceof StratumSocketEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof StratumSocketEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function parseSource(document: string): AsicInitializationEvidence {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", "initialization source projection is malformed");
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", "initialization source projection must be an object");
  }
  return value as AsicInitializationEvidence;
}

function validateSourceFacts(source: AsicInitializationEvidence, attemptSourceCommit: string): void {
  const initialization = source.initialization;
  if (source.schema_version !== "bitaxe-asic-initialization-evidence-v1"
    || source.board !== 205
    || source.attempt_source_commit !== attemptSourceCommit
    || !initialization.all_preparation_steps_completed
    || !initialization.live_initialized_work_observed
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
    throw failure("evidence_invalid", "Stratum socket source quorum is incomplete");
  }
}

function requireUniqueFragments(document: string, fragments: readonly string[]): void {
  for (const fragment of fragments) {
    const first = document.indexOf(fragment);
    if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
      throw failure("evidence_invalid", "Stratum socket semantic fragment is not unique");
    }
  }
}

function extractUniqueSpan(document: string, start: string, end: string): string {
  const startIndex = document.indexOf(start);
  if (startIndex === -1 || document.indexOf(start, startIndex + start.length) !== -1) {
    throw failure("evidence_invalid", "Stratum socket span start is not unique");
  }
  const endIndex = document.indexOf(end, startIndex + start.length);
  if (endIndex === -1 || document.indexOf(end, endIndex + end.length) !== -1) {
    throw failure("evidence_invalid", "Stratum socket span end is not unique");
  }
  return document.slice(startIndex, endIndex);
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  attemptSourceCommit: string,
  currentSourceCommit: string,
): Promise<void> {
  await childText(processPort, gitProgram,
    ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", transportPath],
    "Stratum transport module compatibility");
  const [transport, attemptOwner, currentOwner, attemptLifecycle, currentLifecycle] = await Promise.all([
    childText(processPort, gitProgram, ["show", `${currentSourceCommit}:${transportPath}`],
      "Stratum transport source admission"),
    childText(processPort, gitProgram, ["show", `${attemptSourceCommit}:${ownerPath}`],
      "Stratum owner source admission"),
    childText(processPort, gitProgram, ["show", `${currentSourceCommit}:${ownerPath}`],
      "Stratum owner source admission"),
    childText(processPort, gitProgram, ["show", `${attemptSourceCommit}:${lifecyclePath}`],
      "Stratum lifecycle source admission"),
    childText(processPort, gitProgram, ["show", `${currentSourceCommit}:${lifecyclePath}`],
      "Stratum lifecycle source admission"),
  ]);
  requireUniqueFragments(transport, transportFragments);
  for (const [sourcePath, attemptDocument, currentDocument] of [
    [ownerPath, attemptOwner, currentOwner],
    [lifecyclePath, attemptLifecycle, currentLifecycle],
  ] as const) {
    for (const [start, end] of compatibleSpans.get(sourcePath) ?? []) {
      if (extractUniqueSpan(attemptDocument, start, end)
        !== extractUniqueSpan(currentDocument, start, end)) {
        throw failure("evidence_invalid", "Stratum socket owner or lifecycle span drifted");
      }
    }
  }
}

export async function projectStratumSocketEvidence(
  workspaceRoot: string,
  options: StratumSocketEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  sourceValidatorProgram: string,
  validatorProgram: string,
  admittedSourceSha256 = expectedSourceProjectionSha256,
): Promise<StratumSocketEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const sourceProjection = assertWithinWorkspace(workspaceRoot, options.sourceProjection);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, sourceProjection) !== expectedSourceProjection) {
    throw failure("evidence_invalid", "initialization source projection path is invalid");
  }
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");

  const sourceDocument = await readFile(sourceProjection, "utf8");
  const sourceProjectionSha256 = sha256(sourceDocument);
  if (sourceProjectionSha256 !== admittedSourceSha256) {
    throw failure("evidence_invalid", "initialization source projection digest is invalid");
  }
  const source = parseSource(sourceDocument);
  await childText(processPort, sourceValidatorProgram, [sourceProjection],
    "initialization source validation");
  validateSourceFacts(source, options.attemptSourceCommit);

  const [currentSourceCommit, referenceCommit] = await Promise.all([
    childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity"),
    childText(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference source identity"),
  ]);
  if (!lowerHex(currentSourceCommit, 40) || !lowerHex(referenceCommit, 40)) {
    throw failure("evidence_invalid", "current source identity is invalid");
  }
  if (source.reference_commit !== referenceCommit) {
    throw failure("evidence_invalid", "reference source identity drifted");
  }
  await childText(processPort, gitProgram,
    ["cat-file", "-e", `${options.attemptSourceCommit}^{commit}`], "attempt source admission");
  await childText(processPort, gitProgram,
    ["merge-base", "--is-ancestor", options.attemptSourceCommit, currentSourceCommit],
    "attempt source ancestry");
  await validateSourceCompatibility(
    processPort, gitProgram, options.attemptSourceCommit, currentSourceCommit,
  );
  const relevantPaths = [transportPath, ownerPath, lifecyclePath, expectedSourceProjection];
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths], "Stratum socket worktree state");
  if (dirty !== "") throw failure("evidence_invalid", "Stratum socket paths have uncommitted drift");

  const requestSha256 = sha256(JSON.stringify({
    command: "project-stratum-socket-evidence",
    source_projection: expectedSourceProjection,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: StratumSocketEvidence = {
    schema_version: "bitaxe-stratum-socket-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-stratum-socket-evidence",
      request_sha256: requestSha256,
    },
    source: {
      initialization_projection_sha256: sourceProjectionSha256,
      initialization_projection_current_commit: source.current_source_commit,
      initialization_projection_valid: true,
    },
    socket: {
      command_capacity: 8,
      connect_timeout_ms: 5000,
      read_timeout_ms: 50,
      write_timeout_ms: 2000,
      read_buffer_bytes: 2048,
      tcp_nodelay_enabled: true,
      typed_connect_write_close_commands: true,
      typed_connected_bytes_failed_closed_events: true,
      transport_epoch_isolation: true,
      authorized_session_required_before_submit: true,
      accepted_submit_observed: true,
      transport_module_unchanged: true,
      owner_and_lifecycle_spans_compatible: true,
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
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`,
      { encoding: "utf8", flag: "wx", mode: 0o600 });
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
