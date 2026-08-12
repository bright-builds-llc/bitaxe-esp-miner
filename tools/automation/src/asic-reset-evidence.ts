import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AsicInitializationEvidence,
  type AsicResetEvidence,
  type AutomationCategory,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type AsicResetEvidenceOptions = {
  readonly sourceProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedSourceProjection =
  "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json";
const expectedSourceProjectionSha256 =
  "eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4";
const expectedPlan = "docs/parity/work-plans/20260812T185214Z-PWR-001/PLAN.md";
const expectedPlanSha256 =
  "3b3fb9ca3ae38156b006863a8b3ffded8ebfea43995fa3e3ef9cbec8e3911a79";
const activeTask = "task-parity-pwr001-asic-reset-evidence-audit";
const resetPaths = [
  "crates/bitaxe-asic/src/bm1366/command.rs",
  "crates/bitaxe-asic/src/bm1366/init_plan.rs",
  "firmware/bitaxe/src/asic_adapter.rs",
  "firmware/bitaxe/src/asic_adapter/reset.rs",
  "firmware/bitaxe/src/mining_actuation.rs",
  "firmware/bitaxe/src/mining_actuation_adapter.rs",
] as const;
const semanticFragments = new Map<string, readonly string[]>([
  [resetPaths[0], [
    "pub const RESET_PULSE: Self = Self::ResetPulse {\n        low_ms: 100,\n        high_ms: 100,\n    };",
    "pub const HOLD_RESET_LOW: Self = Self::HoldResetLow;",
  ]],
  [resetPaths[1], [
    "actions.insert(1, Bm1366AdapterAction::reset_pulse());",
    "pub(crate) fn fail_closed(reason: &'static str, action: FailClosedAction) -> Self {\n        let status = AsicInitStatus::FailClosed { reason };\n        Self {\n            stages: vec![Bm1366InitStage::Preflight],\n            actions: vec![\n                Bm1366AdapterAction::HoldResetLow,",
  ]],
  [resetPaths[2], [
    "Bm1366AdapterAction::ResetPulse { low_ms, high_ms } => {\n            reset.reset_pulse(*low_ms, *high_ms)?;",
    "Bm1366AdapterAction::HoldResetLow => {\n            reset.hold_reset_low()?;\n            Ok(ActionOutcome::Continue)",
  ]],
  [resetPaths[3], [
    "pub const RESET_PULSE_LOW_MS: u32 = 100;",
    "pub const RESET_PULSE_HIGH_MS: u32 = 100;",
    "pub fn reset_pulse(&mut self, low_ms: u32, high_ms: u32)",
    "self.reset.set_low()?;\n        std::thread::sleep(std::time::Duration::from_millis(u64::from(low_ms)));\n        self.reset.set_high()?;\n        std::thread::sleep(std::time::Duration::from_millis(u64::from(high_ms)));",
    "pub fn hold_reset_low(&mut self) -> Result<()> {",
  ]],
  [resetPaths[4], [
    "pub const fn preparation_plan(profile: MiningHardwareProfile) -> [PreparationStep; 9]",
    "PreparationStep::ResetAndDetectExactlyOneChip,",
    "pub const fn safe_shutdown_plan() -> [SafeShutdownStep; 8]",
    "SafeShutdownStep::HoldResetLow,",
  ]],
  [resetPaths[5], [
    "skip_reset_pulse: false,",
    "PreparationStep::ResetAndDetectExactlyOneChip =>",
    "SafeShutdownStep::HoldResetLow =>",
  ]],
]);

export class AsicResetEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "AsicResetEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): AsicResetEvidenceError {
  return new AsicResetEvidenceError(category, message, {
    stage: "sealed_asic_reset_projection",
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
    if (error instanceof AsicResetEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof AsicResetEvidenceError) throw error;
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
    || initialization.planned_step_count !== 9
    || initialization.accepted_preparation_event_count !== 18
    || initialization.invalid_preparation_event_count !== 0
    || initialization.terminal_preparation_step !== "retain_production_uart"
    || initialization.terminal_preparation_outcome !== "completed"
    || !initialization.all_preparation_steps_completed
    || !initialization.exactly_one_chip_detected
    || !initialization.mining_ready_initialization_completed
    || !initialization.production_uart_retained
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
    throw failure("evidence_invalid", "ASIC reset source quorum is incomplete");
  }
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "ASIC reset semantic fragment is not unique");
  }
}

