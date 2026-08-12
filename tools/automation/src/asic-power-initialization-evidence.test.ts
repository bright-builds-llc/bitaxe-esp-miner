import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  AsicPowerInitializationEvidenceError,
  projectAsicPowerInitializationEvidence,
} from "./asic-power-initialization-evidence.js";
import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";

const attemptCommit = "a".repeat(40);
const sourceCommit = "b".repeat(40);
const currentCommit = "c".repeat(40);
const referenceCommit = "d".repeat(40);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

const sourceDocuments = new Map<string, string>([
  ["firmware/bitaxe/src/mining_actuation.rs", `
pub const CORE_VOLTAGE_STABILIZATION_MS: u16 = 500;
pub const fn preparation_plan(profile: MiningHardwareProfile) -> [PreparationStep; 9] {
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
}
pub const fn safe_shutdown_plan() -> [SafeShutdownStep; 8] {
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
}
let maybe_safe_shutdown_failure = execute_safe_shutdown(backend).err();
if maybe_earliest_failure.is_none() {
            maybe_earliest_failure = Some(SafeShutdownFailure { step, source });
        }
`],
  ["firmware/bitaxe/src/mining_actuation_adapter.rs", `
PreparationStep::RequireFreshSafetyObservations => {
PreparationStep::SetFanDutyTo100Percent => self.set_fan_full(),
PreparationStep::RequireFreshNonzeroFanRpm => self.wait_for_post_command_fan_proof(),
SafetyActuationCommand::SetCoreVoltage(Self::core_voltage(voltage)?)
CORE_VOLTAGE_STABILIZATION_MS,
crate::asic_adapter::production::set_asic_power_enabled(true)
PreparationStep::ResetAndDetectExactlyOneChip => {
PreparationStep::InitializeMiningReadyWithFrequencyRamp(frequency) => {
PreparationStep::RetainProductionUart => {
SafeShutdownStep::DisableCoreVoltage | SafeShutdownStep::DisableAsic => {
`],
  ["firmware/bitaxe/src/asic_adapter/reset.rs", `
/// Active-low Ultra 205 ASIC power-enable owner.
let mut enable = PinDriver::output(enable_pin)?;
        enable.set_high()?;
pub fn enable(&mut self) -> Result<()> {
        self.enable.set_low()?;
pub fn disable(&mut self) -> Result<()> {
        self.enable.set_high()?;
`],
  ["firmware/bitaxe/src/safety_adapter/ds4432u.rs", `
Self::Conservative1100Millivolts => 1_100,
bus.write_ds4432u(Ds4432uWriteRegister::Output0, code)
`],
  ["firmware/bitaxe/src/safety_adapter/emc2101.rs", `
pub(crate) fn write_fan_duty_percent<Bus>(bus: &mut Bus, percent: u8)
bus.write_emc2101(Emc2101WriteRegister::FanSetting, fan_duty_code(percent))
`],
  ["firmware/bitaxe/src/safety_adapter/request_queue.rs", "bounded request queue\n"],
  ["crates/bitaxe-stratum/src/v1/production_session/campaign.rs", `
Self::Conservative => (400, 1_100, 100),
(400, 1_100, 100) | (485, 1_200, 100)
`],
  ["firmware/bitaxe/src/safety_adapter.rs", `
SafetyActuationCommand::SetFanDuty(percent) => {
            emc2101::write_fan_duty_percent(&mut bus, percent.get())
        }
SafetyActuationCommand::SetCoreVoltage(voltage) => {
            ds4432u::write_core_voltage(&mut bus, voltage)
        }
`],
  ["firmware/bitaxe/src/asic_adapter/production.rs", `
pub fn set_asic_power_enabled(enabled: bool) -> Result<(), ProductionAsicBlocker> {
if enabled {
        enable
            .enable()
} else {
        enable
            .disable()
`],
]);

function sourceEvidence(complete = true) {
  return {
    schema_version: "bitaxe-asic-initialization-evidence-v1",
    board: 205,
    attempt_source_commit: attemptCommit,
    current_source_commit: sourceCommit,
    reference_commit: referenceCommit,
    source_task_sha256: "e".repeat(64),
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-initialization-evidence",
      request_sha256: "f".repeat(64),
    },
    attempt: {
      campaign_result_sha256: "0".repeat(64), diagnostics_sha256: "1".repeat(64),
      observations_sha256: "2".repeat(64), result_seal_valid: true,
      private_digests_valid: true, protected_modes_valid: true,
    },
    initialization: {
      planned_step_count: 9,
      accepted_preparation_event_count: 18,
      invalid_preparation_event_count: 0,
      terminal_preparation_step: "retain_production_uart",
      terminal_preparation_outcome: "completed",
      all_preparation_steps_completed: complete,
      exactly_one_chip_detected: true,
      mining_ready_initialization_completed: true,
      production_uart_retained: true,
      live_initialized_work_observed: true,
      initialization_paths_unchanged: true,
      compatible_path_count: 7,
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
}

async function fixture(name: string, complete = true) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-asic-power-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const sourceProjection = path.join(root,
    "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json");
  await mkdir(path.dirname(sourceProjection), { recursive: true });
  const sourceDocument = `${JSON.stringify(sourceEvidence(complete), null, 2)}\n`;
  await writeFile(sourceProjection, sourceDocument);
  const plan = path.join(root, "docs/parity/work-plans/20260812T193941Z-PWR-002/PLAN.md");
  await mkdir(path.dirname(plan), { recursive: true });
  const planDocument = `# Plan

- Parity row: \`PWR-002\`
- Active task: \`task-parity-pwr002-asic-power-initialization-audit\`
`;
  await writeFile(plan, planDocument);
  await writeFile(path.join(root, "TASKS.md"), `
### task-parity-pwr002-asic-power-initialization-audit | 2026-08-12 | Audit

Plan: \`docs/parity/work-plans/20260812T193941Z-PWR-002/PLAN.md\`

Promotion requires fresh-safety, fan/RPM, 1100 mV, 500 ms stabilization, active-low ASIC-enable.

Safety: This is a software-only audit without device effects.

### next-task | later | Other
`);
  const projection = path.join(root,
    "docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json");
  return {
    root,
    projection,
    sourceSha256: createHash("sha256").update(sourceDocument).digest("hex"),
    planSha256: createHash("sha256").update(planDocument).digest("hex"),
    options: { sourceProjection, attemptSourceCommit: attemptCommit, projection },
  };
}

