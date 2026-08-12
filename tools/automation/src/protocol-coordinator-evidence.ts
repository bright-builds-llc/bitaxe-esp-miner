import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import type {
  AsicInitializationEvidence,
  AsicResultParsingEvidence,
  AsicWorkSendEvidence,
  AutomationCategory,
  ProtocolCoordinatorEvidence,
  StratumSocketEvidence,
} from "./contracts.generated.js";
import { internalCommandSpec } from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type ProtocolCoordinatorEvidenceOptions = {
  readonly initializationProjection: string;
  readonly workSendProjection: string;
  readonly resultParsingProjection: string;
  readonly socketProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

export type ProtocolCoordinatorSourceValidators = {
  readonly initialization: string;
  readonly workSend: string;
  readonly resultParsing: string;
  readonly socket: string;
  readonly evidence: string;
};

export type ProtocolCoordinatorAdmittedDigests = {
  readonly initialization: string;
  readonly workSend: string;
  readonly resultParsing: string;
  readonly socket: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const sourceSpecifications = {
  initialization: {
    path: "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    sha256: "eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4",
  },
  workSend: {
    path: "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json",
    sha256: "447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c",
  },
  resultParsing: {
    path: "docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json",
    sha256: "e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7",
  },
  socket: {
    path: "docs/parity/evidence/str001-socket/stratum-socket-projection.json",
    sha256: "dcb3eed396a268114b017d7ef4fbca9c427a390d7acf405fc52fbef6472122b8",
  },
} as const;

const cadencePath = "crates/bitaxe-core/src/runtime_orchestration.rs";
const recoveryPath = "crates/bitaxe-stratum/src/v1/recovery_policy.rs";
const runtimePath = "crates/bitaxe-stratum/src/v1/production_session/runtime.rs";
const orchestrationPath = "crates/bitaxe-stratum/src/v1/production_session/orchestration.rs";
const asicRuntimePath = "crates/bitaxe-stratum/src/v1/production_session/runtime/asic.rs";
const ownerPath = "firmware/bitaxe/src/production_mining_session.rs";
const asicWorkerPath = "firmware/bitaxe/src/production_mining_session/asic_worker.rs";
const coordinatorPaths = [
  cadencePath,
  recoveryPath,
  runtimePath,
  orchestrationPath,
  asicRuntimePath,
  ownerPath,
  asicWorkerPath,
] as const;

const sourceFragments = new Map<string, readonly string[]>([
  [cadencePath, ["pub const PRODUCTION_REREAD_CADENCE_MS: u64 = 1_000;"]],
  [recoveryPath, [
    "return Some(ProductionSessionBlocker::OperatorPaused);",
    "return Some(ProductionSessionBlocker::NetworkUnavailable);",
    "return Some(ProductionSessionBlocker::StratumV1Unsupported);",
    "return Some(ProductionSessionBlocker::SafetyPrerequisitesStale);",
    "return Some(ProductionSessionBlocker::CampaignLeaseUnavailable);",
    "return Some(ProductionSessionBlocker::ActuationUnqualified);",
  ]],
  [runtimePath, [
    "effects.push(ProductionSessionEffect::PrepareHardware {",
    "if self.hardware_state != MiningHardwareState::Ready {",
    "effects.push(ProductionSessionEffect::SafeStopHardware { lease_id });",
  ]],
  [orchestrationPath, [
    "self.bridge.note_listener_armed();",
    "effects.push(ProductionSessionEffect::DispatchAsic {",
    "runtime.submits.insert(request_id, PendingSubmit { intent });",
    "self.stop_after_first_submit_response(effects)?;",
  ]],
  [asicRuntimePath, [
    ".apply_bridge_observation_with_receipt(observation)",
    "BridgeObservationOutcome::SubmitQueued => AsicCorrelation::Correlated,",
  ]],
  [ownerPath, [
    "const NOTIFICATION_CAPACITY: usize = 16;",
    "let mut session = ProductionMiningSession::new();",
    "while let Some(event) = events.pop_front() {",
    "task_watchdog.feed(crate::runtime_uptime::millis());",
  ]],
  [asicWorkerPath, ["AsicWorkerCommand::Dispatch {"]],
]);

export class ProtocolCoordinatorEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "ProtocolCoordinatorEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): ProtocolCoordinatorEvidenceError {
  return new ProtocolCoordinatorEvidenceError(category, message, {
    stage: "sealed_protocol_coordinator_projection",
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
    if (error instanceof ProtocolCoordinatorEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof ProtocolCoordinatorEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function parseProjection<T>(document: string, label: string): T {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", `${label} source projection is malformed`);
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${label} source projection must be an object`);
  }
  return value as T;
}

type CampaignProjection = {
  readonly board: number;
  readonly attempt_source_commit: string;
  readonly reference_commit: string;
  readonly package_admitted: boolean;
  readonly runtime_identity: string;
  readonly runtime_attestation_status: string;
  readonly campaign_terminal_category: string;
  readonly submit_outcome: string;
  readonly safety_status: string;
  readonly mine_on_boot_disabled: boolean;
  readonly safe_stop_confirmed: boolean;
  readonly lease_cleanup_confirmed: boolean;
  readonly usb_cleanup_ready: boolean;
  readonly hardware_rerun_used: boolean;
  readonly redaction_status: string;
};

function validateCampaignFacts(
  source: CampaignProjection,
  attemptSourceCommit: string,
  referenceCommit: string,
): void {
  if (source.board !== 205
    || source.attempt_source_commit !== attemptSourceCommit
    || source.reference_commit !== referenceCommit
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
    throw failure("evidence_invalid", "protocol coordinator source lineage or quorum is incomplete");
  }
}

function validateSourceFacts(
  initialization: AsicInitializationEvidence,
  workSend: AsicWorkSendEvidence,
  resultParsing: AsicResultParsingEvidence,
  socket: StratumSocketEvidence,
  attemptSourceCommit: string,
  admittedDigests: ProtocolCoordinatorAdmittedDigests,
): void {
  const referenceCommit = initialization.reference_commit;
  for (const source of [initialization, workSend, resultParsing, socket]) {
    validateCampaignFacts(source, attemptSourceCommit, referenceCommit);
  }
  if (initialization.schema_version !== "bitaxe-asic-initialization-evidence-v1"
    || !initialization.initialization.all_preparation_steps_completed
    || !initialization.initialization.mining_ready_initialization_completed
    || !initialization.initialization.live_initialized_work_observed
    || workSend.schema_version !== "bitaxe-asic-work-send-evidence-v1"
    || workSend.source.initialization_projection_sha256 !== admittedDigests.initialization
    || !workSend.work_send.production_ready_gate_required
    || !workSend.work_send.live_work_observed
    || !workSend.work_send.qualified_result_observed
    || !workSend.work_send.accepted_submit_observed
    || resultParsing.schema_version !== "bitaxe-asic-result-parsing-evidence-v1"
    || resultParsing.source.work_send_projection_sha256 !== admittedDigests.workSend
    || !resultParsing.result_parsing.job_lookup_validation
    || !resultParsing.result_parsing.core_validation
    || !resultParsing.result_parsing.live_qualified_result_observed
    || !resultParsing.result_parsing.accepted_submit_observed
    || !resultParsing.result_parsing.correlation_semantics_compatible
    || socket.schema_version !== "bitaxe-stratum-socket-evidence-v1"
    || socket.source.initialization_projection_sha256 !== admittedDigests.initialization
    || !socket.socket.transport_epoch_isolation
    || !socket.socket.authorized_session_required_before_submit
    || !socket.socket.accepted_submit_observed) {
    throw failure("evidence_invalid", "protocol coordinator source facts are incomplete");
  }
}

function requireUniqueFragments(document: string, fragments: readonly string[]): void {
  for (const fragment of fragments) {
    const first = document.indexOf(fragment);
    if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
      throw failure("evidence_invalid", "protocol coordinator semantic fragment is not unique");
    }
  }
}

function requireOrderedFragments(document: string, fragments: readonly string[]): void {
  let cursor = 0;
  for (const fragment of fragments) {
    const index = document.indexOf(fragment, cursor);
    if (index === -1) {
      throw failure("evidence_invalid", "protocol coordinator safe-stop ordering is incomplete");
    }
    cursor = index + fragment.length;
  }
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  baselineCommit: string,
  currentSourceCommit: string,
): Promise<void> {
  await childText(processPort, gitProgram,
    ["diff", "--quiet", baselineCommit, currentSourceCommit, "--", ...coordinatorPaths],
    "protocol coordinator module compatibility");
  const documents = await Promise.all(coordinatorPaths.map(async (sourcePath) => [
    sourcePath,
    await childText(processPort, gitProgram, ["show", `${currentSourceCommit}:${sourcePath}`],
      "protocol coordinator source admission"),
  ] as const));
  const byPath = new Map<string, string>(documents);
  for (const [sourcePath, fragments] of sourceFragments) {
    const document = byPath.get(sourcePath);
    if (document === undefined) {
      throw failure("evidence_invalid", "protocol coordinator source is missing");
    }
    requireUniqueFragments(document, fragments);
  }
  const runtime = byPath.get(runtimePath) ?? "";
  requireOrderedFragments(runtime, [
    "RecoveryAction::BlockSubmissions,",
    "RecoveryAction::InvalidateWorkAndSubmissions,",
    "RecoveryAction::StopAsicInteraction,",
    "effects.push(ProductionSessionEffect::SafeStopHardware { lease_id });",
  ]);
}

async function readAdmittedProjection<T>(
  workspaceRoot: string,
  sourcePath: string,
  expectedPath: string,
  expectedSha256: string,
  label: string,
): Promise<{ readonly document: string; readonly sha256: string; readonly evidence: T }> {
  const resolved = assertWithinWorkspace(workspaceRoot, sourcePath);
  if (path.relative(workspaceRoot, resolved) !== expectedPath) {
    throw failure("evidence_invalid", `${label} source projection path is invalid`);
  }
  const document = await readFile(resolved, "utf8");
  const documentSha256 = sha256(document);
  if (documentSha256 !== expectedSha256) {
    throw failure("evidence_invalid", `${label} source projection digest is invalid`);
  }
  return { document, sha256: documentSha256, evidence: parseProjection<T>(document, label) };
}

export async function projectProtocolCoordinatorEvidence(
  workspaceRoot: string,
  options: ProtocolCoordinatorEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  validators: ProtocolCoordinatorSourceValidators,
  admittedDigests: ProtocolCoordinatorAdmittedDigests = {
    initialization: sourceSpecifications.initialization.sha256,
    workSend: sourceSpecifications.workSend.sha256,
    resultParsing: sourceSpecifications.resultParsing.sha256,
    socket: sourceSpecifications.socket.sha256,
  },
): Promise<ProtocolCoordinatorEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");

  const [initializationInput, workSendInput, resultParsingInput, socketInput] = await Promise.all([
    readAdmittedProjection<AsicInitializationEvidence>(workspaceRoot,
      options.initializationProjection, sourceSpecifications.initialization.path,
      admittedDigests.initialization, "initialization"),
    readAdmittedProjection<AsicWorkSendEvidence>(workspaceRoot,
      options.workSendProjection, sourceSpecifications.workSend.path,
      admittedDigests.workSend, "work-send"),
    readAdmittedProjection<AsicResultParsingEvidence>(workspaceRoot,
      options.resultParsingProjection, sourceSpecifications.resultParsing.path,
      admittedDigests.resultParsing, "result-parsing"),
    readAdmittedProjection<StratumSocketEvidence>(workspaceRoot,
      options.socketProjection, sourceSpecifications.socket.path,
      admittedDigests.socket, "socket"),
  ]);
  const projectionPaths = [
    assertWithinWorkspace(workspaceRoot, options.initializationProjection),
    assertWithinWorkspace(workspaceRoot, options.workSendProjection),
    assertWithinWorkspace(workspaceRoot, options.resultParsingProjection),
    assertWithinWorkspace(workspaceRoot, options.socketProjection),
  ];
  await Promise.all([
    childText(processPort, validators.initialization, [projectionPaths[0] ?? ""],
      "initialization source validation"),
    childText(processPort, validators.workSend, [projectionPaths[1] ?? ""],
      "work-send source validation"),
    childText(processPort, validators.resultParsing, [projectionPaths[2] ?? ""],
      "result-parsing source validation"),
    childText(processPort, validators.socket, [projectionPaths[3] ?? ""],
      "socket source validation"),
  ]);
  validateSourceFacts(
    initializationInput.evidence,
    workSendInput.evidence,
    resultParsingInput.evidence,
    socketInput.evidence,
    options.attemptSourceCommit,
    admittedDigests,
  );

  const [currentSourceCommit, referenceCommit] = await Promise.all([
    childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity"),
    childText(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference source identity"),
  ]);
  if (!lowerHex(currentSourceCommit, 40) || !lowerHex(referenceCommit, 40)
    || initializationInput.evidence.reference_commit !== referenceCommit) {
    throw failure("evidence_invalid", "protocol coordinator source identity drifted");
  }
  await childText(processPort, gitProgram,
    ["cat-file", "-e", `${options.attemptSourceCommit}^{commit}`], "attempt source admission");
  await childText(processPort, gitProgram,
    ["merge-base", "--is-ancestor", options.attemptSourceCommit, currentSourceCommit],
    "attempt source ancestry");
  await validateSourceCompatibility(
    processPort,
    gitProgram,
    initializationInput.evidence.current_source_commit,
    currentSourceCommit,
  );
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...coordinatorPaths,
      ...Object.values(sourceSpecifications).map((source) => source.path)],
    "protocol coordinator worktree state");
  if (dirty !== "") {
    throw failure("evidence_invalid", "protocol coordinator paths have uncommitted drift");
  }

  const requestSha256 = sha256(JSON.stringify({
    command: "project-protocol-coordinator-evidence",
    initialization_projection: sourceSpecifications.initialization.path,
    work_send_projection: sourceSpecifications.workSend.path,
    result_parsing_projection: sourceSpecifications.resultParsing.path,
    socket_projection: sourceSpecifications.socket.path,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: ProtocolCoordinatorEvidence = {
    schema_version: "bitaxe-protocol-coordinator-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-protocol-coordinator-evidence",
      request_sha256: requestSha256,
    },
    source: {
      initialization_projection_sha256: initializationInput.sha256,
      initialization_projection_current_commit: initializationInput.evidence.current_source_commit,
      initialization_projection_valid: true,
      work_send_projection_sha256: workSendInput.sha256,
      work_send_projection_current_commit: workSendInput.evidence.current_source_commit,
      work_send_projection_valid: true,
      result_parsing_projection_sha256: resultParsingInput.sha256,
      result_parsing_projection_current_commit: resultParsingInput.evidence.current_source_commit,
      result_parsing_projection_valid: true,
      socket_projection_sha256: socketInput.sha256,
      socket_projection_current_commit: socketInput.evidence.current_source_commit,
      socket_projection_valid: true,
    },
    coordinator: {
      owner_inbox_capacity: 16,
      readiness_reread_cadence_ms: 1000,
      readiness_gate_count: 6,
      single_owner_serialization: true,
      hardware_prepared_before_pool_access: true,
      authorized_before_asic_dispatch: true,
      qualified_result_before_submit: true,
      accepted_submit_observed: true,
      ordered_terminal_safe_stop: true,
      watchdog_feed_in_owner_loop: true,
      coordinator_modules_unchanged: true,
      lifecycle_spans_compatible: true,
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
    await childText(processPort, validators.evidence, [candidate],
      "independent evidence validation");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
  } catch (error) {
    await unlink(candidate).catch(() => undefined);
    throw error;
  }
  return evidence;
}
