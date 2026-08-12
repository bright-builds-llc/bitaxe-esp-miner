import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import type {
  AutomationCategory,
  MiningCriteriaEvidence,
  ProtocolCoordinatorEvidence,
} from "./contracts.generated.js";
import { internalCommandSpec } from "./contracts.generated.js";
import { optionValue, type ParsedInvocation } from "./invocation.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type MiningCriteriaEvidenceOptions = {
  readonly summary: string;
  readonly smoke: string;
  readonly soak: string;
  readonly coordinatorProjection: string;
  readonly projection: string;
};

export type MiningCriteriaValidators = {
  readonly coordinator: string;
  readonly evidence: string;
};

export type MiningCriteriaAdmittedDigests = {
  readonly summary: string;
  readonly smoke: string;
  readonly soak: string;
  readonly coordinator: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;
const sourceSpecifications = {
  summary: {
    path: "docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md",
    sha256: "b411ed3d8a1ce427231ec2818ed74fb590e6b29e4539a0e131bfdc7bc7acec0c",
  },
  smoke: {
    path: "docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md",
    sha256: "faec052c13b55cc7a53a1206c25c2094d93945d4b17d69c17c8a976e860655ff",
  },
  soak: {
    path: "docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak.md",
    sha256: "fc8904a9d9e2132789d70a9886c8aef05be96134e1ccd4d29bc793c9efa66003",
  },
  coordinator: {
    path: "docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json",
    sha256: "f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7",
  },
} as const;

const campaignPath = "tools/flash/src/campaign.rs";
const admissionPath = "tools/flash/src/campaign/admission.rs";
const markersPath = "tools/flash/src/campaign/markers.rs";
const soakPath = "tools/flash/src/campaign/markers/soak.rs";
const evidencePath = "tools/flash/src/campaign/evidence.rs";
const networkPath = "tools/flash/src/campaign/network/model.rs";
const lifecycleTestPath = "crates/bitaxe-stratum/src/v1/production_session/tests/lifecycle.rs";
const campaignTestPath = "tools/flash/src/tests/campaign.rs";
const criteriaPaths = [
  campaignPath,
  admissionPath,
  markersPath,
  soakPath,
  evidencePath,
  networkPath,
  lifecycleTestPath,
  campaignTestPath,
] as const;

const sourceFragments = new Map<string, readonly string[]>([
  [campaignPath, [
    "const MINING_DURATION_SECONDS: u64 = 600;",
    "let cleanup_result = environment.finish_usb_session();",
  ]],
  [admissionPath, [
    "command.board != BoardId::Ultra205 || !command.redact_evidence",
    "MiningCampaignStage::LiveShare | MiningCampaignStage::Soak => MINING_DURATION_SECONDS,",
    "command.profile == Some(MiningCampaignProfile::UpstreamDefault)",
  ]],
  [markersPath, [
    "marker.safe_stop == SafeStopMarker::Confirmed;",
    "assess_soak_terminal(marker, admission.duration_seconds)",
  ]],
  [soakPath, [
    "marker.accepted_share_count == 0",
    "marker.active_ms < duration_seconds.saturating_mul(1_000)",
  ]],
  [evidencePath, [
    "set_private_directory_mode(root)?;",
    "write_private_new_bytes(&paths.result, &result_bytes)",
    "redacted: true,",
  ]],
  [networkPath, [
    "self.close_elapsed_windows(600_000, serial);",
    "covered_window_count == REQUIRED_WINDOWS",
    "watchdog_valid: self.watchdog_valid,",
    "terminal_http_valid: self.terminal_http_valid,",
    "terminal_websocket_valid: self.terminal_websocket_valid,",
  ]],
  [lifecycleTestPath, ["fn active_duration_counts_from_authorized_mining()"]],
  [campaignTestPath, ["fn soak_requires_full_active_duration()"]],
]);

export class MiningCriteriaEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "MiningCriteriaEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): MiningCriteriaEvidenceError {
  return new MiningCriteriaEvidenceError(category, message, {
    stage: "sealed_mining_criteria_projection",
    hardware_rerun_used: false,
    terminal_attempt_reopened: false,
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
    if (error instanceof MiningCriteriaEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof MiningCriteriaEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function requireExactLines(document: string, lines: readonly string[], label: string): void {
  const actual = document.split(/\r?\n/u);
  for (const line of lines) {
    if (actual.filter((candidate) => candidate === line).length !== 1) {
      throw failure("evidence_invalid", `${label} facts are incomplete`);
    }
  }
}

function validateHistoricalFacts(summary: string, smoke: string, soak: string): void {
  requireExactLines(summary, [
    "phase21_status: passed",
    "phase21_evidence_closure: approved_controlled_no_share_soak",
    "redaction_status: passed",
    "raw_artifacts_committed: no",
    "reference_clean: passed",
  ], "Phase 21 summary");
  requireExactLines(smoke, [
    "live_mining_smoke_status: controlled-no-share",
    "controlled_package_boot_status: trusted",
    "controlled_runtime_harness_status: observed",
    "pool_lifecycle_status: active",
    "subscribe_status: sent",
    "authorize_status: sent",
    "notify_job_status: accepted work_enqueued=true",
    "bm1366_work_dispatch_status: typed_action_ready",
    "result_receive_status: bounded_no_result",
    "share_submission_status: bounded_no_share",
    "api_websocket_telemetry_update_status: ready",
    "watchdog_status: bounded observations present",
    "safe_stop_status: complete mining=disabled hardware_control=disabled work_submission=disabled",
    "redaction_status: passed",
  ], "Phase 21 smoke");
  requireExactLines(soak, [
    "bounded_soak_status: approved_controlled_no_share_soak",
    "duration_seconds: 300",
    "live_smoke_prerequisite: controlled-no-share",
    "controlled_package_boot_status: trusted",
    "controlled_runtime_harness_status: observed",
    "watchdog_responsiveness_status: passed",
    "api_snapshot_status: redacted_sample_captured",
    "websocket_frame_status: passed frames=5",
    "safe_stop_status: complete mining=disabled hardware_control=disabled work_submission=disabled",
    "redaction_status: passed",
  ], "Phase 21 soak");
}

function parseCoordinator(document: string): ProtocolCoordinatorEvidence {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", "protocol coordinator projection is malformed");
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", "protocol coordinator projection must be an object");
  }
  return value as ProtocolCoordinatorEvidence;
}

function validateCoordinatorFacts(coordinator: ProtocolCoordinatorEvidence): void {
  if (coordinator.schema_version !== "bitaxe-protocol-coordinator-evidence-v1"
    || coordinator.board !== 205
    || !coordinator.coordinator.single_owner_serialization
    || !coordinator.coordinator.authorized_before_asic_dispatch
    || !coordinator.coordinator.qualified_result_before_submit
    || !coordinator.coordinator.accepted_submit_observed
    || !coordinator.coordinator.ordered_terminal_safe_stop
    || !coordinator.coordinator.watchdog_feed_in_owner_loop
    || !coordinator.coordinator.lifecycle_spans_compatible
    || coordinator.hardware_rerun_used
    || coordinator.redaction_status !== "passed") {
    throw failure("evidence_invalid", "protocol coordinator facts are incomplete");
  }
}

function requireUniqueFragments(document: string, fragments: readonly string[]): void {
  for (const fragment of fragments) {
    const first = document.indexOf(fragment);
    if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
      throw failure("evidence_invalid", "mining criteria semantic fragment is not unique");
    }
  }
}

function requireOrderedFragments(document: string, fragments: readonly string[]): void {
  let cursor = 0;
  for (const fragment of fragments) {
    const index = document.indexOf(fragment, cursor);
    if (index === -1) throw failure("evidence_invalid", "mining criteria ordering is incomplete");
    cursor = index + fragment.length;
  }
}

async function validateCurrentSource(
  processPort: ProcessPort,
  gitProgram: string,
  currentSourceCommit: string,
): Promise<void> {
  const documents = await Promise.all(criteriaPaths.map(async (sourcePath) => [
    sourcePath,
    await childText(processPort, gitProgram, ["show", `${currentSourceCommit}:${sourcePath}`],
      "mining criteria source admission"),
  ] as const));
  const byPath = new Map<string, string>(documents);
  for (const [sourcePath, fragments] of sourceFragments) {
    const document = byPath.get(sourcePath);
    if (document === undefined) throw failure("evidence_invalid", "mining criteria source is missing");
    requireUniqueFragments(document, fragments);
  }
  requireOrderedFragments(byPath.get(admissionPath) ?? "", sourceFragments.get(admissionPath) ?? []);
  requireOrderedFragments(byPath.get(soakPath) ?? "", sourceFragments.get(soakPath) ?? []);
}

async function readAdmitted(
  workspaceRoot: string,
  sourcePath: string,
  expectedPath: string,
  expectedSha256: string,
  label: string,
): Promise<{ readonly resolved: string; readonly document: string; readonly sha256: string }> {
  const resolved = assertWithinWorkspace(workspaceRoot, sourcePath);
  if (path.relative(workspaceRoot, resolved) !== expectedPath) {
    throw failure("evidence_invalid", `${label} path is invalid`);
  }
  const document = await readFile(resolved, "utf8");
  const documentSha256 = sha256(document);
  if (documentSha256 !== expectedSha256) throw failure("evidence_invalid", `${label} digest is invalid`);
  return { resolved, document, sha256: documentSha256 };
}

export async function projectMiningCriteriaEvidence(
  workspaceRoot: string,
  options: MiningCriteriaEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  validators: MiningCriteriaValidators,
  admittedDigests: MiningCriteriaAdmittedDigests = {
    summary: sourceSpecifications.summary.sha256,
    smoke: sourceSpecifications.smoke.sha256,
    soak: sourceSpecifications.soak.sha256,
    coordinator: sourceSpecifications.coordinator.sha256,
  },
): Promise<MiningCriteriaEvidence> {
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");

  const [summary, smoke, soak, coordinator] = await Promise.all([
    readAdmitted(workspaceRoot, options.summary, sourceSpecifications.summary.path,
      admittedDigests.summary, "Phase 21 summary"),
    readAdmitted(workspaceRoot, options.smoke, sourceSpecifications.smoke.path,
      admittedDigests.smoke, "Phase 21 smoke"),
    readAdmitted(workspaceRoot, options.soak, sourceSpecifications.soak.path,
      admittedDigests.soak, "Phase 21 soak"),
    readAdmitted(workspaceRoot, options.coordinatorProjection, sourceSpecifications.coordinator.path,
      admittedDigests.coordinator, "protocol coordinator projection"),
  ]);
  validateHistoricalFacts(summary.document, smoke.document, soak.document);
  await childText(processPort, validators.coordinator, [coordinator.resolved],
    "protocol coordinator source validation");
  validateCoordinatorFacts(parseCoordinator(coordinator.document));

  const [currentSourceCommit, referenceCommit] = await Promise.all([
    childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity"),
    childText(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference source identity"),
  ]);
  if (!lowerHex(currentSourceCommit, 40) || !lowerHex(referenceCommit, 40)) {
    throw failure("evidence_invalid", "mining criteria source identity is invalid");
  }
  await validateCurrentSource(processPort, gitProgram, currentSourceCommit);
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...criteriaPaths,
      ...Object.values(sourceSpecifications).map((source) => source.path)],
    "mining criteria worktree state");
  if (dirty !== "") throw failure("evidence_invalid", "mining criteria paths have uncommitted drift");

  const requestSha256 = sha256(JSON.stringify({
    command: "project-mining-criteria-evidence",
    summary: sourceSpecifications.summary.path,
    smoke: sourceSpecifications.smoke.path,
    soak: sourceSpecifications.soak.path,
    coordinator_projection: sourceSpecifications.coordinator.path,
    current_source_commit: currentSourceCommit,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: MiningCriteriaEvidence = {
    schema_version: "bitaxe-mining-criteria-evidence-v1",
    board: 205,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-mining-criteria-evidence",
      request_sha256: requestSha256,
    },
    source: {
      phase21_summary_sha256: summary.sha256,
      phase21_summary_valid: true,
      phase21_smoke_sha256: smoke.sha256,
      phase21_smoke_valid: true,
      phase21_soak_sha256: soak.sha256,
      phase21_soak_valid: true,
      protocol_coordinator_sha256: coordinator.sha256,
      protocol_coordinator_valid: true,
    },
    criteria: {
      historical_smoke_controlled_no_share: true,
      historical_soak_duration_seconds: 300,
      historical_watchdog_passed: true,
      historical_telemetry_observed: true,
      historical_safe_stop_confirmed: true,
      current_duration_seconds: 600,
      upstream_default_profile_required: true,
      active_duration_accounting: true,
      full_duration_required: true,
      accepted_share_required: true,
      network_correlation_required: true,
      safe_stop_required: true,
      private_evidence_required: true,
      redaction_required: true,
      source_spans_compatible: true,
      terminal_attempt_reopened: false,
    },
    hardware_rerun_used: false,
    redaction_status: "passed",
  };

  await mkdir(path.dirname(projection), { recursive: true });
  try {
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`,
      { encoding: "utf8", flag: "wx", mode: 0o600 });
    await chmod(candidate, 0o600);
    await childText(processPort, validators.evidence, [candidate], "independent evidence validation");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
  } catch (error) {
    await unlink(candidate).catch(() => undefined);
    throw error;
  }
  return evidence;
}

export function projectMiningCriteriaEvidenceFromInvocation(
  workspaceRoot: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
  resolveTool: (root: string, relative: string) => string,
): Promise<MiningCriteriaEvidence> {
  return projectMiningCriteriaEvidence(workspaceRoot, {
    summary: optionValue(invocation, "--summary"),
    smoke: optionValue(invocation, "--smoke"),
    soak: optionValue(invocation, "--soak"),
    coordinatorProjection: optionValue(invocation, "--coordinator-projection"),
    projection: optionValue(invocation, "--projection"),
  }, processPort, "git", {
    coordinator: resolveTool(workspaceRoot,
      "crates/bitaxe-automation-contracts/validate_protocol_coordinator_evidence"),
    evidence: resolveTool(workspaceRoot,
      "crates/bitaxe-automation-contracts/validate_mining_criteria_evidence"),
  });
}