function fakePort(options: {
  readonly sourceDrift?: boolean;
  readonly semanticDrift?: boolean;
  readonly dirty?: boolean;
  readonly validatorFailure?: boolean;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (options.launchFailure) throw new Error("launch failed");
    if (options.validatorFailure && spec.program === "validator") {
      return { exitCode: 1, stdout: "", stderr: "rejected", timedOut: false };
    }
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "status") return ok(options.dirty ? " M mining_actuation.rs\n" : "");
    if (spec.args[0] === "diff" && options.sourceDrift) {
      return { exitCode: 1, stdout: "", stderr: "", timedOut: false };
    }
    if (spec.args[0] === "show") {
      const target = spec.args[1] ?? "";
      const separator = target.indexOf(":");
      const sourcePath = separator === -1 ? "" : target.slice(separator + 1);
      const source = sourceDocuments.get(sourcePath);
      if (source !== undefined) {
        const currentSafety = target.startsWith(currentCommit)
          && sourcePath === "firmware/bitaxe/src/safety_adapter.rs";
        return ok(options.semanticDrift && currentSafety
          ? source.replace("ds4432u::write_core_voltage", "drifted::write_core_voltage")
          : source);
      }
    }
    return ok();
  });
}

async function captureError(
  promise: Promise<unknown>,
): Promise<AsicPowerInitializationEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof AsicPowerInitializationEvidenceError);
    return error;
  }
}

async function projectFixture(
  value: Awaited<ReturnType<typeof fixture>>,
  processPort: ProcessPort,
) {
  return projectAsicPowerInitializationEvidence(
    value.root, value.options, processPort, "git", "source-validator", "validator",
    value.sourceSha256, value.planSha256,
  );
}

test("complete power transaction emits only closed row evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectFixture(value, fakePort());

  // Assert
  assert.equal(evidence.power_initialization.core_voltage_command_mv, 1_100);
  assert.equal(evidence.power_initialization.core_voltage_stabilization_ms, 500);
  assert.equal(evidence.power_initialization.initial_preparation_failure_primary, true);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /hostname|origin|usbmodem|ssid|password|private\/|scratch\//iu);
});

for (const [name, complete, options, category] of [
  ["incomplete-source", false, {}, "evidence_invalid"],
  ["source-drift", true, { sourceDrift: true }, "evidence_invalid"],
  ["semantic-drift", true, { semanticDrift: true }, "evidence_invalid"],
  ["dirty-source", true, { dirty: true }, "evidence_invalid"],
  ["validator-rejected", true, { validatorFailure: true }, "evidence_invalid"],
  ["launch-failed", true, { launchFailure: true }, "process_failed"],
] as const) {
  test(`${name} withholds final power evidence`, async () => {
    // Arrange
    const value = await fixture(name, complete);

    // Act
    const error = await captureError(projectFixture(value, fakePort(options)));

    // Assert
    assert.equal(error.category, category);
    assert.deepEqual(error.publicValue, {
      stage: "sealed_asic_power_initialization_projection",
      hardware_rerun_used: false,
      projection_published: false,
    });
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    await assert.rejects(readFile(`${value.projection}.candidate`, "utf8"), { code: "ENOENT" });
  });
}

test("mutated active task binding is rejected before publication", async () => {
  // Arrange
  const value = await fixture("task-drift");
  await writeFile(path.join(value.root, "TASKS.md"), "### unrelated | task\n");

  // Act
  const error = await captureError(projectFixture(value, fakePort()));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("real child validators must accept source and candidate files", async () => {
  // Arrange
  const value = await fixture("real-child");
  const validator = path.join(value.root, "validator-child.sh");
  await writeFile(validator, "#!/bin/sh\ntest -s \"$1\"\n");
  await chmod(validator, 0o700);
  const localPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });
  const gitPort = fakePort();
  const processPort: ProcessPort = {
    loadEspEnvironment: () => localPort.loadEspEnvironment(),
    run: (spec, maybeTimeoutMs) => spec.program === "git-fixture"
      ? gitPort.run(spec, maybeTimeoutMs)
      : localPort.run(spec, maybeTimeoutMs),
  };

  // Act
  const evidence = await projectAsicPowerInitializationEvidence(
    value.root, value.options, processPort, "git-fixture", validator, validator,
    value.sourceSha256, value.planSha256,
  );

  // Assert
  assert.equal(evidence.source.initialization_projection_valid, true);
  assert.equal(evidence.power_initialization.source_semantics_admitted, true);
});
