import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AsicPowerInitializationEvidence,
  type AutomationCategory,
  type CoreVoltageControlEvidence,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type CoreVoltageControlEvidenceOptions = {
  readonly sourceProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedSourceProjection =
  "docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json";
const expectedSourceProjectionSha256 =
  "0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe";
const expectedPlan = "docs/parity/work-plans/20260812T212218Z-PWR-003/PLAN.md";
const expectedPlanSha256 =
  "dbd5d3a620726f270fd2827d4c8f53f0301834cea4999107964c22c711cb277e";
const activeTask = "task-parity-pwr003-core-voltage-control-evidence-retry";

const unchangedPaths = [
  "firmware/bitaxe/src/safety_adapter/ds4432u.rs",
  "firmware/bitaxe/src/mining_actuation.rs",
  "firmware/bitaxe/src/mining_actuation_adapter.rs",
] as const;
const semanticPaths = [
  "firmware/bitaxe/src/safety_adapter.rs",
  "firmware/bitaxe/src/safety_adapter/i2c_bus.rs",
] as const;
const referencePaths = [
  "main/power/DS4432U.c",
  "main/power/vcore.c",
] as const;

const sourceFragments = new Map<string, readonly string[]>([
  [unchangedPaths[0], [
    "Self::Conservative1100Millivolts => 1_100,",
    "Self::Output0 => 0xf8,",
    "let code = core_voltage_code(voltage.millivolts());",
    "bus.write_ds4432u(Ds4432uWriteRegister::Output0, code)",
    "assert_eq!(writer.writes, [(Ds4432uWriteRegister::Output0, 0xe1)]);",
  ]],
  [unchangedPaths[1], [
    "pub const CORE_VOLTAGE_STABILIZATION_MS: u16 = 500;",
    "PreparationStep::SetCoreVoltage(profile.core_voltage()),",
    "PreparationStep::WaitForCoreVoltageStabilization500Ms,",
    "PreparationStep::EnableAsic,",
    "pub const fn safe_shutdown_plan() -> [SafeShutdownStep; 8] {\n    [\n        SafeShutdownStep::StopDispatch,\n        SafeShutdownStep::ReduceFrequencyAndResetNonce,\n        SafeShutdownStep::HoldResetLow,\n        SafeShutdownStep::DisableCoreVoltage,\n        SafeShutdownStep::DisableAsic,\n        SafeShutdownStep::SetFanDutyTo100Percent,\n        SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C,\n        SafeShutdownStep::SetFanDutyTo30Percent,\n    ]\n}",
  ]],
  [unchangedPaths[2], [
    "SafeShutdownStep::DisableCoreVoltage | SafeShutdownStep::DisableAsic =>",
    "crate::asic_adapter::production::set_asic_power_enabled(false)",
    "Pinned upstream VCORE_set_voltage(0) performs no DS4432U write;",
  ]],
  [semanticPaths[0], [
    "Ok(()) => SafetyActuationReply::Applied,",
    "Err(_) => SafetyActuationReply::HardwareWriteFailed,",
  ]],
  [semanticPaths[1], [
    "const DS4432U_I2C_ADDRESS: u8 = 0x48;",
    "self.write_register(DS4432U_I2C_ADDRESS, register.address(), value)",
  ]],
]);

const referenceFragments = new Map<string, readonly string[]>([
  [referencePaths[0], [
    "#define DS4432U_SENSOR_ADDR 0x48",
    "#define DS4432U_OUT0_REG 0xF8",
    "DS4432U_set_current_code(0, reg)",
  ]],
  [referencePaths[1], [
    "gpio_set_level(GPIO_ASIC_ENABLE, core_voltage == 0.0f ? 1 : 0);",
    "if (core_voltage != 0.0f) {",
    "DS4432U_set_voltage(core_voltage)",
  ]],
]);

export class CoreVoltageControlEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "CoreVoltageControlEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): CoreVoltageControlEvidenceError {
  return new CoreVoltageControlEvidenceError(category, message, {
    stage: "sealed_core_voltage_control_projection",
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
    if (error instanceof CoreVoltageControlEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof CoreVoltageControlEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function parseSource(document: string): AsicPowerInitializationEvidence {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", "power-initialization source projection is malformed");
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", "power-initialization source projection must be an object");
  }
  return value as AsicPowerInitializationEvidence;
}

function validateSourceFacts(
  source: AsicPowerInitializationEvidence,
  attemptSourceCommit: string,
): void {
  const power = source.power_initialization;
  if (source.schema_version !== "bitaxe-asic-power-initialization-evidence-v1"
    || source.board !== 205
    || source.attempt_source_commit !== attemptSourceCommit
    || power.profile !== "conservative"
    || power.core_voltage_command_mv !== 1_100
    || power.preparation_step_count !== 9
    || power.accepted_preparation_event_count !== 18
    || !power.fresh_safety_required_before_effects
    || power.core_voltage_stabilization_ms !== 500
    || !power.asic_enable_active_low
    || !power.mining_ready_initialization_completed
    || !power.production_uart_retained
    || !power.accepted_submit_observed
    || !power.safe_stop_asic_disable_commanded
    || !power.source_semantics_admitted
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
    throw failure("evidence_invalid", "core-voltage-control source quorum is incomplete");
  }
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "core-voltage-control semantic fragment is not unique");
  }
}

function requireVoltagePreparation(adapter: string, actuation: string): boolean {
  const voltage = actuation.indexOf("PreparationStep::SetCoreVoltage(profile.core_voltage()),");
  const stabilization = actuation.indexOf("PreparationStep::WaitForCoreVoltageStabilization500Ms,");
  const enable = actuation.indexOf("PreparationStep::EnableAsic,");
  if (voltage < 0 || stabilization <= voltage || enable <= stabilization) {
    throw failure("evidence_invalid", "voltage stabilization must precede ASIC enable");
  }
  // Historical snapshots retain their exact blocking form. The current form
  // additionally proves the complete cancellable wait and generation guards;
  // neither form bypasses the unchanged-module Git comparison below.
  const cancellable = adapter.includes("fn cancellable_delay(");
  const fragments = cancellable ? [
    "self.request_preparation_voltage(Self::core_voltage(voltage)?)",
    "SafetyActuationCommand::SetCoreVoltageForGeneration {\n                voltage,\n                permit: crate::production_mining_session::revocation::stamp(\n                    self.maybe_worker_generation,\n                ),\n            }",
    "PreparationStep::WaitForCoreVoltageStabilization500Ms => {\n                self.cancellable_delay(u64::from(CORE_VOLTAGE_STABILIZATION_MS))\n            }",
    "crate::mining_actuation::wait_with_cancellation(\n            duration_ms,\n            crate::runtime_uptime::millis,\n            |milliseconds| thread::sleep(Duration::from_millis(milliseconds)),\n            || self.check_preparation_admission(),\n        )",
    "if !crate::production_mining_session::revocation::permits(self.maybe_worker_generation) {\n            return Err(MiningActuationAdapterError::WorkerGenerationRevoked);\n        }",
    "crate::asic_adapter::production::set_asic_power_enabled_guarded(\n                    true,\n                    self.maybe_worker_generation,\n                )",
  ] : [
    "SafetyActuationCommand::SetCoreVoltage(Self::core_voltage(voltage)?)",
    "thread::sleep(Duration::from_millis(u64::from(\n                    CORE_VOLTAGE_STABILIZATION_MS,\n                )));",
    "crate::asic_adapter::production::set_asic_power_enabled(true)",
  ];
  for (const fragment of fragments) requireUniqueFragment(adapter, fragment);
  if (cancellable) {
    requireUniqueFragment(actuation, "let deadline = now_ms().saturating_add(duration_ms);");
    requireUniqueFragment(actuation, "loop {\n        check()?;\n        let remaining = deadline.saturating_sub(now_ms());\n        if remaining == 0 {\n            return Ok(());\n        }\n        sleep_ms(remaining.min(50));\n    }");
  }
  return cancellable;
}

function requireVoltageOwner(document: string, generationRequired: boolean): void {
  const stamped = document.includes("SetCoreVoltageForGeneration");
  if (generationRequired && !stamped) throw failure("evidence_invalid", "generation voltage route is absent");
  const fragment = stamped
    ? "SafetyActuationCommand::SetCoreVoltageForGeneration { voltage, permit } => {\n            if !crate::production_mining_session::revocation::permits_work(permit) {\n                return SafetyActuationReply::HardwareWriteFailed;\n            }\n            ds4432u::write_core_voltage(&mut bus, voltage)\n        }"
    : "SafetyActuationCommand::SetCoreVoltage(voltage) => {\n            ds4432u::write_core_voltage(&mut bus, voltage)\n        }";
  requireUniqueFragment(document, fragment);
}

function validateTaskAndPlan(
  taskDocument: string,
  planDocument: string,
  admittedPlanSha256: string,
): void {
  const heading = `### ${activeTask} |`;
  const start = taskDocument.indexOf(heading);
  if (start === -1 || taskDocument.indexOf(heading, start + heading.length) !== -1) {
    throw failure("evidence_invalid", "PWR-003 active task binding is invalid");
  }
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const taskBlock = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  for (const required of [
    expectedPlan,
    "DS4432U address/register/code and write route",
    "This is a software-only evidence audit",
  ]) {
    if (!taskBlock.includes(required)) {
      throw failure("evidence_invalid", "PWR-003 active task binding is incomplete");
    }
  }
  if (sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `PWR-003`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "PWR-003 immutable plan binding is invalid");
  }
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  workspaceRoot: string,
  attemptSourceCommit: string,
  currentSourceCommit: string,
  referenceCommit: string,
): Promise<void> {
  const unchangedDocuments = new Map<string, string>();
  for (const sourcePath of unchangedPaths) {
    await childText(processPort, gitProgram,
      ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", sourcePath],
      "core-voltage-control module compatibility");
    const document = await childText(processPort, gitProgram,
      ["show", `${currentSourceCommit}:${sourcePath}`], "core-voltage-control source admission");
    unchangedDocuments.set(sourcePath, document);
    for (const fragment of sourceFragments.get(sourcePath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }
  const adapter = unchangedDocuments.get(unchangedPaths[2]);
  const actuation = unchangedDocuments.get(unchangedPaths[1]);
  if (adapter === undefined || actuation === undefined) throw failure("evidence_invalid", "voltage source pair is incomplete");
  const generationRequired = requireVoltagePreparation(adapter, actuation);
  for (const sourcePath of semanticPaths) {
    for (const commit of [attemptSourceCommit, currentSourceCommit]) {
      const document = await childText(processPort, gitProgram,
        ["show", `${commit}:${sourcePath}`], "core-voltage-control semantic admission");
      if (sourcePath === semanticPaths[0]) requireVoltageOwner(document, generationRequired);
      for (const fragment of sourceFragments.get(sourcePath) ?? []) {
        requireUniqueFragment(document, fragment);
      }
    }
  }
  for (const referencePath of referencePaths) {
    const document = await childText(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "show",
        `${referenceCommit}:${referencePath}`], "core-voltage-control reference admission");
    for (const fragment of referenceFragments.get(referencePath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }
}

export async function projectCoreVoltageControlEvidence(
  workspaceRoot: string,
  options: CoreVoltageControlEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  sourceValidatorProgram: string,
  validatorProgram: string,
  admittedSourceSha256 = expectedSourceProjectionSha256,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<CoreVoltageControlEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const sourceProjection = assertWithinWorkspace(workspaceRoot, options.sourceProjection);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, sourceProjection) !== expectedSourceProjection) {
    throw failure("evidence_invalid", "power-initialization source projection path is invalid");
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
    throw failure("evidence_invalid", "power-initialization source projection digest is invalid");
  }
  validateTaskAndPlan(taskDocument, planDocument, admittedPlanSha256);
  const source = parseSource(sourceDocument);
  await childText(processPort, sourceValidatorProgram, [sourceProjection],
    "power-initialization source validation");
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
    processPort, gitProgram, workspaceRoot, options.attemptSourceCommit,
    currentSourceCommit, referenceCommit,
  );
  const relevantPaths = [
    ...unchangedPaths,
    ...semanticPaths,
    expectedSourceProjection,
    expectedPlan,
    "TASKS.md",
  ];
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths], "core-voltage-control worktree state");
  if (dirty !== "") {
    throw failure("evidence_invalid", "core-voltage-control paths have uncommitted drift");
  }

  const requestSha256 = sha256(JSON.stringify({
    command: "project-core-voltage-control-evidence",
    source_projection: expectedSourceProjection,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    plan_sha256: admittedPlanSha256,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: CoreVoltageControlEvidence = {
    schema_version: "bitaxe-core-voltage-control-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-core-voltage-control-evidence",
      request_sha256: requestSha256,
    },
    source: {
      power_initialization_projection_sha256: sourceProjectionSha256,
      power_initialization_projection_current_commit: source.current_source_commit,
      power_initialization_projection_valid: true,
      source_task_sha256: sha256(taskDocument),
      plan_sha256: admittedPlanSha256,
    },
    voltage_control: {
      target_millivolts: 1_100,
      i2c_address: 0x48,
      output_register: 0xf8,
      register_code: 0xe1,
      register_write_count: 1,
      typed_command_routed: true,
      stabilization_millis: 500,
      stabilization_before_asic_enable: true,
      zero_voltage_skips_ds4432u_write: true,
      active_low_disable: true,
      successful_initialized_work_observed: true,
      accepted_submit_observed: true,
      compatible_path_count: 5,
      reference_semantics_admitted: true,
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
