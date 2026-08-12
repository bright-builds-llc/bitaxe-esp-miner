import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AsicInitializationEvidence,
  type AsicPowerInitializationEvidence,
  type AutomationCategory,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type AsicPowerInitializationEvidenceOptions = {
  readonly sourceProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedSourceProjection =
  "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json";
const expectedSourceProjectionSha256 =
  "eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4";
const expectedPlan = "docs/parity/work-plans/20260812T193941Z-PWR-002/PLAN.md";
const expectedPlanSha256 =
  "7ff2ca77e4967f2f823033ef68cfab264863fc20caad841a1ac30c8ecf5d14ff";
const activeTask = "task-parity-pwr002-asic-power-initialization-audit";

const unchangedPaths = [
  "firmware/bitaxe/src/mining_actuation.rs",
  "firmware/bitaxe/src/mining_actuation_adapter.rs",
  "firmware/bitaxe/src/asic_adapter/reset.rs",
  "firmware/bitaxe/src/safety_adapter/ds4432u.rs",
  "firmware/bitaxe/src/safety_adapter/emc2101.rs",
  "firmware/bitaxe/src/safety_adapter/request_queue.rs",
] as const;

const semanticPaths = [
  "crates/bitaxe-stratum/src/v1/production_session/campaign.rs",
  "firmware/bitaxe/src/safety_adapter.rs",
  "firmware/bitaxe/src/asic_adapter/production.rs",
] as const;

const semanticFragments = new Map<string, readonly string[]>([
  [unchangedPaths[0], [
    "pub const CORE_VOLTAGE_STABILIZATION_MS: u16 = 500;",
    `pub const fn preparation_plan(profile: MiningHardwareProfile) -> [PreparationStep; 9] {
    [
        PreparationStep::RequireFreshSafetyObservations,
        PreparationStep::SetFanDutyTo100Percent,
        PreparationStep::RequireFreshNonzeroFanRpm,
        PreparationStep::SetCoreVoltage(profile.core_voltage()),
        PreparationStep::WaitForCoreVoltageStabilization500Ms,
        PreparationStep::EnableAsic,
        PreparationStep::ResetAndDetectExactlyOneChip,
        PreparationStep::InitializeMiningReadyWithFrequencyRamp(profile.frequency()),
        PreparationStep::RetainProductionUart,
    ]
}`,
    `pub const fn safe_shutdown_plan() -> [SafeShutdownStep; 8] {
    [
        SafeShutdownStep::StopDispatch,
        SafeShutdownStep::ReduceFrequencyAndResetNonce,
        SafeShutdownStep::HoldResetLow,
        SafeShutdownStep::DisableCoreVoltage,
        SafeShutdownStep::DisableAsic,
        SafeShutdownStep::SetFanDutyTo100Percent,
        SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C,
        SafeShutdownStep::SetFanDutyTo30Percent,
    ]
}`,
    "let maybe_safe_shutdown_failure = execute_safe_shutdown(backend).err();",
    "if maybe_earliest_failure.is_none() {\n            maybe_earliest_failure = Some(SafeShutdownFailure { step, source });\n        }",
  ]],
  [unchangedPaths[1], [
    "PreparationStep::RequireFreshSafetyObservations => {",
    "PreparationStep::SetFanDutyTo100Percent => self.set_fan_full(),",
    "PreparationStep::RequireFreshNonzeroFanRpm => self.wait_for_post_command_fan_proof(),",
    "SafetyActuationCommand::SetCoreVoltage(Self::core_voltage(voltage)?)",
    "CORE_VOLTAGE_STABILIZATION_MS,",
    "crate::asic_adapter::production::set_asic_power_enabled(true)",
    "PreparationStep::ResetAndDetectExactlyOneChip => {",
    "PreparationStep::InitializeMiningReadyWithFrequencyRamp(frequency) => {",
    "PreparationStep::RetainProductionUart => {",
    "SafeShutdownStep::DisableCoreVoltage | SafeShutdownStep::DisableAsic => {",
  ]],
  [unchangedPaths[2], [
    "/// Active-low Ultra 205 ASIC power-enable owner.",
    "let mut enable = PinDriver::output(enable_pin)?;\n        enable.set_high()?;",
    "pub fn enable(&mut self) -> Result<()> {\n        self.enable.set_low()?;",
    "pub fn disable(&mut self) -> Result<()> {\n        self.enable.set_high()?;",
  ]],
  [unchangedPaths[3], [
    "Self::Conservative1100Millivolts => 1_100,",
    "bus.write_ds4432u(Ds4432uWriteRegister::Output0, code)",
  ]],
  [unchangedPaths[4], [
    "pub(crate) fn write_fan_duty_percent<Bus>(bus: &mut Bus, percent: u8)",
    "bus.write_emc2101(Emc2101WriteRegister::FanSetting, fan_duty_code(percent))",
  ]],
  [semanticPaths[0], [
    "Self::Conservative => (400, 1_100, 100),",
    "(400, 1_100, 100) | (485, 1_200, 100)",
  ]],
  [semanticPaths[1], [
    "SafetyActuationCommand::SetFanDuty(percent) => {\n            emc2101::write_fan_duty_percent(&mut bus, percent.get())\n        }",
    "SafetyActuationCommand::SetCoreVoltage(voltage) => {\n            ds4432u::write_core_voltage(&mut bus, voltage)\n        }",
  ]],
  [semanticPaths[2], [
    "pub fn set_asic_power_enabled(enabled: bool) -> Result<(), ProductionAsicBlocker> {",
    "if enabled {\n        enable\n            .enable()",
    "} else {\n        enable\n            .disable()",
  ]],
]);

export class AsicPowerInitializationEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "AsicPowerInitializationEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): AsicPowerInitializationEvidenceError {
  return new AsicPowerInitializationEvidenceError(category, message, {
    stage: "sealed_asic_power_initialization_projection",
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
    if (error instanceof AsicPowerInitializationEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof AsicPowerInitializationEvidenceError) throw error;
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
    throw failure("evidence_invalid", "ASIC power initialization source quorum is incomplete");
  }
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "ASIC power initialization semantic fragment is not unique");
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
    throw failure("evidence_invalid", "PWR-002 active task binding is invalid");
  }
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const taskBlock = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  for (const required of [
    expectedPlan,
    "fresh-safety, fan/RPM, 1100 mV, 500 ms stabilization, active-low ASIC-enable",
    "This is a software-only audit",
  ]) {
    if (!taskBlock.includes(required)) {
      throw failure("evidence_invalid", "PWR-002 active task binding is incomplete");
    }
  }
  if (sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `PWR-002`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "PWR-002 immutable plan binding is invalid");
  }
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  attemptSourceCommit: string,
  currentSourceCommit: string,
): Promise<void> {
  for (const sourcePath of unchangedPaths) {
    await childText(processPort, gitProgram,
      ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", sourcePath],
      "ASIC power initialization module compatibility");
    const document = await childText(processPort, gitProgram,
      ["show", `${currentSourceCommit}:${sourcePath}`], "ASIC power initialization source admission");
    for (const fragment of semanticFragments.get(sourcePath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }
  for (const sourcePath of semanticPaths) {
    for (const commit of [attemptSourceCommit, currentSourceCommit]) {
      const document = await childText(processPort, gitProgram,
        ["show", `${commit}:${sourcePath}`], "ASIC power initialization semantic admission");
      for (const fragment of semanticFragments.get(sourcePath) ?? []) {
        requireUniqueFragment(document, fragment);
      }
    }
  }
}

export async function projectAsicPowerInitializationEvidence(
  workspaceRoot: string,
  options: AsicPowerInitializationEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  sourceValidatorProgram: string,
  validatorProgram: string,
  admittedSourceSha256 = expectedSourceProjectionSha256,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<AsicPowerInitializationEvidence> {
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
  const relevantPaths = [
    ...unchangedPaths,
    ...semanticPaths,
    expectedSourceProjection,
    expectedPlan,
    "TASKS.md",
  ];
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths], "ASIC power initialization worktree state");
  if (dirty !== "") {
    throw failure("evidence_invalid", "ASIC power initialization paths have uncommitted drift");
  }

  const requestSha256 = sha256(JSON.stringify({
    command: "project-asic-power-initialization-evidence",
    source_projection: expectedSourceProjection,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    plan_sha256: admittedPlanSha256,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: AsicPowerInitializationEvidence = {
    schema_version: "bitaxe-asic-power-initialization-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-power-initialization-evidence",
      request_sha256: requestSha256,
    },
    source: {
      initialization_projection_sha256: sourceProjectionSha256,
      initialization_projection_current_commit: source.current_source_commit,
      initialization_projection_valid: true,
      source_task_sha256: sha256(taskDocument),
      plan_sha256: admittedPlanSha256,
    },
    power_initialization: {
      profile: "conservative",
      frequency_mhz: 400,
      core_voltage_command_mv: 1_100,
      fan_duty_command_percent: 100,
      preparation_step_count: 9,
      accepted_preparation_event_count: 18,
      fresh_safety_required_before_effects: true,
      fan_full_commanded_before_voltage: true,
      post_command_nonzero_fan_rpm_required: true,
      core_voltage_stabilization_ms: 500,
      asic_enable_active_low: true,
      reset_and_detect_completed: true,
      exactly_one_chip_detected_after_reset: true,
      mining_ready_initialization_completed: true,
      production_uart_retained: true,
      accepted_submit_observed: true,
      rollback_step_count: 8,
      rollback_attempts_all_steps: true,
      initial_preparation_failure_primary: true,
      safe_stop_asic_disable_commanded: true,
      unchanged_path_count: unchangedPaths.length,
      semantic_path_count: semanticPaths.length,
      source_semantics_admitted: true,
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