function validateTaskAndPlan(
  taskDocument: string,
  planDocument: string,
  admittedPlanSha256: string,
): void {
  const heading = `### ${activeTask} |`;
  const start = taskDocument.indexOf(heading);
  if (start === -1 || taskDocument.indexOf(heading, start + heading.length) !== -1) {
    throw failure("evidence_invalid", "PWR-001 active task binding is invalid");
  }
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const taskBlock = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  for (const required of [
    `Plan: \`${expectedPlan}\``,
    "active-low 100 ms/100 ms reset semantics",
    "No hardware interaction is permitted or required",
  ]) {
    if (!taskBlock.includes(required)) {
      throw failure("evidence_invalid", "PWR-001 active task binding is incomplete");
    }
  }
  if (sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `PWR-001`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "PWR-001 immutable plan binding is invalid");
  }
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  attemptSourceCommit: string,
  currentSourceCommit: string,
): Promise<void> {
  for (const sourcePath of resetPaths) {
    await childText(processPort, gitProgram,
      ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", sourcePath],
      "ASIC reset module compatibility");
    const document = await childText(processPort, gitProgram,
      ["show", `${currentSourceCommit}:${sourcePath}`], "ASIC reset source admission");
    for (const fragment of semanticFragments.get(sourcePath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }
}

export async function projectAsicResetEvidence(
  workspaceRoot: string,
  options: AsicResetEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  sourceValidatorProgram: string,
  validatorProgram: string,
  admittedSourceSha256 = expectedSourceProjectionSha256,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<AsicResetEvidence> {
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

  const [sourceDocument, taskDocument, planDocument] = await Promise.all([
    readFile(sourceProjection, "utf8"),
    readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
    readFile(path.join(workspaceRoot, expectedPlan), "utf8"),
  ]);
  const sourceProjectionSha256 = sha256(sourceDocument);
  if (sourceProjectionSha256 !== admittedSourceSha256) {
    throw failure("evidence_invalid", "initialization source projection digest is invalid");
  }
  validateTaskAndPlan(taskDocument, planDocument, admittedPlanSha256);
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
  if (!lowerHex(currentSourceCommit, 40) || !lowerHex(referenceCommit, 40)
    || source.reference_commit !== referenceCommit) {
    throw failure("evidence_invalid", "source identity is invalid or drifted");
  }
  await childText(processPort, gitProgram,
    ["cat-file", "-e", `${options.attemptSourceCommit}^{commit}`], "attempt source admission");
  await childText(processPort, gitProgram,
    ["merge-base", "--is-ancestor", options.attemptSourceCommit, currentSourceCommit],
    "attempt source ancestry");
  await validateSourceCompatibility(
    processPort, gitProgram, options.attemptSourceCommit, currentSourceCommit,
  );
  const relevantPaths = [...resetPaths, expectedSourceProjection, expectedPlan, "TASKS.md"];
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths], "ASIC reset worktree state");
  if (dirty !== "") throw failure("evidence_invalid", "ASIC reset paths have uncommitted drift");

  const requestSha256 = sha256(JSON.stringify({
    command: "project-asic-reset-evidence",
    source_projection: expectedSourceProjection,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    plan_sha256: admittedPlanSha256,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: AsicResetEvidence = {
    schema_version: "bitaxe-asic-reset-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-reset-evidence",
      request_sha256: requestSha256,
    },
    source: {
      initialization_projection_sha256: sourceProjectionSha256,
      initialization_projection_current_commit: source.current_source_commit,
      initialization_projection_valid: true,
      source_task_sha256: sha256(taskDocument),
      plan_sha256: admittedPlanSha256,
    },
    reset: {
      active_low: true,
      low_duration_ms: 100,
      high_duration_ms: 100,
      reset_and_detect_completed: true,
      exactly_one_chip_detected_after_reset: true,
      accepted_submit_observed: true,
      fail_closed_hold_low: true,
      safe_stop_hold_low: true,
      reset_paths_unchanged: true,
      compatible_path_count: resetPaths.length,
      adapter_semantics_admitted: true,
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
